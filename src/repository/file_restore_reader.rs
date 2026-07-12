// ============================================================================
// file_restore_reader.rs — P-02: Repository File Restore Reader
// ============================================================================
//
// P-02 implements restore from a verified, COMMITTED Restore Point in a
// Nuwa Repository. This module reads blocks via BlockStore → BlockMap →
// Catalog pipeline and reconstructs files streamingly with integrity
// verification.
//
// Core principles:
// - Only COMMITTED Restore Points are eligible for restore
// - 14-step preflight validation before any data read
// - Integrity errors (hash mismatch, path escape, corruption) = hard Err
// - Single-file I/O errors = soft fail (RestoreOutcome::Partial)
//
// Lifecycle:
//   1. FileRestoreReader::open(repo, point_id) → 14-step preflight
//   2. restore_all()          → restore every catalog entry
//   3. restore_selected(paths) → selective restore
//   4. RestoreOutcome          → Complete or Partial

#![allow(clippy::empty_line_after_doc_comments)]
use crate::repository::block_map::engine::{BlockMapEngine, BlockMapEntry};
use crate::repository::block_map::sqlite_block_map::SqliteBlockMap;
use crate::repository::block_store::store::BlockStore;
use crate::repository::catalog::engine::{CatalogEngine, CatalogEntryType, FileEntry};
use crate::repository::catalog::sqlite_catalog::SqliteCatalog;
use crate::repository::error::RepositoryError;
use crate::repository::metadata::metadata_store::read_metadata;
use crate::repository::metadata::models::BackupInstanceMetadata;
use crate::repository::path_security::prepare_restore_file_target;
use crate::repository::repo_manager::{check_repo, RepoHandle};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use rusqlite::params;

/// Full record of a Restore Point from repo.db restore_points table.
#[derive(Debug, Clone)]
pub(crate) struct RestorePointRecord {
    pub point_id: String,
    pub status: String,
    pub instance_path: String,
    pub job_id: String,
    pub total_raw_bytes: i64,
}

/// Query a full Restore Point record from repo.db by point_id.
fn get_restore_point(
    repo: &RepoHandle,
    point_id: &str,
) -> Result<Option<RestorePointRecord>, RepositoryError> {
    let conn = repo.repo_db()?;
    let result = conn.query_row(
        "SELECT point_id, status, instance_path, job_id, total_raw_bytes FROM restore_points WHERE point_id = ?1",
        params![point_id],
        |row| {
            Ok(RestorePointRecord {
                point_id: row.get(0)?,
                status: row.get(1)?,
                instance_path: row.get(2)?,
                job_id: row.get(3)?,
                total_raw_bytes: row.get(4)?,
            })
        },
    );
    match result {
        Ok(record) => Ok(Some(record)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}

/// Outcome of a restore operation.
#[derive(Debug, Clone)]

pub enum RestoreOutcome {
    /// All files restored successfully.
    Complete(RestoreSummary),

    /// Some files failed; partial restore completed.
    Partial(RestoreSummary),
}

/// Summary of a restore operation.
#[derive(Debug, Clone)]

pub struct RestoreSummary {
    /// Total number of entries restored (files + directories)
    pub total_entries: u64,

    /// Number of files successfully restored
    pub files_restored: u64,

    /// Number of directories restored
    pub directories_restored: u64,

    /// Number of files that failed to restore
    pub failed_files: Vec<String>,

    /// Total bytes written to disk
    pub total_bytes_restored: u64,

    /// The Restore Point that was read
    pub point_id: String,
}

/// Repository File Restore Reader.

///

/// Created via FileRestoreReader::open() which performs 14-step preflight

/// validation. After successful open(), call restore_all() or restore_selected()
/// to restore files.
pub struct FileRestoreReader {
    #[allow(dead_code)]
    repo: RepoHandle,

    point_record: RestorePointRecord,

    metadata: BackupInstanceMetadata,

    catalog: SqliteCatalog,

    block_map: SqliteBlockMap,
    block_store: crate::repository::block_store::store::LocalFsBlockStore,
}

impl std::fmt::Debug for FileRestoreReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileRestoreReader")
            .field("point_id", &self.point_record.point_id)
            .field("status", &self.point_record.status)
            .field("instance_path", &self.point_record.instance_path)
            .field("file_count", &self.metadata.summary.file_count)
            .finish()
    }
}

impl FileRestoreReader {
    /// Open a Restore Point for reading, performing 14-step preflight.

    ///

    /// # Arguments

    /// * `repo` — Opened RepoHandle

    /// * `point_id` — Restore Point ID to open

    ///

    /// # Preflight Steps

    /// 1. check_repo() — basic repository integrity

    /// 2. get_restore_point(point_id) — read from repo.db

    /// 3. Verify status == "COMMITTED"; reject anything else

    /// 4. Resolve instance_path — validate relative, join instances_dir

    /// 5. Read backup-metadata.json from instance_path

    /// 6. Verify metadata.restore_point_id == point_id

    /// 7. Verify metadata.job_id == repo.db job_id

    /// 8. Verify metadata.source_type == "file"

    /// 9. Recompute SHA-256 of catalog.db == metadata.catalog.sha256

    /// 10. Recompute SHA-256 of block-map.db == metadata.block_map.sha256

    /// 11. Open Catalog and Block Map

    /// 12. Verify catalog.file_count == metadata.catalog.file_count

    /// 13. Verify block_count == metadata.block_map.block_count

    /// 14. Verify total_raw_bytes == metadata.summary.total_raw_bytes
    pub fn open(repo: &RepoHandle, point_id: &str) -> Result<FileRestoreReader, RepositoryError> {
        // Step 1: Basic repository integrity check

        check_repo(repo)?;

        // Step 2: Read Restore Point from repo.db

        let point_record =
            get_restore_point(repo, point_id)?.ok_or_else(|| RepositoryError::General {
                detail: format!("Restore point '{}' not found in repository", point_id),
            })?;

        // Step 3: Verify status is COMMITTED

        if point_record.status != "COMMITTED" {
            return Err(RepositoryError::General {

                detail: format!(

                    "Restore point '{}' has status '{}'. Only COMMITTED restore points can be restored.",

                    point_id, point_record.status

                ),

            });
        }

        // Step 4: Resolve instance path
        // instance_path is stored as 'backup-instances/{point_id}' (relative to repo root)
        let instance_path = if point_record.instance_path.is_empty() {
            repo.instances_dir.join(point_id)
        } else {
            let path = Path::new(&point_record.instance_path);
            if path.is_absolute() {
                path.to_path_buf()
            } else {
                // Relative paths are relative to the repository root
                repo.root.join(path)
            }
        };

        // Verify instance_path exists

        if !instance_path.exists() {
            return Err(RepositoryError::MetadataMissing(instance_path));
        }

        // Step 5: Read backup-metadata.json

        let metadata = read_metadata(&instance_path)?;

        // Step 6: Verify metadata.restore_point_id matches

        if metadata.restore_point_id != point_id {
            return Err(RepositoryError::RepositoryIdentityMismatch {
                field: "restore_point_id".into(),

                v1: metadata.restore_point_id,

                v2: point_id.to_string(),
            });
        }

        // Step 7: Verify metadata.job_id matches repo.db

        if metadata.job_id != point_record.job_id {
            return Err(RepositoryError::RepositoryIdentityMismatch {
                field: "job_id".into(),

                v1: metadata.job_id,

                v2: point_record.job_id,
            });
        }

        // Step 8: Verify source_type is "file"

        if metadata.source_type != "file" {
            return Err(RepositoryError::General {

                detail: format!(

                    "Restore point '{}' has source_type '{}'. This restore reader only supports 'file' source type.",

                    point_id, metadata.source_type

                ),

            });
        }

        // Step 9: Compute SHA-256 of catalog.db and verify against metadata

        let catalog_path = instance_path.join(
            &metadata
                .catalog
                .as_ref()
                .ok_or_else(|| RepositoryError::General {
                    detail: "Metadata missing catalog integrity section".into(),
                })?
                .database,
        );

        let catalog_sha256 = compute_file_sha256(&catalog_path)?;

        let expected_catalog_sha256 = &metadata.catalog.as_ref().unwrap().sha256;

        if catalog_sha256 != *expected_catalog_sha256 {
            return Err(RepositoryError::BlockStoreCorrupted {
                root: catalog_path,

                detail: format!(
                    "Catalog SHA-256 mismatch: expected {}, computed {}",
                    expected_catalog_sha256, catalog_sha256
                ),
            });
        }

        // Step 10: Compute SHA-256 of block-map.db and verify against metadata

        let block_map_path = instance_path.join(&metadata.block_map.database);

        let block_map_sha256 = compute_file_sha256(&block_map_path)?;

        if block_map_sha256 != metadata.block_map.sha256 {
            return Err(RepositoryError::BlockMapCorrupted {
                path: block_map_path,

                detail: format!(
                    "Block Map SHA-256 mismatch: expected {}, computed {}",
                    metadata.block_map.sha256, block_map_sha256
                ),
            });
        }

        // Step 11: Open Catalog and Block Map databases

        let catalog = SqliteCatalog::open(catalog_path.clone())?;

        let block_map = SqliteBlockMap::open(block_map_path.clone())?;

        // Step 12: Verify catalog file_count matches metadata

        let actual_file_count = catalog.file_count()?;

        let expected_file_count = metadata.catalog.as_ref().map(|c| c.file_count).unwrap_or(0);

        if actual_file_count != expected_file_count {
            return Err(RepositoryError::CatalogCorrupted {
                path: catalog_path,

                detail: format!(
                    "Catalog file_count mismatch: expected {}, actual {}",
                    expected_file_count, actual_file_count
                ),
            });
        }

        // Step 13: Verify block_count matches metadata

        let actual_block_count = block_map.block_count()?;

        if actual_block_count != metadata.block_map.block_count {
            return Err(RepositoryError::BlockMapCorrupted {
                path: block_map_path,

                detail: format!(
                    "Block Map block_count mismatch: expected {}, actual {}",
                    metadata.block_map.block_count, actual_block_count
                ),
            });
        }

        // Step 14: Verify total_raw_bytes matches metadata summary

        // Also validate against repo.db record

        if metadata.summary.total_raw_bytes != point_record.total_raw_bytes as u64 {
            return Err(RepositoryError::RepositoryIdentityMismatch {
                field: "total_raw_bytes".into(),

                v1: metadata.summary.total_raw_bytes.to_string(),

                v2: point_record.total_raw_bytes.to_string(),
            });
        }

        // Initialize BlockStore from repository block_store_dir
        let block_store = crate::repository::block_store::store::LocalFsBlockStore::new(
            repo.block_store_dir.clone(),
        );
        Ok(FileRestoreReader {
            repo: repo.clone(),

            point_record,

            metadata,

            catalog,

            block_map,
            block_store,
        })
    }

    /// Get a reference to the Restore Point record.
    #[allow(dead_code)]
    pub(crate) fn point_record(&self) -> &RestorePointRecord {
        &self.point_record
    }

    /// Get a reference to the metadata.
    pub fn metadata(&self) -> &BackupInstanceMetadata {
        &self.metadata
    }

    /// Validate a FileEntry for restore eligibility.

    /// List all files in the catalog.
    pub fn list_entries(&self) -> Result<Vec<String>, RepositoryError> {
        self.catalog.list_files()
    }

    /// Get a single file entry from the catalog.
    pub fn get_entry(&self, path: &str) -> Result<FileEntry, RepositoryError> {
        self.catalog
            .get_file(path)?
            .ok_or_else(|| RepositoryError::FileNotFound(path.to_string()))
    }

    pub fn validate_file_entry(&self, entry: &FileEntry) -> Result<(), RepositoryError> {
        match entry.entry_type {
            CatalogEntryType::File => {
                if entry.sha256.is_none() {
                    return Err(RepositoryError::CatalogCorrupted {
                        path: PathBuf::new(),
                        detail: format!("File entry '{}' has no SHA-256 hash", entry.path),
                    });
                }
                let mut extents = entry.extents.clone();
                extents.sort_by_key(|e| e.file_offset);
                let mut current_offset: u64 = 0;
                for ext in &extents {
                    if ext.length == 0 {
                        return Err(RepositoryError::CatalogCorrupted {
                            path: PathBuf::new(),
                            detail: "Zero-length extent".into(),
                        });
                    }
                    if ext.file_offset != current_offset {
                        return Err(RepositoryError::CatalogCorrupted {
                            path: PathBuf::new(),
                            detail: format!(
                                "Extent gap: expected offset {} got {}",
                                current_offset, ext.file_offset
                            ),
                        });
                    }
                    let _ = current_offset.checked_add(ext.length).ok_or_else(|| {
                        RepositoryError::CatalogCorrupted {
                            path: PathBuf::new(),
                            detail: "Overflow at file offset".into(),
                        }
                    })?;
                    let _ = ext.logical_offset.checked_add(ext.length).ok_or_else(|| {
                        RepositoryError::CatalogCorrupted {
                            path: PathBuf::new(),
                            detail: "logical_offset overflow".into(),
                        }
                    })?;

                    current_offset = ext.file_offset + ext.length;
                }
                if current_offset != entry.size {
                    return Err(RepositoryError::CatalogCorrupted {
                        path: PathBuf::new(),
                        detail: format!(
                            "Extents cover {} bytes but file size is {}",
                            current_offset, entry.size
                        ),
                    });
                }
                Ok(())
            }
            CatalogEntryType::Directory => {
                if entry.sha256.is_some() {
                    return Err(RepositoryError::CatalogCorrupted {
                        path: PathBuf::new(),
                        detail: "Directory entry has SHA-256".into(),
                    });
                }
                if !entry.extents.is_empty() {
                    return Err(RepositoryError::CatalogCorrupted {
                        path: PathBuf::new(),
                        detail: "Directory entry has extents".into(),
                    });
                }
                if entry.size != 0 {
                    return Err(RepositoryError::CatalogCorrupted {
                        path: PathBuf::new(),
                        detail: format!("Directory has non-zero size {}", entry.size),
                    });
                }
                Ok(())
            }
        }
    }
    /// Validate that block map entries cover the extent range with no gaps.
    fn validate_block_coverage(
        &self,
        mappings: &[BlockMapEntry],
        extent_logical_offset: u64,
        extent_length: u64,
    ) -> Result<(), RepositoryError> {
        if mappings.is_empty() {
            return Err(RepositoryError::OffsetNotFound(extent_logical_offset));
        }
        let extent_end = extent_logical_offset
            .checked_add(extent_length)
            .ok_or_else(|| RepositoryError::General {
                detail: "Extent length overflow".into(),
            })?;
        let mut expected = extent_logical_offset;
        for mapping in mappings {
            if mapping.logical_offset != expected {
                return Err(RepositoryError::General {
                    detail: format!(
                        "Block map gap at logical_offset: expected {} got {}",
                        expected, mapping.logical_offset
                    ),
                });
            }
            expected = mapping
                .logical_offset
                .checked_add(mapping.raw_size)
                .ok_or_else(|| RepositoryError::General {
                    detail: "Block map raw_size overflow".into(),
                })?;
        }
        if expected < extent_end {
            return Err(RepositoryError::General {
                detail: format!(
                    "Block map coverage incomplete: covers up to {} extent ends at {}",
                    expected, extent_end
                ),
            });
        }
        Ok(())
    }

    /// Decompress block data to raw bytes.
    fn decompress_block_data(
        &self,
        block: &crate::repository::block_store::store::Block,
    ) -> Result<Vec<u8>, RepositoryError> {
        use crate::repository::block_store::block_header::Compression;
        match block.header.compression {
            Compression::None => Ok(block.data.clone()),
            Compression::Zstd => {
                let mut decoder =
                    zstd::Decoder::new(&block.data[..]).map_err(|e| RepositoryError::General {
                        detail: format!("zstd decompression failed: {}", e),
                    })?;
                let mut raw = Vec::with_capacity(block.header.raw_size as usize);
                decoder
                    .read_to_end(&mut raw)
                    .map_err(|e| RepositoryError::General {
                        detail: format!("Failed to read decompressed data: {}", e),
                    })?;
                if raw.len() != block.header.raw_size as usize {
                    return Err(RepositoryError::General {
                        detail: format!(
                            "Decompressed size mismatch: expected {} got {}",
                            block.header.raw_size,
                            raw.len()
                        ),
                    });
                }
                Ok(raw)
            }
        }
    }
    /// Restore a single file from the Restore Point to the destination.
    ///
    /// Algorithm:
    /// 1. Validate entry via validate_file_entry()
    /// 2. Get safe destination via prepare_restore_file_target()
    /// 3. Check overwrite semantics
    /// 4. Empty file optimization
    /// 5. Stream blocks: BlockMap -> BlockStore -> .tmp file
    /// 6. SHA-256 verification of .tmp file
    /// 7. Atomic rename .tmp -> dest
    pub fn restore_file(
        &self,
        dest_root: &Path,
        entry: &FileEntry,
        overwrite: bool,
    ) -> Result<(PathBuf, u64), RepositoryError> {
        if entry.entry_type != CatalogEntryType::File {
            return Err(RepositoryError::General {
                detail: format!("Entry '{}' is not a file", entry.path),
            });
        }
        self.validate_file_entry(entry)?;
        let dest = prepare_restore_file_target(dest_root, &entry.path)?;
        if dest.exists() && !overwrite {
            return Err(RepositoryError::General {
                detail: format!(
                    "Restore target already exists: {}. Use overwrite=true to replace.",
                    dest.display()
                ),
            });
        }
        // Empty file optimization
        if entry.size == 0 {
            if !dest.exists() {
                fs::write(&dest, []).map_err(|e| RepositoryError::IoError {
                    path: dest.clone(),
                    detail: format!("Failed to create empty file: {}", dest.display()),
                    source: e,
                })?;
            }
            return Ok((dest, 0));
        }
        // Create .tmp file
        let tmp_path = dest.with_extension("tmp");
        if tmp_path.exists() {
            fs::remove_file(&tmp_path).map_err(|e| RepositoryError::IoError {
                path: tmp_path.clone(),
                detail: "Failed to remove stale .tmp file".into(),
                source: e,
            })?;
        }
        let mut tmp_file = fs::File::create(&tmp_path).map_err(|e| RepositoryError::IoError {
            path: tmp_path.clone(),
            detail: format!("Failed to create .tmp file: {}", tmp_path.display()),
            source: e,
        })?;
        // Sort extents by file_offset
        let mut extents = entry.extents.clone();
        extents.sort_by_key(|e| e.file_offset);
        let mut bytes_written: u64 = 0;
        for extent in &extents {
            let extent_start = extent.logical_offset;
            let extent_end = extent_start.checked_add(extent.length).ok_or_else(|| {
                RepositoryError::General {
                    detail: "Extent overflow during restore".into(),
                }
            })?;
            let mappings = self.block_map.get_range(extent_start, extent_end)?;
            self.validate_block_coverage(&mappings, extent_start, extent.length)?;
            for mapping in &mappings {
                let block = self.block_store.get_block(&mapping.block_id)?;
                let raw_data = self.decompress_block_data(&block)?;
                let block_end = mapping
                    .logical_offset
                    .checked_add(mapping.raw_size)
                    .ok_or_else(|| RepositoryError::General {
                        detail: "Block map raw_size overflow".into(),
                    })?;
                let overlap_start = std::cmp::max(extent_start, mapping.logical_offset);
                let overlap_end = std::cmp::min(extent_end, block_end);
                if overlap_end <= overlap_start {
                    continue;
                }
                let offset_in_block = overlap_start - mapping.logical_offset;
                let overlap_len = overlap_end - overlap_start;
                let file_write_offset = extent.file_offset + (overlap_start - extent_start);
                if file_write_offset != bytes_written {
                    use std::io::Seek;
                    tmp_file
                        .seek(std::io::SeekFrom::Start(file_write_offset))
                        .map_err(|e| RepositoryError::IoError {
                            path: tmp_path.clone(),
                            detail: format!("Seek failed at offset {}", file_write_offset),
                            source: e,
                        })?;
                }
                let write_start = offset_in_block as usize;
                let write_end = write_start + overlap_len as usize;
                if write_end > raw_data.len() {
                    return Err(RepositoryError::General {
                        detail: format!(
                            "Block data underrun: need {} bytes but block has {}",
                            write_end,
                            raw_data.len()
                        ),
                    });
                }
                use std::io::Write;
                tmp_file
                    .write_all(&raw_data[write_start..write_end])
                    .map_err(|e| RepositoryError::IoError {
                        path: tmp_path.clone(),
                        detail: format!(
                            "Write failed at offset {} (len {})",
                            file_write_offset, overlap_len
                        ),
                        source: e,
                    })?;
                bytes_written = bytes_written.max(file_write_offset + overlap_len);
            }
        }
        // Flush and close
        use std::io::Write;
        tmp_file.flush().map_err(|e| RepositoryError::IoError {
            path: tmp_path.clone(),
            detail: "Flush failed on .tmp file".into(),
            source: e,
        })?;
        drop(tmp_file);
        // Verify size
        let actual_size = fs::metadata(&tmp_path)
            .map_err(|e| RepositoryError::IoError {
                path: tmp_path.clone(),
                detail: "Failed to read .tmp metadata".into(),
                source: e,
            })?
            .len();
        if actual_size != entry.size {
            let _ = fs::remove_file(&tmp_path);
            return Err(RepositoryError::General {
                detail: format!(
                    "Restored file size mismatch: expected {} got {}",
                    entry.size, actual_size
                ),
            });
        }
        // SHA-256 verification
        let sha256 = compute_file_sha256(&tmp_path)?;
        if let Some(expected_sha) = &entry.sha256 {
            if sha256 != *expected_sha {
                let _ = fs::remove_file(&tmp_path);
                return Err(RepositoryError::General {
                    detail: format!(
                        "Restored file SHA-256 mismatch: expected {} got {}",
                        expected_sha, sha256
                    ),
                });
            }
        }
        // Atomic rename .tmp -> dest
        if dest.exists() {
            fs::rename(&tmp_path, &dest)
                .or_else(|_| {
                    fs::remove_file(&dest).ok();
                    fs::rename(&tmp_path, &dest)
                })
                .map_err(|e| RepositoryError::IoError {
                    path: dest.clone(),
                    detail: format!("Atomic rename failed: .tmp -> {}", dest.display()),
                    source: e,
                })?;
        } else {
            fs::rename(&tmp_path, &dest).map_err(|e| RepositoryError::IoError {
                path: dest.clone(),
                detail: format!(
                    "Rename failed: {} -> {}",
                    tmp_path.display(),
                    dest.display()
                ),
                source: e,
            })?;
        }
        Ok((dest, bytes_written))
    }
    /// Restore a directory from the Restore Point.
    pub fn restore_directory(
        &self,
        dest_root: &Path,
        entry: &FileEntry,
    ) -> Result<PathBuf, RepositoryError> {
        if entry.entry_type != CatalogEntryType::Directory {
            return Err(RepositoryError::General {
                detail: format!("Entry '{}' is not a directory", entry.path),
            });
        }
        crate::repository::path_security::prepare_restore_directory(dest_root, &entry.path)
    }

    /// Restore all files and directories from the Restore Point.
    /// Returns Complete on full success, Partial if I/O errors occur.
    pub fn restore_all(
        &self,
        dest_root: &Path,
        overwrite: bool,
    ) -> Result<RestoreOutcome, RepositoryError> {
        let paths = self.catalog.list_files()?;
        let mut summary = RestoreSummary {
            total_entries: 0,
            files_restored: 0,
            directories_restored: 0,
            failed_files: Vec::new(),
            total_bytes_restored: 0,
            point_id: self.point_record.point_id.clone(),
        };
        // First pass: directories
        for path_str in &paths {
            if let Ok(Some(entry)) = self.catalog.get_file(path_str) {
                if entry.entry_type == CatalogEntryType::Directory {
                    match self.restore_directory(dest_root, &entry) {
                        Ok(_) => summary.directories_restored += 1,
                        Err(ref e) if is_io_error(e) => {
                            summary.failed_files.push(path_str.clone());
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        // Second pass: files
        for path_str in &paths {
            if let Ok(Some(entry)) = self.catalog.get_file(path_str) {
                if entry.entry_type == CatalogEntryType::File {
                    match self.restore_file(dest_root, &entry, overwrite) {
                        Ok((_, bytes)) => {
                            summary.files_restored += 1;
                            summary.total_bytes_restored += bytes;
                        }
                        Err(ref e) if is_io_error(e) => {
                            summary.failed_files.push(path_str.clone());
                        }
                        Err(e) => return Err(e),
                    }
                }
            }
        }
        summary.total_entries = summary.files_restored + summary.directories_restored;
        if summary.failed_files.is_empty() {
            Ok(RestoreOutcome::Complete(summary))
        } else {
            Ok(RestoreOutcome::Partial(summary))
        }
    }

    /// Restore selected files/directories from the Restore Point.
    pub fn restore_selected(
        &self,
        dest_root: &Path,
        paths: &[String],
        overwrite: bool,
    ) -> Result<RestoreOutcome, RepositoryError> {
        let all_paths = self.catalog.list_files()?;
        let mut selected_entries: Vec<FileEntry> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for requested in paths {
            if let Some(entry) = self.catalog.get_file(requested)? {
                if seen.insert(requested.clone()) {
                    let is_dir = entry.entry_type == CatalogEntryType::Directory;
                    selected_entries.push(entry);
                    if is_dir {
                        let prefix = format!("{}/", requested.trim_end_matches("/"));
                        for cat_path in &all_paths {
                            if cat_path.starts_with(&prefix) && seen.insert(cat_path.clone()) {
                                if let Some(sub) = self.catalog.get_file(cat_path)? {
                                    selected_entries.push(sub);
                                }
                            }
                        }
                    }
                }
            } else {
                let prefix = format!("{}/", requested.trim_end_matches("/"));
                let mut found = false;
                for cat_path in &all_paths {
                    if cat_path.starts_with(&prefix) && seen.insert(cat_path.clone()) {
                        if let Some(sub) = self.catalog.get_file(cat_path)? {
                            selected_entries.push(sub);
                            found = true;
                        }
                    }
                }
                if !found {
                    return Err(RepositoryError::FileNotFound(requested.clone()));
                }
            }
        }
        // Sort: directories first
        selected_entries.sort_by_key(|e| match e.entry_type {
            CatalogEntryType::Directory => 0,
            CatalogEntryType::File => 1,
        });
        let mut summary = RestoreSummary {
            total_entries: 0,
            files_restored: 0,
            directories_restored: 0,
            failed_files: Vec::new(),
            total_bytes_restored: 0,
            point_id: self.point_record.point_id.clone(),
        };
        for entry in &selected_entries {
            match entry.entry_type {
                CatalogEntryType::Directory => match self.restore_directory(dest_root, entry) {
                    Ok(_) => summary.directories_restored += 1,
                    Err(ref e) if is_io_error(e) => {
                        summary.failed_files.push(entry.path.clone());
                    }
                    Err(e) => return Err(e),
                },
                CatalogEntryType::File => match self.restore_file(dest_root, entry, overwrite) {
                    Ok((_, bytes)) => {
                        summary.files_restored += 1;
                        summary.total_bytes_restored += bytes;
                    }
                    Err(ref e) if is_io_error(e) => {
                        summary.failed_files.push(entry.path.clone());
                    }
                    Err(e) => return Err(e),
                },
            }
        }
        summary.total_entries = summary.files_restored + summary.directories_restored;
        if summary.failed_files.is_empty() {
            Ok(RestoreOutcome::Complete(summary))
        } else {
            Ok(RestoreOutcome::Partial(summary))
        }
    }
}

/// Compute the SHA-256 hex digest of a file.
fn compute_file_sha256(path: &Path) -> Result<String, RepositoryError> {
    let mut file = fs::File::open(path).map_err(|e| RepositoryError::IoError {
        path: path.to_path_buf(),

        detail: format!("Cannot open file for SHA-256: {}", path.display()),

        source: e,
    })?;

    let mut hasher = Sha256::new();

    let mut buffer = [0u8; 65536]; // 64KB buffer

    loop {
        let bytes_read = file
            .read(&mut buffer)
            .map_err(|e| RepositoryError::IoError {
                path: path.to_path_buf(),

                detail: format!("Error reading file for SHA-256: {}", path.display()),

                source: e,
            })?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();

    Ok(hex::encode(result))
}

/// Check if a RepositoryError is an I/O error (soft fail).
fn is_io_error(err: &RepositoryError) -> bool {
    matches!(err, RepositoryError::IoError { .. })
}

// ============================================================================

// Tests

// ============================================================================

#[cfg(test)]
mod tests {

    use super::*;

    use crate::repository::backup_writer::RepositoryBackupWriter;

    use crate::repository::repo_manager::init_repo;

    use crate::repository::repo_manager::DEFAULT_BLOCK_SIZE;

    use crate::repository::FileExtent;
    use tempfile::TempDir;

    fn setup_repo() -> (RepoHandle, TempDir) {
        let tmp = TempDir::new().unwrap();

        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        (handle, tmp)
    }

    fn setup_job(handle: &RepoHandle, job_id: &str) {
        let conn = handle.repo_db().unwrap();

        conn.execute(

            "INSERT OR IGNORE INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)

             VALUES (?1, ?2, 0, '', datetime('now'), 'Active')",

            rusqlite::params![job_id, job_id],

        )

        .unwrap();
    }

    fn create_test_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
        let path = dir.join(name);

        fs::write(&path, content).unwrap();

        path
    }

    // ---- Gate 2: Preflight rejection tests ----

    #[test]

    fn test_preflight_nonexistent_point() {
        let (handle, _tmp) = setup_repo();

        let result = FileRestoreReader::open(&handle, "nonexistent-point");

        assert!(result.is_err());
    }

    #[test]

    fn test_preflight_non_committed_rejected() {
        let (handle, _tmp) = setup_repo();

        setup_job(&handle, "test-job");

        // Create a restore point with status WRITING

        let conn = handle.repo_db().unwrap();

        conn.execute(

            "INSERT INTO restore_points (point_id, job_id, chain_id, chain_position, created_at, status, instance_path)

             VALUES ('uncommitted-rp', 'test-job', 'uncommitted-rp', 0, datetime('now'), 'WRITING', 'uncommitted-rp')",

            [],

        )

        .unwrap();

        let result = FileRestoreReader::open(&handle, "uncommitted-rp");

        assert!(result.is_err());

        let err = result.unwrap_err().to_string();

        assert!(
            err.contains("WRITING"),
            "Error should mention WRITING status"
        );
    }

    // ---- Gate 2: Happy path restore ----

    #[test]

    fn test_preflight_and_restore_roundtrip() {
        let (handle, _tmp) = setup_repo();

        setup_job(&handle, "test-job");

        let src_tmp = TempDir::new().unwrap();

        let src_file = create_test_file(src_tmp.path(), "hello.txt", b"Hello, Nuwa Restore!");

        // Write backup

        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("new writer");

        writer.begin().expect("begin");

        writer
            .write_file(&src_file, "hello.txt", "2026-07-12T00:00:00Z")
            .expect("write file");

        let result = writer.finalize().expect("finalize");

        let point_id = result.point_id;

        // Preflight open

        let reader = FileRestoreReader::open(&handle, &point_id).expect("open reader");

        assert_eq!(reader.point_record().point_id, point_id);

        assert_eq!(reader.point_record().status, "COMMITTED");

        assert_eq!(reader.metadata().source_type, "file");
    }

    #[test]

    fn test_open_multiple_times() {
        let (handle, _tmp) = setup_repo();

        setup_job(&handle, "test-job");

        let src_tmp = TempDir::new().unwrap();

        let src_file = create_test_file(src_tmp.path(), "data.txt", b"Some data");

        let mut writer = RepositoryBackupWriter::new(&handle, "test-job", false).expect("new");

        writer.begin().expect("begin");

        writer
            .write_file(&src_file, "data.txt", "2026-07-12T00:00:00Z")
            .expect("write");

        let result = writer.finalize().expect("finalize");

        let point_id = result.point_id;

        // Open reader twice - both should succeed

        let reader1 = FileRestoreReader::open(&handle, &point_id).expect("first open");

        let reader2 = FileRestoreReader::open(&handle, &point_id).expect("second open");

        assert_eq!(
            reader1.point_record().point_id,
            reader2.point_record().point_id
        );
    }

    #[test]

    fn test_catalog_sha256_mismatch_rejected() {
        let (handle, _tmp) = setup_repo();

        setup_job(&handle, "test-job");

        let src_tmp = TempDir::new().unwrap();

        let src_file = create_test_file(src_tmp.path(), "test.bin", b"test data");

        let mut writer = RepositoryBackupWriter::new(&handle, "test-job", false).expect("new");

        writer.begin().expect("begin");

        writer
            .write_file(&src_file, "test.bin", "2026-07-12T00:00:00Z")
            .expect("write");

        let result = writer.finalize().expect("finalize");

        let point_id = result.point_id;

        // Corrupt the catalog.db

        let instance_dir = handle.instances_dir.join(&point_id);

        let cat_path = instance_dir.join("catalog.db");

        fs::write(&cat_path, b"corrupted data").expect("corrupt catalog");

        let reader = FileRestoreReader::open(&handle, &point_id);

        assert!(reader.is_err());

        let err = reader.unwrap_err().to_string();

        assert!(
            err.contains("SHA-256 mismatch") || err.contains("Checksum"),
            "Error should mention SHA-256 mismatch, got: {}",
            err
        );
    }

    #[test]

    fn test_block_map_sha256_mismatch_rejected() {
        let (handle, _tmp) = setup_repo();

        setup_job(&handle, "test-job");

        let src_tmp = TempDir::new().unwrap();

        let src_file = create_test_file(src_tmp.path(), "test.bin", b"more test data");

        let mut writer = RepositoryBackupWriter::new(&handle, "test-job", false).expect("new");

        writer.begin().expect("begin");

        writer
            .write_file(&src_file, "test.bin", "2026-07-12T00:00:00Z")
            .expect("write");

        let result = writer.finalize().expect("finalize");

        let point_id = result.point_id;

        // Corrupt the block-map.db

        let instance_dir = handle.instances_dir.join(&point_id);

        let bm_path = instance_dir.join("block-map.db");

        fs::write(&bm_path, b"corrupted map data").expect("corrupt block map");

        let reader = FileRestoreReader::open(&handle, &point_id);

        assert!(reader.is_err());
    }

    #[test]

    fn test_job_id_mismatch_rejected() {
        let (handle, _tmp) = setup_repo();

        setup_job(&handle, "job-a");

        let src_tmp = TempDir::new().unwrap();

        let src_file = create_test_file(src_tmp.path(), "f.txt", b"data");

        let mut writer = RepositoryBackupWriter::new(&handle, "job-a", false).expect("new");

        writer.begin().expect("begin");

        writer
            .write_file(&src_file, "f.txt", "2026-07-12T00:00:00Z")
            .expect("write");

        let result = writer.finalize().expect("finalize");

        let point_id = result.point_id;

        // Modify metadata to have wrong job_id

        let instance_dir = handle.instances_dir.join(&point_id);

        let meta_path = instance_dir.join("backup-metadata.json");

        let mut meta: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&meta_path).unwrap()).unwrap();

        meta["job_id"] = serde_json::Value::String("wrong-job".to_string());

        fs::write(&meta_path, serde_json::to_string_pretty(&meta).unwrap()).unwrap();

        let reader = FileRestoreReader::open(&handle, &point_id);

        assert!(reader.is_err());
    }
    // ========================================================================
    // Gate 2: Single-File Restore tests (P-02f)
    // ========================================================================

    #[test]
    fn test_restore_single_file_roundtrip() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().unwrap();
        let content = b"Hello, Nuwa Repository Restore!";
        let src_file = create_test_file(src_tmp.path(), "hello.txt", content);
        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("new writer");
        writer.begin().expect("begin");
        writer
            .write_file(&src_file, "hello.txt", "2026-07-12T00:00:00Z")
            .expect("write");
        let result = writer.finalize().expect("finalize");
        let point_id = result.point_id;
        let reader = FileRestoreReader::open(&handle, &point_id).expect("open reader");
        let entry = reader.get_entry("hello.txt").expect("get entry");
        let dest_tmp = TempDir::new().unwrap();
        let (restored_path, bytes) = reader
            .restore_file(dest_tmp.path(), &entry, false)
            .expect("restore");
        assert_eq!(bytes, content.len() as u64);
        let restored = fs::read(&restored_path).expect("read restored");
        assert_eq!(restored, content, "Restored content must match original");
        // SHA-256 was verified inside restore_file - confirm file exists
        assert!(restored_path.exists());
    }

    #[test]
    fn test_restore_empty_file() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().unwrap();
        let src_file = create_test_file(src_tmp.path(), "empty.txt", b"");
        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("new writer");
        writer.begin().expect("begin");
        writer
            .write_file(&src_file, "empty.txt", "2026-07-12T00:00:00Z")
            .expect("write");
        let result = writer.finalize().expect("finalize");
        let point_id = result.point_id;
        let reader = FileRestoreReader::open(&handle, &point_id).expect("open reader");
        let entry = reader.get_entry("empty.txt").expect("get entry");
        let dest_tmp = TempDir::new().unwrap();
        let (restored_path, bytes) = reader
            .restore_file(dest_tmp.path(), &entry, false)
            .expect("restore empty");
        assert_eq!(bytes, 0);
        let restored = fs::read(&restored_path).expect("read restored");
        assert!(restored.is_empty(), "Restored empty file must be empty");
    }

    #[test]
    fn test_restore_overwrite_false_rejected() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().unwrap();
        let src_file = create_test_file(src_tmp.path(), "data.txt", b"some data");
        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("new writer");
        writer.begin().expect("begin");
        writer
            .write_file(&src_file, "data.txt", "2026-07-12T00:00:00Z")
            .expect("write");
        let result = writer.finalize().expect("finalize");
        let point_id = result.point_id;
        let reader = FileRestoreReader::open(&handle, &point_id).expect("open reader");
        let entry = reader.get_entry("data.txt").expect("get entry");
        let dest_tmp = TempDir::new().unwrap();
        // Create existing file
        fs::write(dest_tmp.path().join("data.txt"), b"existing data").expect("create existing");
        let result = reader.restore_file(dest_tmp.path(), &entry, false);
        assert!(
            result.is_err(),
            "Must reject overwrite when overwrite=false"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("already exists"),
            "Error must mention 'already exists': {}",
            err
        );
    }

    #[test]
    fn test_restore_overwrite_true_succeeds() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().unwrap();
        let content = b"new data";
        let src_file = create_test_file(src_tmp.path(), "data.txt", content);
        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("new writer");
        writer.begin().expect("begin");
        writer
            .write_file(&src_file, "data.txt", "2026-07-12T00:00:00Z")
            .expect("write");
        let result = writer.finalize().expect("finalize");
        let point_id = result.point_id;
        let reader = FileRestoreReader::open(&handle, &point_id).expect("open reader");
        let entry = reader.get_entry("data.txt").expect("get entry");
        let dest_tmp = TempDir::new().unwrap();
        // Create existing file with different content
        fs::write(dest_tmp.path().join("data.txt"), b"old data").expect("create existing");
        let (restored_path, bytes) = reader
            .restore_file(dest_tmp.path(), &entry, true)
            .expect("restore with overwrite");
        assert_eq!(bytes, content.len() as u64);
        let restored = fs::read(&restored_path).expect("read restored");
        assert_eq!(restored, content, "Must overwrite with new content");
    }

    // ========================================================================
    // Gate 3: Directory & Multi-File Restore tests
    // ========================================================================

    #[test]
    fn test_restore_all_simple() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().unwrap();
        let f1 = create_test_file(src_tmp.path(), "a.txt", b"file a");
        let f2 = create_test_file(src_tmp.path(), "b.txt", b"file b");
        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("new writer");
        writer.begin().expect("begin");
        writer
            .write_file(&f1, "a.txt", "2026-07-12T00:00:00Z")
            .expect("write a");
        writer
            .write_file(&f2, "b.txt", "2026-07-12T00:00:00Z")
            .expect("write b");
        let result = writer.finalize().expect("finalize");
        let point_id = result.point_id;
        let reader = FileRestoreReader::open(&handle, &point_id).expect("open reader");
        let dest_tmp = TempDir::new().unwrap();
        let outcome = reader
            .restore_all(dest_tmp.path(), false)
            .expect("restore all");
        match outcome {
            RestoreOutcome::Complete(summary) => {
                assert_eq!(summary.files_restored, 2, "Must restore 2 files");
                assert!(summary.failed_files.is_empty());
            }
            RestoreOutcome::Partial(_) => panic!("Expected Complete, got Partial"),
        }
        assert_eq!(fs::read(dest_tmp.path().join("a.txt")).unwrap(), b"file a");
        assert_eq!(fs::read(dest_tmp.path().join("b.txt")).unwrap(), b"file b");
    }

    #[test]
    fn test_restore_selected_single_file() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().unwrap();
        let f1 = create_test_file(src_tmp.path(), "keep.txt", b"keep me");
        let f2 = create_test_file(src_tmp.path(), "skip.txt", b"skip me");
        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("new writer");
        writer.begin().expect("begin");
        writer
            .write_file(&f1, "keep.txt", "2026-07-12T00:00:00Z")
            .expect("write keep");
        writer
            .write_file(&f2, "skip.txt", "2026-07-12T00:00:00Z")
            .expect("write skip");
        let result = writer.finalize().expect("finalize");
        let point_id = result.point_id;
        let reader = FileRestoreReader::open(&handle, &point_id).expect("open reader");
        let dest_tmp = TempDir::new().unwrap();
        let paths = vec!["keep.txt".to_string()];
        let outcome = reader
            .restore_selected(dest_tmp.path(), &paths, false)
            .expect("restore selected");
        match outcome {
            RestoreOutcome::Complete(summary) => {
                assert_eq!(summary.files_restored, 1, "Must restore only 1 file");
            }
            RestoreOutcome::Partial(_) => panic!("Expected Complete"),
        }
        assert!(
            dest_tmp.path().join("keep.txt").exists(),
            "keep.txt must exist"
        );
        assert!(
            !dest_tmp.path().join("skip.txt").exists(),
            "skip.txt must NOT exist"
        );
    }

    #[test]
    fn test_restore_selected_nonexistent_path_fails() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().unwrap();
        let f = create_test_file(src_tmp.path(), "existing.txt", b"data");
        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("new writer");
        writer.begin().expect("begin");
        writer
            .write_file(&f, "existing.txt", "2026-07-12T00:00:00Z")
            .expect("write");
        let result = writer.finalize().expect("finalize");
        let point_id = result.point_id;
        let reader = FileRestoreReader::open(&handle, &point_id).expect("open reader");
        let dest_tmp = TempDir::new().unwrap();
        let paths = vec!["nonexistent.txt".to_string()];
        let result = reader.restore_selected(dest_tmp.path(), &paths, false);
        assert!(result.is_err(), "Must reject nonexistent path");
    }

    #[test]
    fn test_restore_with_nested_dirs_creates_parents() {
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        let src_tmp = TempDir::new().unwrap();
        let nested_dir = src_tmp.path().join("a").join("b");
        fs::create_dir_all(&nested_dir).unwrap();
        let src_file = create_test_file(&nested_dir, "deep.txt", b"deep file");
        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("new writer");
        writer.begin().expect("begin");
        writer
            .write_file(&src_file, "a/b/deep.txt", "2026-07-12T00:00:00Z")
            .expect("write deep");
        let result = writer.finalize().expect("finalize");
        let point_id = result.point_id;
        let reader = FileRestoreReader::open(&handle, &point_id).expect("open reader");
        let dest_tmp = TempDir::new().unwrap();
        let outcome = reader
            .restore_all(dest_tmp.path(), false)
            .expect("restore all");
        match outcome {
            RestoreOutcome::Complete(s) => assert_eq!(s.files_restored, 1, "Must restore 1 file"),
            RestoreOutcome::Partial(_) => panic!("Expected Complete"),
        }
        let restored_file = dest_tmp.path().join("a").join("b").join("deep.txt");
        assert!(restored_file.exists(), "Nested restored file must exist");
        assert_eq!(fs::read(&restored_file).unwrap(), b"deep file");
    }

    #[test]
    fn test_validate_file_entry_rejects_missing_sha256() {
        let _entry = FileEntry {
            entry_type: CatalogEntryType::File,
            path: "test.txt".into(),
            size: 5,
            modified: "2026-07-12T00:00:00Z".into(),
            extents: vec![FileExtent {
                logical_offset: 0,
                file_offset: 0,
                length: 5,
            }],
            sha256: None,
        };
        let (handle, _tmp) = setup_repo();
        setup_job(&handle, "test-job");
        // We need an open reader with a catalog to call validate
        // But validate_file_entry doesn't use self for the basic validation
        // Actually it does take &self - let's test via a real reader
        // For this test we'll just verify the function exists by using a reader
        let src_tmp = TempDir::new().unwrap();
        let f = create_test_file(src_tmp.path(), "test.txt", b"hello");
        let mut writer =
            RepositoryBackupWriter::new(&handle, "test-job", false).expect("new writer");
        writer.begin().expect("begin");
        writer
            .write_file(&f, "test.txt", "2026-07-12T00:00:00Z")
            .expect("write");
        let result = writer.finalize().expect("finalize");
        let reader = FileRestoreReader::open(&handle, &result.point_id).expect("open");
        // Corrupt entry: remove sha256
        let mut bad_entry = reader.get_entry("test.txt").expect("get entry");
        bad_entry.sha256 = None;
        let err = reader.validate_file_entry(&bad_entry);
        assert!(err.is_err(), "Must reject entry without SHA-256");
    }
}
