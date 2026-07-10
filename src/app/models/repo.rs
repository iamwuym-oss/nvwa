// ============================================================================
// repo.rs -- Repository API models for Settings and Backup pages
//
// These types form the "contract" between the Tauri command layer and the
// React UI for Repository management operations.
//
// Design:
//   - String-based IDs and paths for Tauri JSON serialization
//   - Separate View (read) and Request (write) types per project convention
//   - Retention status is read-only (no mutation in v1)
//   - All Phase S frozen parameters are read-only display fields
// ============================================================================

use serde::{Deserialize, Serialize};

/// Repository registry entry (persisted in repositories.json).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRecord {
    /// Stable Repository UUID
    pub id: String,
    /// User-friendly display name
    pub name: String,
    /// Filesystem path where the Repository lives
    pub path: String,
    /// ISO 8601 timestamp of creation
    pub created_at: String,
    /// ISO 8601 timestamp of last successful open
    pub last_opened: String,
    /// "active" | "missing"
    pub status: String,
}

/// Full Repository information shown in the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoInfoResponse {
    /// Registry UUID
    pub id: String,
    /// Display name
    pub name: String,
    /// Filesystem path
    pub path: String,
    /// Repository Engine internal UUID (mirrors info.repository_id)
    pub repo_uuid: String,
    /// Repository format version
    pub format_version: u32,
    /// ISO 8601 creation timestamp
    pub created_at: String,
    /// Read-only: "256 KiB (Fixed)"
    pub block_size: String,
    /// Read-only: "zstd"
    pub compression: String,
    /// Capability flags for UI display
    pub capabilities: Vec<String>,
    /// Total chunks stored
    pub total_chunks: u64,
    /// Total raw data size in bytes
    pub total_size_bytes: u64,
    /// Number of backup instances
    pub instance_count: u32,
    /// Retention status (read-only)
    pub retention: RetentionStatusSummary,
}

/// Read-only retention status shown in Repository details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionStatusSummary {
    pub total_restore_points: u32,
    pub active_restore_points: u32,
    pub deleted_restore_points: u32,
    pub orphan_candidates: u64,
}

/// Result of a Repository verification operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub passed: bool,
    pub checked_instances: u32,
    pub checked_blocks: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
}

/// Request payload for creating a new Repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepoRequest {
    /// User-friendly display name (must be non-empty)
    pub name: String,
    /// Filesystem path for the Repository (must be non-empty, must not exist as repo)
    pub path: String,
}

/// Request payload for Repository verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyOptionsRequest {
    /// Quick check (metadata only) vs full check (all blocks)
    pub quick: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_record_serde() {
        let record = RepoRecord {
            id: "a1b2c3d4-e5f6-7890-abcd-ef1234567890".into(),
            name: "Test Repo".into(),
            path: "/tmp/test-repo".into(),
            created_at: "2026-07-10T10:00:00Z".into(),
            last_opened: "2026-07-10T10:00:00Z".into(),
            status: "active".into(),
        };
        let json = serde_json::to_string(&record).unwrap();
        let decoded: RepoRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, record.id);
        assert_eq!(decoded.name, record.name);
        assert_eq!(decoded.status, "active");
    }

    #[test]
    fn test_verify_response_serde() {
        let resp = VerifyResponse {
            passed: true,
            checked_instances: 5,
            checked_blocks: 10000,
            errors: vec![],
            warnings: vec!["orphan candidates: 42".into()],
            duration_ms: 1500,
        };
        let json = serde_json::to_string(&resp).unwrap();
        let decoded: VerifyResponse = serde_json::from_str(&json).unwrap();
        assert!(decoded.passed);
        assert_eq!(decoded.checked_instances, 5);
    }

    #[test]
    fn test_create_repo_request_validation() {
        let req = CreateRepoRequest {
            name: "My Backup".into(),
            path: "D:\\NuwaRepo\\main".into(),
        };
        assert!(!req.name.is_empty());
        assert!(!req.path.is_empty());
    }
}
