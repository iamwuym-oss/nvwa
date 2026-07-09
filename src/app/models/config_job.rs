// ============================================================================
// config_job.rs -- Configuration job API models for Settings page
//
// These types form the "contract" between the Tauri command layer and the
// React Settings page for backup job CRUD operations.
//
// Design:
//   - String-based paths (not PathBuf) for Tauri JSON serialization
//   - Flat retention fields (not nested RetentionPolicy) for simple UI forms
//   - Separate View (read) and Request (write) types per project convention
// ============================================================================

use serde::{Deserialize, Serialize};

/// A backup job configuration as seen by the Settings page.
///
/// This is the "configuration perspective" — unlike BackupJobView (runtime
/// perspective with status/last-run info), this shows raw config fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfigView {
    /// Unique job name
    pub name: String,
    /// Backup source path (filesystem path string)
    pub source: String,
    /// Backup destination path (filesystem path string)
    pub dest: String,
    /// Whether compression is enabled
    pub compress: bool,
    /// Keep last N backup versions (None = no count-based limit)
    pub retention_keep_count: Option<u32>,
    /// Keep backups from last N days (None = no day-based limit)
    pub retention_keep_days: Option<u64>,
    /// Schedule profile ID (None = manual only)
    pub schedule_id: Option<String>,
}

/// Request payload for creating or updating a backup job configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfigRequest {
    /// Unique job name (must be non-empty)
    pub name: String,
    /// Backup source path (must be non-empty)
    pub source: String,
    /// Backup destination path (must be non-empty)
    pub dest: String,
    /// Whether compression is enabled
    pub compress: bool,
    /// Keep last N backup versions (None = no count-based limit)
    pub retention_keep_count: Option<u32>,
    /// Keep backups from last N days (None = no day-based limit)
    pub retention_keep_days: Option<u64>,
    /// Schedule profile ID (None = manual only)
    pub schedule_id: Option<String>,
}
