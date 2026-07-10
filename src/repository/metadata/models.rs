// ============================================================================
// models.rs — Phase S Metadata data models
// ============================================================================
//
// Data model hierarchy: Backup Job → Restore Point → Backup Instance
// See Architecture v1.0 §4.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Source data type for a backup job
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SourceType {
    File = 0,
    Volume = 1,
    Disk = 2,
}

impl SourceType {
    pub fn from_u32(v: u32) -> Option<Self> {
        match v {
            0 => Some(SourceType::File),
            1 => Some(SourceType::Volume),
            2 => Some(SourceType::Disk),
            _ => None,
        }
    }

    pub fn to_u32(self) -> u32 {
        self as u32
    }
}

/// Job status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum JobStatus {
    Active,
    Deleted,
}

/// Restore Point status (transaction + retention state machine).
///
/// Transaction states (Wave 3): CREATING → WRITING → VERIFYING → COMMITTED
/// Failure terminal state: FAILED
/// Retention states (S-09):  COMMITTED → DELETING → DELETED
///
/// DELETING/DELETED are written by Retention Engine only.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PointStatus {
    Creating,
    Writing,
    Verifying,
    #[serde(rename = "COMMITTED")]
    Committed,
    Failed,
    /// Retention Engine phase 1: deletion in progress (crash-recoverable)
    #[serde(rename = "DELETING")]
    Deleting,
    /// Retention Engine phase 3: deletion complete
    #[serde(rename = "DELETED")]
    Deleted,
}

impl PointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            PointStatus::Creating => "CREATING",
            PointStatus::Writing => "WRITING",
            PointStatus::Verifying => "VERIFYING",
            PointStatus::Committed => "COMMITTED",
            PointStatus::Failed => "FAILED",
            PointStatus::Deleting => "DELETING",
            PointStatus::Deleted => "DELETED",
        }
    }

    pub fn parse_from_str(s: &str) -> Option<Self> {
        match s {
            "CREATING" => Some(PointStatus::Creating),
            "WRITING" => Some(PointStatus::Writing),
            "VERIFYING" => Some(PointStatus::Verifying),
            "COMMITTED" => Some(PointStatus::Committed),
            "FAILED" => Some(PointStatus::Failed),
            "DELETING" => Some(PointStatus::Deleting),
            "DELETED" => Some(PointStatus::Deleted),
            _ => None,
        }
    }
}

/// Backup Job — user-defined backup task.
/// Persists until user deletes it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupJob {
    pub job_id: String,
    pub job_name: String,
    pub source_type: SourceType,
    pub source_path: String,
    pub created_at: String, // ISO-8601
    pub status: JobStatus,
}

/// Restore Point — one execution result.
/// Tracks chain relationships and transaction state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestorePoint {
    pub point_id: String,
    pub job_id: String,
    pub chain_id: String,
    pub chain_position: u32, // 0 = Full, 1+ = Incremental
    pub created_at: String,  // ISO-8601
    pub status: PointStatus,
    pub instance_path: PathBuf,
    pub block_count: u64,
    pub total_raw_bytes: u64,
    pub parent_point_id: Option<String>,
}

/// Chunk policy metadata stored in repo.db
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkPolicyMetadata {
    pub policy_type: String, // "fixed"
    pub block_size: u32,     // 262144 for 256KB
}

/// Block Map integrity manifest (stored in backup-metadata.json)
/// Purpose: corruption DETECTION only. NOT for reconstruction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMapIntegrity {
    pub database: String, // "block-map.db"
    pub block_count: u64,
    pub sha256: String,
    pub first_offset: u64,
    pub last_offset: u64,
}

/// Catalog integrity manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogIntegrity {
    pub database: String,
    pub file_count: u64,
    pub sha256: String,
}

/// Backup Instance Metadata — written as backup-metadata.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInstanceMetadata {
    pub schema_version: String,
    pub restore_point_id: String,
    pub job_id: String,
    pub source_type: String,
    pub source_description: String,

    /// Enterprise asset identity. Empty string in single-machine mode.
    /// Future: workstation/server/volume/disk identifier
    pub asset_id: String,
    /// Enterprise asset type classification
    pub asset_type: String,
    pub created_at: String,
    pub status: String,

    pub block_chunk_policy: ChunkPolicyMetadata,

    pub block_map: BlockMapIntegrity,

    pub catalog: Option<CatalogIntegrity>,

    pub summary: BackupInstanceSummary,
}

/// Repository capability flags.
/// Written at repository init time, READ-ONLY after creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryCapabilities {
    pub compression: bool,
    pub encryption: bool,
    pub dedup: bool,
    pub immutable_storage: bool,
}

impl RepositoryCapabilities {
    /// Default Phase S capabilities
    pub fn phase_s_default() -> Self {
        RepositoryCapabilities {
            compression: true,
            encryption: false,
            dedup: false,
            immutable_storage: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInstanceSummary {
    pub total_raw_bytes: u64,
    pub file_count: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_status_roundtrip() {
        for status in &[
            PointStatus::Creating,
            PointStatus::Writing,
            PointStatus::Verifying,
            PointStatus::Committed,
            PointStatus::Failed,
            PointStatus::Deleting,
            PointStatus::Deleted,
        ] {
            let s = status.as_str();
            let back = PointStatus::parse_from_str(s).unwrap();
            assert_eq!(*status, back);
        }
    }

    #[test]
    fn test_source_type_roundtrip() {
        assert_eq!(SourceType::from_u32(0), Some(SourceType::File));
        assert_eq!(SourceType::from_u32(1), Some(SourceType::Volume));
        assert_eq!(SourceType::from_u32(2), Some(SourceType::Disk));
        assert!(SourceType::from_u32(99).is_none());
    }
}
