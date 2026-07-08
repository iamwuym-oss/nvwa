// ============================================================================
// commands/config.rs -- Tauri commands for the Settings / Job Config page
//
// This layer ONLY:
//   1. Receives parameters from the frontend invoke()
//   2. Calls the corresponding Application Service
//   3. Converts errors to AppError and returns them
//
// It MUST NOT contain any business logic.
// ============================================================================

use nuwa_backup::app::error::AppError;
use nuwa_backup::app::models::config_job::{JobConfigRequest, JobConfigView};
use nuwa_backup::app::services::config_service;

/// List all job configurations (sorted by name).
#[tauri::command]
pub fn list_job_configs() -> Result<Vec<JobConfigView>, AppError> {
    config_service::list_job_configs()
}

/// Get a single job configuration by name.
#[tauri::command]
pub fn get_job_config(name: String) -> Result<JobConfigView, AppError> {
    config_service::get_job_config(&name)
}

/// Create a new job configuration.
#[tauri::command]
pub fn create_job_config(request: JobConfigRequest) -> Result<JobConfigView, AppError> {
    config_service::create_job_config(&request)
}

/// Update an existing job configuration.
#[tauri::command]
pub fn update_job_config(name: String, request: JobConfigRequest) -> Result<JobConfigView, AppError> {
    config_service::update_job_config(&name, &request)
}

/// Delete a job configuration by name.
#[tauri::command]
pub fn delete_job_config(name: String) -> Result<(), AppError> {
    config_service::delete_job_config(&name)
}
