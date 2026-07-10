// ============================================================================
// engine.rs — CatalogEngine trait and data models
// ============================================================================
//
// CatalogEngine provides file-level metadata for a Restore Point.
// It answers: "what files were backed up, and where are their extents?"
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
use std::path::PathBuf;

/// A contiguous range in the logical address space occupied by a file.
///
/// The logical address space is defined per-data-source-type:
/// - File backup: logical_offset = byte offset within the backup stream
/// - Volume backup: logical_offset = byte offset from volume start (future)
/// - Disk backup: logical_offset = byte offset from disk start (future)
///
/// To find the actual block(s) for this extent, pass [logical_offset, logical_offset + length)
/// to BlockMapEngine::get_range().
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileExtent {
    /// Starting offset in the logical address space
    pub logical_offset: u64,
    /// Number of bytes in this extent
    pub length: u64,
}

/// Metadata about one backed-up file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    /// Full relative path from the backup source root
    pub path: String,
    /// Total file size in bytes
    pub size: u64,
    /// ISO-8601 modification timestamp
    pub modified: String,
    /// List of extents describing where this file's data lives
    /// in the logical address space
    pub extents: Vec<FileExtent>,
}

/// CatalogEngine trait — provides file-level metadata for a Restore Point.
///
/// # Contract
/// - add_file: records a file and its extents
/// - get_file: retrieves a file entry by its path
/// - list_files: returns all file paths in the catalog
/// - file_count: returns the total number of files
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
    /// # Arguments
    /// * `path` — Relative path from backup source root
    /// * `size` — Total file size in bytes
    /// * `modified` — ISO-8601 modification timestamp
    /// * `extents` — Ordered list of extents for this file
    fn add_file(
        &mut self,
        path: &str,
        size: u64,
        modified: &str,
        extents: Vec<FileExtent>,
    ) -> Result<(), RepositoryError>;

    /// Retrieve a file entry by its path.
    /// Returns None if the file is not in the catalog.
    fn get_file(&self, path: &str) -> Result<Option<FileEntry>, RepositoryError>;

    /// List all file paths in the catalog.
    /// Returns them in lexicographic order.
    fn list_files(&self) -> Result<Vec<String>, RepositoryError>;

    /// Return the total number of file entries in the catalog.
    fn file_count(&self) -> Result<u64, RepositoryError>;

    /// Close the CatalogEngine and return the path to its database file.
    fn close(self: Box<Self>) -> Result<PathBuf, RepositoryError>;
}
