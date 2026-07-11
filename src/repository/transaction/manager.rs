// ============================================================================
// manager.rs - CrashConsistencyManager: transaction state machine (P-00C)
// ============================================================================
//
// CrashConsistencyManager guarantees crash-safe backup transactions.
// It tracks the lifecycle of a Restore Point across multiple components:
//
//   BEGIN -> CREATING -> WRITING -> VERIFYING -> COMMITTED
//                                                 |
//                                              FAILED
//
// Key design principles:
// - Transaction journal tracks component completion
// - repo.db restore_points.status is the user-facing truth
// - On crash, startup recovery scans BOTH journals AND repo.db
// - COMMIT sequence: repo.db UPDATE -> journal COMMITTED -> remove journal
// - FAILED transactions are preserved for orphan candidate tracking
//
// P-00C changes (per P-00_File_Backup_Repository_Data_Contract.md v0.7):
// - Section 3.2: begin() sets repo.db=WRITING, journal=CREATING
// - Section 3.2: enter_writing() transitions journal CREATING->WRITING
// - Section 5.3#1: recover_at_startup() never auto-commits from non-terminal states
// - Section 5.3#2: commit() reordered: repo.db before journal
// - Section 5.3#3: commit/verify use RepoHandle API, not raw SQL
// - Section 3.3: RecoveryReport has journal_cleaned + state_mismatch (not auto_committed)
//
// Architecture constraints (Section 2.3, Section 11):
// - block-map.db and catalog.db are NOT rebuildable from block-store
// - Transaction state machine prevents half-written metadata
// - Each Restore Point has exactly one transaction lifecycle

use crate::repository::error::RepositoryError;
use crate::repository::repo_manager::RepoHandle;
use crate::repository::transaction::journal::{
    remove_journal, scan_journals, write_journal, ComponentPhase, ComponentStatus,
    TransactionJournal, TransactionState,
};
use std::path::PathBuf;

/// Summary of recovery actions taken during startup recovery.
///
/// P-00C Section 5.3#1: RecoveryReport tracks journal_cleaned and state_mismatch
/// separately. auto_committed is REMOVED - non-terminal states never auto-commit.
#[derive(Debug, Clone, Default)]
pub struct RecoveryReport {
    /// Journals that were cleaned up (repo.db already COMMITTED)
    pub journal_cleaned: Vec<String>,
    /// Journals with state mismatch (Journal=COMMITTED but repo.db != COMMITTED)
    pub state_mismatch: Vec<String>,
    /// Restore Points marked as FAILED (incomplete/unverified)
    pub marked_failed: Vec<String>,
    /// Orphans: repo.db non-terminal entries with no journal file (P-00 §3.3)
    pub orphans_cleaned: Vec<String>,
    /// Total non-terminal journals or restore points found
    pub total_incomplete: usize,
    /// Any errors encountered during recovery (non-fatal, reported to user)
    pub errors: Vec<String>,
}

/// CrashConsistencyManager - manages the lifecycle of one backup transaction.
///
/// # Usage
/// ```ignore
/// let mgr = CrashConsistencyManager::begin(&repo, "point-001", "job-001")?;
/// mgr.enter_writing(&repo)?;
/// // ... write blocks, block-map, catalog ...
/// mgr.complete_block_store()?;
/// mgr.complete_block_map()?;
/// mgr.complete_catalog()?;
/// mgr.complete_metadata()?;
/// mgr.enter_verify(&repo)?;
/// mgr.commit(&repo)?;   // or mgr.fail(&repo)?;
/// ```
///
/// # Thread Safety
/// NOT thread-safe. Each Restore Point should have at most one active manager.
pub struct CrashConsistencyManager {
    repo_root: PathBuf,
    point_id: String,
    journal: TransactionJournal,
}

impl CrashConsistencyManager {
    /// Begin a new backup transaction (P-00C Section 3.2 Step 2).
    ///
    /// Creates restore_points row in repo.db with status=WRITING
    /// and a transaction journal in state CREATING.
    pub fn begin(repo: &RepoHandle, point_id: &str, job_id: &str) -> Result<Self, RepositoryError> {
        // P-00 Section 3.2 Step 1: repo.db=CREATING (durable before any data write)
        repo.create_restore_point(
            point_id,
            job_id,
            point_id,
            0,
            &chrono::Utc::now().to_rfc3339(),
            "CREATING",
            &format!("backup-instances/{}", point_id),
        )?;

        // P-00 Section 3.2 Step 2: journal=CREATING
        // enter_writing() transitions both repo.db and journal to WRITING
        let journal = TransactionJournal::new(point_id);
        write_journal(&repo.root, &journal)?;

        Ok(CrashConsistencyManager {
            repo_root: repo.root.clone(),
            point_id: point_id.to_string(),
            journal,
        })
    }

    /// Update the journal state and write it to disk.
    fn update_state(&mut self, state: TransactionState) -> Result<(), RepositoryError> {
        self.journal.state = state;
        write_journal(&self.repo_root, &self.journal)
    }

    /// Transition to WRITING state (P-00C Section 3.2 Step 3).
    ///
    /// Updates BOTH repo.db and Journal to WRITING.
    /// Must be called before any data write.
    pub fn enter_writing(&mut self, repo: &RepoHandle) -> Result<(), RepositoryError> {
        if self.journal.state != TransactionState::Creating {
            return Err(RepositoryError::General {
                detail: format!(
                    "enter_writing: expected CREATING state, got {:?}",
                    self.journal.state
                ),
            });
        }
        repo.set_restore_point_status(&self.point_id, "WRITING")?;
        self.update_state(TransactionState::Writing)
    }

    /// Mark the block-store component as completed.
    /// P-00C Section 3.2: Component completion only valid in WRITING or VERIFYING state.
    pub fn complete_block_store(&mut self) -> Result<(), RepositoryError> {
        if self.journal.state != TransactionState::Writing {
            return Err(RepositoryError::General {
                detail: format!(
                    "complete_block_store: expected WRITING or VERIFYING state, got {:?}",
                    self.journal.state
                ),
            });
        }
        self.journal.components.block_store = ComponentPhase::Completed;
        write_journal(&self.repo_root, &self.journal)
    }

    /// Mark the block-map component as completed.
    pub fn complete_block_map(&mut self) -> Result<(), RepositoryError> {
        if self.journal.state != TransactionState::Writing {
            return Err(RepositoryError::General {
                detail: format!(
                    "complete_block_map: expected WRITING or VERIFYING state, got {:?}",
                    self.journal.state
                ),
            });
        }
        self.journal.components.block_map = ComponentPhase::Completed;
        write_journal(&self.repo_root, &self.journal)
    }

    /// Mark the catalog component as completed.
    pub fn complete_catalog(&mut self) -> Result<(), RepositoryError> {
        if self.journal.state != TransactionState::Writing {
            return Err(RepositoryError::General {
                detail: format!(
                    "complete_catalog: expected WRITING or VERIFYING state, got {:?}",
                    self.journal.state
                ),
            });
        }
        self.journal.components.catalog = ComponentPhase::Completed;
        write_journal(&self.repo_root, &self.journal)
    }

    /// Mark the metadata component as completed.
    pub fn complete_metadata(&mut self) -> Result<(), RepositoryError> {
        if self.journal.state != TransactionState::Writing {
            return Err(RepositoryError::General {
                detail: format!(
                    "complete_metadata: expected WRITING or VERIFYING state, got {:?}",
                    self.journal.state
                ),
            });
        }
        self.journal.components.metadata = ComponentPhase::Completed;
        write_journal(&self.repo_root, &self.journal)
    }

    /// Transition to VERIFYING state (P-00C Section 3.2 Step 5).
    ///
    /// All components must be completed before entering verification.
    /// Updates BOTH repo.db and Journal to VERIFYING.
    /// Only from WRITING state - CREATING->VERIFYING is illegal.
    pub fn enter_verify(&mut self, repo: &RepoHandle) -> Result<(), RepositoryError> {
        if self.journal.state != TransactionState::Writing {
            return Err(RepositoryError::General {
                detail: format!(
                    "enter_verify: expected WRITING state, got {:?}",
                    self.journal.state
                ),
            });
        }
        if !self.journal.components.all_completed() {
            return Err(RepositoryError::General {
                detail: format!(
                    "Cannot enter VERIFYING: components incomplete (store={:?}, map={:?}, catalog={:?}, metadata={:?})",
                    self.journal.components.block_store.as_str(),
                    self.journal.components.block_map.as_str(),
                    self.journal.components.catalog.as_str(),
                    self.journal.components.metadata.as_str(),
                ),
            });
        }
        repo.set_restore_point_status(&self.point_id, "VERIFYING")?;
        self.update_state(TransactionState::Verifying)
    }

    /// Commit the transaction (P-00C Section 5.3 #2).
    ///
    /// Sequence (P-00C Section 3.2 Steps 6-9):
    /// 1. repo.db UPDATE SET status=COMMITTED (repo.db is terminal truth)
    /// 2. Write journal with COMMITTED state
    /// 3. Remove journal file
    ///
    /// # Crash Safety
    /// - Crash after step 1: recover_at_startup sees repo.db=COMMITTED -> journal cleanup
    /// - Crash after step 2: same path - repo.db is the terminal truth
    pub fn commit(mut self, repo: &RepoHandle) -> Result<(), RepositoryError> {
        if self.journal.state != TransactionState::Verifying {
            return Err(RepositoryError::General {
                detail: format!(
                    "commit: expected VERIFYING state, got {:?}",
                    self.journal.state
                ),
            });
        }
        // Step 1: repo.db - the terminal truth (P-00C Section 3.2 Step 6)
        repo.set_restore_point_status(&self.point_id, "COMMITTED")?;
        // Step 2: journal state update (P-00C Section 3.2 Step 7)
        self.update_state(TransactionState::Committed)?;
        // Step 3: journal cleanup (P-00C Section 3.2 Step 9)
        remove_journal(&self.repo_root, &self.point_id)
            .map_err(|e| RepositoryError::General {
                detail: format!(
                    "Transaction committed but journal cleanup failed: {}. recover_at_startup() will clean up.",
                    e,
                ),
            })
    }

    /// Fail the transaction.
    ///
    /// Updates repo.db to FAILED and writes FAILED state to journal.
    /// The journal file is preserved for orphan candidate tracking.
    pub fn fail(mut self, repo: &RepoHandle) -> Result<(), RepositoryError> {
        // P-00C fix: guard against terminal states (COMMITTED cannot be failed)
        if self.journal.state.is_terminal() {
            return Err(RepositoryError::General {
                detail: format!(
                    "fail: cannot fail from terminal state {:?}",
                    self.journal.state
                ),
            });
        }
        repo.set_restore_point_status(&self.point_id, "FAILED")?;
        self.update_state(TransactionState::Failed)?;
        // Keep the journal for orphan candidate tracking (P-00C Section 3.2 FAILURE PATH)
        Ok(())
    }

    /// Return the restore point ID for this transaction.
    pub fn point_id(&self) -> &str {
        &self.point_id
    }

    /// Return the current transaction state.
    pub fn state(&self) -> TransactionState {
        self.journal.state
    }

    /// Return the current component status.
    pub fn component_status(&self) -> &ComponentStatus {
        &self.journal.components
    }

    /// Recover incomplete transactions at startup (P-00C Section 5.3 #1).
    ///
    /// Scans BOTH sources (P-00C Section 3.3):
    /// - Phase 1: Existing journal files
    /// - Phase 2: repo.db non-terminal restore_points without journals (orphans)
    ///
    /// Key rule: Non-terminal states (CREATING/WRITING/VERIFYING) ALWAYS resolve to FAILED,
    /// regardless of component completion flags. No auto-commit.
    pub fn recover_at_startup(repo: &RepoHandle) -> Result<RecoveryReport, RepositoryError> {
        let mut report = RecoveryReport::default();

        // Phase 1: Process existing journals
        let journals = scan_journals(&repo.root)?;
        report.total_incomplete = journals.len();

        for journal in journals {
            let point_id = journal.restore_point_id.clone();
            // P-00C fix: propagate DB errors, do not swallow
            let db_status = repo.get_restore_point_status(&point_id)?;

            match (journal.state, db_status.as_deref()) {
                // Journal COMMITTED + repo.db COMMITTED: normal cleanup
                (TransactionState::Committed, Some("COMMITTED")) => {
                    if let Err(e) = remove_journal(&repo.root, &point_id) {
                        report.errors.push(format!(
                            "Failed to remove COMMITTED journal for {}: {}",
                            point_id, e
                        ));
                    } else {
                        report.journal_cleaned.push(point_id.clone());
                    }
                }

                // Journal COMMITTED + repo.db absent: state mismatch, preserve evidence
                (TransactionState::Committed, None) => {
                    report.state_mismatch.push(point_id.clone());
                    report.errors.push(format!(
                        "STATE MISMATCH: Journal for {} is COMMITTED but repo.db entry is missing",
                        point_id
                    ));
                }

                // repo.db COMMITTED (journal in any state): safe cleanup
                (_, Some("COMMITTED")) => {
                    if let Err(e) = remove_journal(&repo.root, &point_id) {
                        report.errors.push(format!(
                            "Failed to remove journal for COMMITTED point {}: {}",
                            point_id, e
                        ));
                    } else {
                        report.journal_cleaned.push(point_id.clone());
                    }
                }

                // Journal COMMITTED + repo.db != COMMITTED: state mismatch
                (TransactionState::Committed, _) => {
                    report.state_mismatch.push(point_id.clone());
                    report.errors.push(format!(
                        "STATE MISMATCH: Journal for {} is COMMITTED but repo.db status is {:?}",
                        point_id, db_status
                    ));
                }

                // Journal FAILED: preserve for orphan tracking
                (TransactionState::Failed, _) => {}

                // Non-terminal (CREATING/WRITING/VERIFYING): always FAILED
                // P-00C fix: journal state is ALSO updated to FAILED for idempotent recovery.
                // Both repo.db and journal must reflect the terminal state.
                _ => {
                    repo.set_restore_point_status(&point_id, "FAILED")?;
                    let mut updated_journal = journal.clone();
                    updated_journal.state = TransactionState::Failed;
                    write_journal(&repo.root, &updated_journal)?;
                    report.marked_failed.push(point_id.clone());
                }
            }
        }

        // Phase 2: Scan repo.db for non-terminal restore points without journals
        // Error is propagated (not swallowed) per P-00C fix: critical DB errors must fail
        let orphans = repo.list_non_terminal_restore_points()?;
        for (point_id, _status) in orphans {
            let journal_path = repo
                .root
                .join(".nuwarepo")
                .join("transactions")
                .join(format!("txn-{}.log", point_id));
            if !journal_path.exists() {
                report.total_incomplete += 1;
                repo.set_restore_point_status(&point_id, "FAILED")?;
                report.marked_failed.push(point_id.clone());
                report.orphans_cleaned.push(point_id);
            }
        }

        Ok(report)
    }
}

// ---------- unit tests ----------
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::repo_manager::{init_repo, RepoHandle, DEFAULT_BLOCK_SIZE};
    use tempfile::TempDir;

    fn setup() -> (RepoHandle, TempDir, String) {
        let tmp = TempDir::new().expect("temp dir");
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).expect("init_repo");
        let job_id = "test-job-gen".to_string();
        // Create backup_job entry to satisfy FOREIGN KEY constraint
        let conn = handle.repo_db().expect("repo_db");
        let _ = conn.execute(
            "INSERT OR IGNORE INTO backup_jobs (job_id, job_name, source_type, created_at, status) VALUES (?1, 'TestJob', 0, '2026-07-11T00:00:00Z', 'active')",
            rusqlite::params![&job_id],
        );
        (handle, tmp, job_id)
    }

    #[test]
    fn test_begin_creates_journal() {
        let (repo, _tmp, _job_id) = setup();
        let mgr = CrashConsistencyManager::begin(&repo, "point-begin", &_job_id).unwrap();
        assert_eq!(mgr.state(), TransactionState::Creating);
    }

    #[test]
    fn test_enter_writing_transitions_state() {
        let (repo, _tmp, _job_id) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-enter", &_job_id).unwrap();
        assert_eq!(mgr.state(), TransactionState::Creating);
        mgr.enter_writing(&repo).unwrap();
        assert_eq!(mgr.state(), TransactionState::Writing);
    }

    #[test]
    fn test_enter_writing_requires_creating() {
        let (repo, _tmp, _job_id) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-bad-enter", &_job_id).unwrap();
        mgr.enter_writing(&repo).unwrap();
        // Second call should fail
        let result = mgr.enter_writing(&repo);
        assert!(result.is_err());
    }

    #[test]
    fn test_complete_block_store() {
        let (repo, _tmp, _job_id) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-store", &_job_id).unwrap();
        mgr.enter_writing(&repo).unwrap();
        mgr.complete_block_store().unwrap();
        assert_eq!(
            mgr.component_status().block_store,
            ComponentPhase::Completed
        );
    }

    #[test]
    fn test_complete_all_components() {
        let (repo, _tmp, _job_id) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-all", &_job_id).unwrap();
        mgr.enter_writing(&repo).unwrap();
        mgr.complete_block_store().unwrap();
        mgr.complete_block_map().unwrap();
        mgr.complete_catalog().unwrap();
        mgr.complete_metadata().unwrap();
        assert!(mgr.component_status().all_completed());
    }

    #[test]
    fn test_enter_verify_after_all_completed() {
        let (repo, _tmp, _job_id) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-verify", &_job_id).unwrap();
        mgr.enter_writing(&repo).unwrap();
        mgr.complete_block_store().unwrap();
        mgr.complete_block_map().unwrap();
        mgr.complete_catalog().unwrap();
        mgr.complete_metadata().unwrap();
        mgr.enter_verify(&repo).unwrap();
        assert_eq!(mgr.state(), TransactionState::Verifying);
    }

    #[test]
    fn test_enter_verify_before_all_completed_fails() {
        let (repo, _tmp, _job_id) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-verify-fail", &_job_id).unwrap();
        mgr.enter_writing(&repo).unwrap();
        mgr.complete_block_store().unwrap();
        // block_map, catalog, metadata NOT completed
        let result = mgr.enter_verify(&repo);
        assert!(result.is_err());
    }

    #[test]
    fn test_commit_removes_journal() {
        let (repo, tmp, _job_id) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-commit", &_job_id).unwrap();
        mgr.enter_writing(&repo).unwrap();
        mgr.complete_block_store().unwrap();
        mgr.complete_block_map().unwrap();
        mgr.complete_catalog().unwrap();
        mgr.complete_metadata().unwrap();
        mgr.enter_verify(&repo).unwrap();
        mgr.commit(&repo).unwrap();
        // Journal should be gone
        let journals = scan_journals(tmp.path()).unwrap();
        assert!(journals.is_empty());
    }

    #[test]
    fn test_fail_keeps_journal() {
        let (repo, tmp, _job_id) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-fail", &_job_id).unwrap();
        mgr.enter_writing(&repo).unwrap();
        mgr.fail(&repo).unwrap();
        // Journal should still exist (for orphan tracking)
        let journals = scan_journals(tmp.path()).unwrap();
        assert_eq!(journals.len(), 1);
        assert_eq!(journals[0].state, TransactionState::Failed);
    }

    #[test]
    fn test_recover_no_journals() {
        let (repo, _tmp, _job_id) = setup();
        let report = CrashConsistencyManager::recover_at_startup(&repo).unwrap();
        assert_eq!(report.total_incomplete, 0);
        assert!(report.journal_cleaned.is_empty());
        assert!(report.marked_failed.is_empty());
    }

    #[test]
    fn test_recover_non_terminal_marked_failed() {
        let (repo, _tmp, _job_id) = setup();
        // P-00C Section 5.3#1: non-terminal + all components completed -> FAILED (not auto-commit)
        let mut mgr = CrashConsistencyManager::begin(&repo, "recover-nonterm", &_job_id).unwrap();
        mgr.enter_writing(&repo).unwrap();
        mgr.complete_block_store().unwrap();
        mgr.complete_block_map().unwrap();
        mgr.complete_catalog().unwrap();
        mgr.complete_metadata().unwrap();
        drop(mgr); // simulate crash
        let report = CrashConsistencyManager::recover_at_startup(&repo).unwrap();
        assert_eq!(report.marked_failed.len(), 1);
        assert_eq!(report.marked_failed[0], "recover-nonterm");
        assert_eq!(
            report.journal_cleaned.len(),
            0,
            "must NOT auto-commit from non-terminal"
        );
    }

    #[test]
    fn test_recover_mark_failed() {
        let (repo, _tmp, _job_id) = setup();
        let mgr = CrashConsistencyManager::begin(&repo, "recover-failed", &_job_id).unwrap();
        drop(mgr); // nothing completed
        let report = CrashConsistencyManager::recover_at_startup(&repo).unwrap();
        assert_eq!(report.marked_failed.len(), 1);
        assert_eq!(report.marked_failed[0], "recover-failed");
    }

    #[test]
    fn test_recover_committed_cleans_journal() {
        let (repo, tmp, _job_id) = setup();
        let point_id = "clean-committed";
        // Create both COMMITTED repo.db entry and COMMITTED journal
        repo.create_restore_point(
            point_id,
            &_job_id,
            point_id,
            0,
            "2026-07-11T00:00:00Z",
            "COMMITTED",
            &format!("backup-instances/{}", point_id),
        )
        .expect("create restore point");
        let mut j = TransactionJournal::new(point_id);
        j.state = TransactionState::Committed;
        write_journal(tmp.path(), &j).unwrap();
        drop(j);
        let report = CrashConsistencyManager::recover_at_startup(&repo).unwrap();
        assert_eq!(report.journal_cleaned.len(), 1);
        assert_eq!(report.journal_cleaned[0], point_id);
        let remaining = scan_journals(tmp.path()).unwrap();
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_full_lifecycle() {
        let (repo, _tmp, _job_id) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "full-lifecycle", &_job_id).unwrap();
        assert_eq!(mgr.state(), TransactionState::Creating);
        mgr.enter_writing(&repo).unwrap();
        mgr.complete_block_store().unwrap();
        mgr.complete_block_map().unwrap();
        mgr.complete_catalog().unwrap();
        mgr.complete_metadata().unwrap();
        mgr.enter_verify(&repo).unwrap();
        assert_eq!(mgr.state(), TransactionState::Verifying);
        mgr.commit(&repo).unwrap();
    }
}
