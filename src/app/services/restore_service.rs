// ============================================================================
// restore_service.rs -- Restore domain service (storage engine redesign)
//
// The restore service is temporarily unavailable as the storage engine
// is being redesigned to use .nwb single-file format.
// ============================================================================

use crate::app::error::AppError;
use crate::app::models::restore::{
    RestoreOperationResult, RestorePointView, RestorePreview, RestoreRequest,
};

pub fn list_restore_points() -> Result<Vec<RestorePointView>, AppError> {
    Ok(Vec::new())
}

pub fn get_restore_preview(_backup_id: &str) -> Result<RestorePreview, AppError> {
    Err(AppError::config(
        "Restore is temporarily unavailable during storage engine redesign.",
    ))
}

pub fn execute_restore(_request: RestoreRequest) -> Result<RestoreOperationResult, AppError> {
    Err(AppError::internal(
        "Restore is temporarily unavailable during storage engine redesign. \
         The new .nwb single-file engine is being built.",
    ))
}

pub fn delete_backup_set(_backup_id: &str) -> Result<(), AppError> {
    Err(AppError::config(
        "Delete is temporarily unavailable during storage engine redesign.",
    ))
}
