// ============================================================================
// engine.rs — BlockMapEngine trait definition
// ============================================================================
//
// BlockMapEngine provides the mapping from logical_offset → block_id.
// This is the critical index for reconstructing data from blocks.
//
// Key architecture rules:
// - BlockMapEngine does NOT know about files, only logical addresses
// - logical_offset is a position in the source data address space
// - NOT rebuildable from block-store — see Architecture §2.3
// - Per-Backup-Instance isolation: one BlockMap per Restore Point

use crate::repository::block_store::block_id::BlockId;
use crate::repository::error::RepositoryError;
use std::path::PathBuf;

/// A single mapping entry from logical_offset to block_id
#[derive(Debug, Clone)]
pub struct BlockMapEntry {
    /// Starting offset in the logical address space
    pub logical_offset: u64,
    /// Block identity
    pub block_id: BlockId,
    /// Size of the raw (uncompressed) data in this block
    pub raw_size: u64,
}

/// BlockMapEngine trait — maps logical addresses to block identifiers.
///
/// # Contract
/// - insert_mapping: records that data at logical_offset is stored in block_id
/// - get_block: finds which block covers the given logical_offset (exact match on start)
/// - get_range: returns all entries in [start, end) offset range
/// - close: finalizes and returns the path to the database file
///
/// # Architecture constraints (§2.3, §2.4)
/// - Block Map is NOT rebuildable from block-store alone
/// - Metadata (block-map.db) is the sole logical view of data
/// - Losing block-map.db leaves data uninterpretable
///
/// # Per-Backup-Instance
/// Each Restore Point has its own BlockMap. There is no global block index.
pub trait BlockMapEngine {
    /// Insert a mapping from logical_offset to block_id.
    ///
    /// # Arguments
    /// * `logical_offset` — Starting offset in the logical address space
    /// * `block_id` — Identity of the block containing this offset's data
    /// * `raw_size` — Size of raw (uncompressed) data in this block
    fn insert_mapping(
        &mut self,
        logical_offset: u64,
        block_id: &BlockId,
        raw_size: u64,
    ) -> Result<(), RepositoryError>;

    /// Find which block covers the given logical_offset.
    /// Returns None if no mapping exists at that exact offset.
    fn get_block(&self, logical_offset: u64) -> Result<Option<BlockMapEntry>, RepositoryError>;

    /// Return all mappings in the [start, end) offset range.
    /// Entries are returned in ascending logical_offset order.
    fn get_range(&self, start: u64, end: u64) -> Result<Vec<BlockMapEntry>, RepositoryError>;

    /// Return the total number of mappings in this BlockMap.
    fn block_count(&self) -> Result<u64, RepositoryError>;

    /// Close the BlockMapEngine and return the path to its database file.
    /// After calling close, the engine should not be used further.
    fn close(self: Box<Self>) -> Result<PathBuf, RepositoryError>;
}
