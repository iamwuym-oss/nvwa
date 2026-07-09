// ============================================================================
// schedule.rs -- Tauri commands for Schedule page
//
// This is a thin IPC bridge. All business logic lives in the Application
// Service Layer (schedule_service.rs), never in this command file.
// ============================================================================

use crate::app::error::AppError;
use crate::app::models::schedule::{
    ScheduleDeleteResult, ScheduleProfileRequest, ScheduleProfileView,
};
use crate::app::services::schedule_service;
use tauri::command;

/// List all schedule profiles.
#[command]
pub fn list_schedules() -> Result<Vec<ScheduleProfileView>, AppError> {
    schedule_service::list_schedules()
}

/// Get a single schedule profile by ID.
#[command]
pub fn get_schedule(id: String) -> Result<ScheduleProfileView, AppError> {
    schedule_service::get_schedule(&id)
}

/// Create a new schedule profile.
#[command]
pub fn create_schedule(request: ScheduleProfileRequest) -> Result<ScheduleProfileView, AppError> {
    schedule_service::create_schedule(&request)
}

/// Update an existing schedule profile.
#[command]
pub fn update_schedule(id: String, request: ScheduleProfileRequest) -> Result<ScheduleProfileView, AppError> {
    let view = schedule_service::update_schedule(&id, &request)?;
    // Sync schtasks for referencing jobs when trigger changes.
    // The service layer handles this, but we also log any errors here.
    if let Err(e) = schedule_service::sync_schedule_trigger_update(&id) {
        eprintln!("Warning: schtasks update sync failed for schedule '{}': {}", id, e);
    }
    Ok(view)
}

/// Delete a schedule profile.
///
/// Safety: if any backup jobs reference this schedule, deletion is BLOCKED
/// and an error is returned. The user must first remove the schedule reference
/// from those jobs in Settings.
#[command]
pub fn delete_schedule(id: String, confirmed: bool) -> Result<ScheduleDeleteResult, AppError> {
    schedule_service::delete_schedule(&id, confirmed)
}

/// Enable a schedule profile and create Windows scheduled tasks for associated jobs.
#[command]
pub fn enable_schedule(id: String) -> Result<ScheduleProfileView, AppError> {
    let view = schedule_service::enable_schedule(&id)?;
    // Sync schtasks for associated jobs.
    // Errors are NOT silently swallowed ? but sync failure doesn't undo the config change.
    // The frontend can check task_sync_status on the ScheduleProfileView.
    if let Err(e) = schedule_service::sync_create_tasks(&id) {
        eprintln!("Warning: schtasks sync failed after enabling schedule '{}': {}", id, e);
    }
    Ok(view)
}

/// Disable a schedule profile and remove Windows scheduled tasks for associated jobs.
#[command]
pub fn disable_schedule(id: String) -> Result<ScheduleProfileView, AppError> {
    let view = schedule_service::disable_schedule(&id)?;
    // Remove schtasks for associated jobs.
    if let Err(e) = schedule_service::sync_remove_tasks(&id) {
        eprintln!("Warning: schtasks removal failed after disabling schedule '{}': {}", id, e);
    }
    Ok(view)
}
