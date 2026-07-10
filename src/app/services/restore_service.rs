// ============================================================================
// restore_service.rs -- Restore domain service
//
// Responsibilities:
//   - List available restore points (from history database)
//   - Preview files in a backup point
//   - Execute restore from a backup point to a destination
//
// This service is the single entry point for all restore operations.
// It orchestrates history queries, core engine calls, and results.
// ============================================================================

use std::path::Path;
use std::time::Instant;

use crate::app::error::AppError;
use crate::app::models::restore::{
    RestoreFileEntry, RestoreOperationResult, RestorePointView, RestorePreview, RestoreRequest,
};
use crate::config::Config;
use crate::history::{HistoryDb, OperationRecord};
use crate::storage;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all available restore points across all configured jobs.
///
/// Returns restore points sorted by timestamp (newest first).
/// Returns an empty Vec if no history exists or no config is found.
pub fn list_restore_points() -> Result<Vec<RestorePointView>, AppError> {
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };

    let mut points: Vec<RestorePointView> = Vec::new();

    for (job_name, job_cfg) in &config.job {
        let db_path = HistoryDb::history_db_path(&job_cfg.dest);
        let db = match HistoryDb::open_or_create(&db_path) {
            Ok(db) => db,
            Err(_) => continue,
        };

        let records = match db.query_history(100, Some("backup")) {
            Ok(r) => r,
            Err(_) => continue,
        };

        for record in &records {
            // Verify the backup directory still exists
            let backup_dir = job_cfg.dest.join(&record.backup_id);
            if !backup_dir.exists() {
                continue;
            }

            points.push(RestorePointView {
                backup_id: record.backup_id.clone(),
                job_name: record.job_name.clone().or(Some(job_name.clone())),
                timestamp: record.timestamp.clone(),
                source_root: record.source_root.clone(),
                dest_path: job_cfg.dest.to_string_lossy().into_owned(),
                file_count: record.file_count,
                total_bytes: record.total_bytes,
                status: record.status.clone(),
            });
        }
    }

    // Sort newest first
    points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(points)
}

/// Get a preview of files that would be restored from a backup point.
pub fn get_restore_preview(backup_id: &str) -> Result<RestorePreview, AppError> {
    // Find the backup point by searching all job destinations
    let config = Config::load().map_err(AppError::from)?;

    for job_cfg in config.job.values() {
        let backup_dir = job_cfg.dest.join(backup_id);
        let manifest_path = backup_dir.join("manifest.json");

        if !backup_dir.exists() || !manifest_path.exists() {
            continue;
        }

        let manifest = storage::read_manifest(&backup_dir)
            .map_err(|e| AppError::internal(format!("Failed to read manifest: {}", e)))?;

        let mut files: Vec<RestoreFileEntry> = Vec::new();
        let mut total_files: u64 = 0;
        let mut total_bytes: u64 = 0;

        for file_entry in &manifest.files {
            files.push(RestoreFileEntry {
                relative_path: file_entry.relative_path.clone(),
                size_bytes: file_entry.size_bytes,
                modified_time: file_entry.modified_time.clone(),
            });
            total_files += 1;
            total_bytes += file_entry.size_bytes;
        }

        // Build point view from manifest
        let point = RestorePointView {
            backup_id: backup_id.to_string(),
            job_name: None,
            timestamp: manifest.created_at.clone(),
            source_root: manifest.source_root.clone(),
            dest_path: job_cfg.dest.to_string_lossy().into_owned(),
            file_count: total_files,
            total_bytes,
            status: "success".into(),
        };

        return Ok(RestorePreview {
            point,
            files,
            total_files,
            total_bytes,
        });
    }

    Err(AppError::config(format!(
        "Backup point '{}' not found. The backup directory may have been moved or deleted.",
        backup_id
    )))
}

/// Execute a restore operation.
///
/// Flow:
///   1. Find the backup point directory
///   2. Call core restore::execute_restore()
///   3. Record the result in history
///   4. Return RestoreOperationResult
pub fn execute_restore(request: &RestoreRequest) -> Result<RestoreOperationResult, AppError> {
    if request.backup_id.trim().is_empty() {
        return Err(AppError::config("Backup point ID must not be empty"));
    }
    if request.dest.trim().is_empty() {
        return Err(AppError::config("Destination path must not be empty"));
    }

    let dest = Path::new(&request.dest);
    let backup_dir = find_backup_dir(&request.backup_id)?;

    // Ensure destination exists
    if !dest.exists() {
        std::fs::create_dir_all(dest).map_err(|e| {
            AppError::storage(format!("Cannot create destination directory: {}", e))
        })?;
    }

    let start = Instant::now();

    // Execute the restore via core engine
    let core_result = crate::restore::execute_restore(&backup_dir, dest, request.overwrite)
        .map_err(|e| {
            AppError::internal(format!("Restore failed: {}", e)).with_detail(e.to_string())
        })?;

    let duration_ms = start.elapsed().as_millis() as u64;

    // Determine status
    let status = if core_result.checksum_failures > 0 {
        "partial"
    } else {
        "success"
    };

    // Record in history (for the first job that matches)
    if let Some(job_cfg) = find_job_for_backup_dir(&backup_dir) {
        let db_path = HistoryDb::history_db_path(&job_cfg.dest);
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        if let Ok(db) = HistoryDb::open_or_create(&db_path) {
            let timestamp = chrono::Local::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string();

            let record = OperationRecord {
                backup_id: request.backup_id.clone(),
                operation: "restore".into(),
                timestamp: timestamp.clone(),
                source_root: String::new(),
                dest_path: request.dest.clone(),
                job_name: None,
                file_count: core_result.restored_count,
                total_bytes: 0,
                duration_ms,
                exit_code: if status == "success" { 0 } else { 1 },
                status: status.into(),
            };

            db.record_operation(&record).ok();
        }
    }

    let timestamp = chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    Ok(RestoreOperationResult {
        restore_id: format!("rst-{}", chrono::Local::now().format("%Y%m%d%H%M%S")),
        restored_count: core_result.restored_count,
        skipped_count: core_result.skipped_count,
        checksum_failures: core_result.checksum_failures,
        timestamp,
        duration_ms,
        status: status.into(),
        error: if core_result.checksum_failures > 0 {
            Some(format!(
                "{} files have checksum mismatches",
                core_result.checksum_failures
            ))
        } else {
            None
        },
    })
}

/// Delete a backup set by backup_id.
///
/// This permanently removes:
///   1. The backup directory (all files + manifest)
///   2. The history record
///
/// It does NOT affect:
///   - Job configuration
///   - Other backup sets
///   - The config file
pub fn delete_backup_set(backup_id: &str) -> Result<(), AppError> {
    if backup_id.trim().is_empty() {
        return Err(AppError::config("Backup point ID must not be empty"));
    }

    // 1. Find the backup directory and read manifest metadata BEFORE deletion
    let backup_dir = find_backup_dir(backup_id)?;
    let manifest_path = backup_dir.join("manifest.json");

    // Read manifest to capture file metadata for history
    let (source_root, file_count, total_bytes) = if manifest_path.exists() {
        match crate::manifest::Manifest::from_file(&manifest_path) {
            Ok(m) => (
                m.source_root.clone(),
                m.summary.file_count,
                m.summary.total_bytes,
            ),
            Err(_) => (String::new(), 0, 0),
        }
    } else {
        (String::new(), 0, 0)
    };

    // Find job name before deleting the config reference
    let job_name = find_job_name_for_backup_dir(&backup_dir);

    eprintln!(
        "Deleting backup set: {} at {}",
        backup_id,
        backup_dir.display()
    );

    // 2. Delete associated history records and record the deletion event
    if let Some(job_cfg) = find_job_for_backup_dir(&backup_dir) {
        let db_path = crate::history::HistoryDb::history_db_path(&job_cfg.dest);
        if let Ok(db) = crate::history::HistoryDb::open_or_create(&db_path) {
            let deleted = db.delete_operation_by_backup_id(backup_id).unwrap_or(0);
            eprintln!(
                "Deleted {} history records for backup set {}",
                deleted, backup_id
            );

            // Record a new history entry so the deletion is visible in the History page
            let timestamp = chrono::Local::now()
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string();
            let _ = db.record_operation(&crate::history::OperationRecord {
                backup_id: backup_id.into(),
                operation: "delete_backup_set".into(),
                timestamp,
                source_root,
                dest_path: job_cfg.dest.to_string_lossy().into_owned(),
                job_name,
                file_count,
                total_bytes,
                duration_ms: 0,
                exit_code: 0,
                status: "success".into(),
            });
        }
    }

    // 3. Delete the backup directory (after reading manifest and recording history)
    std::fs::remove_dir_all(&backup_dir)
        .map_err(|e| AppError::storage(format!("Failed to delete backup directory: {}", e)))?;

    Ok(())
}
// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Find the backup directory for a given backup_id by searching all job destinations.
fn find_backup_dir(backup_id: &str) -> Result<std::path::PathBuf, AppError> {
    let config = Config::load().map_err(AppError::from)?;

    for job_cfg in config.job.values() {
        let backup_dir = job_cfg.dest.join(backup_id);
        let manifest_path = backup_dir.join("manifest.json");

        if backup_dir.exists() && manifest_path.exists() {
            return Ok(backup_dir);
        }
    }

    Err(AppError::config(format!(
        "Backup point '{}' not found. The backup directory may have been moved or deleted.",
        backup_id
    )))
}

/// Find the job config that contains a specific backup directory.
fn find_job_for_backup_dir(backup_dir: &Path) -> Option<crate::config::JobConfig> {
    let config = Config::load().ok()?;
    for job_cfg in config.job.values() {
        let expected = job_cfg.dest.join(backup_dir.file_name()?);
        if expected == backup_dir {
            return Some(job_cfg.clone());
        }
    }
    None
}

/// Find the job NAME for a backup directory by searching all job destinations.
/// Returns None if the backup dir does not correspond to any configured job.
fn find_job_name_for_backup_dir(backup_dir: &Path) -> Option<String> {
    let config = Config::load().ok()?;
    for (name, job_cfg) in &config.job {
        let expected = job_cfg.dest.join(backup_dir.file_name()?);
        if expected == backup_dir {
            return Some(name.clone());
        }
    }
    None
}
