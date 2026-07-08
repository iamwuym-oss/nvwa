// ============================================================================
// restore.rs -- Restore domain API models
//
// These types form the "contract" between the Tauri command layer and the
// React Restore page. They are designed to be:
//   - Independent of core restore.rs types
//   - Serializable to JSON for Tauri IPC
//   - Extensible for future features (volume restore, etc.)
// ============================================================================

use serde::{Deserialize, Serialize};

/// A backup point available for restore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePointView {
    /// Unique backup point identifier
    pub backup_id: String,
    /// Backup job name this point belongs to
    pub job_name: Option<String>,
    /// ISO 8601 timestamp when the backup was created
    pub timestamp: String,
    /// Source path that was backed up
    pub source_root: String,
    /// Destination path where the backup is stored
    pub dest_path: String,
    /// Number of files in this backup point
    pub file_count: u64,
    /// Total data size in bytes
    pub total_bytes: u64,
    /// Status: "success" | "failure" | "partial"
    pub status: String,
}

/// A single file entry inside a backup point (for restore preview).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreFileEntry {
    /// Relative path from the source root
    pub relative_path: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Last modification time
    pub modified_time: String,
}

/// Preview of files that will be restored from a backup point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePreview {
    /// Backup point metadata
    pub point: RestorePointView,
    /// List of files in this backup point
    pub files: Vec<RestoreFileEntry>,
    /// Total file count
    pub total_files: u64,
    /// Total data size
    pub total_bytes: u64,
}

/// Request to execute a restore operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreRequest {
    /// Backup point ID to restore from
    pub backup_id: String,
    /// Destination path for restored files
    pub dest: String,
    /// Whether to overwrite existing files
    pub overwrite: bool,
}

/// Result of a completed restore operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreOperationResult {
    /// Unique restore operation identifier
    pub restore_id: String,
    /// Number of files successfully restored
    pub restored_count: u64,
    /// Number of files skipped
    pub skipped_count: u64,
    /// Number of checksum failures
    pub checksum_failures: u64,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Status: "success" | "failure" | "partial"
    pub status: String,
    /// Error detail if failed
    pub error: Option<String>,
}
