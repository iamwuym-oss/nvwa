// ============================================================================
// restore_provider.rs -- RestoreProvider trait and implementations
//
// Abstraction layer that unifies restore operations from different storage
// backends. Each provider is responsible for listing/preview/restore/delete
// for its storage type.
//
// Current providers:
//   FlatFileRestoreProvider  -- Phase 1/2 flat-file + manifest.json (extracted from restore_service.rs)
//   RepositoryRestoreProvider -- Phase S Repository Engine (read-only, uses existing public APIs)
//
// Design:
//   - Providers are self-contained: each knows how to discover its restore points
//   - restore_service.rs calls ALL providers and merges results
//   - No Phase S frozen core is modified
//   - No repository::recovery is called
// ============================================================================

use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::app::error::AppError;
use crate::app::models::restore::{
    RestoreFileEntry, RestoreOperationResult, RestorePointView, RestorePreview, RestoreRequest,
};
use crate::app::services::repo_registry::RepoRegistry;
use crate::config::{Config, JobConfig};
use crate::history::{HistoryDb, OperationRecord};
use crate::repository::CatalogEngine;

// ---------------------------------------------------------------------------
// RestoreProvider trait
// ---------------------------------------------------------------------------

pub trait RestoreProvider {
    /// List all restore points this provider can handle.
    fn list_restore_points(&self) -> Result<Vec<RestorePointView>, AppError>;

    /// Preview files in a specific restore point.
    fn get_preview(&self, backup_id: &str) -> Result<RestorePreview, AppError>;

    /// Execute a restore from a restore point to a destination.
    fn execute_restore(&self, request: &RestoreRequest)
        -> Result<RestoreOperationResult, AppError>;

    /// Delete a backup set (remove from view, preserve data when appropriate).
    fn delete_backup_set(&self, backup_id: &str) -> Result<(), AppError>;
}

// ---------------------------------------------------------------------------
// FlatFileRestoreProvider -- Phase 1 & 2 flat-file storage
// ---------------------------------------------------------------------------

pub struct FlatFileRestoreProvider;

impl FlatFileRestoreProvider {
    /// Find the backup directory for a given backup_id.
    fn find_backup_dir(backup_id: &str) -> Result<PathBuf, AppError> {
        let config = Config::load().map_err(AppError::from)?;
        for job_cfg in config.job.values() {
            let backup_dir = job_cfg.dest.join(backup_id);
            let manifest_path = backup_dir.join("manifest.json");
            if backup_dir.exists() && manifest_path.exists() {
                return Ok(backup_dir);
            }
        }
        Err(AppError::config(format!(
            "Backup point '{}' not found in flat-file storage.",
            backup_id
        )))
    }

    /// Find the job config containing a backup directory.
    fn find_job_for_backup_dir(backup_dir: &Path) -> Option<JobConfig> {
        let config = Config::load().ok()?;
        for job_cfg in config.job.values() {
            let expected = job_cfg.dest.join(backup_dir.file_name()?);
            if expected == backup_dir {
                return Some(job_cfg.clone());
            }
        }
        None
    }

    /// Find the job name for a backup directory.
    fn find_job_name_for_backup_dir(backup_dir: &Path) -> Option<String> {
        let config = Config::load().ok()?;
        for (name, job_cfg) in &config.job {
            let expected = job_cfg.dest.join(backup_dir.file_name()?);
            if expected == backup_dir {
                return Some(name.clone());
            }
        }
        None
    }
}

impl RestoreProvider for FlatFileRestoreProvider {
    fn list_restore_points(&self) -> Result<Vec<RestorePointView>, AppError> {
        let config = match Config::load() {
            Ok(c) => c,
            Err(_) => return Ok(Vec::new()),
        };

        let mut points: Vec<RestorePointView> = Vec::new();

        for (job_name, job_cfg) in &config.job {
            // Only include flat-file jobs
            let storage_type = job_cfg.storage_type.as_deref().unwrap_or("flat-file");
            if storage_type != "flat-file" {
                continue;
            }

            let db_path = HistoryDb::history_db_path(&job_cfg.dest);
            let db = match HistoryDb::open_or_create(&db_path) {
                Ok(db) => db,
                Err(_) => continue,
            };

            let records = match db.query_history(100, Some("backup")) {
                Ok(r) => r,
                Err(_) => continue,
            };

            for record in &records {
                let backup_dir = job_cfg.dest.join(&record.backup_id);
                if !backup_dir.exists() {
                    continue;
                }

                points.push(RestorePointView {
                    backup_id: record.backup_id.clone(),
                    job_name: record.job_name.clone().or(Some(job_name.clone())),
                    timestamp: record.timestamp.clone(),
                    source_root: record.source_root.clone(),
                    dest_path: job_cfg.dest.to_string_lossy().into_owned(),
                    file_count: record.file_count,
                    total_bytes: record.total_bytes,
                    status: record.status.clone(),
                });
            }
        }
        Ok(points)
    }

    fn get_preview(&self, backup_id: &str) -> Result<RestorePreview, AppError> {
        let config = Config::load().map_err(AppError::from)?;
        for job_cfg in config.job.values() {
            let backup_dir = job_cfg.dest.join(backup_id);
            let manifest_path = backup_dir.join("manifest.json");
            if !backup_dir.exists() || !manifest_path.exists() {
                continue;
            }

            let manifest = crate::storage::read_manifest(&backup_dir)
                .map_err(|e| AppError::internal(format!("Failed to read manifest: {}", e)))?;

            let mut files: Vec<RestoreFileEntry> = Vec::new();
            let mut total_files: u64 = 0;
            let mut total_bytes: u64 = 0;

            for file_entry in &manifest.files {
                files.push(RestoreFileEntry {
                    relative_path: file_entry.relative_path.clone(),
                    size_bytes: file_entry.size_bytes,
                    modified_time: file_entry.modified_time.clone(),
                });
                total_files += 1;
                total_bytes += file_entry.size_bytes;
            }

            let point = RestorePointView {
                backup_id: backup_id.to_string(),
                job_name: None,
                timestamp: manifest.created_at.clone(),
                source_root: manifest.source_root.clone(),
                dest_path: job_cfg.dest.to_string_lossy().into_owned(),
                file_count: total_files,
                total_bytes,
                status: "success".into(),
            };

            return Ok(RestorePreview {
                point,
                files,
                total_files,
                total_bytes,
            });
        }

        Err(AppError::config(format!(
            "Backup point '{}' not found in flat-file storage.",
            backup_id
        )))
    }

    fn execute_restore(
        &self,
        request: &RestoreRequest,
    ) -> Result<RestoreOperationResult, AppError> {
        if request.backup_id.trim().is_empty() {
            return Err(AppError::config("Backup ID must not be empty"));
        }
        if request.dest.trim().is_empty() {
            return Err(AppError::config("Restore destination must not be empty"));
        }

        let backup_dir = Self::find_backup_dir(&request.backup_id)?;
        let dest_path = Path::new(&request.dest);
        let start = Instant::now();

        let core_result =
            crate::restore::execute_restore(&backup_dir, dest_path, request.overwrite)
                .map_err(|e| AppError::internal(format!("Restore failed: {}", e)))?;

        let duration = start.elapsed().as_millis() as u64;

        // Record history
        let job_cfg = Self::find_job_for_backup_dir(&backup_dir);
        let job_name = Self::find_job_name_for_backup_dir(&backup_dir);
        let timestamp = chrono::Local::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string();

        if let Some(ref cfg) = job_cfg {
            let db_path = HistoryDb::history_db_path(&cfg.dest);
            if let Ok(db) = HistoryDb::open_or_create(&db_path) {
                let status = if core_result.checksum_failures == 0 {
                    "success"
                } else {
                    "partial"
                };
                let _ = db.record_operation(&OperationRecord {
                    backup_id: request.backup_id.clone(),
                    operation: "restore".into(),
                    timestamp: timestamp.clone(),
                    source_root: String::new(),
                    dest_path: request.dest.clone(),
                    job_name,
                    file_count: core_result.restored_count,
                    total_bytes: 0,
                    duration_ms: duration,
                    exit_code: if core_result.checksum_failures == 0 {
                        0
                    } else {
                        1
                    },
                    status: status.into(),
                });
            }
        }

        Ok(RestoreOperationResult {
            restore_id: format!("restore-{}", &request.backup_id),
            restored_count: core_result.restored_count,
            skipped_count: core_result.skipped_count,
            checksum_failures: core_result.checksum_failures,
            timestamp,
            duration_ms: duration,
            status: if core_result.checksum_failures == 0 {
                "success".into()
            } else {
                "partial".into()
            },
            error: if core_result.checksum_failures > 0 {
                Some(format!(
                    "{} files have checksum mismatches",
                    core_result.checksum_failures
                ))
            } else {
                None
            },
        })
    }

    fn delete_backup_set(&self, backup_id: &str) -> Result<(), AppError> {
        if backup_id.trim().is_empty() {
            return Err(AppError::config("Backup point ID must not be empty"));
        }

        let backup_dir = Self::find_backup_dir(backup_id)?;
        let manifest_path = backup_dir.join("manifest.json");

        // Read manifest metadata BEFORE deletion
        let (source_root, file_count, total_bytes) = if manifest_path.exists() {
            match crate::manifest::Manifest::from_file(&manifest_path) {
                Ok(m) => (
                    m.source_root.clone(),
                    m.summary.file_count,
                    m.summary.total_bytes,
                ),
                Err(_) => (String::new(), 0, 0),
            }
        } else {
            (String::new(), 0, 0)
        };

        let job_name = Self::find_job_name_for_backup_dir(&backup_dir);

        // Delete history records
        if let Some(job_cfg) = Self::find_job_for_backup_dir(&backup_dir) {
            let db_path = HistoryDb::history_db_path(&job_cfg.dest);
            if let Ok(db) = HistoryDb::open_or_create(&db_path) {
                let _ = db.delete_operation_by_backup_id(backup_id);
                let timestamp = chrono::Local::now()
                    .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                    .to_string();
                let _ = db.record_operation(&OperationRecord {
                    backup_id: backup_id.into(),
                    operation: "delete_backup_set".into(),
                    timestamp,
                    source_root,
                    dest_path: job_cfg.dest.to_string_lossy().into_owned(),
                    job_name,
                    file_count,
                    total_bytes,
                    duration_ms: 0,
                    exit_code: 0,
                    status: "success".into(),
                });
            }
        }

        // Delete the backup directory
        std::fs::remove_dir_all(&backup_dir)
            .map_err(|e| AppError::storage(format!("Failed to delete backup directory: {}", e)))?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// RepositoryRestoreProvider -- Phase S Repository Engine (read-only)
// ---------------------------------------------------------------------------

#[cfg(feature = "repository")]
pub struct RepositoryRestoreProvider;

#[cfg(feature = "repository")]
impl RepositoryRestoreProvider {
    /// Collect all restore points from repository-type jobs.
    /// Queries HistoryDb for entries matching repository job configs.
    fn collect_repo_restore_points() -> Result<Vec<RestorePointView>, AppError> {
        let config = match Config::load() {
            Ok(c) => c,
            Err(_) => return Ok(Vec::new()),
        };

        let mut points: Vec<RestorePointView> = Vec::new();

        for (job_name, job_cfg) in &config.job {
            let storage_type = job_cfg.storage_type.as_deref().unwrap_or("flat-file");
            if storage_type != "repository" {
                continue;
            }

            let repo_id = match &job_cfg.repository_id {
                Some(id) => id,
                None => continue,
            };

            // Resolve repository path
            let registry = RepoRegistry::load();
            let repo_path = match registry.resolve_path(repo_id) {
                Ok(p) => p,
                Err(_) => continue,
            };

            if !crate::repository::repo_manager::is_repository(&repo_path) {
                continue;
            }

            // Read backup instances from the repository instances directory
            let handle = match crate::repository::repo_manager::open_repo(&repo_path) {
                Ok(h) => h,
                Err(_) => continue,
            };

            // Iterate over instances directory
            let instances_dir = &handle.instances_dir;
            if !instances_dir.exists() {
                continue;
            }

            let entries = match std::fs::read_dir(instances_dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let instance_path = entry.path();
                if !instance_path.is_dir() {
                    continue;
                }

                // Read backup-metadata.json
                let meta_path = instance_path.join("backup-metadata.json");
                if !meta_path.exists() {
                    continue;
                }

                match crate::repository::metadata::metadata_store::read_metadata(&instance_path) {
                    Ok(meta) => {
                        let instance_id = instance_path
                            .file_name()
                            .map(|n| n.to_string_lossy().into_owned())
                            .unwrap_or_default();

                        points.push(RestorePointView {
                            backup_id: instance_id,
                            job_name: Some(job_name.clone()),
                            timestamp: meta.created_at.clone(),
                            source_root: meta.source_description.clone(),
                            dest_path: repo_path.to_string_lossy().into_owned(),
                            file_count: 0,  // filled lazily in preview
                            total_bytes: 0, // filled lazily
                            status: meta.status.clone(),
                        });
                    }
                    Err(_) => continue,
                }
            }
        }
        Ok(points)
    }

    /// Open a Repository instance's catalog.db and read its file list.
    fn read_instance_catalog(instance_dir: &Path) -> Result<Vec<RestoreFileEntry>, AppError> {
        let catalog_path = instance_dir.join("catalog.db");
        if !catalog_path.exists() {
            return Err(AppError::config(format!(
                "Catalog not found for instance: {}",
                instance_dir.display()
            )));
        }

        let catalog = crate::repository::catalog::sqlite_catalog::SqliteCatalog::open(catalog_path)
            .map_err(|e| AppError::internal(format!("Failed to open catalog: {}", e)))?;

        let file_paths = catalog
            .list_files()
            .map_err(|e| AppError::internal(format!("Failed to list catalog files: {}", e)))?;

        let mut files: Vec<RestoreFileEntry> = Vec::new();

        for path in file_paths {
            if let Ok(Some(entry)) = catalog.get_file(&path) {
                files.push(RestoreFileEntry {
                    relative_path: entry.path.clone(),
                    size_bytes: entry.size,
                    modified_time: entry.modified.clone(),
                });
            }
        }

        Ok(files)
    }
}

#[cfg(feature = "repository")]
impl RestoreProvider for RepositoryRestoreProvider {
    fn list_restore_points(&self) -> Result<Vec<RestorePointView>, AppError> {
        Self::collect_repo_restore_points()
    }

    fn get_preview(&self, backup_id: &str) -> Result<RestorePreview, AppError> {
        // Find the instance directory by searching all repository jobs
        let config = Config::load().map_err(AppError::from)?;

        for job_cfg in config.job.values() {
            let storage_type = job_cfg.storage_type.as_deref().unwrap_or("flat-file");
            if storage_type != "repository" {
                continue;
            }

            let repo_id = match &job_cfg.repository_id {
                Some(id) => id,
                None => continue,
            };

            let registry = RepoRegistry::load();
            let repo_path = match registry.resolve_path(repo_id) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let handle = match crate::repository::repo_manager::open_repo(&repo_path) {
                Ok(h) => h,
                Err(_) => continue,
            };

            let instance_dir = handle.instances_dir.join(backup_id);
            let meta_path = instance_dir.join("backup-metadata.json");
            if !instance_dir.exists() || !meta_path.exists() {
                continue;
            }

            let meta =
                match crate::repository::metadata::metadata_store::read_metadata(&instance_dir) {
                    Ok(m) => m,
                    Err(_) => continue,
                };

            let files = Self::read_instance_catalog(&instance_dir)?;

            let total_files = files.len() as u64;
            let total_bytes: u64 = files.iter().map(|f| f.size_bytes).sum();

            let point = RestorePointView {
                backup_id: backup_id.to_string(),
                job_name: None,
                timestamp: meta.created_at,
                source_root: meta.source_description,
                dest_path: repo_path.to_string_lossy().into_owned(),
                file_count: total_files,
                total_bytes,
                status: "success".into(),
            };

            return Ok(RestorePreview {
                point,
                files,
                total_files,
                total_bytes,
            });
        }

        Err(AppError::config(format!(
            "Backup point '{}' not found in any Repository.",
            backup_id
        )))
    }

    fn execute_restore(
        &self,
        _request: &RestoreRequest,
    ) -> Result<RestoreOperationResult, AppError> {
        // Repository restore is NOT implemented in Phase 2.5.
        // Phase S provides block-level storage. File-level restore from
        // Repository requires assembling blocks into files via Catalog + BlockMap + BlockStore.
        // This will be implemented in a future phase.
        Err(AppError::config(
            "Restore from Repository is not yet supported in this version. \
             Please use flat-file backup for file-level restore, \
             or check back in a future release.",
        ))
    }

    fn delete_backup_set(&self, backup_id: &str) -> Result<(), AppError> {
        if backup_id.trim().is_empty() {
            return Err(AppError::config("Backup point ID must not be empty"));
        }

        // Find the instance in any repository job config
        let config = Config::load().map_err(AppError::from)?;
        let mut found = false;

        for job_cfg in config.job.values() {
            let storage_type = job_cfg.storage_type.as_deref().unwrap_or("flat-file");
            if storage_type != "repository" {
                continue;
            }

            let repo_id = match &job_cfg.repository_id {
                Some(id) => id,
                None => continue,
            };

            let registry = RepoRegistry::load();
            let repo_path = match registry.resolve_path(repo_id) {
                Ok(p) => p,
                Err(_) => continue,
            };

            let instance_dir = repo_path.join("backup-instances").join(backup_id);
            let meta_path = instance_dir.join("backup-metadata.json");

            if !instance_dir.exists() || !meta_path.exists() {
                continue;
            }

            // Delete history records for this backup_id
            if job_cfg.dest.exists() {
                let db_path = HistoryDb::history_db_path(&job_cfg.dest);
                if let Ok(db) = HistoryDb::open_or_create(&db_path) {
                    let _ = db.delete_operation_by_backup_id(backup_id);
                    let timestamp = chrono::Local::now()
                        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                        .to_string();
                    let _ = db.record_operation(&OperationRecord {
                        backup_id: backup_id.into(),
                        operation: "delete_backup_set".into(),
                        timestamp,
                        source_root: String::new(),
                        dest_path: repo_path.to_string_lossy().into_owned(),
                        job_name: Some(job_cfg.source.to_string_lossy().into_owned()),
                        file_count: 0,
                        total_bytes: 0,
                        duration_ms: 0,
                        exit_code: 0,
                        status: "success".into(),
                    });
                }
            }

            // Delete the instance directory
            std::fs::remove_dir_all(&instance_dir).map_err(|e| {
                AppError::storage(format!("Failed to delete backup instance: {}", e))
            })?;

            found = true;
            break;
        }

        if !found {
            return Err(AppError::config(format!(
                "Backup point '{}' not found in any Repository instance.",
                backup_id
            )));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Composite provider that tries all registered providers
// ---------------------------------------------------------------------------

/// Run a backup_id lookup across all providers until one handles it.
#[allow(dead_code)]
fn find_provider_for_backup(backup_id: &str) -> Result<Box<dyn RestoreProvider>, AppError> {
    // Try FlatFile first (most common, fastest)
    let flat = FlatFileRestoreProvider;
    if flat.get_preview(backup_id).is_ok() {
        return Ok(Box::new(FlatFileRestoreProvider));
    }

    // Try Repository
    let repo = RepositoryRestoreProvider;
    if repo.get_preview(backup_id).is_ok() {
        return Ok(Box::new(RepositoryRestoreProvider));
    }

    Err(AppError::config(format!(
        "Backup point '{}' not found in any storage backend.",
        backup_id
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- FlatFile Provider tests ----

    #[test]
    fn test_flat_file_list_empty_when_no_config() {
        // Without a valid config, list should return empty (not error)
        let provider = FlatFileRestoreProvider;
        let result = provider.list_restore_points();
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_flat_file_preview_not_found() {
        let provider = FlatFileRestoreProvider;
        let result = provider.get_preview("nonexistent-backup-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_flat_file_delete_empty_id() {
        let provider = FlatFileRestoreProvider;
        let result = provider.delete_backup_set("");
        assert!(result.is_err());
    }

    #[test]
    fn test_flat_file_execute_empty_backup_id() {
        let provider = FlatFileRestoreProvider;
        let req = RestoreRequest {
            backup_id: "".into(),
            dest: "/tmp/restore".into(),
            overwrite: false,
        };
        let result = provider.execute_restore(&req);
        assert!(result.is_err());
    }

    #[test]
    fn test_flat_file_execute_empty_dest() {
        let provider = FlatFileRestoreProvider;
        let req = RestoreRequest {
            backup_id: "some-id".into(),
            dest: "".into(),
            overwrite: false,
        };
        let result = provider.execute_restore(&req);
        assert!(result.is_err());
    }

    // ---- Repository Provider tests ----
    #[cfg(feature = "repository")]
    #[test]
    fn test_repo_restore_not_yet_supported() {
        let provider = RepositoryRestoreProvider;
        let req = RestoreRequest {
            backup_id: "any-id".into(),
            dest: "/tmp/restore".into(),
            overwrite: false,
        };
        let result = provider.execute_restore(&req);
        assert!(result.is_err());
        // Message should explain it's not supported yet
        let err = result.unwrap_err();
        assert!(err.message.contains("not yet supported"));
    }

    #[test]
    fn test_repo_list_empty_when_no_config() {
        let provider = RepositoryRestoreProvider;
        let result = provider.list_restore_points();
        assert!(result.is_ok());
    }

    #[test]
    fn test_repo_preview_not_found() {
        let provider = RepositoryRestoreProvider;
        let result = provider.get_preview("nonexistent-backup-id");
        assert!(result.is_err());
    }

    #[test]
    fn test_repo_delete_empty_id() {
        let provider = RepositoryRestoreProvider;
        let result = provider.delete_backup_set("");
        assert!(result.is_err());
    }

    // ---- find_provider_for_backup ----

    #[test]
    fn test_find_provider_nonexistent() {
        let result = find_provider_for_backup("nonexistent-id");
        assert!(result.is_err());
    }
}
