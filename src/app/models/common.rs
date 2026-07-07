// ============================================================================
// common.rs -- Shared enum types used across all Application Layer models
//
// These types are the "product language" for protection status, health, and
// job state. They are intentionally decoupled from any UI display strings.
// ============================================================================

use serde::{Deserialize, Serialize};

/// Overall data protection status for the Dashboard hero section
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProtectionStatus {
    /// All configured jobs are running within acceptable parameters
    Protected,
    /// At least one job has a recent failure or warning
    AtRisk,
    /// No recent successful backups, or core protection is not configured
    Critical,
    /// State cannot be determined (e.g., no config file)
    Unknown,
}

/// Per-job runtime status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    /// Job is configured and last backup succeeded
    Active,
    /// Job is configured but user has paused it
    Paused,
    /// Job has persistent failures
    Error,
    /// Status not yet determined
    Unknown,
}

/// Platform health indicator
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    /// All subsystems (config, storage, scheduler) operating normally
    Healthy,
    /// Minor issue detected (e.g., low disk space, one failed job)
    Warning,
    /// Critical failure detected
    Critical,
}

/// Operation type recorded in history
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    Backup,
    Restore,
    Verify,
}
