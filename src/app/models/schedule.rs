// ============================================================================
// schedule.rs -- Schedule domain API models
//
// These types form the "contract" between the Tauri command layer and the
// React Schedule page. They are designed to be:
//   - Independent of core config.rs ScheduleProfileConfig (no struct leak)
//   - Serializable to JSON for Tauri IPC
//   - The ScheduleProfileView includes runtime state derived by ScheduleService
// ============================================================================

use serde::{Deserialize, Serialize};

/// Trigger type discriminator for UI display (no runtime values).
/// This mirrors src/config.rs ScheduleTriggerConfig but as a pure UI enum.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TriggerType {
    Once,
    Daily,
    Weekly,
    Monthly,
    OnLogon,
}

/// A single schedule profile as seen by the UI.
///
/// This is a view model: used_by_jobs, next_run_at, last_run_at, and
/// last_run_status are computed at query time, not persisted in config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleProfileView {
    /// Unique schedule identifier (derived from name on creation)
    pub id: String,
    /// Human-readable display name (also serves as the primary key)
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Whether this schedule is enabled
    pub enabled: bool,
    /// Trigger type for display
    pub trigger_type: TriggerType,
    /// Human-readable trigger summary (e.g. "Daily at 21:00", "Weekly on Mon,Wed,Fri")
    pub trigger_summary: String,
    /// Number of backup jobs currently referencing this schedule
    pub used_by_count: u32,
    /// Names of backup jobs referencing this schedule
    pub used_by_jobs: Vec<String>,
    /// Next estimated run time (from Windows Task Scheduler or trigger calc)
    pub next_run_at: Option<String>,
    /// Timestamp of last run for any job using this schedule (from History)
    pub last_run_at: Option<String>,
    /// Status of last run: "success" | "failure" | "partial" | null
    pub last_run_status: Option<String>,
    /// Whether the Windows scheduled tasks are in sync with config
    pub task_sync_status: TaskSyncStatus,
    /// Creation timestamp (ISO 8601)
    pub created_at: Option<String>,
    /// Last update timestamp (ISO 8601)
    pub updated_at: Option<String>,
}

/// Task synchronization status with Windows Task Scheduler
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskSyncStatus {
    /// All associated backup jobs have Windows tasks created
    Synced,
    /// Some or all associated jobs are missing Windows tasks
    Partial,
    /// No associated jobs (nothing to sync)
    NoJobs,
    /// Sync status could not be determined
    Unknown,
}

/// Request to create or update a schedule profile.
///
/// NOTE: The id field is intentionally removed from this request.
///   - On CREATE: the backend generates the ID from name via name_to_id()
///   - On UPDATE: name changes are forbidden; the existing ID is preserved
///   - The Tauri command update_schedule(id, request) passes the ID separately
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleProfileRequest {
    /// Human-readable display name (used as unique identifier)
    pub name: String,
    /// Optional description
    pub description: Option<String>,
    /// Whether this schedule is enabled after creation
    pub enabled: bool,
    /// Trigger type
    pub trigger_type: TriggerType,
    /// Trigger parameters as JSON string (parsed by service layer)
    /// This avoids enum variant complexity in IPC serialization.
    pub trigger_params: String,
}

/// Result of a schedule deletion attempt, with affected job information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDeleteResult {
    /// Whether the deletion was executed
    pub deleted: bool,
    /// Names of affected backup jobs (will lose their schedule reference)
    pub affected_jobs: Vec<String>,
    /// User-facing message
    pub message: String,
}

// ---------------------------------------------------------------------------
// Helper: build a human-readable trigger summary from config trigger
// ---------------------------------------------------------------------------

/// Build a trigger summary string from config trigger type and parameters.
/// Used by ScheduleService when assembling ScheduleProfileView.
pub fn summarize_trigger(trigger_type: &TriggerType, params: &str) -> String {
    match trigger_type {
        TriggerType::Once => {
            format!("Once at {}", params)
        }
        TriggerType::Daily => {
            format!("Daily at {}", params)
        }
        TriggerType::Weekly => {
            // params format: "Mon,Wed,Fri@21:00" or "days@time"
            if let Some((days, time)) = params.split_once('@') {
                let display_days: Vec<&str> = days.split(',').collect();
                format!("Weekly on {} at {}", display_days.join(", "), time)
            } else {
                format!("Weekly ({})", params)
            }
        }
        TriggerType::Monthly => {
            format!("Monthly on day {} at {}", params, "")
        }
        TriggerType::OnLogon => {
            if let Ok(secs) = params.parse::<u32>() {
                if secs == 0 {
                    "On logon (no delay)".to_string()
                } else {
                    format!("On logon ({}s delay)", secs)
                }
            } else {
                "On logon".to_string()
            }
        }
    }
}

/// Convert core ScheduleTriggerConfig trigger_type to UI TriggerType
pub fn trigger_type_from_config(config_id: &str) -> TriggerType {
    match config_id {
        "once" => TriggerType::Once,
        "daily" => TriggerType::Daily,
        "weekly" => TriggerType::Weekly,
        "monthly" => TriggerType::Monthly,
        "on_logon" | "onlogon" => TriggerType::OnLogon,
        _ => TriggerType::Daily,
    }
}

/// Generate a machine-readable identifier from a human-readable schedule name.
///
/// This is the inverse of the user-friendly name: trim, lowercase, replace
/// non-alphanumeric chars with hyphens, and strip leading/trailing hyphens.
///
/// Examples:
///   "Daily Evening Backup" -> "daily-evening-backup"
///   "On Login Backup"      -> "on-login-backup"
pub fn name_to_id(name: &str) -> String {
    name.trim()
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}
