// ============================================================================
// mod.rs — Block Map Engine module entry point
// ============================================================================
//
// Phase S Wave 2: BlockMap maps logical_offset → block_id.
// Per-Backup-Instance SQLite database, NOT rebuildable from block-store.
// See Architecture v1.0 §9.

pub mod engine;
pub mod sqlite_block_map;

pub use engine::{BlockMapEngine, BlockMapEntry};
pub use sqlite_block_map::SqliteBlockMap;
