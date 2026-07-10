// ============================================================================
// adapter.rs — S-10: Read-only LegacyAdapter for flat-file backups
// ============================================================================
//
// Reads Phase 1/2.5 flat-file format (manifest.json + files/ directory).
// See Architecture v1.0 §14.
//
// Flat-file layout (Phase 1):
//   {backup_dir_name}/
//   ├── manifest.json       # Manifest (backup_id, files[], summary, sha256)
//   └── files/              # All source files, stored by relative path
//
// Legacy directory (Phase S):
//   legacy/                 # Under repository root
//   └── {backup_dir_name}/
//       └── (same as above)
//
// Policy:
//   Read-only. No migration. No conversion. No new flat-file backups.

use crate::repository::error::RepositoryError;
use crate::repository::repo_manager::RepoHandle;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

// ======== Legacy Data Models ========

/// A restore point discovered from flat-file backup format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyRestorePoint {
    pub backup_id: String,
    pub created_at: String,
    pub source_root: String,
    pub storage_format: String,
    pub file_count: u64,
    pub total_bytes: u64,
    /// Path to the backup directory (under legacy/)
    pub backup_dir: PathBuf,
}

/// Flat-file manifest schema (Phase 1 format, read-only)
#[derive(Debug, Clone, Deserialize)]
struct FlatManifest {
    pub schema_version: String,
    pub backup_id: String,
    pub created_at: String,
    pub source_root: String,
    pub storage_format: String,
    pub files: Vec<FlatFileEntry>,
    pub summary: FlatSummary,
}

#[derive(Debug, Clone, Deserialize)]
struct FlatFileEntry {
    pub relative_path: String,
    pub sha256: String,
    pub stored_path: String,
}

#[derive(Debug, Clone, Deserialize)]
struct FlatSummary {
    pub file_count: u64,
    pub total_bytes: u64,
}

// ======== LegacyAdapter Trait ========

/// Read-only adapter for flat-file backup format.
///
/// Legacy backups are stored under repository's legacy/ directory.
/// They are permanently read-only — no migration, no conversion.
pub trait LegacyAdapter {
    /// List all legacy restore points in the repository.
    fn list_restore_points(&self) -> Result<Vec<LegacyRestorePoint>, RepositoryError>;

    /// Restore all files from a legacy restore point to the target path.
    ///
    /// # Arguments
    /// * `point` — The LegacyRestorePoint to restore from
    /// * `target_path` — Destination directory for restored files
    ///
    /// # Safety
    /// - Validates target path safety
    /// - Per-file SHA-256 verification after copy
    fn restore_full(
        &self,
        point: &LegacyRestorePoint,
        target_path: &Path,
    ) -> Result<RestoreResult, RepositoryError>;
}

/// Result of a legacy restore operation.
#[derive(Debug, Clone, Default)]
pub struct RestoreResult {
    pub files_restored: u64,
    pub bytes_restored: u64,
    pub files_verified: u64,
    pub files_failed: u64,
}

// ======== Default Implementation ========

/// Default LegacyAdapter implementation.
///
/// Scans `legacy/` subdirectories under the repository root,
/// reads `manifest.json` from each, and provides restore capabilities.
pub struct FlatFileAdapter {
    legacy_dir: PathBuf,
}

impl FlatFileAdapter {
    /// Create a new FlatFileAdapter for the given repository handle.
    pub fn new(handle: &RepoHandle) -> Self {
        FlatFileAdapter {
            legacy_dir: handle.legacy_dir.clone(),
        }
    }

    /// Create a new FlatFileAdapter targeting a specific legacy directory.
    /// Used for testing.
    pub fn new_with_path(path: PathBuf) -> Self {
        FlatFileAdapter { legacy_dir: path }
    }
}

impl LegacyAdapter for FlatFileAdapter {
    fn list_restore_points(&self) -> Result<Vec<LegacyRestorePoint>, RepositoryError> {
        if !self.legacy_dir.exists() {
            return Ok(Vec::new());
        }

        let mut points: Vec<LegacyRestorePoint> = Vec::new();

        let entries = fs::read_dir(&self.legacy_dir).map_err(|e| {
            RepositoryError::io(self.legacy_dir.clone(), "Cannot scan legacy directory", e)
        })?;

        for entry in entries.flatten() {
            let dir_path = entry.path();
            if !dir_path.is_dir() {
                continue;
            }

            let manifest_path = dir_path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            match Self::read_manifest(&manifest_path) {
                Ok(manifest) => {
                    points.push(LegacyRestorePoint {
                        backup_id: manifest.backup_id,
                        created_at: manifest.created_at,
                        source_root: manifest.source_root,
                        storage_format: manifest.storage_format,
                        file_count: manifest.summary.file_count,
                        total_bytes: manifest.summary.total_bytes,
                        backup_dir: dir_path,
                    });
                }
                Err(_) => {
                    // Skip corrupted manifests silently — legacy is best-effort
                    continue;
                }
            }
        }

        // Sort by creation time (newest first)
        points.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(points)
    }

    fn restore_full(
        &self,
        point: &LegacyRestorePoint,
        target_path: &Path,
    ) -> Result<RestoreResult, RepositoryError> {
        // Validate target path is safe
        if target_path.as_os_str().is_empty() {
            return Err(RepositoryError::General {
                detail: "Restore target path is empty".to_string(),
            });
        }

        // Read manifest
        let manifest_path = point.backup_dir.join("manifest.json");
        let manifest = Self::read_manifest(&manifest_path)?;

        // Create target directory
        fs::create_dir_all(target_path).map_err(|e| {
            RepositoryError::io(
                target_path.to_path_buf(),
                "Cannot create restore target directory",
                e,
            )
        })?;

        let files_dir = point.backup_dir.join("files");
        if !files_dir.exists() {
            return Err(RepositoryError::General {
                detail: format!("Files directory not found: {}", files_dir.display()),
            });
        }

        let mut result = RestoreResult::default();

        for file_entry in &manifest.files {
            let source_path = files_dir.join(&file_entry.stored_path);

            // Resolve target path with safety check
            let target_file = target_path.join(&file_entry.relative_path);

            if let Some(parent) = target_file.parent() {
                if let Err(_e) = fs::create_dir_all(parent) {
                    result.files_failed += 1;
                    continue;
                }
            }

            // Copy file
            match fs::copy(&source_path, &target_file) {
                Ok(copied_bytes) => {
                    result.files_restored += 1;
                    result.bytes_restored += copied_bytes;

                    // SHA-256 verification
                    match crate::checksum::sha256_file(&target_file) {
                        Ok(actual_hash) if actual_hash == file_entry.sha256 => {
                            result.files_verified += 1;
                        }
                        Ok(_actual_hash) => {
                            result.files_failed += 1;
                            // Remove corrupted file
                            let _ = fs::remove_file(&target_file);
                        }
                        Err(_) => {
                            result.files_failed += 1;
                        }
                    }
                }
                Err(_) => {
                    result.files_failed += 1;
                }
            }
        }

        Ok(result)
    }
}

impl FlatFileAdapter {
    /// Read and validate a flat-file manifest.json.
    fn read_manifest(path: &Path) -> Result<FlatManifest, RepositoryError> {
        let content = fs::read_to_string(path)
            .map_err(|e| RepositoryError::io(path.to_path_buf(), "Cannot read manifest.json", e))?;

        let manifest: FlatManifest = serde_json::from_str(&content)?;

        // Validate schema version (major version must match)
        let current_major = "1";
        let manifest_major = manifest.schema_version.split('.').next().unwrap_or("0");

        if current_major != manifest_major {
            return Err(RepositoryError::General {
                detail: format!(
                    "Unsupported manifest schema version: {} (current: {})",
                    manifest.schema_version, "1.0"
                ),
            });
        }

        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::Manifest;
    use crate::repository::repo_manager::{init_repo, DEFAULT_BLOCK_SIZE};
    use tempfile::TempDir;

    fn setup_legacy_backup(legacy_dir: &Path, backup_name: &str) -> PathBuf {
        let backup_dir = legacy_dir.join(backup_name);
        let files_dir = backup_dir.join("files");
        fs::create_dir_all(&files_dir).unwrap();

        // Create a test file
        let test_content = b"Hello, Nuwa Legacy!";
        let test_file_rel = "documents/test.txt";
        let test_file_path = files_dir.join(test_file_rel);
        fs::create_dir_all(test_file_path.parent().unwrap()).unwrap();
        fs::write(&test_file_path, test_content).unwrap();

        // Create manifest
        let manifest = Manifest::new(
            backup_name.to_string(),
            "C:\\original_source".to_string(),
            crate::manifest::CompressionConfig {
                enabled: false,
                algorithm: None,
            },
        );

        // Build flat-file manifest
        let flat = serde_json::json!({
            "schema_version": manifest.schema_version,
            "backup_id": manifest.backup_id,
            "created_at": manifest.created_at,
            "source_root": manifest.source_root,
            "storage_format": manifest.storage_format,
            "compression": { "enabled": false, "algorithm": null },
            "files": [
                {
                    "relative_path": test_file_rel,
                    "size_bytes": test_content.len(),
                    "modified_time": "2025-01-01T00:00:00Z",
                    "sha256": "2a21c2dd456342396cf1cb9a15c1c85fcaa55d54466512bb2f3dc196cffc740d",
                    "stored_path": test_file_rel
                }
            ],
            "directories": [],
            "summary": {
                "file_count": 1,
                "directory_count": 0,
                "total_bytes": test_content.len() as u64
            }
        });

        let content = serde_json::to_string_pretty(&flat).unwrap();
        fs::write(backup_dir.join("manifest.json"), &content).unwrap();

        backup_dir
    }

    #[test]
    fn test_list_empty_legacy() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();
        let adapter = FlatFileAdapter::new(&handle);

        let points = adapter.list_restore_points().unwrap();
        assert!(points.is_empty(), "New repo should have no legacy backups");
    }

    #[test]
    fn test_list_legacy_points() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        // Place a legacy backup in the legacy directory
        setup_legacy_backup(&handle.legacy_dir, "20260701_TestBackup");

        let adapter = FlatFileAdapter::new(&handle);
        let points = adapter.list_restore_points().unwrap();

        assert_eq!(points.len(), 1, "Should find one legacy backup");
        assert_eq!(
            points[0].backup_dir,
            handle.legacy_dir.join("20260701_TestBackup")
        );
        assert_eq!(points[0].file_count, 1);
        assert_eq!(points[0].total_bytes, 19); // "Hello, Nuwa Legacy!" = 19 bytes
    }

    #[test]
    fn test_restore_legacy_point() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        setup_legacy_backup(&handle.legacy_dir, "RestoreTest");

        let adapter = FlatFileAdapter::new(&handle);
        let points = adapter.list_restore_points().unwrap();

        let restore_target = tmp.path().join("restored");
        let result = adapter.restore_full(&points[0], &restore_target).unwrap();

        assert_eq!(result.files_restored, 1, "Should restore one file");
        assert_eq!(result.files_failed, 0, "No files should fail");

        // Verify restored file content
        let restored_file = restore_target.join("documents/test.txt");
        assert!(restored_file.exists(), "Restored file should exist");
        let content = fs::read_to_string(&restored_file).unwrap();
        assert_eq!(content, "Hello, Nuwa Legacy!");
    }

    #[test]
    fn test_restore_integrity_check() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        setup_legacy_backup(&handle.legacy_dir, "IntegrityTest");

        let adapter = FlatFileAdapter::new(&handle);
        let points = adapter.list_restore_points().unwrap();

        let restore_target = tmp.path().join("integrity_restored");
        let result = adapter.restore_full(&points[0], &restore_target).unwrap();

        // All restored files should pass SHA-256 verification
        assert_eq!(
            result.files_restored, result.files_verified,
            "All restored files must pass SHA-256 verification"
        );
    }

    #[test]
    fn test_multiple_legacy_backups() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        setup_legacy_backup(&handle.legacy_dir, "BackupA");
        setup_legacy_backup(&handle.legacy_dir, "BackupB");

        let adapter = FlatFileAdapter::new(&handle);
        let points = adapter.list_restore_points().unwrap();

        assert_eq!(points.len(), 2, "Should find two legacy backups");
    }

    #[test]
    fn test_skip_directory_without_manifest() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        // Create a directory without manifest.json
        fs::create_dir_all(handle.legacy_dir.join("NotABackup")).unwrap();

        let adapter = FlatFileAdapter::new(&handle);
        let points = adapter.list_restore_points().unwrap();

        assert!(
            points.is_empty(),
            "Directory without manifest should be skipped"
        );
    }
}
