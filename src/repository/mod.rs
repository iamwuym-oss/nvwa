// ============================================================================
// mod.rs — Phase S Repository Engine module entry point
// ============================================================================
//
// This module implements Nuwa unified backup Repository Engine.
// Architecture v1.0 frozen baseline. See docs/phase-s/ for full documentation.
//
// Current: Wave 1 (S-01 + S-02 + S-03)
// Repository init, Block Store, Metadata Engine
//
// Key principles:
// - Repository Engine does NOT know data source types
// - Block identity = SHA-256(raw data), NOT compressed data
// - Block size is fixed at repository init time

pub mod block_store;
pub mod error;
pub mod metadata;
pub mod repo_manager;

// Re-export key types at the repository level for convenience
pub use block_store::block_id::BlockId;
pub use block_store::store::{Block, BlockStore};
pub use error::RepositoryError;
pub use repo_manager::{
    check_repo, init_repo, is_repository, open_repo, RepoHandle, RepositoryInfo, DEFAULT_BLOCK_SIZE,
};
