// ============================================================================
// mod.rs — Catalog Engine module entry point (P-00C: added CatalogEntryType)
// ============================================================================
//
// Phase S Wave 2: Catalog provides file-level metadata for Restore Points.
// Per-Backup-Instance SQLite database, NOT rebuildable from block-store.
// See Architecture v1.0 §10.

pub mod engine;
pub mod sqlite_catalog;

pub use engine::{CatalogEngine, CatalogEntryType, FileEntry, FileExtent};
pub use sqlite_catalog::SqliteCatalog;
