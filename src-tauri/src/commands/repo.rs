// ============================================================================
// commands/repo.rs -- Tauri commands for Repository management
//
// 4 commands in Phase 1:
//   create_repo  -> create + register
//   list_repos   -> list all
//   get_repo_info -> detail
//   verify_repo  -> integrity verify
//
// These are thin wrappers. All business logic lives in RepoService.
// ============================================================================

use nuwa_backup::app::models::repo::{
    CreateRepoRequest, RepoInfoResponse, VerifyOptionsRequest, VerifyResponse,
};
use nuwa_backup::app::services::repo_service;

// ---------------------------------------------------------------------------
// Phase 1: 4 core commands
// ---------------------------------------------------------------------------

/// Create a new Repository at the given path and register it.
#[tauri::command]
pub fn create_repo(name: String, path: String) -> Result<RepoInfoResponse, String> {
    let req = CreateRepoRequest { name, path };
    repo_service::create_repo(req).map_err(|e| e.message)
}

/// List all registered Repositories with live status.
#[tauri::command]
pub fn list_repos() -> Result<Vec<RepoInfoResponse>, String> {
    repo_service::list_repos().map_err(|e| e.message)
}

/// Get detailed information for a single Repository.
#[tauri::command]
pub fn get_repo_info(id: String) -> Result<RepoInfoResponse, String> {
    repo_service::get_repo_info(&id).map_err(|e| e.message)
}

/// Run integrity verification on a Repository.
#[tauri::command]
pub fn verify_repo(id: String, quick: bool) -> Result<VerifyResponse, String> {
    let options = VerifyOptionsRequest { quick };
    repo_service::verify_repo(&id, options).map_err(|e| e.message)
}
