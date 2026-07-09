// ============================================================================
// dashboard_service.rs -- Dashboard data aggregation service
//
// Responsibilities:
// - Aggregate data from config, history, scheduler, and diskspace
// - Produce a single DashboardOverview for the Tauri command layer
//
// This service is READ-ONLY. It never creates, modifies, or deletes any
// configuration, history records, scheduled tasks, or backup data.
// ============================================================================

use std::collections::BTreeSet;

use crate::app::error::AppError;
use crate::app::models::common::{HealthStatus, OperationType, ProtectionStatus};
use crate::app::models::dashboard::{
    ActivityRecord, BackupSummary, DashboardOverview, StorageStatus,
};
use crate::config::Config;
use crate::history::HistoryDb;
use crate::scheduler;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Build the full DashboardOverview by aggregating across subsystems.
///
/// Data flow:
///   Config::load()         -> job count & unique destination paths
///   HistoryDb::query()     -> last backup + recent activity
///   scheduler::list_tasks() -> scheduled task count
///   diskspace::free_space() -> per-destination storage usage
pub fn get_overview() -> Result<DashboardOverview, AppError> {
    // ---- Config ----

    let config = Config::load().map_err(AppError::from)?;

    let total_jobs = config.job.len() as u32;
    eprintln!("DEBUG[dashboard_service]: loaded {} jobs", total_jobs);
    for n in config.job.keys() {
        eprintln!("  job: {}", n);
    }

    // Collect unique destination paths for storage & history queries
    let dests: BTreeSet<&std::path::PathBuf> = config.job.values().map(|j| &j.dest).collect();

    // ---- History (last backup + recent activity) ----
    let (last_backup, recent_activity) = query_history_aggregate(&dests);

    // ---- Storage (per destination) ----
    let storage: Vec<StorageStatus> = dests
        .iter()
        .filter_map(|d| get_storage_status(d).ok())
        .collect();

    // ---- Scheduler ----
    // scheduled_count = number of active Windows Task Scheduler entries (schtasks),
    // NOT the number of ScheduleProfile configs. One job with one schedule = one task.
    // This is a real-time query, not a config-derived count.
    let scheduled_count = match scheduler::list_tasks() {
        Ok(tasks) => tasks.len() as u32,
        Err(_) => 0,
    };

    // ---- Derived state ----
    let protection_status = derive_protection(&last_backup);
    let health = derive_health(&protection_status, &storage);

    Ok(DashboardOverview {
        protection_status,
        total_jobs,
        last_backup,
        storage,
        recent_activity,
        scheduled_count,
        health,
    })
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Query history from every unique destination and aggregate results.
///
/// Returns (last_backup, recent_activity) where last_backup is the single
/// most recent backup across all destinations.
fn query_history_aggregate(
    dests: &BTreeSet<&std::path::PathBuf>,
) -> (Option<BackupSummary>, Vec<ActivityRecord>) {
    let mut all_records: Vec<crate::history::OperationRecord> = Vec::new();

    for dest in dests {
        let db_path = HistoryDb::history_db_path(dest);
        match HistoryDb::open_or_create(&db_path) {
            Ok(db) => {
                if let Ok(records) = db.query_history(10, None) {
                    all_records.extend(records);
                }
            }
            Err(_) => continue,
        }
    }

    // Sort by timestamp descending (newest first)
    all_records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    let last_backup = all_records
        .iter()
        .find(|r| r.operation == "backup")
        .map(|r| BackupSummary {
            timestamp: r.timestamp.clone(),
            status: r.status.clone(),
            job_name: r.job_name.clone().unwrap_or_default(),
            file_count: r.file_count,
            total_bytes: r.total_bytes,
            duration_ms: r.duration_ms,
        });

    let recent_activity: Vec<ActivityRecord> = all_records
        .into_iter()
        .take(10)
        .map(|r| {
            let op = match r.operation.as_str() {
                "backup" => OperationType::Backup,
                "restore" => OperationType::Restore,
                "verify" => OperationType::Verify,
                _ => OperationType::Backup,
            };
            ActivityRecord {
                timestamp: r.timestamp,
                operation: op,
                job_name: r.job_name,
                status: r.status,
                file_count: r.file_count,
                total_bytes: r.total_bytes,
            }
        })
        .collect();

    (last_backup, recent_activity)
}

/// Get storage usage for a single destination path.
fn get_storage_status(dest: &std::path::Path) -> Result<StorageStatus, AppError> {
    let (free_bytes, total_bytes) =
        crate::diskspace::free_space(dest).map_err(|e| AppError::storage(e.to_string()))?;

    let used_bytes = total_bytes.saturating_sub(free_bytes);
    let used_pct = if total_bytes > 0 {
        (used_bytes as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    Ok(StorageStatus {
        path: dest.to_string_lossy().into_owned(),
        free_bytes,
        total_bytes,
        used_bytes,
        used_pct,
    })
}

/// Derive overall protection status from the last backup record.
fn derive_protection(last_backup: &Option<BackupSummary>) -> ProtectionStatus {
    match last_backup {
        Some(b) => match b.status.as_str() {
            "success" => ProtectionStatus::Protected,
            "partial" => ProtectionStatus::AtRisk,
            _ => ProtectionStatus::AtRisk,
        },
        None => ProtectionStatus::Critical,
    }
}

/// Derive platform health from protection status and storage state.
fn derive_health(protection: &ProtectionStatus, storage: &[StorageStatus]) -> HealthStatus {
    // Critical protection -> Critical health
    if *protection == ProtectionStatus::Critical {
        return HealthStatus::Critical;
    }

    // Any destination with < 5% free space -> Warning
    for s in storage {
        let free_pct = if s.total_bytes > 0 {
            (s.free_bytes as f64 / s.total_bytes as f64) * 100.0
        } else {
            0.0
        };
        if free_pct < 5.0 {
            return HealthStatus::Warning;
        }
    }

    if *protection == ProtectionStatus::AtRisk {
        return HealthStatus::Warning;
    }

    HealthStatus::Healthy
}
