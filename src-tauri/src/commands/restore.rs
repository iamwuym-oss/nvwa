// ============================================================================
// commands/restore.rs -- Tauri commands for the Restore page
//
// This layer ONLY:
//   1. Receives parameters from the frontend invoke()
//   2. Calls the corresponding Application Service
//   3. Converts errors to AppError and returns them
//
// It MUST NOT contain any business logic.
// ============================================================================

use nuwa_backup::app::error::AppError;
use nuwa_backup::app::models::restore::{
    RestoreOperationResult, RestorePointView, RestorePreview, RestoreRequest,
};
use nuwa_backup::app::services::restore_service;

/// List all available restore points (newest first).
#[tauri::command]
pub fn list_restore_points() -> Result<Vec<RestorePointView>, AppError> {
    restore_service::list_restore_points()
}

/// Get a preview of files that would be restored from a backup point.
#[tauri::command]
pub fn get_restore_preview(backup_id: String) -> Result<RestorePreview, AppError> {
    restore_service::get_restore_preview(&backup_id)
}

/// Execute a restore operation.
#[tauri::command]
pub fn execute_restore(request: RestoreRequest) -> Result<RestoreOperationResult, AppError> {
    restore_service::execute_restore(&request)
}
