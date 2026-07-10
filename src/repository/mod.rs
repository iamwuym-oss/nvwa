// ============================================================================
// mod.rs - Phase S Repository Engine module entry point
// ============================================================================
//
// This module implements Nuwa unified backup Repository Engine.
// Architecture v1.1 frozen baseline. See docs/phase-s/ for full documentation.
//
// Completed Modules:
//   Wave 1: S-01 repo_manager, S-02 block_store, S-03 metadata
//   Wave 2: S-04 block_map, S-05 catalog
//   Wave 3: S-06 chain, S-07 transaction/crash_consistency
//   Wave 4: S-08 verify, S-09 retention, S-13 recovery
//   Wave 5: S-10 legacy (partial)
//
// Key principles:
// - Repository Engine does NOT know data source types
// - Block identity = SHA-256(raw data), NOT compressed data
// - Block size is fixed at repository init time
// - Block Map is NOT rebuildable from block-store
// - Catalog stores FileExtent, NOT block references
// - Transaction journal is the SOURCE OF TRUTH for crash recovery
// - Verify Engine is READ-ONLY - never modifies data

pub mod block_map;
pub mod block_store;
pub mod catalog;
pub mod chunk_engine;
pub mod cli;
pub mod error;
pub mod legacy;
pub mod metadata;
pub mod recovery;
pub mod repo_manager;
pub mod retention;
pub mod transaction;
pub mod verify;

// Re-export key types at the repository level for convenience
pub use block_map::engine::{BlockMapEngine, BlockMapEntry};
pub use block_map::sqlite_block_map::SqliteBlockMap;
pub use block_store::block_id::BlockId;
pub use block_store::store::{Block, BlockStore};
pub use catalog::engine::{CatalogEngine, FileEntry, FileExtent};
pub use catalog::sqlite_catalog::SqliteCatalog;
pub use chunk_engine::{ChunkEngine, ChunkPolicy, ChunkResult, FixedChunkPolicy};
pub use error::RepositoryError;
pub use legacy::adapter::{FlatFileAdapter, LegacyAdapter, LegacyRestorePoint, RestoreResult};
pub use recovery::integrity_check::{
    check_integrity, CheckDetail, CheckResult, CheckStatus, IntegrityReport, IntegritySummary,
};
pub use recovery::rebuild::rebuild_repo;
pub use repo_manager::{
    check_repo, init_repo, is_repository, open_repo, RepoHandle, RepositoryInfo, DEFAULT_BLOCK_SIZE,
};
pub use retention::engine::{
    apply_retention, count_orphan_candidates, list_orphan_candidates, recover_incomplete_deletions,
    OrphanSummary, RetentionPolicy, RetentionResult,
};
pub use transaction::journal::{
    ComponentPhase, ComponentStatus, TransactionJournal, TransactionState,
};
pub use transaction::manager::{CrashConsistencyManager, RecoveryReport};
pub use verify::engine::{verify_repo, VerifyLevel, VerifyReport};
