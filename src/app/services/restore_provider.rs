// ============================================================================
// restore_provider.rs -- RestoreProvider trait and implementations
//
// P-07: Repository-only. Only RepositoryRestoreProvider
// remains for all restore operations via Repository Engine.
// ============================================================================

use std::path::{Path, PathBuf};

use crate::app::error::AppError;
use crate::app::models::restore::{
    RestoreFileEntry, RestoreOperationResult, RestorePointView, RestorePreview, RestoreRequest,
};
use crate::app::services::repo_registry::RepoRegistry;
use crate::repository::file_restore_reader::{FileRestoreReader, RestoreOutcome};

// ---------------------------------------------------------------------------
// RestoreProvider trait
// ---------------------------------------------------------------------------

pub trait RestoreProvider {
    fn list_restore_points(&self) -> Result<Vec<RestorePointView>, AppError>;
    fn get_preview(&self, backup_id: &str) -> Result<RestorePreview, AppError>;
    fn execute_restore(&self, request: &RestoreRequest)
        -> Result<RestoreOperationResult, AppError>;
    fn delete_backup_set(&self, backup_id: &str) -> Result<(), AppError>;
}

// ---------------------------------------------------------------------------
// RepositoryRestoreProvider -- Phase S Repository Engine
// ---------------------------------------------------------------------------

pub struct RepositoryRestoreProvider;

impl RepositoryRestoreProvider {
    fn find_repo_for_backup(
        backup_id: &str,
    ) -> Result<(crate::repository::RepoHandle, PathBuf), AppError> {
        let registry = RepoRegistry::load();
        for record in registry.list() {
            let repo_path = PathBuf::from(&record.path);
            if !repo_path.exists() {
                continue;
            }
            let handle = crate::repository::open_repo(&repo_path)
                .map_err(|e| AppError::storage(format!("Cannot open repository: {}", e)))?;

            // Check via FileRestoreReader
            if FileRestoreReader::open(&handle, backup_id).is_ok() {
                return Ok((handle, repo_path));
            }
        }
        Err(AppError::config(format!(
            "Backup point '{}' not found in any Repository instance.",
            backup_id
        )))
    }
}

impl RestoreProvider for RepositoryRestoreProvider {
    fn list_restore_points(&self) -> Result<Vec<RestorePointView>, AppError> {
        let mut points: Vec<RestorePointView> = Vec::new();
        let registry = RepoRegistry::load();

        for record in registry.list() {
            let repo_path = PathBuf::from(&record.path);
            if !repo_path.exists() {
                continue;
            }
            let handle = match crate::repository::open_repo(&repo_path) {
                Ok(h) => h,
                Err(_) => continue,
            };

            {
                let conn = match handle.repo_db() {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let mut stmt = match conn.prepare(
                "SELECT point_id, job_id, created_at, status, block_count, total_raw_bytes, instance_path FROM restore_points ORDER BY created_at DESC LIMIT 100"
            ) {
                Ok(s) => s,
                Err(_) => continue,
            };

                let _ = stmt
                    .query_map([], |row| {
                        Ok(RestorePointView {
                            backup_id: row.get::<_, String>(0)?,
                            job_name: Some(row.get::<_, String>(1)?),
                            timestamp: row.get::<_, String>(2)?,
                            status: row.get::<_, String>(3)?,
                            file_count: row.get::<_, i64>(4).unwrap_or(0) as u64,
                            total_bytes: row.get::<_, i64>(5).unwrap_or(0) as u64,
                            source_root: record.name.clone(),
                            dest_path: record.path.clone(),
                        })
                    })
                    .map(|rows| {
                        for row in rows.flatten() {
                            points.push(row);
                        }
                    });
            }
        }

        points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(points)
    }

    fn get_preview(&self, backup_id: &str) -> Result<RestorePreview, AppError> {
        if backup_id.trim().is_empty() {
            return Err(AppError::config("Backup ID must not be empty"));
        }

        let (handle, repo_path) = Self::find_repo_for_backup(backup_id)?;
        let reader = FileRestoreReader::open(&handle, backup_id)
            .map_err(|e| AppError::storage(format!("Cannot open restore point: {}", e)))?;

        let paths = reader
            .list_entries()
            .map_err(|e| AppError::storage(format!("Cannot list files: {}", e)))?;

        let mut files: Vec<RestoreFileEntry> = Vec::new();
        for path in &paths {
            if let Ok(entry) = reader.get_entry(path) {
                if entry.entry_type == crate::repository::catalog::CatalogEntryType::File {
                    files.push(RestoreFileEntry {
                        relative_path: entry.path.clone(),
                        size_bytes: entry.size,
                        modified_time: entry.modified.clone(),
                    });
                }
            }
        }

        let total_bytes: u64 = files.iter().map(|e| e.size_bytes).sum();

        Ok(RestorePreview {
            point: RestorePointView {
                backup_id: backup_id.to_string(),
                job_name: Some(String::new()),
                timestamp: String::new(),
                source_root: repo_path.to_string_lossy().to_string(),
                dest_path: repo_path.to_string_lossy().to_string(),
                file_count: files.len() as u64,
                total_bytes,
                status: "success".to_string(),
            },
            total_files: files.len() as u64,
            files,
            total_bytes,
        })
    }

    fn execute_restore(
        &self,
        request: &RestoreRequest,
    ) -> Result<RestoreOperationResult, AppError> {
        if request.backup_id.trim().is_empty() {
            return Err(AppError::config("Backup ID must not be empty"));
        }
        if request.dest.trim().is_empty() {
            return Err(AppError::config("Destination path must not be empty"));
        }

        let (handle, _repo_path) = Self::find_repo_for_backup(&request.backup_id)?;
        let dest_path = Path::new(&request.dest);

        let reader = FileRestoreReader::open(&handle, &request.backup_id)
            .map_err(|e| AppError::storage(format!("Cannot open restore point: {}", e)))?;

        let outcome = reader
            .restore_all(dest_path, request.overwrite)
            .map_err(|e| AppError::storage(format!("Restore failed: {}", e)))?;

        let (restored_count, checksum_failures, status) = match outcome {
            RestoreOutcome::Complete(summary) => (
                summary.files_restored,
                summary.failed_files.len() as u64,
                "success".to_string(),
            ),
            RestoreOutcome::Partial(summary) => (
                summary.files_restored,
                summary.failed_files.len() as u64,
                "partial".to_string(),
            ),
        };

        Ok(RestoreOperationResult {
            restore_id: request.backup_id.clone(),
            restored_count,
            skipped_count: 0,
            checksum_failures,
            timestamp: String::new(),
            duration_ms: 0,
            status,
            error: None,
        })
    }

    fn delete_backup_set(&self, _backup_id: &str) -> Result<(), AppError> {
        Err(AppError::config("Backup set deletion through Repository retention is not yet implemented. Use 'nuwa repo' CLI commands."))
    }
}

// ---------------------------------------------------------------------------
// Provider discovery
// ---------------------------------------------------------------------------

pub fn find_provider_for_backup(_backup_id: &str) -> Result<Box<dyn RestoreProvider>, AppError> {
    Ok(Box::new(RepositoryRestoreProvider))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_restore_empty_backup_id() {
        let provider = RepositoryRestoreProvider;
        let req = RestoreRequest {
            backup_id: "".into(),
            dest: "/tmp/restore".into(),
            overwrite: false,
        };
        let result = provider.execute_restore(&req);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message.contains("Backup ID must not be empty"));
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

    #[test]
    fn test_find_provider_succeeds() {
        let result = find_provider_for_backup("any-id");
        assert!(result.is_ok());
    }
}
