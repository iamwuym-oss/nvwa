// ============================================================================
// commands/dashboard.rs -- Tauri commands for the Dashboard page
//
// This layer ONLY:
//   1. Receives parameters from the frontend invoke()
//   2. Calls the corresponding Application Service
//   3. Converts errors to AppError and returns them
//
// It MUST NOT contain any business logic.
// ============================================================================

use nuwa_backup::app::models::dashboard::DashboardOverview;
use nuwa_backup::app::error::AppError;
use nuwa_backup::app::services::dashboard_service;

/// Return the full Dashboard overview (protection status, jobs, storage, activity).
///
/// This is the single Tauri command that powers the Dashboard page.
/// It calls dashboard_service::get_overview() which aggregates data from
/// config, history, scheduler, and diskspace.
#[tauri::command]
pub fn get_dashboard_overview() -> Result<DashboardOverview, AppError> {
    dashboard_service::get_overview()
}
