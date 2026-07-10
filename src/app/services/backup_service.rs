// ============================================================================
// backup_service.rs -- Backup domain service
//
// Responsibilities:
//   - List all configured backup jobs with derived status
//   - Execute a backup for a given job through the Core Engine
//   - Record backup results in the history database
//
// This service is the single entry point for all backup operations.
// It orchestrates config reading, core execution, and history recording.
// ============================================================================

use std::path::Path;
use std::time::Instant;

use crate::app::error::AppError;
use crate::app::models::backup::{BackupJobStatus, BackupJobView, BackupResult};
use crate::config::{Config, JobConfig};
use crate::history::{HistoryDb, OperationRecord};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all configured backup jobs with derived status.
///
/// Each job's status is derived from its configuration validity and
/// the last recorded backup operation in the history database.
pub fn list_jobs() -> Result<Vec<BackupJobView>, AppError> {
    let config = match Config::load() {
        Ok(c) => c,
        // If no config file exists, return empty list (clean first-time experience)
        Err(_) => return Ok(Vec::new()),
    };

    let jobs: Vec<BackupJobView> = config
        .job
        .iter()
        .map(|(name, job_cfg)| build_job_view(name, job_cfg))
        .collect();

    Ok(jobs)
}

/// Get a single job's detailed view by name.
pub fn get_job_detail(name: &str) -> Result<BackupJobView, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let job_cfg = config
        .job
        .get(name)
        .ok_or_else(|| AppError::config(format!("Job '{}' not found", name)))?;

    Ok(build_job_view(name, job_cfg))
}

/// Execute a backup for the given job name.
///
/// Flow:
///   1. Load config and find the job
///   2. Validate source and destination paths
///   3. Call core backup::execute_backup()
///   4. Record the result in history
///   5. Return BackupResult
pub fn run_backup(job_name: &str) -> Result<BackupResult, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let job_cfg = config
        .job
        .get(job_name)
        .ok_or_else(|| AppError::config(format!("Job '{}' not found", job_name)))?;

    // Validate source exists
    if !job_cfg.source.exists() {
        return Err(AppError::config(format!(
            "Source path '{}' does not exist",
            job_cfg.source.display()
        )));
    }

    let start = Instant::now();

    // Execute the backup via core engine
    let backup_id = crate::backup::execute_backup(&job_cfg.source, &job_cfg.dest, job_cfg.compress)
        .map_err(|e| {
            AppError::internal(format!("Backup failed: {}", e)).with_detail(e.to_string())
        })?;

    let duration_ms = start.elapsed().as_millis() as u64;

    // Record in history
    let history_result = record_backup_in_history(
        &backup_id,
        job_name,
        &job_cfg.source,
        &job_cfg.dest,
        0, // file_count — unknown from execute_backup return
        0, // total_bytes — unknown from execute_backup return
        duration_ms,
        "success",
    );

    if let Err(e) = history_result {
        eprintln!("Warning: failed to record backup history: {}", e);
    }

    let timestamp = chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    Ok(BackupResult {
        backup_id,
        timestamp,
        file_count: 0,
        total_bytes: 0,
        duration_ms,
        status: "success".into(),
        error: None,
    })
}

/// Run a backup without recording history (for testing/validation).
pub fn run_backup_dry(job_name: &str) -> Result<BackupResult, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let job_cfg = config
        .job
        .get(job_name)
        .ok_or_else(|| AppError::config(format!("Job '{}' not found", job_name)))?;

    if !job_cfg.source.exists() {
        return Err(AppError::config(format!(
            "Source path '{}' does not exist",
            job_cfg.source.display()
        )));
    }

    Ok(BackupResult {
        backup_id: "dry-run".into(),
        timestamp: chrono::Local::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string(),
        file_count: 0,
        total_bytes: 0,
        duration_ms: 0,
        status: "dry_run".into(),
        error: None,
    })
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Build a BackupJobView from config data + history query.
fn build_job_view(name: &str, cfg: &JobConfig) -> BackupJobView {
    let retention = match &cfg.retention {
        Some(r) => match (r.keep_count, r.keep_days) {
            (Some(c), None) => format!("Keep last {} versions", c),
            (None, Some(d)) => format!("Keep {} days", d),
            (Some(c), Some(d)) => format!("Keep {} versions or {} days", c, d),
            (None, None) => "Keep all".into(),
        },
        None => "Keep all".into(),
    };

    // Query history for last backup of this job
    let (last_time, last_status, last_files, last_bytes) =
        query_last_backup_for_job(name, &cfg.dest);

    let status = derive_job_status(cfg, &last_status);

    BackupJobView {
        name: name.into(),
        source: cfg.source.to_string_lossy().into_owned(),
        dest: cfg.dest.to_string_lossy().into_owned(),
        compress: cfg.compress,
        retention,
        status,
        last_backup_time: last_time,
        last_backup_status: last_status,
        last_backup_files: last_files,
        last_backup_bytes: last_bytes,
    }
}

/// Query the history database for the last backup of a specific job.
fn query_last_backup_for_job(
    job_name: &str,
    dest: &Path,
) -> (Option<String>, Option<String>, u64, u64) {
    let db_path = HistoryDb::history_db_path(dest);
    let db = match HistoryDb::open_or_create(&db_path) {
        Ok(db) => db,
        Err(_) => return (None, None, 0, 0),
    };

    let records = match db.query_history(10, Some("backup")) {
        Ok(r) => r,
        Err(_) => return (None, None, 0, 0),
    };

    // Find the most recent backup for this job
    for r in &records {
        if r.job_name.as_deref() == Some(job_name) {
            return (
                Some(r.timestamp.clone()),
                Some(r.status.clone()),
                r.file_count,
                r.total_bytes,
            );
        }
    }

    (None, None, 0, 0)
}

/// Derive the job's display status from config validity and last backup.
fn derive_job_status(cfg: &JobConfig, last_status: &Option<String>) -> BackupJobStatus {
    // Check for misconfiguration
    if !cfg.source.exists() {
        return BackupJobStatus::Misconfigured;
    }

    // Check parent directory of dest exists
    if let Some(parent) = cfg.dest.parent() {
        if !parent.exists() {
            return BackupJobStatus::Misconfigured;
        }
    }

    match last_status {
        None => BackupJobStatus::NeverRun,
        Some(s) if s == "success" => BackupJobStatus::Active,
        _ => BackupJobStatus::Error,
    }
}

/// Record a backup operation in the history database.
#[allow(clippy::too_many_arguments)]
fn record_backup_in_history(
    backup_id: &str,
    job_name: &str,
    source: &Path,
    dest: &Path,
    file_count: u64,
    total_bytes: u64,
    duration_ms: u64,
    status: &str,
) -> Result<(), AppError> {
    let db_path = HistoryDb::history_db_path(dest);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let db = HistoryDb::open_or_create(&db_path)
        .map_err(|e| AppError::history(format!("Cannot open history DB: {}", e)))?;

    let timestamp = chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    db.record_operation(&OperationRecord {
        backup_id: backup_id.into(),
        operation: "backup".into(),
        timestamp,
        source_root: source.to_string_lossy().into_owned(),
        dest_path: dest.to_string_lossy().into_owned(),
        job_name: Some(job_name.into()),
        file_count,
        total_bytes,
        duration_ms,
        exit_code: if status == "success" { 0 } else { 1 },
        status: status.into(),
    })
    .map_err(|e| AppError::history(format!("Failed to record history: {}", e)))?;

    Ok(())
}
