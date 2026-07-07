// ============================================================================
// dashboard.rs -- Dashboard API model (DashboardOverview + its sub-types)
//
// This is the "contract" between the backend and the React Dashboard page.
// It aggregates data from config, history, scheduler, and diskspace into a
// single view model. The UI never sees raw core types.
// ============================================================================

use crate::app::models::common::{HealthStatus, OperationType, ProtectionStatus};
use serde::{Deserialize, Serialize};

/// Summary of a single backup operation, used in Dashboard last-backup display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSummary {
    /// Timestamp of the backup (ISO 8601)
    pub timestamp: String,
    /// Status string: "success" | "failure" | "partial"
    pub status: String,
    /// Name of the job that produced this backup
    pub job_name: String,
    /// Number of files in this backup
    pub file_count: u64,
    /// Total data size in bytes
    pub total_bytes: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Per-destination storage usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStatus {
    /// Destination path
    pub path: String,
    /// Free space in bytes
    pub free_bytes: u64,
    /// Total capacity in bytes
    pub total_bytes: u64,
    /// Used space in bytes
    pub used_bytes: u64,
    /// Usage percentage (0.0 – 100.0)
    pub used_pct: f64,
}

/// A single row in the Recent Activity list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRecord {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Operation type (backup / restore / verify)
    pub operation: OperationType,
    /// Job name (may be empty for manual operations)
    pub job_name: Option<String>,
    /// Status: "success" | "failure" | "partial"
    pub status: String,
    /// Number of files processed
    pub file_count: u64,
    /// Total data size in bytes
    pub total_bytes: u64,
}

/// Top-level Dashboard view model -- returned by dashboard_service
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    /// Overall protection state (derived from last-backup status)
    pub protection_status: ProtectionStatus,
    /// Number of configured backup jobs
    pub total_jobs: u32,
    /// Most recent backup (None if no backups exist)
    pub last_backup: Option<BackupSummary>,
    /// Storage usage per unique destination
    pub storage: Vec<StorageStatus>,
    /// Recent activity feed (newest first, max 10)
    pub recent_activity: Vec<ActivityRecord>,
    /// Number of scheduled tasks
    pub scheduled_count: u32,
    /// Platform health
    pub health: HealthStatus,
}
