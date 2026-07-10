// ============================================================================
// metadata_store.rs — backup-metadata.json read/write for Backup Instances
// ============================================================================
//
// Each Backup Instance has a backup-metadata.json that contains:
// - Lightweight metadata (no block list)
// - Block Map integrity manifest (DETECTION only, NOT reconstruction)
// - Catalog integrity manifest

use crate::repository::error::RepositoryError;
use crate::repository::metadata::models::BackupInstanceMetadata;
use std::fs;
use std::path::Path;

/// Write backup-metadata.json to the Backup Instance directory.
/// Uses atomic write (.tmp → rename) for crash safety.
pub fn write_metadata(
    instance_dir: &Path,
    metadata: &BackupInstanceMetadata,
) -> Result<(), RepositoryError> {
    // Ensure the instance directory exists
    fs::create_dir_all(instance_dir).map_err(|e| {
        RepositoryError::io(
            instance_dir.to_path_buf(),
            "Cannot create instance directory",
            e,
        )
    })?;

    let json = serde_json::to_string_pretty(metadata)?;

    let meta_path = instance_dir.join("backup-metadata.json");
    let tmp_path = instance_dir.join("backup-metadata.json.tmp");

    // Atomic write: .tmp → rename
    fs::write(&tmp_path, &json).map_err(|e| {
        RepositoryError::io(tmp_path.clone(), "Cannot write metadata temporary file", e)
    })?;

    fs::rename(&tmp_path, &meta_path).map_err(|e| {
        // Clean up .tmp on rename failure
        let _ = fs::remove_file(&tmp_path);
        RepositoryError::io(
            meta_path,
            "Cannot rename metadata file to final location",
            e,
        )
    })?;

    Ok(())
}

/// Read backup-metadata.json from the Backup Instance directory.
pub fn read_metadata(instance_dir: &Path) -> Result<BackupInstanceMetadata, RepositoryError> {
    let meta_path = instance_dir.join("backup-metadata.json");

    if !meta_path.exists() {
        return Err(RepositoryError::MetadataMissing(meta_path));
    }

    let content = fs::read_to_string(&meta_path)
        .map_err(|e| RepositoryError::io(meta_path.clone(), "Cannot read metadata file", e))?;

    let metadata: BackupInstanceMetadata = serde_json::from_str(&content)?;

    // Validate schema version
    if metadata.schema_version != "1.0" {
        return Err(RepositoryError::UnsupportedSchemaVersion(
            metadata.schema_version,
        ));
    }

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::metadata::models::*;
    use tempfile::TempDir;

    fn sample_metadata() -> BackupInstanceMetadata {
        BackupInstanceMetadata {
            schema_version: "1.0".to_string(),
            restore_point_id: "test-point-001".to_string(),
            job_id: "test-job-001".to_string(),
            source_type: "file".to_string(),
            source_description: "C:\\test".to_string(),
            asset_id: "".to_string(),
            asset_type: "".to_string(),
            created_at: "2026-07-10T10:00:00Z".to_string(),
            status: "COMMITTED".to_string(),
            block_chunk_policy: ChunkPolicyMetadata {
                policy_type: "fixed".to_string(),
                block_size: 262144,
            },
            block_map: BlockMapIntegrity {
                database: "block-map.db".to_string(),
                block_count: 100,
                sha256: "abc123".to_string(),
                first_offset: 0,
                last_offset: 26214400,
            },
            catalog: Some(CatalogIntegrity {
                database: "catalog.db".to_string(),
                file_count: 10,
                sha256: "def456".to_string(),
            }),
            summary: BackupInstanceSummary {
                total_raw_bytes: 26214400,
                file_count: 10,
            },
        }
    }

    #[test]
    fn test_write_read_metadata_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let instance_dir = tmp.path().join("backup-instances").join("test-point-001");

        let meta = sample_metadata();
        write_metadata(&instance_dir, &meta).unwrap();

        let read_back = read_metadata(&instance_dir).unwrap();
        assert_eq!(read_back.restore_point_id, meta.restore_point_id);
        assert_eq!(read_back.schema_version, "1.0");
        assert_eq!(read_back.block_chunk_policy.block_size, 262144);
        assert_eq!(read_back.block_map.block_count, 100);
    }

    #[test]
    fn test_read_metadata_not_found() {
        let tmp = TempDir::new().unwrap();
        let result = read_metadata(tmp.path());
        assert!(matches!(result, Err(RepositoryError::MetadataMissing(_))));
    }

    #[test]
    fn test_unsupported_schema_version() {
        let tmp = TempDir::new().unwrap();
        let mut meta = sample_metadata();
        meta.schema_version = "2.0".to_string();
        write_metadata(tmp.path(), &meta).unwrap();
        let result = read_metadata(tmp.path());
        assert!(matches!(
            result,
            Err(RepositoryError::UnsupportedSchemaVersion(_))
        ));
    }

    #[test]
    fn test_atomic_write_crash_safety() {
        let tmp = TempDir::new().unwrap();
        let instance_dir = tmp.path().join("atomic-test");
        let meta = sample_metadata();

        // Normal write should not leave .tmp residue
        write_metadata(&instance_dir, &meta).unwrap();
        assert!(!instance_dir.join("backup-metadata.json.tmp").exists());
        assert!(instance_dir.join("backup-metadata.json").exists());
    }
}
