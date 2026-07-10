// ============================================================================
// engine.rs 鈥?Verify Engine: three-level verification
// ============================================================================
//
// The Verify Engine provides three levels of repository integrity checking.
// See Architecture v1.0 搂12.
//
// Level 1 (Metadata):
//   - repo.db integrity check (PRAGMA integrity_check)
//   - backup-instances directory presence scan
//   - Each instance has backup-metadata.json
//
// Level 2 (Sampling):
//   - Level 1 checks
//   - Randomly select N blocks from block-store
//   - Full SHA-256 verification of each sampled block
//
// Level 3 (Full):
//   - All Level 1 + Level 2 checks
//   - Verify EVERY block in block-store
//
// Key design principles (搂12.1):
// - Verify Engine does NOT modify any data
// - Verification is read-only
// - Failed blocks are reported but NOT deleted
// - Data loss detection is informational, not destructive

use crate::repository::block_store::block_id::BlockId;
use crate::repository::block_store::store::BlockStore;
use crate::repository::error::RepositoryError;
use crate::repository::repo_manager::RepoHandle;
use std::fs;
use std::path::Path;

/// Verification level selector
#[derive(Debug, Clone)]
pub enum VerifyLevel {
    /// Level 1: repo.db + metadata structure only
    Metadata,
    /// Level 2: Level 1 + verify N randomly sampled blocks
    Sampling(u32),
    /// Level 3: Level 1 + verify ALL blocks in block-store
    Full,
}

/// Detailed report of verification results
#[derive(Debug, Clone, Default)]
pub struct VerifyReport {
    // ======== Structure Checks ========
    /// Total restore points found in repo.db
    pub total_restore_points: u64,
    /// Restore Points where backup-instances directory is missing
    pub missing_instances: Vec<String>,
    /// Restore Points where backup-metadata.json is missing
    pub missing_metadata: Vec<String>,
    /// Restore Points where metadata status is not COMMITTED
    pub incomplete_transactions: Vec<String>,

    // ======== Block Verification ========
    /// Total block files found in block-store
    pub total_blocks: u64,
    /// Number of blocks actually verified in this run
    pub verified_blocks: u64,
    /// Number of blocks that failed verification
    pub failed_blocks: u64,
    /// Block IDs that failed (for diagnostics)
    pub failed_block_ids: Vec<String>,

    // ======== Overall ========
    /// True if any corruption or unrecoverable state was detected
    pub data_loss_detected: bool,
    /// Human-readable summary
    pub summary: String,
}

/// Run three-level verification on a Repository.
///
/// # Arguments
/// * `handle` 鈥?Opened RepoHandle
/// * `block_store` 鈥?BlockStore implementation to verify blocks against
/// * `level` 鈥?Verification depth (Metadata/Sampling/Full)
///
/// # Returns
/// VerifyReport with all findings. Errors are captured in the report
/// rather than propagated, except for fundamental access problems.
pub fn verify_repo(
    handle: &RepoHandle,
    block_store: &dyn BlockStore,
    level: VerifyLevel,
) -> Result<VerifyReport, RepositoryError> {
    let mut report = VerifyReport::default();

    // ======== Phase 1: Metadata Verification ========
    verify_metadata(handle, &mut report)?;

    // ======== Phase 2: Block Verification ========
    if !matches!(level, VerifyLevel::Metadata) {
        verify_blocks(handle, block_store, &level, &mut report)?;
    }

    // ======== Report ========
    report.data_loss_detected = !report.failed_block_ids.is_empty()
        || !report.missing_instances.is_empty()
        || !report.missing_metadata.is_empty();

    report.summary = build_summary(&report, &level);

    Ok(report)
}

/// Phase 1: Verify repository metadata structure.
fn verify_metadata(handle: &RepoHandle, report: &mut VerifyReport) -> Result<(), RepositoryError> {
    // 1. Check repo.db integrity
    let conn = handle.repo_db()?;
    let integrity: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|e| RepositoryError::General {
            detail: format!("Cannot run repo.db integrity check: {}", e),
        })?;

    if integrity != "ok" {
        return Err(RepositoryError::SelfCheckFailed(format!(
            "repo.db integrity check failed: {}",
            integrity
        )));
    }

    // 2. Count restore points in repo.db
    report.total_restore_points = conn
        .query_row("SELECT COUNT(*) FROM restore_points", [], |row| {
            row.get::<_, u64>(0)
        })
        .unwrap_or(0);

    // 3. Check each restore point has its instance directory
    let mut stmt = conn
        .prepare("SELECT point_id, instance_path, status FROM restore_points")
        .map_err(|e| RepositoryError::General {
            detail: format!("Cannot query restore points: {}", e),
        })?;

    let rows = stmt
        .query_map([], |row| {
            let point_id: String = row.get(0)?;
            let instance_path: String = row.get(1)?;
            let status: String = row.get(2)?;
            Ok((point_id, instance_path, status))
        })
        .map_err(|e| RepositoryError::General {
            detail: format!("Cannot iterate restore points: {}", e),
        })?;

    for row in rows {
        let (point_id, instance_path, status) = row.map_err(|e| RepositoryError::General {
            detail: format!("Cannot read restore point row: {}", e),
        })?;

        // Check if status is terminal
        if status != "COMMITTED" {
            report.incomplete_transactions.push(point_id.clone());
        }

        let instance_dir = Path::new(&instance_path);
        if !instance_dir.exists() {
            report.missing_instances.push(point_id.clone());
            continue;
        }

        // Check backup-metadata.json exists
        let meta_path = instance_dir.join("backup-metadata.json");
        if !meta_path.exists() {
            report.missing_metadata.push(point_id);
        }
    }

    Ok(())
}

/// Phase 2: Verify blocks in block-store.
fn verify_blocks(
    _handle: &RepoHandle,
    block_store: &dyn BlockStore,
    level: &VerifyLevel,
    report: &mut VerifyReport,
) -> Result<(), RepositoryError> {
    // Walk block-store and collect all block files
    let mut all_blocks: Vec<BlockId> = Vec::new();
    walk_block_store(block_store.root_path(), &mut all_blocks)?;

    report.total_blocks = all_blocks.len() as u64;

    // Determine which blocks to verify
    let to_verify: Vec<&BlockId> = match level {
        VerifyLevel::Full => all_blocks.iter().collect(),
        VerifyLevel::Sampling(n) => {
            let sample_count = (*n as usize).min(all_blocks.len());
            if sample_count == 0 {
                Vec::new()
            } else if sample_count >= all_blocks.len() {
                all_blocks.iter().collect()
            } else {
                // Take evenly spaced samples for deterministic behavior
                let step = all_blocks.len() / sample_count;
                all_blocks.iter().step_by(step).take(sample_count).collect()
            }
        }
        _ => Vec::new(), // Metadata level handled above
    };

    // Verify each selected block
    for block_id in &to_verify {
        report.verified_blocks += 1;

        match block_store.verify_block(block_id) {
            Ok(true) => { /* block is intact */ }
            Ok(false) => {
                report.failed_blocks += 1;
                report.failed_block_ids.push(block_id.to_hex());
            }
            Err(e) => {
                report.failed_blocks += 1;
                report.failed_block_ids.push(block_id.to_hex());
                // Log error but continue verifying
                let _ = e;
            }
        }
    }

    Ok(())
}

/// Recursively walk the block-store directory tree and collect all Block IDs.
///
/// Block-store layout:
///   block-store/{hash[0:2]}/{hash[2:4]}/{full_hash}.block
///
/// This traversal is used by Verify Engine and is NOT for production
/// data-path usage (it's slow for 100M+ blocks).
fn walk_block_store(dir: &Path, blocks: &mut Vec<BlockId>) -> Result<(), RepositoryError> {
    if !dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|e| {
        RepositoryError::io(dir.to_path_buf(), "Cannot read block-store directory", e)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            RepositoryError::io(dir.to_path_buf(), "Cannot read directory entry", e)
        })?;

        let path = entry.path();

        if path.is_dir() {
            // Recurse into subdirectory
            walk_block_store(&path, blocks)?;
        } else if path.extension().and_then(|s| s.to_str()) == Some("block") {
            // Extract block_id from filename (without extension)
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Ok(block_id) = stem.parse::<BlockId>() {
                    blocks.push(block_id);
                }
                // Silently skip files that don't parse as valid hex (non-block files)
            }
        }
    }

    Ok(())
}

/// Build a human-readable summary string
fn build_summary(report: &VerifyReport, level: &VerifyLevel) -> String {
    let level_name = match level {
        VerifyLevel::Metadata => "Metadata",
        VerifyLevel::Sampling(n) => &format!("Sampling({})", n),
        VerifyLevel::Full => "Full",
    };

    let mut parts = vec![format!(
        "Verify Engine [{}]: {} restore points, {} blocks in store, {} verified.",
        level_name, report.total_restore_points, report.total_blocks, report.verified_blocks,
    )];

    if !report.failed_block_ids.is_empty() {
        parts.push(format!(
            "FAILED: {} blocks failed verification.",
            report.failed_blocks
        ));
    }
    if !report.missing_instances.is_empty() {
        parts.push(format!(
            "WARNING: {} restore points missing instance directories.",
            report.missing_instances.len()
        ));
    }
    if !report.missing_metadata.is_empty() {
        parts.push(format!(
            "WARNING: {} restore points missing backup-metadata.json.",
            report.missing_metadata.len()
        ));
    }
    if !report.incomplete_transactions.is_empty() {
        parts.push(format!(
            "WARNING: {} incomplete transactions detected.",
            report.incomplete_transactions.len()
        ));
    }

    if report.data_loss_detected {
        parts.push("DATA LOSS DETECTED".to_string());
    } else {
        parts.push("Repository integrity verified.".to_string());
    }

    parts.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::block_store::block_header::{BlockHeader, Compression};
    use crate::repository::block_store::block_id::BlockId;
    use crate::repository::block_store::store::{Block, LocalFsBlockStore};
    use crate::repository::repo_manager::{init_repo, DEFAULT_BLOCK_SIZE};
    use tempfile::TempDir;

    fn setup() -> (RepoHandle, LocalFsBlockStore, TempDir) {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();
        let store = LocalFsBlockStore::new(handle.block_store_dir.clone());
        (handle, store, tmp)
    }

    fn write_test_block(store: &LocalFsBlockStore, data: &[u8]) -> BlockId {
        let header = BlockHeader::new(Compression::None, data.len() as u64, data.len() as u64);
        let block = Block {
            header,
            data: data.to_vec(),
        };
        store.put_block(&block).unwrap()
    }

    #[test]
    fn test_metadata_level_empty_repo() {
        let (handle, store, _tmp) = setup();
        let report = verify_repo(&handle, &store, VerifyLevel::Metadata).unwrap();
        assert_eq!(report.total_restore_points, 0);
        assert_eq!(report.total_blocks, 0);
        assert!(!report.data_loss_detected);
    }

    #[test]
    fn test_metadata_level_with_blocks_no_instances() {
        let (handle, store, _tmp) = setup();
        write_test_block(&store, b"test block data for verification");

        let report = verify_repo(&handle, &store, VerifyLevel::Metadata).unwrap();
        // Blocks exist but Metadata level doesn't verify them
        assert_eq!(report.total_blocks, 0);
        assert_eq!(report.verified_blocks, 0);
    }

    #[test]
    fn test_sampling_level_verifies_blocks() {
        let (handle, store, _tmp) = setup();

        // Write multiple blocks
        for i in 0..10 {
            write_test_block(&store, &format!("block data {}", i).into_bytes());
        }

        let report = verify_repo(&handle, &store, VerifyLevel::Sampling(5)).unwrap();
        assert_eq!(report.total_blocks, 10);
        assert!(report.verified_blocks > 0);
        assert!(report.verified_blocks <= 5);
        assert_eq!(report.failed_blocks, 0);
    }

    #[test]
    fn test_full_level_verifies_all_blocks() {
        let (handle, store, _tmp) = setup();

        for i in 0..5 {
            write_test_block(&store, &format!("full verify block {}", i).into_bytes());
        }

        let report = verify_repo(&handle, &store, VerifyLevel::Full).unwrap();
        assert_eq!(report.total_blocks, 5);
        assert_eq!(report.verified_blocks, 5);
        assert_eq!(report.failed_blocks, 0);
    }

    #[test]
    fn test_walk_empty_block_store() {
        let tmp = TempDir::new().unwrap();
        let mut blocks = Vec::new();
        walk_block_store(tmp.path(), &mut blocks).unwrap();
        assert!(blocks.is_empty());
    }

    #[test]
    fn test_sampling_none_requested() {
        let (handle, store, _tmp) = setup();
        write_test_block(&store, b"some data");

        // Sampling(0) means verify 0 blocks
        let report = verify_repo(&handle, &store, VerifyLevel::Sampling(0)).unwrap();
        assert_eq!(report.total_blocks, 1);
        assert_eq!(report.verified_blocks, 0);
    }

    #[test]
    fn test_build_summary() {
        let report = VerifyReport {
            total_restore_points: 5,
            total_blocks: 1000,
            verified_blocks: 100,
            ..Default::default()
        };

        let summary = build_summary(&report, &VerifyLevel::Sampling(100));
        assert!(summary.contains("Sampling(100)"));
        assert!(summary.contains("5 restore points"));
        assert!(summary.contains("1000 blocks"));
        assert!(summary.contains("Repository integrity verified."));
    }

    #[test]
    fn test_build_summary_with_failures() {
        let report = VerifyReport {
            failed_blocks: 3,
            failed_block_ids: vec!["abc".to_string(), "def".to_string()],
            data_loss_detected: true,
            ..Default::default()
        };

        let summary = build_summary(&report, &VerifyLevel::Full);
        assert!(summary.contains("FAILED"));
        assert!(summary.contains("DATA LOSS DETECTED"));
    }
}
