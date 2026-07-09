// ============================================================================
// history.rs -- History domain API models
//
// These types form the "contract" between the Tauri command layer and the
// React History page. They are designed to be:
//   - Independent of core history.rs types
//   - Serializable to JSON for Tauri IPC
//   - Grouped by operation type for UI filtering
// ============================================================================

use serde::{Deserialize, Serialize};

/// A single historical operation record displayed on the History page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryRecordView {
    /// Unique backup/operation identifier
    pub backup_id: String,
    /// Operation type: "backup" | "restore" | "verify"
    pub operation: String,
    /// ISO 8601 timestamp when the operation occurred
    pub timestamp: String,
    /// Source path that was operated on
    pub source_root: String,
    /// Destination/backup storage path
    pub dest_path: String,
    /// Backup job name this record belongs to (if available)
    pub job_name: Option<String>,
    /// Number of files processed
    pub file_count: u64,
    /// Total data size in bytes
    pub total_bytes: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Status: "success" | "failure" | "partial"
    pub status: String,
    /// Human-readable exit code description
    pub exit_info: String,
}

/// Paginated/aggregated history query result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryQueryResult {
    /// Total matching records across all pages
    pub total: u32,
    /// Page of records returned
    pub records: Vec<HistoryRecordView>,
}

/// Supported filter options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryFilter {
    /// Maximum records to return (default 50, max 200)
    pub limit: Option<u32>,
    /// Operation type filter: "backup" | "restore" | "verify" | null (all)
    pub operation: Option<String>,
}
