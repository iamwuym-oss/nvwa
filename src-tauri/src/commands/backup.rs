// ============================================================================
// commands/backup.rs -- Tauri commands for the Backup page
//
// This layer ONLY:
//   1. Receives parameters from the frontend invoke()
//   2. Calls the corresponding Application Service
//   3. Converts errors to AppError and returns them
//
// It MUST NOT contain any business logic.
// ============================================================================

use nuwa_backup::app::error::AppError;
use nuwa_backup::app::models::backup::{BackupJobView, BackupResult};
use nuwa_backup::app::services::backup_service;

/// List all configured backup jobs.
#[tauri::command]
pub fn list_backup_jobs() -> Result<Vec<BackupJobView>, AppError> {
    backup_service::list_jobs()
}

/// Get details for a single backup job.
#[tauri::command]
pub fn get_backup_job_detail(name: String) -> Result<BackupJobView, AppError> {
    backup_service::get_job_detail(&name)
}

/// Execute a backup for the specified job.
#[tauri::command]
pub fn run_backup(job_name: String) -> Result<BackupResult, AppError> {
    backup_service::run_backup(&job_name)
}
