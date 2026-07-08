// ============================================================================
// backup.rs -- Backup domain API models
//
// These types form the "contract" between the Tauri command layer and the
// React Backup page. They are designed to be:
//   - Independent of core backup.rs types (no internal NuwaError leaks)
//   - Serializable to JSON for Tauri IPC
//   - Extensible for future features (volume backup, encryption, etc.)
// ============================================================================

use serde::{Deserialize, Serialize};

/// Status of a single backup job
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BackupJobStatus {
    /// Job configured and last backup succeeded
    Active,
    /// Job configured but never run
    NeverRun,
    /// Job configured but last backup failed
    Error,
    /// Job configuration has issues (source/dest missing)
    Misconfigured,
}

/// Backup job as seen by the UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJobView {
    /// Unique job name from config
    pub name: String,
    /// Source path being backed up
    pub source: String,
    /// Destination path for backup storage
    pub dest: String,
    /// Whether compression is enabled
    pub compress: bool,
    /// Retention policy summary (e.g. "Keep 7 versions" or "Keep 30 days")
    pub retention: String,
    /// Current job status
    pub status: BackupJobStatus,
    /// ISO 8601 timestamp of last backup (None if never run)
    pub last_backup_time: Option<String>,
    /// Status of last backup: "success" | "failure" | "partial" | null
    pub last_backup_status: Option<String>,
    /// Number of files in last backup
    pub last_backup_files: u64,
    /// Size of last backup in bytes
    pub last_backup_bytes: u64,
}

/// Request to execute a backup for a specific job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRequest {
    /// Name of the job to run
    pub job_name: String,
}

/// Result of a completed backup operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupResult {
    /// Unique backup point identifier
    pub backup_id: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Number of files backed up
    pub file_count: u64,
    /// Total data size in bytes
    pub total_bytes: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Status: "success" | "failure" | "partial"
    pub status: String,
    /// Error detail if failed
    pub error: Option<String>,
}
