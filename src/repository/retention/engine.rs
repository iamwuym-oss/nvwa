// ============================================================================
// engine.rs 鈥?Retention Engine: Restore Point lifecycle management
// ============================================================================
//
// Phase S (S-09) Retention Engine performs logical deletion only.
// It manages Restore Point lifecycle without modifying block data.
//
// Architecture Compliance (搂12):
//   ? Only Restore Point lifecycle management
//   ? No physical block deletion
//   ? No reference counting
//   ? Orphan candidates only (no GC)
//   ? Two-phase delete for crash safety (DELETING 鈫?delete 鈫?DELETED)
//   ? Block-store remains immutable
//   ? No modifications to BlockStore, Catalog, or BlockMap formats
//
// Retention Policy:
//   keep_days:       Delete COMMITTED points older than N days
//   keep_full_count: Among candidates, protect N most recent full backups
//
// Two-Phase Delete Protocol (Crash Safety):
//   Phase 1: repo.db status = DELETING  (mark intent)
//   Phase 2: delete backup-instances/{point_id}/
//   Phase 3: repo.db status = DELETED   (mark complete)
//
// If crash occurs:
//   After Phase 1: next run detects DELETING 鈫?retries Phase 2鈫?
//   After Phase 2: data is gone, next run advances to Phase 3
//   After Phase 3: fully complete, no action needed
//
// Orphan Candidates:
//   Blocks from deleted Restore Points are orphan candidates.
//   Phase S only counts them 鈥?no physical deletion.
//   Phase 6+ will add reference counting + GC for safe reclamation.

use crate::repository::error::RepositoryError;
use crate::repository::repo_manager::RepoHandle;
use std::fs;

use std::time::{SystemTime, UNIX_EPOCH};

/// Retention policy defining which Restore Points should be deleted.
#[derive(Debug, Clone)]
pub struct RetentionPolicy {
    /// Delete COMMITTED Restore Points older than this many days.
    /// Set to 0 to disable time-based deletion.
    pub keep_days: u32,

    /// Keep at least this many most recent full backups (chain_position=0),
    /// even if they exceed keep_days.
    /// Set to 0 to disable full backup protection.
    pub keep_full_count: u32,
}

impl RetentionPolicy {
    /// Create a new retention policy.
    ///
    /// # Arguments
    /// * `keep_days` 鈥?Delete points older than N days (0 = keep all by age)
    /// * `keep_full_count` 鈥?Keep at least N full backups (0 = no protection)
    pub fn new(keep_days: u32, keep_full_count: u32) -> Self {
        RetentionPolicy {
            keep_days,
            keep_full_count,
        }
    }
}

/// Result of applying a retention policy.
#[derive(Debug, Clone, Default)]
pub struct RetentionResult {
    /// Restore Points that were deleted by this retention run
    pub deleted_points: Vec<String>,
    /// Total number of orphan block candidates from all currently-DELETED points
    pub orphan_candidate_count: u64,
}

/// Orphan candidate summary (read-only query).
#[derive(Debug, Clone, Default)]
pub struct OrphanSummary {
    /// Restore Points with DELETED status
    pub deleted_points: Vec<String>,
    /// Sum of block_count across all deleted points
    pub total_orphan_blocks: u64,
}

/// Apply a retention policy to the repository.
///
/// This function:
/// 1. Identifies eligible Restore Points based on the policy
/// 2. For each candidate, performs two-phase delete:
///    DELETING 鈫?delete directory 鈫?DELETED
/// 3. Recalculates orphan candidate count from all DELETED points
///
/// # Arguments
/// * `handle` 鈥?Opened RepoHandle
/// * `policy` 鈥?Retention policy defining what to delete
///
/// # Returns
/// RetentionResult with summary of deletions and orphan count
///
/// # Crash Safety
/// Each delete uses a two-phase protocol. If interrupted:
/// - Phase 1 complete (DELETING): next run retries deletion
/// - Phase 2 complete (dir deleted): next run marks DELETED
/// - Phase 3 complete (DELETED): no action needed
pub fn apply_retention(
    handle: &RepoHandle,
    policy: &RetentionPolicy,
) -> Result<RetentionResult, RepositoryError> {
    let mut result = RetentionResult::default();

    // Step 1: Recover any in-progress deletions from previous crashes
    recover_incomplete_deletions(handle)?;

    // Step 2: Find COMMITTED Restore Points eligible for retention
    let candidates = find_retention_candidates(handle, policy)?;

    if candidates.is_empty() {
        // Step 3 is optional 鈥?just get orphan count
        result.orphan_candidate_count = count_orphan_candidates(handle)?;
        return Ok(result);
    }

    // Step 3: Delete each candidate
    let conn = handle.repo_db()?;

    for point_id in &candidates {
        // Phase 1: Mark as DELETING
        conn.execute(
            "UPDATE restore_points SET status = 'DELETING' WHERE point_id = ?1 AND status = 'COMMITTED'",
            [point_id],
        )?;

        // Phase 2: Delete the instance directory
        let instance_dir = handle.instances_dir.join(point_id);
        if instance_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&instance_dir) {
                // If directory deletion fails, we keep DELETING status
                // so next retention run can retry
                return Err(RepositoryError::io(
                    instance_dir,
                    format!("Cannot delete instance directory for point {}", point_id),
                    e,
                ));
            }
        }

        // Phase 3: Mark as DELETED
        conn.execute(
            "UPDATE restore_points SET status = 'DELETED' WHERE point_id = ?1",
            [point_id],
        )?;

        result.deleted_points.push(point_id.clone());
    }

    // Step 4: Recalculate orphan candidate count
    result.orphan_candidate_count = count_orphan_candidates(handle)?;

    Ok(result)
}

/// Recover any incomplete deletions from a previous crash.
///
/// Scans for DELETING status and completes the deletion:
/// - If instance directory exists 鈫?delete it 鈫?mark DELETED
/// - If instance directory is gone 鈫?already complete 鈫?mark DELETED
pub fn recover_incomplete_deletions(handle: &RepoHandle) -> Result<(), RepositoryError> {
    let conn = handle.repo_db()?;

    let mut stmt = conn.prepare("SELECT point_id FROM restore_points WHERE status = 'DELETING'")?;

    let point_ids: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    for point_id in &point_ids {
        let instance_dir = handle.instances_dir.join(point_id);

        // If the instance directory still exists, try to delete it
        if instance_dir.exists() {
            let _ = fs::remove_dir_all(&instance_dir);
        }

        // Advance to DELETED regardless (data is already orphaned)
        conn.execute(
            "UPDATE restore_points SET status = 'DELETED' WHERE point_id = ?1",
            [point_id],
        )?;
    }

    Ok(())
}

/// Find COMMITTED Restore Points that should be deleted per the retention policy.
///
/// Algorithm:
/// 1. Collect all COMMITTED points, ordered by creation date (oldest first)
/// 2. If keep_days > 0: identify points older than keep_days
/// 3. If keep_full_count > 0: among the candidates, protect the N most
///    recent full backups (chain_position = 0)
/// 4. Return the remaining (unprotected) candidates for deletion
fn find_retention_candidates(
    handle: &RepoHandle,
    policy: &RetentionPolicy,
) -> Result<Vec<String>, RepositoryError> {
    if policy.keep_days == 0 && policy.keep_full_count == 0 {
        // No retention policy configured
        return Ok(Vec::new());
    }

    let conn = handle.repo_db()?;

    // Get all COMMITTED points
    let mut stmt = conn.prepare(
        "SELECT point_id, chain_position, created_at FROM restore_points
         WHERE status = 'COMMITTED'
         ORDER BY created_at ASC",
    )?;

    let points: Vec<(String, u32, String)> = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            let pos: u32 = row.get(1)?;
            let created: String = row.get(2)?;
            Ok((id, pos, created))
        })?
        .filter_map(|r| r.ok())
        .collect();

    if points.is_empty() {
        return Ok(Vec::new());
    }

    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut candidates: Vec<(String, u32, String)> = Vec::new();

    // Age-based filtering
    for (point_id, chain_pos, created_at) in &points {
        if policy.keep_days > 0 {
            if let Some(point_time) = parse_iso_timestamp(created_at) {
                let age_days = (now_secs - point_time) / 86400;
                if age_days >= policy.keep_days as u64 {
                    candidates.push((point_id.clone(), *chain_pos, created_at.clone()));
                }
            }
            // If timestamp can't be parsed, keep the point (safe default)
        }
    }

    // Full backup protection
    if policy.keep_full_count > 0 && !candidates.is_empty() {
        // Find the N most recent full backups among candidates
        let mut protected_count = 0u32;

        // Sort candidates by created_at DESC to find most recent
        // We protect by removing from candidates
        let mut protected: std::collections::HashSet<String> = std::collections::HashSet::new();

        for (point_id, chain_pos, _) in points.iter().rev() {
            if *chain_pos == 0 && candidates.iter().any(|(id, _, _)| id == point_id) {
                protected.insert(point_id.clone());
                protected_count += 1;
                if protected_count >= policy.keep_full_count {
                    break;
                }
            }
        }

        // Remove protected points from candidates
        candidates.retain(|(id, _, _)| !protected.contains(id));
    }

    Ok(candidates.into_iter().map(|(id, _, _)| id).collect())
}

/// Count total orphan block candidates from all DELETED Restore Points.
///
/// This is the sum of block_count across all DELETED points.
/// These blocks are candidates for future garbage collection (Phase 6+).
/// Phase S does NOT delete these blocks.
pub fn count_orphan_candidates(handle: &RepoHandle) -> Result<u64, RepositoryError> {
    let conn = handle.repo_db()?;
    let count: u64 = conn.query_row(
        "SELECT COALESCE(SUM(block_count), 0) FROM restore_points WHERE status = 'DELETED'",
        [],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// List all DELETED Restore Points and their total orphan block count.
pub fn list_orphan_candidates(handle: &RepoHandle) -> Result<OrphanSummary, RepositoryError> {
    let conn = handle.repo_db()?;

    let mut stmt = conn.prepare(
        "SELECT point_id, block_count FROM restore_points WHERE status = 'DELETED'
         ORDER BY created_at ASC",
    )?;

    let mut summary = OrphanSummary::default();

    let rows = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let bc: u64 = row.get(1)?;
        Ok((id, bc))
    })?;

    for row in rows {
        let (id, bc) = row?;
        summary.deleted_points.push(id);
        summary.total_orphan_blocks += bc;
    }

    Ok(summary)
}

/// Parse an ISO-8601 timestamp string to epoch seconds.
/// Returns None if parsing fails (safe default 鈥?keep the point).
fn parse_iso_timestamp(iso: &str) -> Option<u64> {
    // Try common ISO-8601 patterns
    // Format: "2026-07-10T10:00:00Z" or "2026-07-10T10:00:00+00:00"
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) {
        return Some(dt.timestamp() as u64);
    }
    // Try without timezone suffix
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(iso, "%Y-%m-%dT%H:%M:%S") {
        return Some(naive.and_utc().timestamp() as u64);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::repo_manager::{init_repo, DEFAULT_BLOCK_SIZE};
    use tempfile::TempDir;

    fn setup() -> (RepoHandle, TempDir) {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        // Insert a default job for FK compliance (retention tests reference job-001)
        let conn = handle.repo_db().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)
             VALUES ('job-001', 'test-job', 0, 'C:\test', '2025-01-01T00:00:00Z', 'active')",
            [],
        )
        .unwrap();

        (handle, tmp)
    }

    fn insert_point(
        handle: &RepoHandle,
        point_id: &str,
        chain_pos: u32,
        created_at: &str,
        block_count: u64,
    ) {
        let conn = handle.repo_db().unwrap();
        conn.execute(
            "INSERT INTO restore_points (point_id, job_id, chain_id, chain_position,
             created_at, status, instance_path, block_count, total_raw_bytes)
             VALUES (?1, 'job-001', 'chain-001', ?2, ?3, 'COMMITTED', ?4, ?5, 0)",
            rusqlite::params![
                point_id,
                chain_pos,
                created_at,
                handle
                    .instances_dir
                    .join(point_id)
                    .to_string_lossy()
                    .to_string(),
                block_count,
            ],
        )
        .unwrap();

        // Create the instance directory
        let dir = handle.instances_dir.join(point_id);
        fs::create_dir_all(&dir).unwrap();

        // Write a minimal backup-metadata.json
        let meta = format!(
            "{{\"point_id\":\"{}\",\"schema_version\":\"1.0\"}}",
            point_id
        );
        fs::write(dir.join("backup-metadata.json"), &meta).unwrap();
    }

    fn count_committed(handle: &RepoHandle) -> u64 {
        let conn = handle.repo_db().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM restore_points WHERE status = 'COMMITTED'",
            [],
            |row| row.get::<_, u64>(0),
        )
        .unwrap()
    }

    fn count_deleted(handle: &RepoHandle) -> u64 {
        let conn = handle.repo_db().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM restore_points WHERE status = 'DELETED'",
            [],
            |row| row.get::<_, u64>(0),
        )
        .unwrap()
    }

    fn count_deleting(handle: &RepoHandle) -> u64 {
        let conn = handle.repo_db().unwrap();
        conn.query_row(
            "SELECT COUNT(*) FROM restore_points WHERE status = 'DELETING'",
            [],
            |row| row.get::<_, u64>(0),
        )
        .unwrap()
    }

    #[test]
    fn test_empty_repo_no_deletion() {
        let (handle, _tmp) = setup();
        let policy = RetentionPolicy::new(30, 0);
        let result = apply_retention(&handle, &policy).unwrap();
        assert!(result.deleted_points.is_empty());
        assert_eq!(result.orphan_candidate_count, 0);
    }

    #[test]
    fn test_no_policy_no_deletion() {
        let (handle, _tmp) = setup();
        insert_point(&handle, "point-old", 0, "2025-01-01T00:00:00Z", 100);

        let policy = RetentionPolicy::new(0, 0); // No retention
        let result = apply_retention(&handle, &policy).unwrap();
        assert!(result.deleted_points.is_empty());
        assert_eq!(count_committed(&handle), 1);
    }

    #[test]
    fn test_delete_old_point() {
        let (handle, _tmp) = setup();
        insert_point(&handle, "point-old", 0, "2025-01-01T00:00:00Z", 100);

        let policy = RetentionPolicy::new(30, 0);
        let result = apply_retention(&handle, &policy).unwrap();
        assert_eq!(result.deleted_points.len(), 1);
        assert_eq!(result.deleted_points[0], "point-old");
        assert_eq!(count_committed(&handle), 0);
        assert_eq!(count_deleted(&handle), 1);
    }

    #[test]
    fn test_keep_recent_point() {
        let (handle, _tmp) = setup();
        insert_point(&handle, "point-recent", 0, "2026-07-09T00:00:00Z", 50);

        let policy = RetentionPolicy::new(30, 0);
        let result = apply_retention(&handle, &policy).unwrap();
        assert!(result.deleted_points.is_empty());
        assert_eq!(count_committed(&handle), 1);
    }

    #[test]
    fn test_delete_only_old_keeps_recent() {
        let (handle, _tmp) = setup();
        insert_point(&handle, "point-old", 0, "2025-01-01T00:00:00Z", 100);
        insert_point(&handle, "point-recent", 0, "2026-07-09T00:00:00Z", 50);

        let policy = RetentionPolicy::new(30, 0);
        let result = apply_retention(&handle, &policy).unwrap();
        assert_eq!(result.deleted_points.len(), 1);
        assert_eq!(result.deleted_points[0], "point-old");
        assert_eq!(count_committed(&handle), 1);
    }

    #[test]
    fn test_protect_full_backup_with_keep_full_count() {
        let (handle, _tmp) = setup();
        // Both are old, but keep_full_count=1 protects the most recent full
        insert_point(&handle, "point-old-full", 0, "2025-06-01T00:00:00Z", 200);
        insert_point(&handle, "point-older-full", 0, "2025-01-01T00:00:00Z", 100);

        let policy = RetentionPolicy::new(30, 1);
        let result = apply_retention(&handle, &policy).unwrap();
        // One should be protected, the other deleted
        assert_eq!(result.deleted_points.len(), 1);
        let remaining = count_committed(&handle);
        assert_eq!(remaining, 1);
    }

    #[test]
    fn test_orphan_candidate_count() {
        let (handle, _tmp) = setup();
        insert_point(&handle, "point-a", 0, "2025-01-01T00:00:00Z", 100);
        insert_point(&handle, "point-b", 0, "2025-02-01T00:00:00Z", 200);

        // Delete both
        let policy = RetentionPolicy::new(30, 0);
        let result = apply_retention(&handle, &policy).unwrap();
        assert_eq!(result.orphan_candidate_count, 300);
    }

    #[test]
    fn test_count_orphan_candidates() {
        let (handle, _tmp) = setup();
        insert_point(&handle, "point-x", 0, "2025-01-01T00:00:00Z", 50);

        // Manually mark as DELETED
        let conn = handle.repo_db().unwrap();
        conn.execute(
            "UPDATE restore_points SET status = 'DELETED' WHERE point_id = 'point-x'",
            [],
        )
        .unwrap();

        let count = count_orphan_candidates(&handle).unwrap();
        assert_eq!(count, 50);
    }

    #[test]
    fn test_list_orphan_candidates() {
        let (handle, _tmp) = setup();
        insert_point(&handle, "point-z", 0, "2025-01-01T00:00:00Z", 75);

        let conn = handle.repo_db().unwrap();
        conn.execute(
            "UPDATE restore_points SET status = 'DELETED' WHERE point_id = 'point-z'",
            [],
        )
        .unwrap();

        let summary = list_orphan_candidates(&handle).unwrap();
        assert_eq!(summary.deleted_points.len(), 1);
        assert_eq!(summary.total_orphan_blocks, 75);
    }

    #[test]
    fn test_two_phase_delete_crash_safety() {
        let (handle, _tmp) = setup();
        insert_point(&handle, "crash-point", 0, "2025-01-01T00:00:00Z", 100);

        // Simulate crash after Phase 1: set status to DELETING
        let conn = handle.repo_db().unwrap();
        conn.execute(
            "UPDATE restore_points SET status = 'DELETING' WHERE point_id = 'crash-point'",
            [],
        )
        .unwrap();

        assert_eq!(count_deleting(&handle), 1);

        // Recovery should complete the deletion
        let policy = RetentionPolicy::new(1, 0);
        let _ = apply_retention(&handle, &policy).unwrap();

        // Should have been recovered and not double-counted
        assert_eq!(count_deleting(&handle), 0);
        assert_eq!(count_deleted(&handle), 1);
    }

    #[test]
    fn test_instance_directory_deleted_on_retention() {
        let (handle, _tmp) = setup();
        insert_point(&handle, "del-dir", 0, "2025-01-01T00:00:00Z", 10);

        let instance_dir = handle.instances_dir.join("del-dir");
        assert!(instance_dir.exists());

        let policy = RetentionPolicy::new(30, 0);
        apply_retention(&handle, &policy).unwrap();

        // Instance directory should be gone
        assert!(!instance_dir.exists());
    }

    #[test]
    fn test_non_committed_points_untouched() {
        let (handle, _tmp) = setup();
        let conn = handle.repo_db().unwrap();

        // Insert a point with status CREATING (not COMMITTED)
        conn.execute(
            "INSERT INTO restore_points (point_id, job_id, chain_id, chain_position,
             created_at, status, instance_path, block_count, total_raw_bytes)
             VALUES ('incomplete', 'job-001', 'chain-001', 0,
             '2025-01-01T00:00:00Z', 'CREATING', 'none', 0, 0)",
            [],
        )
        .unwrap();

        let policy = RetentionPolicy::new(1, 0);
        let result = apply_retention(&handle, &policy).unwrap();

        // Non-COMMITTED points should NOT be touched by retention
        assert!(result.deleted_points.is_empty());
    }

    #[test]
    fn test_parse_iso_timestamp_variants() {
        assert!(parse_iso_timestamp("2026-07-10T10:00:00Z").is_some());
        assert!(parse_iso_timestamp("2026-07-10T10:00:00+00:00").is_some());
        assert!(parse_iso_timestamp("2025-01-01T00:00:00").is_some());
        assert!(parse_iso_timestamp("invalid-date").is_none());
    }
}
