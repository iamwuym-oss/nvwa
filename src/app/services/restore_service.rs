// ============================================================================
// restore_service.rs -- Restore domain service
//
// Responsibilities:
//   - List available restore points (from ALL storage backends)
//   - Preview files in a backup point
//   - Execute restore from a backup point to a destination
//   - Delete backup sets
//
// This service is the single entry point for all restore operations.
// It delegates to RestoreProvider implementations for each storage backend.
// P-07: Repository-only. All restore operations use Repository Engine.
// ============================================================================

use std::time::Instant;

use crate::app::error::AppError;
use crate::app::models::restore::{
    RestoreOperationResult, RestorePointView, RestorePreview, RestoreRequest,
};
use crate::app::services::restore_provider::{RepositoryRestoreProvider, RestoreProvider};
use crate::history::{HistoryDb, OperationRecord};

// ---------------------------------------------------------------------------
// Providers
// ---------------------------------------------------------------------------

/// All registered restore providers (Repository Engine only).
fn all_providers() -> Vec<Box<dyn RestoreProvider>> {
    vec![Box::new(RepositoryRestoreProvider)]
}

/// Find the provider that can handle a specific backup_id.
fn provider_for_backup(backup_id: &str) -> Result<Box<dyn RestoreProvider>, AppError> {
    for provider in all_providers() {
        if provider.get_preview(backup_id).is_ok() {
            return Ok(provider);
        }
    }
    Err(AppError::config(format!(
        "Backup point '{}' not found in any storage backend.",
        backup_id
    )))
}

// ---------------------------------------------------------------------------
// Public API (unchanged contract)
// ---------------------------------------------------------------------------

/// List all available restore points across all configured jobs.
///
/// Returns restore points sorted by timestamp (newest first).
/// Returns an empty Vec if no restore points are found.
pub fn list_restore_points() -> Result<Vec<RestorePointView>, AppError> {
    let mut all_points: Vec<RestorePointView> = Vec::new();

    for provider in all_providers() {
        match provider.list_restore_points() {
            Ok(points) => all_points.extend(points),
            Err(_) => continue, // Skip providers that fail; don't block the list
        }
    }

    // Sort newest first
    all_points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(all_points)
}

/// Get a preview of files that would be restored from a backup point.
/// Searches across all storage backends.
pub fn get_restore_preview(backup_id: &str) -> Result<RestorePreview, AppError> {
    let provider = provider_for_backup(backup_id)?;
    provider.get_preview(backup_id)
}

/// Execute a restore operation.
///
/// Flow:
///   1. Find the provider that owns this backup_id
///   2. Call provider's execute_restore()
///   3. Return RestoreOperationResult
pub fn execute_restore(request: RestoreRequest) -> Result<RestoreOperationResult, AppError> {
    // Validate input before looking up providers
    if request.backup_id.trim().is_empty() {
        return Err(AppError::config("Backup ID must not be empty"));
    }
    if request.dest.trim().is_empty() {
        return Err(AppError::config("Destination path must not be empty"));
    }

    let start = Instant::now();

    let provider = provider_for_backup(&request.backup_id)?;
    let mut result = provider.execute_restore(&request)?;

    // Record the restore in history
    let timestamp = chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();
    result.duration_ms = start.elapsed().as_millis() as u64;
    result.timestamp = timestamp.clone();

    // Try to record history (non-fatal on failure)
    if let Ok(config) = crate::config::Config::load() {
        for job_cfg in config.job.values() {
            let db_path = HistoryDb::history_db_path(&job_cfg.dest);
            if let Ok(db) = HistoryDb::open_or_create(&db_path) {
                let status = if result.checksum_failures == 0 {
                    "success"
                } else {
                    "partial"
                };
                let _ = db.record_operation(&OperationRecord {
                    backup_id: request.backup_id.clone(),
                    operation: "restore".into(),
                    timestamp: timestamp.clone(),
                    source_root: String::new(),
                    dest_path: request.dest.clone(),
                    job_name: None,
                    file_count: result.restored_count,
                    total_bytes: 0,
                    duration_ms: result.duration_ms,
                    exit_code: if result.checksum_failures == 0 { 0 } else { 1 },
                    status: status.into(),
                });
            }
        }
    }

    Ok(result)
}

/// Delete a backup set by backup_id.
///
/// Finds the provider that owns this backup_id and delegates deletion.
/// Repository: removes instance directory + history records (does NOT run GC).
pub fn delete_backup_set(backup_id: &str) -> Result<(), AppError> {
    let provider = provider_for_backup(backup_id)?;
    provider.delete_backup_set(backup_id)
}
