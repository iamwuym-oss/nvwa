// ============================================================================
// engine.rs — CatalogEngine trait and data models (P-00C: entry_type, sha256, file_offset)
// ============================================================================
//
// P-00C changes per P-00_File_Backup_Repository_Data_Contract.md v0.7:
// - Added CatalogEntryType (File | Directory) to distinguish files from dirs
// - Added file_offset to FileExtent for per-file extent reconstruction
// - Added sha256 to FileEntry for per-file integrity verification
// - Added add_directory() to trait for explicit directory recording
//
// Key architecture rules:
// - Catalog stores FileExtent (logical_offset + length), NOT block references
// - Catalog does NOT know about blocks — BlockMap maps offset → block
// - Per-Backup-Instance isolation: one Catalog per Restore Point
// - NOT rebuildable from block-store — see Architecture §2.3
//
// Catalog ↔ BlockMap bridge:
//   Catalog: file → [FileExtent{logical_offset, length}]
//   BlockMap: logical_offset → block_id
//
// To restore a file:
//   1. Catalog.get_file(path) → FileEntry with extents
//   2. For each extent: BlockMap.get_range(offset, offset+length) → block_ids
//   3. BlockStore.get_block(block_id) → raw data

use crate::repository::error::RepositoryError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Type of entry in the catalog.
///
/// P-00 §4.5: Distinguishes files (with SHA-256) from directories (no data).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CatalogEntryType {
    /// A regular file. Must have sha256 = Some(...), extents may be empty (empty file).
    File,
    /// An empty directory. size = 0, sha256 = None, extents = [].
    Directory,
}

impl CatalogEntryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CatalogEntryType::File => "file",
            CatalogEntryType::Directory => "directory",
        }
    }
}

/// A contiguous range in the logical address space occupied by a file.
///
/// The logical address space is defined per-data-source-type:
/// - File backup: logical_offset = byte offset within the backup stream
/// - Volume backup: logical_offset = byte offset from volume start (future)
/// - Disk backup: logical_offset = byte offset from disk start (future)
///
/// To find the actual block(s) for this extent, pass [logical_offset, logical_offset + length)
/// to BlockMapEngine::get_range().
///
/// P-00C added file_offset: per-file offset within the source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileExtent {
    /// Starting offset in the logical address space (global per Backup Instance)
    pub logical_offset: u64,
    /// Offset within the source file (P-00 §1.4: for per-file reconstruction)
    pub file_offset: u64,
    /// Number of bytes in this extent
    pub length: u64,
}

/// Metadata about one backed-up file.
///
/// P-00C added entry_type and sha256 fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// Type of catalog entry: file or directory
    pub entry_type: CatalogEntryType,
    /// Full relative path from the backup source root
    pub path: String,
    /// Total file size in bytes
    pub size: u64,
    /// ISO-8601 modification timestamp
    pub modified: String,
    /// List of extents describing where this file's data lives
    /// in the logical address space.
    /// Empty for directories and empty files.
    pub extents: Vec<FileExtent>,
    /// SHA-256 hex digest. Required for files; None for directories (P-00 §4.5)
    pub sha256: Option<String>,
}

/// CatalogEngine trait — provides file-level metadata for a Restore Point.
///
/// # Contract
/// - add_file: records a file and its extents
/// - add_directory: records an empty directory (no data)
/// - get_file: retrieves a file entry by its path
/// - list_files: returns all file paths in the catalog
/// - file_count: returns the total number of entries
/// - close: finalizes and returns the database path
///
/// # Per-Backup-Instance
/// Each Restore Point has its own Catalog. There is no global file index.
pub trait CatalogEngine {
    /// Add or update a file entry in the catalog.
    ///
    /// If the file already exists (by path), the existing entry is replaced.
    /// All extents for this file are provided together.
    ///
    /// P-00 §4.5: sha256 is required for files. For directories, use add_directory().
    /// P-00 §1.4: Each extent must include file_offset for per-file reconstruction.
    ///
    /// # Arguments
    /// * `path` — Relative path from backup source root
    /// * `size` — Total file size in bytes
    /// * `modified` — ISO-8601 modification timestamp
    /// * `sha256` — SHA-256 hex digest of file content
    /// * `extents` — Ordered list of extents for this file
    fn add_file(
        &mut self,
        path: &str,
        size: u64,
        modified: &str,
        sha256: Option<String>,
        extents: Vec<FileExtent>,
    ) -> Result<(), RepositoryError>;

    /// Add an empty directory entry to the catalog.
    ///
    /// P-00 §4.5: Directories have size=0, sha256=None, extents=[].
    /// The directory is recorded so that empty directories can be recreated during restore.
    fn add_directory(&mut self, path: &str, modified: &str) -> Result<(), RepositoryError>;

    /// Retrieve a file entry by its path.
    /// Returns None if the file is not in the catalog.
    fn get_file(&self, path: &str) -> Result<Option<FileEntry>, RepositoryError>;

    /// List all file paths in the catalog.
    /// Returns them in lexicographic order.
    fn list_files(&self) -> Result<Vec<String>, RepositoryError>;

    /// Return the total number of entries in the catalog.
    fn file_count(&self) -> Result<u64, RepositoryError>;

    /// Close the CatalogEngine and return the path to its database file.
    fn close(self: Box<Self>) -> Result<PathBuf, RepositoryError>;
}
