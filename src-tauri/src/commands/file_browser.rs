// ============================================================================
// commands/file_browser.rs -- Tauri commands for in-app file browser
//
// Thin wrapper: receives params, calls file_browser_service, returns result.
// No business logic.
// ============================================================================

use nuwa_backup::app::error::AppError;
use nuwa_backup::app::services::file_browser_service;

#[tauri::command]
pub fn list_roots() -> Result<Vec<file_browser_service::RootEntry>, AppError> {
    file_browser_service::list_roots()
}

#[tauri::command]
pub fn list_directory(path: String) -> Result<Vec<file_browser_service::FsEntry>, AppError> {
    file_browser_service::list_directory(&path)
}
