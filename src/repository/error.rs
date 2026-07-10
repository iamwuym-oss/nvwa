// ============================================================================
// error.rs — RepositoryError types for Phase S Repository Engine
// ============================================================================

use std::path::PathBuf;
use thiserror::Error;

/// Repository-level errors.
/// Each variant provides a user-facing message and a source where applicable.
/// CLI commands map these to exit codes for script consumption.
#[derive(Error, Debug)]
pub enum RepositoryError {
    // ======== Repository Initialization ========
    #[error("Repository already exists at {0}")]
    AlreadyExists(PathBuf),

    #[error("No repository found at {0}. Use 'nuwa repo init' to create one.")]
    NotFound(PathBuf),

    #[error("Invalid repository at {0}: {1}")]
    InvalidRepository(PathBuf, String),

    #[error("Repository version {0} is not supported (expected 1)")]
    UnsupportedVersion(u32),

    #[error("Repository format version {0} is not supported by this software version (min_compatible={1})")]
    UnsupportedRepositoryVersion(u32, u32),

    #[error("Repository identity mismatch: {field} ({v1} vs {v2})")]
    RepositoryIdentityMismatch {
        field: String,
        v1: String,
        v2: String,
    },

    // ======== Block Store ========
    #[error("Block not found: {0}")]
    BlockNotFound(String),

    #[error("Block checksum mismatch: expected {expected}, got {actual}")]
    BlockCorrupted { expected: String, actual: String },

    #[error("Block header invalid at {path}: {detail}")]
    InvalidBlockHeader { path: PathBuf, detail: String },

    #[error("Block store corrupted at root {root}: {detail}")]
    BlockStoreCorrupted { root: PathBuf, detail: String },

    // ======== Metadata ========
    #[error("Metadata file not found: {0}")]
    MetadataMissing(PathBuf),

    #[error("Metadata schema version {0} is not supported")]
    UnsupportedSchemaVersion(String),

    #[error("Metadata field missing or invalid: {0}")]
    InvalidMetadata(String),

    // ======== Block Map ========
    #[error("Block map corrupted at {path}: {detail}")]
    BlockMapCorrupted { path: PathBuf, detail: String },

    #[error("Logical offset {0} not found in block map")]
    OffsetNotFound(u64),

    // ======== Catalog ========
    #[error("Catalog corrupted at {path}: {detail}")]
    CatalogCorrupted { path: PathBuf, detail: String },

    #[error("File not found in catalog: {0}")]
    FileNotFound(String),

    // ======== Crash Consistency ========
    #[error(
        "Incomplete transaction detected for restore point {0}. Run 'nuwa repo check' to recover."
    )]
    IncompleteTransaction(String),

    #[error("Transaction consistency check failed for {point}: {detail}")]
    TransactionInconsistent { point: String, detail: String },

    // ======== Recovery ========
    #[error(
        "Component not rebuildable: {0}\nblock-map and catalog cannot be rebuilt from block-store."
    )]
    NotRebuildable(String),

    #[error("Repository self-check failed: {0}")]
    SelfCheckFailed(String),

    // ======== I/O ========
    #[error("I/O error at {path}: {detail} ({source})")]
    IoError {
        path: PathBuf,
        detail: String,
        source: std::io::Error,
    },

    // ======== Serialization ========
    #[error("JSON serialization error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("SQLite error: {0}")]
    SqliteError(#[from] rusqlite::Error),

    #[error("General repository error: {detail}")]
    General { detail: String },
}

impl From<std::io::Error> for RepositoryError {
    fn from(e: std::io::Error) -> Self {
        RepositoryError::IoError {
            path: PathBuf::new(),
            detail: e.to_string(),
            source: e,
        }
    }
}

impl RepositoryError {
    /// Create a block not found error from a block ID hex
    pub fn block_not_found(hex: &str) -> Self {
        RepositoryError::BlockNotFound(hex.to_string())
    }

    /// Create an invalid block header error
    pub fn invalid_block_header(path: PathBuf, detail: impl Into<String>) -> Self {
        RepositoryError::InvalidBlockHeader {
            path,
            detail: detail.into(),
        }
    }

    /// Create an I/O error with path context
    pub fn io(path: PathBuf, detail: impl Into<String>, source: std::io::Error) -> Self {
        RepositoryError::IoError {
            path,
            detail: detail.into(),
            source,
        }
    }
}
