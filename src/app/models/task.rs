// ============================================================================
// task.rs -- Unified task/progress model for all backup operations
//
// This model is shared across Backup, Restore, Verify, Prune, and Schedule
// operations. It provides a single contract for:
//   - Task lifecycle (Pending → Running → Completed / Failed / Cancelled)
//   - Progress reporting (percentage, current file, message)
//   - Result types
//
// The React frontend uses a single set of types to render progress bars,
// status badges, and activity feed across all pages.
// ============================================================================

use serde::{Deserialize, Serialize};

/// Type of operation being performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Backup,
    Restore,
    Verify,
    Prune,
    Schedule,
}

/// Current state of a task in its lifecycle
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskState {
    /// Task is queued but not yet started
    Pending,
    /// Task is actively executing
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed with an error
    Failed,
    /// Task was cancelled by user
    Cancelled,
}

/// Progress snapshot for a running task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskProgress {
    /// Progress percentage (0.0 – 100.0)
    pub percent: f64,
    /// Human-readable status message (e.g. "Backing up file 42 of 150")
    pub message: String,
    /// Current file being processed (if applicable)
    pub current_file: Option<String>,
    /// Files processed so far
    pub files_processed: u64,
    /// Total files to process (0 if unknown)
    pub files_total: u64,
    /// Bytes processed so far
    pub bytes_processed: u64,
    /// Total bytes to process (0 if unknown)
    pub bytes_total: u64,
}

/// Result summary for a completed task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Whether the task succeeded
    pub success: bool,
    /// Status string: "success" | "failure" | "partial" | "cancelled"
    pub status: String,
    /// Number of files processed
    pub file_count: u64,
    /// Total data size in bytes
    pub total_bytes: u64,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Backup point ID (only for backup tasks)
    pub backup_id: Option<String>,
}

/// Full task descriptor returned by any operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    /// Unique task identifier
    pub task_id: String,
    /// Type of operation
    pub task_type: TaskType,
    /// Current state
    pub state: TaskState,
    /// Progress (non-null when Running)
    pub progress: Option<TaskProgress>,
    /// Result (non-null when Completed or Failed)
    pub result: Option<TaskResult>,
    /// Job name this task belongs to
    pub job_name: String,
    /// ISO 8601 timestamp when task was created
    pub created_at: String,
    /// ISO 8601 timestamp when task completed (None if not finished)
    pub completed_at: Option<String>,
}
