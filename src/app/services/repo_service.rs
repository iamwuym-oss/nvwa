// ============================================================================
// repo_service.rs -- Repository domain service (Application Layer)
//
// This is the single entry point for Repository management operations from
// the Tauri command layer. Phase 1 exposes 4 core methods:
//
//   create_repo  -- init a new Repository + register in registry
//   list_repos   -- list all registered Repositories with live status
//   get_repo_info -- full details for one Repository
//   verify_repo  -- integrity verification
//
// All operations identify Repositories by UUID (not path) via RepoRegistry.
// Phase S core modules (repository::*) remain FROZEN -- no changes.
// ============================================================================

use std::path::Path;
use std::time::Instant;

use crate::app::error::AppError;
use crate::app::models::repo::{
    CreateRepoRequest, RepoInfoResponse, RepoRecord, RetentionStatusSummary, VerifyOptionsRequest,
    VerifyResponse,
};
use crate::app::services::repo_registry::RepoRegistry;
use crate::repository;
use crate::repository::retention::engine::count_orphan_candidates;

// ---------------------------------------------------------------------------
// Phase 1: 4 core methods
// ---------------------------------------------------------------------------

/// Create a new Repository and register it.
///
/// Flow:
///   1. Validate the target path does not already contain a Repository
///   2. Call repository::init_repo() to create the backing store
///   3. Register the Repository in the registry
///   4. Return full info
pub fn create_repo(req: CreateRepoRequest) -> Result<RepoInfoResponse, AppError> {
    let path = Path::new(&req.path);

    // Validate path
    if req.name.trim().is_empty() {
        return Err(AppError::config("Repository name cannot be empty"));
    }
    if req.path.trim().is_empty() {
        return Err(AppError::config("Repository path cannot be empty"));
    }

    // Check for existing Repository at this path
    if repository::repo_manager::is_repository(path) {
        return Err(AppError::config(format!(
            "A Repository already exists at '{}'. Use list or open instead.",
            path.display()
        )));
    }

    // Create the backing store (Phase S core, frozen)
    let handle =
        repository::repo_manager::init_repo(path, repository::repo_manager::DEFAULT_BLOCK_SIZE)
            .map_err(|e| AppError::internal(format!("Failed to initialize Repository: {}", e)))?;

    // Build registry record
    let now = chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    let record = RepoRecord {
        id: handle.info.repository_id.clone(),
        name: req.name,
        path: req.path,
        created_at: now.clone(),
        last_opened: now,
        status: "active".into(),
    };

    // Register
    let mut registry = RepoRegistry::load();
    registry.register(record)?;

    // Build response
    Ok(build_repo_info(handle))
}

/// List all registered Repositories with live status.
///
/// Each Repository is opened to get current stats.
/// If a Repository cannot be opened, its status is set to "missing"
/// and partial info is returned (rather than failing the entire list).
pub fn list_repos() -> Result<Vec<RepoInfoResponse>, AppError> {
    let registry = RepoRegistry::load();
    let records = registry.list();

    let mut results: Vec<RepoInfoResponse> = Vec::new();

    for record in records {
        let path = Path::new(&record.path);
        if repository::repo_manager::is_repository(path) {
            match repository::repo_manager::open_repo(path) {
                Ok(handle) => {
                    let mut info = build_repo_info(handle);
                    info.id = record.id.clone();
                    info.name = record.name.clone();
                    results.push(info);
                }
                Err(_) => {
                    // Repository exists but cannot be opened -> show as error state
                    results.push(RepoInfoResponse {
                        id: record.id.clone(),
                        name: record.name.clone(),
                        path: record.path.clone(),
                        repo_uuid: record.id.clone(),
                        format_version: 0,
                        created_at: record.created_at.clone(),
                        block_size: "256 KiB (Fixed)".into(),
                        compression: "zstd".into(),
                        capabilities: vec![],
                        total_chunks: 0,
                        total_size_bytes: 0,
                        instance_count: 0,
                        retention: RetentionStatusSummary {
                            total_restore_points: 0,
                            active_restore_points: 0,
                            deleted_restore_points: 0,
                            orphan_candidates: 0,
                        },
                    });
                }
            }
        } else {
            // Path no longer exists
            results.push(RepoInfoResponse {
                id: record.id.clone(),
                name: record.name.clone(),
                path: record.path.clone(),
                repo_uuid: record.id.clone(),
                format_version: 0,
                created_at: record.created_at.clone(),
                block_size: "256 KiB (Fixed)".into(),
                compression: "zstd".into(),
                capabilities: vec![],
                total_chunks: 0,
                total_size_bytes: 0,
                instance_count: 0,
                retention: RetentionStatusSummary {
                    total_restore_points: 0,
                    active_restore_points: 0,
                    deleted_restore_points: 0,
                    orphan_candidates: 0,
                },
            });
        }
    }

    Ok(results)
}

/// Get detailed information for a single Repository by UUID.
pub fn get_repo_info(id: &str) -> Result<RepoInfoResponse, AppError> {
    let registry = RepoRegistry::load();
    let record = registry
        .get(id)
        .ok_or_else(|| AppError::config(format!("Repository '{}' is not registered", id)))?;

    let path = registry.resolve_path(id)?;

    let handle = repository::repo_manager::open_repo(&path).map_err(|e| {
        // Update status to indicate issue
        let mut reg = RepoRegistry::load();
        let _ = reg.update_status(id, "error");
        AppError::internal(format!("Failed to open Repository: {}", e))
    })?;

    let mut info = build_repo_info(handle);
    info.id = record.id.clone();
    info.name = record.name.clone();

    // Touch the registry to update last_opened
    let mut reg = RepoRegistry::load();
    let _ = reg.touch(id);

    Ok(info)
}

/// Run integrity verification on a Repository.
pub fn verify_repo(id: &str, options: VerifyOptionsRequest) -> Result<VerifyResponse, AppError> {
    let registry = RepoRegistry::load();
    let path = registry.resolve_path(id)?;

    let handle = repository::repo_manager::open_repo(&path)
        .map_err(|e| AppError::internal(format!("Failed to open Repository for verify: {}", e)))?;

    // Create BlockStore for the verify engine
    let block_store =
        repository::block_store::store::LocalFsBlockStore::new(handle.block_store_dir.clone());

    let start = Instant::now();

    // Phase S Verify Engine (frozen)
    let verify_level = if options.quick {
        repository::verify::VerifyLevel::Metadata
    } else {
        repository::verify::VerifyLevel::Full
    };

    let report = repository::verify::verify_repo(&handle, &block_store, verify_level)
        .map_err(|e| AppError::internal(format!("Verification failed: {}", e)))?;

    let duration = start.elapsed().as_millis() as u64;

    // Build errors/warnings list from report
    let mut errors: Vec<String> = Vec::new();
    let mut warnings: Vec<String> = Vec::new();

    if report.data_loss_detected {
        errors.push("Data loss detected".into());
    }
    if !report.missing_instances.is_empty() {
        warnings.push(format!(
            "{} restore points missing instance directory",
            report.missing_instances.len()
        ));
    }
    if !report.missing_metadata.is_empty() {
        warnings.push(format!(
            "{} restore points missing metadata",
            report.missing_metadata.len()
        ));
    }
    if !report.incomplete_transactions.is_empty() {
        warnings.push(format!(
            "{} incomplete transactions found",
            report.incomplete_transactions.len()
        ));
    }
    if report.failed_blocks > 0 {
        errors.push(format!(
            "{} blocks failed verification",
            report.failed_blocks
        ));
    }

    Ok(VerifyResponse {
        passed: !report.data_loss_detected && report.failed_blocks == 0,
        checked_instances: report.total_restore_points as u32,
        checked_blocks: report.verified_blocks,
        errors,
        warnings,
        duration_ms: duration,
    })
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Build a RepoInfoResponse from a Repository handle.
fn build_repo_info(handle: repository::RepoHandle) -> RepoInfoResponse {
    let capabilities = vec![
        format!(
            "Fixed Block ({})",
            bytes_to_human(handle.info.chunk_policy.block_size as u64)
        ),
        "SHA-256 Verification".into(),
        "Transaction Log".into(),
    ];

    let now = chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    // Count orphan candidates via Phase S retention engine
    let orphan_count = count_orphan_candidates(&handle).unwrap_or(0);

    RepoInfoResponse {
        id: handle.info.repository_id.clone(),
        name: String::new(), // filled by caller from registry
        path: handle.root.to_string_lossy().into_owned(),
        repo_uuid: handle.info.repository_id.clone(),
        format_version: handle.info.format_version,
        created_at: now,
        block_size: format!("{} KiB (Fixed)", handle.info.chunk_policy.block_size / 1024),
        compression: if handle.info.capabilities.compression {
            "zstd".into()
        } else {
            "none".into()
        },
        capabilities,
        total_chunks: 0, // Phase S does not expose total chunk count directly
        total_size_bytes: handle.info.total_raw_bytes,
        instance_count: handle.info.restore_point_count as u32,
        retention: RetentionStatusSummary {
            total_restore_points: handle.info.restore_point_count as u32,
            active_restore_points: handle.info.restore_point_count as u32,
            deleted_restore_points: 0,
            orphan_candidates: orphan_count,
        },
    }
}

/// Format bytes to human-readable string.
fn bytes_to_human(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_idx])
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_bytes_to_human() {
        assert_eq!(bytes_to_human(0), "0.0 B");
        assert_eq!(bytes_to_human(1024), "1.0 KiB");
        assert_eq!(bytes_to_human(1048576), "1.0 MiB");
        assert_eq!(bytes_to_human(1073741824), "1.0 GiB");
    }

    #[test]
    fn test_create_repo_direct_core() {
        // Test the core init directly (avoids global registry file conflicts)
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("test-repo");

        let handle = crate::repository::repo_manager::init_repo(
            &repo_path,
            crate::repository::repo_manager::DEFAULT_BLOCK_SIZE,
        )
        .unwrap();

        assert_eq!(handle.info.repository_id.len(), 36);
        assert_eq!(handle.info.chunk_policy.block_size, 262144);

        // Test registry in isolation
        let mut registry = RepoRegistry::load_from(&tmp.path().join("repos.json"));
        let now = chrono::Local::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();
        registry
            .register(RepoRecord {
                id: handle.info.repository_id.clone(),
                name: "Test Repo".into(),
                path: repo_path.to_string_lossy().into_owned(),
                created_at: now.clone(),
                last_opened: now,
                status: "active".into(),
            })
            .unwrap();

        let repos = registry.list();
        assert_eq!(repos.len(), 1);
    }

    #[test]
    fn test_is_repository_detection() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("test-repo");

        assert!(!crate::repository::repo_manager::is_repository(&repo_path));

        crate::repository::repo_manager::init_repo(
            &repo_path,
            crate::repository::repo_manager::DEFAULT_BLOCK_SIZE,
        )
        .unwrap();

        assert!(crate::repository::repo_manager::is_repository(&repo_path));
    }

    #[test]
    fn test_build_repo_info_from_handle() {
        let tmp = TempDir::new().unwrap();
        let repo_path = tmp.path().join("test-repo");

        let handle = crate::repository::repo_manager::init_repo(
            &repo_path,
            crate::repository::repo_manager::DEFAULT_BLOCK_SIZE,
        )
        .unwrap();

        let info = super::build_repo_info(handle);
        assert_eq!(info.repo_uuid.len(), 36);
        assert!(info.block_size.contains("256 KiB"));
        assert!(info.compression.contains("zstd"));
    }

    #[test]
    fn test_verify_repo_passes() {
        // Use direct repository + registry test to avoid global registry conflicts
        let tmp = TempDir::new().unwrap();
        let reg_path = tmp.path().join("repos.json");
        let repo_path = tmp.path().join("test-repo");

        // Init repo via core engine
        let handle = crate::repository::repo_manager::init_repo(
            &repo_path,
            crate::repository::repo_manager::DEFAULT_BLOCK_SIZE,
        )
        .unwrap();

        // Register in isolated registry
        let mut registry = RepoRegistry::load_from(&reg_path);
        let now = chrono::Local::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();
        registry
            .register(RepoRecord {
                id: handle.info.repository_id.clone(),
                name: "Verify Test".into(),
                path: repo_path.to_string_lossy().into_owned(),
                created_at: now.clone(),
                last_opened: now,
                status: "active".into(),
            })
            .unwrap();

        // Build BlockStore
        let block_store = crate::repository::block_store::store::LocalFsBlockStore::new(
            handle.block_store_dir.clone(),
        );

        // Verify (metadata level is enough for empty repo)
        let report = crate::repository::verify::verify_repo(
            &handle,
            &block_store,
            crate::repository::verify::VerifyLevel::Metadata,
        )
        .unwrap();
        assert!(!report.data_loss_detected);
    }

    #[test]
    fn test_get_nonexistent_repo_fails() {
        let result = get_repo_info("00000000-0000-0000-0000-000000000000");
        assert!(result.is_err());
    }
}
