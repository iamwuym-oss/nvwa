// ============================================================================
// commands/history.rs -- Tauri commands for the History page
//
// This layer ONLY:
//   1. Receives parameters from the frontend invoke()
//   2. Calls the corresponding Application Service
//   3. Converts errors to AppError and returns them
//
// It MUST NOT contain any business logic.
// ============================================================================

use nuwa_backup::app::error::AppError;
use nuwa_backup::app::models::history::{HistoryFilter, HistoryQueryResult};
use nuwa_backup::app::services::history_service;

/// Query operation history with optional filters.
#[tauri::command]
pub fn query_history(filter: HistoryFilter) -> Result<HistoryQueryResult, AppError> {
    history_service::query_history(filter)
}

/// List available operation types for the frontend filter UI.
#[tauri::command]
pub fn list_history_operation_types() -> Vec<String> {
    history_service::list_operation_types()
}
