// ============================================================================
// backup_writer.rs 閳?P-01: RepositoryBackupWriter high-level backup pipeline
// ============================================================================
//
// P-01 wraps the CrashConsistencyManager lifecycle and Repository Engine
// components into a cohesive file-level backup pipeline.
//
// Lifecycle (P-00 鎼?.2):
//   1. new()   閳?prepare paths, generate point_id
//   2. begin() 閳?CrashConsistencyManager::begin() + open components
//   3. write_file() / write_directory() 閳?process files through pipeline
//   4. finalize() 閳?complete components, write metadata, enter_verify, commit
//   5. fail()   閳?abort on error
//
// P-01 scope (鎼?.1): File source (single directory), full backup only,
// 256KB fixed block, zstd compression per-block (default enabled).

use crate::repository::block_map::engine::BlockMapEngine;
use crate::repository::block_map::sqlite_block_map::SqliteBlockMap;
use crate::repository::block_store::store::LocalFsBlockStore;
use crate::repository::catalog::engine::{CatalogEngine, FileExtent};
use crate::repository::catalog::sqlite_catalog::SqliteCatalog;
use crate::repository::chunk_engine::policy::FixedChunkPolicy;
use crate::repository::chunk_engine::{ChunkEngine, ChunkResult};
use crate::repository::error::RepositoryError;
use crate::repository::metadata::metadata_store::write_metadata;
use crate::repository::metadata::models::{
    BackupInstanceMetadata, BackupInstanceSummary, BlockMapIntegrity, CatalogIntegrity,
    ChunkPolicyMetadata,
};
use crate::repository::repo_manager::RepoHandle;
use crate::repository::transaction::manager::CrashConsistencyManager;
use sha2::Digest;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Summary of a completed backup run.
#[derive(Debug, Clone)]
pub struct BackupResult {
    /// Restore Point ID (UUID v4)
    pub point_id: String,
    /// Total number of blocks written
    pub block_count: u64,
    /// Total raw (uncompressed) bytes
    pub total_raw_bytes: u64,
    /// Number of file entries in catalog
    pub file_count: u64,
    /// Number of directory entries in catalog
    pub directory_count: u64,
}

/// High-level file backup writer for Repository Engine.
pub struct RepositoryBackupWriter {
    repo: RepoHandle,
    job_id: String,
    point_id: String,
    instance_dir: PathBuf,

    // Lifecycle handles
    manager: Option<CrashConsistencyManager>,
    block_store: LocalFsBlockStore,
    catalog: Option<SqliteCatalog>,
    block_map: Option<SqliteBlockMap>,

    // Engine configuration
    compression: bool,
    block_size: u32,

    // Accumulated state
    current_logical_offset: u64,
    total_raw_bytes: u64,
    block_count: u64,
    file_count: u64,
    directory_count: u64,
}
impl RepositoryBackupWriter {
    /// Create a new BackupWriter for the given repository and job.
    ///
    /// Does NOT start the transaction yet 閳?call begin() separately.
    /// point_id is auto-generated as UUID v4.
    ///
    /// # Arguments
    /// * `repo` 閳?Opened RepoHandle
    /// * `job_id` 閳?Job identifier for this backup
    /// * `compression` 閳?Enable per-block zstd compression
    pub fn new(
        repo: &RepoHandle,
        job_id: &str,
        compression: bool,
    ) -> Result<Self, RepositoryError> {
        let point_id = Uuid::new_v4().to_string();
        let instance_dir = repo.instances_dir.join(&point_id);
        let block_store = LocalFsBlockStore::new(repo.block_store_dir.clone());

        Ok(RepositoryBackupWriter {
            repo: repo.clone(),
            job_id: job_id.to_string(),
            point_id,
            instance_dir,
            manager: None,
            block_store,
            catalog: None,
            block_map: None,
            compression,
            block_size: repo.info.chunk_policy.block_size,
            current_logical_offset: 0,
            total_raw_bytes: 0,
            block_count: 0,
            file_count: 0,
            directory_count: 0,
        })
    }

    /// Start the backup transaction (P-00 鎼?.2 Steps 1-2).
    ///
    /// 1. Creates repo.db entry via CrashConsistencyManager
    /// 2. Transitions to WRITING via enter_writing()
    /// 3. Creates instance directory
    /// 4. Opens Catalog and BlockMap databases
    pub fn begin(&mut self) -> Result<(), RepositoryError> {
        let mgr = CrashConsistencyManager::begin(&self.repo, &self.point_id, &self.job_id)?;
        self.manager = Some(mgr);
        self.manager.as_mut().unwrap().enter_writing(&self.repo)?;

        // Create instance directory
        fs::create_dir_all(&self.instance_dir).map_err(|e| {
            RepositoryError::io(
                self.instance_dir.clone(),
                "Cannot create instance directory",
                e,
            )
        })?;

        // Open catalog and block-map databases
        let cat_path = self.instance_dir.join("catalog.db");
        let catalog = SqliteCatalog::open(cat_path)?;
        let bm_path = self.instance_dir.join("block-map.db");
        let block_map = SqliteBlockMap::open(bm_path)?;

        self.catalog = Some(catalog);
        self.block_map = Some(block_map);
        Ok(())
    }

    /// Write a single file to the backup (P-01b).
    ///
    /// 1. Opens the source file
    /// 2. Computes SHA-256 of full content
    /// 3. Passes through ChunkEngine -> BlockStore
    /// 4. Inserts BlockMap entries
    /// 5. Adds Catalog entry with extents
    ///
    /// # Arguments
    /// * `source_path` 閳?Absolute path to the file on disk
    /// * `relative_path` 閳?Relative path for catalog (forward slashes)
    /// * `modified` 閳?ISO-8601 modification timestamp
    pub fn write_file(
        &mut self,
        source_path: &Path,
        relative_path: &str,
        modified: &str,
    ) -> Result<(), RepositoryError> {
        // Validate relative path per P-00 鎼?.4
        self.validate_relative_path(relative_path)?;

        let file_size = source_path
            .metadata()
            .map_err(|e| {
                RepositoryError::io(
                    source_path.to_path_buf(),
                    "Cannot access source file metadata",
                    e,
                )
            })?
            .len();

        // Open source file
        let mut file = fs::File::open(source_path).map_err(|e| {
            RepositoryError::io(source_path.to_path_buf(), "Cannot open source file", e)
        })?;

        // Compute SHA-256 of file content
        let file_sha256 = {
            use std::io::Read;
            let mut hasher = sha2::Sha256::new();
            let mut buf = [0u8; 65536];
            loop {
                let n = file.read(&mut buf).map_err(|e| {
                    RepositoryError::io(
                        source_path.to_path_buf(),
                        "Cannot read source file for SHA-256",
                        e,
                    )
                })?;
                if n == 0 {
                    break;
                }
                hasher.update(&buf[..n]);
            }
            format!("{:x}", hasher.finalize())
        };

        // Rewind file for chunk engine
        use std::io::Seek;
        file.rewind().map_err(|e| {
            RepositoryError::io(source_path.to_path_buf(), "Cannot rewind source file", e)
        })?;

        // Process through ChunkEngine
        let policy = FixedChunkPolicy::new(self.block_size)?;
        let engine = ChunkEngine::new(Box::new(policy), self.compression);
        let chunk_results: Vec<ChunkResult> = engine.process(&mut file, &self.block_store)?;

        let file_start_offset = self.current_logical_offset;

        // Build extents and update BlockMap
        let mut extents = Vec::with_capacity(chunk_results.len());
        let mut extent_file_offset: u64 = 0;

        for result in &chunk_results {
            let extent = FileExtent {
                file_offset: extent_file_offset,
                logical_offset: result.logical_offset + file_start_offset,
                length: result.raw_size,
            };
            extents.push(extent);

            let global_offset = file_start_offset + result.logical_offset;
            self.block_map.as_mut().unwrap().insert_mapping(
                global_offset,
                &result.block_id,
                result.raw_size,
            )?;

            extent_file_offset += result.raw_size;
        }

        // Update running totals
        let total_file_bytes: u64 = chunk_results.iter().map(|r| r.raw_size).sum();
        self.current_logical_offset += total_file_bytes;
        self.total_raw_bytes += total_file_bytes;
        self.block_count += chunk_results.len() as u64;
        self.file_count += 1;

        // Add catalog entry
        self.catalog.as_mut().unwrap().add_file(
            relative_path,
            file_size,
            modified,
            Some(file_sha256),
            extents,
        )?;

        Ok(())
    }

    /// Write an empty directory entry to the catalog (P-01b).
    pub fn write_directory(
        &mut self,
        relative_path: &str,
        modified: &str,
    ) -> Result<(), RepositoryError> {
        self.validate_relative_path(relative_path)?;
        self.catalog
            .as_mut()
            .unwrap()
            .add_directory(relative_path, modified)?;
        self.directory_count += 1;
        Ok(())
    }
}

impl RepositoryBackupWriter {
    pub fn backup_directory(&mut self, source_dir: &Path) -> Result<(), RepositoryError> {
        self.backup_directory_recursive(source_dir, source_dir, "")?;
        Ok(())
    }
    fn backup_directory_recursive(
        &mut self,
        source_root: &Path,
        current_dir: &Path,
        relative_prefix: &str,
    ) -> Result<(), RepositoryError> {
        let entries = fs::read_dir(current_dir).map_err(|e| {
            RepositoryError::io(current_dir.to_path_buf(), "Cannot read directory", e)
        })?;
        for entry in entries {
            let entry = entry.map_err(|e| {
                RepositoryError::io(current_dir.to_path_buf(), "Cannot read directory entry", e)
            })?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .map_err(|e| RepositoryError::io(path.clone(), "Cannot get file type", e))?;
            let rel_path =
                path.strip_prefix(source_root)
                    .map_err(|_| RepositoryError::General {
                        detail: format!(
                            "Path {} is not under source root {:?}",
                            path.display(),
                            source_root
                        ),
                    })?;
            let rel_str = rel_path.to_string_lossy().replace("\\", "/");
            let catalog_path = if relative_prefix.is_empty() {
                rel_str.clone()
            } else {
                format!("{}/{}", relative_prefix, rel_str)
            };
            if file_type.is_symlink() {
                return Err(RepositoryError::General {
                    detail: format!("P-00 §9: Symbolic link not supported: {}.", catalog_path),
                });
            }
            if file_type.is_dir() {
                self.write_directory(&catalog_path, &self.get_modified_time(&path)?)?;
                self.backup_directory_recursive(source_root, &path, relative_prefix)?;
            } else if file_type.is_file() {
                self.write_file(&path, &catalog_path, &self.get_modified_time(&path)?)?;
            }
        }
        Ok(())
    }
    fn get_modified_time(&self, path: &Path) -> Result<String, RepositoryError> {
        let metadata = path
            .metadata()
            .map_err(|e| RepositoryError::io(path.to_path_buf(), "Cannot read metadata", e))?;
        let modified = metadata
            .modified()
            .map_err(|e| RepositoryError::io(path.to_path_buf(), "Cannot read modified time", e))?;
        let datetime: chrono::DateTime<chrono::Utc> = modified.into();
        Ok(datetime.to_rfc3339())
    }
    pub fn finalize(mut self) -> Result<BackupResult, RepositoryError> {
        let mut mgr = self
            .manager
            .take()
            .ok_or_else(|| RepositoryError::General {
                detail: "BackupWriter: not in an active transaction".to_string(),
            })?;
        mgr.complete_block_store()?;
        mgr.complete_block_map()?;
        let _bm_path = Box::new(self.block_map.take().unwrap()).close()?;
        mgr.complete_catalog()?;
        let _cat_path = Box::new(self.catalog.take().unwrap()).close()?;
        let bm_sha256 = Self::compute_file_sha256(&_bm_path)?;
        let cat_sha256 = Self::compute_file_sha256(&_cat_path)?;
        let metadata = self.build_metadata_with_hashes(&bm_sha256, &cat_sha256);
        write_metadata(&self.instance_dir, &metadata)?;
        mgr.complete_metadata()?;
        mgr.enter_verify(&self.repo)?;
        mgr.commit(&self.repo)?;
        self.repo.update_restore_point_stats(
            &self.point_id,
            self.block_count as i64,
            self.total_raw_bytes as i64,
        )?;
        Ok(BackupResult {
            point_id: self.point_id.clone(),
            block_count: self.block_count,
            total_raw_bytes: self.total_raw_bytes,
            file_count: self.file_count,
            directory_count: self.directory_count,
        })
    }
    fn compute_file_sha256(path: &Path) -> Result<String, RepositoryError> {
        use std::io::Read;
        let mut file = std::fs::File::open(path).map_err(|e| {
            RepositoryError::io(path.to_path_buf(), "Cannot open file for SHA-256", e)
        })?;
        let mut hasher = sha2::Sha256::new();
        let mut buf = [0u8; 65536];
        loop {
            let n = file.read(&mut buf).map_err(|e| {
                RepositoryError::io(path.to_path_buf(), "Cannot read file for SHA-256", e)
            })?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }
    fn build_metadata_with_hashes(
        &self,
        bm_sha256: &str,
        cat_sha256: &str,
    ) -> BackupInstanceMetadata {
        BackupInstanceMetadata {
            schema_version: "1.0".to_string(),
            restore_point_id: self.point_id.clone(),
            job_id: self.job_id.clone(),
            source_type: "file".to_string(),
            source_description: String::new(),
            asset_id: String::new(),
            asset_type: "file".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            status: "COMMITTED".to_string(),
            block_chunk_policy: ChunkPolicyMetadata {
                policy_type: "fixed".to_string(),
                block_size: self.block_size,
            },
            block_map: BlockMapIntegrity {
                database: "block-map.db".to_string(),
                block_count: self.block_count,
                sha256: bm_sha256.to_string(),
                first_offset: 0,
                last_offset: self.current_logical_offset.saturating_sub(1),
            },
            catalog: Some(CatalogIntegrity {
                database: "catalog.db".to_string(),
                file_count: self.file_count + self.directory_count,
                sha256: cat_sha256.to_string(),
            }),
            summary: BackupInstanceSummary {
                total_raw_bytes: self.total_raw_bytes,
                file_count: self.file_count,
            },
        }
    }
    pub fn fail(mut self) -> Result<(), RepositoryError> {
        if let Some(mgr) = self.manager.take() {
            mgr.fail(&self.repo)?;
        }
        Ok(())
    }
    pub fn point_id(&self) -> &str {
        &self.point_id
    }
    fn validate_relative_path(&self, path: &str) -> Result<(), RepositoryError> {
        if path.starts_with('/') || path.starts_with('\\') {
            return Err(RepositoryError::General {
                detail: format!("P-00 §1.4: Absolute path not allowed in catalog: {}", path),
            });
        }
        if path.contains("..") {
            return Err(RepositoryError::General {
                detail: format!(
                    "P-00 §1.4: Path containing '..' not allowed in catalog: {}",
                    path
                ),
            });
        }
        if path.len() >= 2 && path.as_bytes()[1] == b':' {
            return Err(RepositoryError::General {
                detail: format!(
                    "P-00 §1.4: Windows drive letter path not allowed in catalog: {}",
                    path
                ),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::init_repo;
    use tempfile::TempDir;

    fn setup_job(handle: &RepoHandle, job_id: &str) {
        let conn = handle.repo_db().expect("repo_db");
        conn.execute(
            "INSERT OR IGNORE INTO backup_jobs (job_id, job_name, source_type, created_at, status) VALUES (?1, ?2, 0, ?3, 'active')",
            rusqlite::params![job_id, job_id, &chrono::Utc::now().to_rfc3339()],
        ).expect("insert job");
    }

    fn setup_repo() -> (RepoHandle, TempDir) {
        let tmp = TempDir::new().expect("temp dir");
        let handle =
            init_repo(tmp.path(), crate::repository::DEFAULT_BLOCK_SIZE).expect("init repo");
        (handle, tmp)
    }

    #[test]
    fn test_backup_writer_new_and_begin() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("create writer");
        writer.begin().expect("begin");
        assert!(!writer.point_id().is_empty());
    }

    #[test]
    fn test_backup_writer_write_file() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().expect("src tmp");
        let src_file = src_tmp.path().join("test.txt");
        fs::write(&src_file, b"Hello P-01").expect("write");
        let mut writer = RepositoryBackupWriter::new(&handle, "test-job", false).expect("new");
        writer.begin().expect("begin");
        writer
            .write_file(&src_file, "test.txt", "2026-07-11T00:00:00Z")
            .expect("write");
        assert_eq!(writer.file_count, 1);
        assert_eq!(writer.block_count, 1);
    }

    #[test]
    fn test_backup_writer_write_directory() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let mut writer = RepositoryBackupWriter::new(&handle, "test-job", false).expect("new");
        writer.begin().expect("begin");
        writer
            .write_directory("empty-dir", "2026-07-11T00:00:00Z")
            .expect("write");
        assert_eq!(writer.directory_count, 1);
    }

    #[test]
    fn test_backup_writer_write_file_and_finalize() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().expect("src tmp");
        let src_file = src_tmp.path().join("test.txt");
        fs::write(&src_file, b"finalize test").expect("write");
        let mut writer = RepositoryBackupWriter::new(&handle, "test-job", false).expect("new");
        writer.begin().expect("begin");
        writer
            .write_file(&src_file, "test.txt", "2026-07-11T00:00:00Z")
            .expect("write");
        let result = writer.finalize().expect("finalize");
        assert_eq!(result.point_id.len(), 36);
        assert_eq!(result.file_count, 1);
        assert_eq!(result.block_count, 1);
        let status = handle
            .get_restore_point_status(&result.point_id)
            .expect("get")
            .expect("exists");
        assert_eq!(status, "COMMITTED");
    }

    #[test]
    fn test_validate_path_rejects_absolute() {
        let (handle, _tmp) = setup_repo();
        let writer = RepositoryBackupWriter::new(&handle, "test-job", false).expect("new");
        assert!(writer.validate_relative_path("/etc/passwd").is_err());
        assert!(writer
            .validate_relative_path("C:\\Windows\\system32")
            .is_err());
        assert!(writer.validate_relative_path("foo/../bar").is_err());
        assert!(writer.validate_relative_path("valid/rel/path.txt").is_ok());
    }

    #[test]
    fn test_backup_writer_write_multi_block_file() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().expect("src tmp");
        let src_file = src_tmp.path().join("large.bin");
        let file_size: u64 = 307200;
        let data: Vec<u8> = (0..file_size).map(|i| (i % 251) as u8).collect();
        fs::write(&src_file, &data).expect("write");
        let mut writer = RepositoryBackupWriter::new(&handle, "test-job", false).expect("new");
        writer.begin().expect("begin");
        writer
            .write_file(&src_file, "large.bin", "2026-07-11T00:00:00Z")
            .expect("write");
        assert_eq!(writer.block_count, 2);
        let result = writer.finalize().expect("finalize");
        assert_eq!(result.block_count, 2);
        assert_eq!(result.total_raw_bytes, file_size);
        let status = handle
            .get_restore_point_status(&result.point_id)
            .expect("get")
            .expect("exists");
        assert_eq!(status, "COMMITTED");
    }

    #[test]
    fn test_backup_directory_walk() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().expect("src tmp");
        let root = src_tmp.path();
        fs::write(root.join("root.txt"), b"rf").expect("w");
        fs::create_dir(root.join("sub")).expect("mkdir");
        fs::write(root.join("sub").join("n.txt"), b"nf").expect("w");
        fs::create_dir(root.join("empty")).expect("mkdir");
        fs::create_dir_all(root.join("a").join("b").join("c")).expect("mkdir");
        fs::write(root.join("a").join("b").join("c").join("d.txt"), b"df").expect("w");
        let mut writer = RepositoryBackupWriter::new(&handle, "test-job", false).expect("new");
        writer.begin().expect("begin");
        writer.backup_directory(root).expect("backup");
        assert_eq!(writer.file_count, 3);
        assert_eq!(writer.directory_count, 5);
        let result = writer.finalize().expect("finalize");
        assert_eq!(result.file_count, 3);
        assert_eq!(result.directory_count, 5);
        let status = handle
            .get_restore_point_status(&result.point_id)
            .expect("get")
            .expect("exists");
        assert_eq!(status, "COMMITTED");
    }
}
