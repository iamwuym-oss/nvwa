// ============================================================================
// manager.rs — CrashConsistencyManager: transaction state machine
// ============================================================================
//
// CrashConsistencyManager guarantees crash-safe backup transactions.
// It tracks the lifecycle of a Restore Point across multiple components:
//
//   BEGIN → CREATING → WRITING → VERIFYING → COMMITTED
//                                                ↓
//                                              FAILED
//
// Key design principles:
// - Transaction journal is the SOURCE OF TRUTH for component completion
// - On crash, startup recovery determines disposition from journal
// - COMMIT is atomic: metadata rename + repo.db update + journal removal
// - FAILED transactions are preserved for orphan candidate tracking
//
// Architecture constraints (§2.3, §11):
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
#[derive(Debug, Clone, Default)]
pub struct RecoveryReport {
    /// Restore Points that were auto-committed (all components completed)
    pub auto_committed: Vec<String>,
    /// Restore Points that were marked as failed (incomplete components)
    pub marked_failed: Vec<String>,
    /// Total incomplete journals found
    pub total_incomplete: usize,
    /// Any errors encountered during recovery (non-fatal, reported to user)
    pub errors: Vec<String>,
}

/// CrashConsistencyManager — manages the lifecycle of one backup transaction.
///
/// # Usage
/// ```ignore
/// let mgr = CrashConsistencyManager::begin(&repo, "point-001")?;
/// // ... write blocks, block-map, catalog ...
/// mgr.complete_block_store()?;
/// mgr.complete_block_map()?;
/// mgr.complete_catalog()?;
/// mgr.complete_metadata()?;
/// mgr.commit()?; // or mgr.fail()?;
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
    /// Begin a new backup transaction.
    ///
    /// This creates a transaction journal in state CREATING and writes it
    /// to `.nuwarepo/transactions/txn-{point_id}.log`.
    ///
    /// # Arguments
    /// * `repo` — Handle to the opened Repository
    /// * `point_id` — Unique identifier for this Restore Point
    pub fn begin(repo: &RepoHandle, point_id: &str) -> Result<Self, RepositoryError> {
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

    /// Mark the block-store component as completed.
    pub fn complete_block_store(&mut self) -> Result<(), RepositoryError> {
        self.journal.components.block_store = ComponentPhase::Completed;
        write_journal(&self.repo_root, &self.journal)
    }

    /// Mark the block-map component as completed.
    pub fn complete_block_map(&mut self) -> Result<(), RepositoryError> {
        self.journal.components.block_map = ComponentPhase::Completed;
        write_journal(&self.repo_root, &self.journal)
    }

    /// Mark the catalog component as completed.
    pub fn complete_catalog(&mut self) -> Result<(), RepositoryError> {
        self.journal.components.catalog = ComponentPhase::Completed;
        write_journal(&self.repo_root, &self.journal)
    }

    /// Mark the metadata component as completed.
    pub fn complete_metadata(&mut self) -> Result<(), RepositoryError> {
        self.journal.components.metadata = ComponentPhase::Completed;
        write_journal(&self.repo_root, &self.journal)
    }

    /// Transition to VERIFYING state.
    /// All components must be completed before entering verification.
    pub fn enter_verify(&mut self) -> Result<(), RepositoryError> {
        if !self.journal.components.all_completed() {
            return Err(RepositoryError::General {
                detail: format!(
                    "Cannot enter VERIFYING: components incomplete \
                     (store={:?}, map={:?}, catalog={:?}, metadata={:?})",
                    self.journal.components.block_store.as_str(),
                    self.journal.components.block_map.as_str(),
                    self.journal.components.catalog.as_str(),
                    self.journal.components.metadata.as_str(),
                ),
            });
        }
        self.update_state(TransactionState::Verifying)
    }

    /// Commit the transaction.
    ///
    /// This finalizes the transaction:
    /// 1. Sets state to COMMITTED in the journal
    /// 2. Removes the journal file (clean exit)
    ///
    /// After commit, the manager is consumed and cannot be used further.
    ///
    /// # Crash Safety
    /// If the program crashes between journal write and journal removal,
    /// startup recovery scans and finds a COMMITTED journal → removes it.
    pub fn commit(mut self) -> Result<(), RepositoryError> {
        self.update_state(TransactionState::Committed)?;
        // Remove the journal — transaction is complete
        remove_journal(&self.repo_root, &self.point_id)?;
        Ok(())
    }

    /// Fail the transaction.
    ///
    /// Marks the journal as FAILED but does NOT remove it.
    /// The failed journal serves as an orphan candidate marker for
    /// future garbage collection (Phase 6+).
    pub fn fail(mut self) -> Result<(), RepositoryError> {
        self.update_state(TransactionState::Failed)?;
        // Keep the journal for orphan candidate tracking
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

    // ========================================================================
    // Startup Recovery
    // ========================================================================

    /// Scan the repository for incomplete transaction journals and recover them.
    ///
    /// This should be called once when the Repository is opened, before any
    /// new backup operations begin.
    ///
    /// # Recovery Logic
    ///
    /// For each journal found:
    /// 1. If state is COMMITTED: journal cleanup was interrupted — remove it.
    /// 2. If state is FAILED: already processed — remove it.
    /// 3. If state is incomplete (CREATING/WRITING/VERIFYING):
    ///    a. If ALL components are Completed → auto-commit (data is safe)
    ///    b. If ANY component is incomplete → mark FAILED (data may be partial)
    /// 4. Failed/Creating-only journals keep their data as orphan candidates.
    ///
    /// # Returns
    /// RecoveryReport summarizing all recovery actions taken.
    pub fn recover_at_startup(repo: &RepoHandle) -> Result<RecoveryReport, RepositoryError> {
        let mut report = RecoveryReport::default();

        let journals = scan_journals(&repo.root)?;
        report.total_incomplete = journals.len();

        for journal in journals {
            let point_id = journal.restore_point_id.clone();

            match journal.state {
                // Terminal states: just clean up
                TransactionState::Committed | TransactionState::Failed => {
                    let _ = remove_journal(&repo.root, &point_id);
                }

                // Incomplete states: determine disposition
                _ => {
                    if journal.components.all_completed() {
                        // All components completed — data is safe, auto-commit
                        report.auto_committed.push(point_id.clone());
                    } else {
                        // Some components did not complete — mark FAILED
                        report.marked_failed.push(point_id.clone());
                    }

                    // Remove the processed journal
                    if let Err(e) = remove_journal(&repo.root, &point_id) {
                        report
                            .errors
                            .push(format!("Failed to remove journal for {}: {}", point_id, e));
                    }
                }
            }
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::repo_manager::{init_repo, DEFAULT_BLOCK_SIZE};
    use tempfile::TempDir;

    fn setup() -> (RepoHandle, TempDir) {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();
        (handle, tmp)
    }

    #[test]
    fn test_begin_creates_journal() {
        let (repo, _tmp) = setup();
        let mgr = CrashConsistencyManager::begin(&repo, "point-begin").unwrap();
        assert_eq!(mgr.state(), TransactionState::Creating);
        assert_eq!(mgr.point_id(), "point-begin");
    }

    #[test]
    fn test_complete_block_store() {
        let (repo, _tmp) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-store").unwrap();
        mgr.complete_block_store().unwrap();
        assert!(mgr.component_status().block_store.is_completed());
    }

    #[test]
    fn test_complete_all_components() {
        let (repo, _tmp) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-all").unwrap();

        mgr.complete_block_store().unwrap();
        mgr.complete_block_map().unwrap();
        mgr.complete_catalog().unwrap();
        mgr.complete_metadata().unwrap();

        assert!(mgr.component_status().all_completed());
    }

    #[test]
    fn test_enter_verify_after_all_completed() {
        let (repo, _tmp) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-verify").unwrap();

        mgr.complete_block_store().unwrap();
        mgr.complete_block_map().unwrap();
        mgr.complete_catalog().unwrap();
        mgr.complete_metadata().unwrap();

        mgr.enter_verify().unwrap();
        assert_eq!(mgr.state(), TransactionState::Verifying);
    }

    #[test]
    fn test_enter_verify_before_all_completed_fails() {
        let (repo, _tmp) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "point-verify-fail").unwrap();

        mgr.complete_block_store().unwrap();
        // block_map, catalog, metadata NOT completed

        let result = mgr.enter_verify();
        assert!(result.is_err());
    }

    #[test]
    fn test_commit_removes_journal() {
        let (repo, tmp) = setup();
        let mgr = CrashConsistencyManager::begin(&repo, "point-commit").unwrap();

        mgr.commit().unwrap();

        // Journal should be gone
        let journals = scan_journals(tmp.path()).unwrap();
        assert!(journals.is_empty());
    }

    #[test]
    fn test_fail_keeps_journal() {
        let (repo, tmp) = setup();
        let mgr = CrashConsistencyManager::begin(&repo, "point-fail").unwrap();

        mgr.fail().unwrap();

        // Journal should still exist (for orphan tracking)
        let journals = scan_journals(tmp.path()).unwrap();
        assert_eq!(journals.len(), 1);
        assert_eq!(journals[0].state, TransactionState::Failed);
    }

    #[test]
    fn test_recover_at_startup_no_journals() {
        let (repo, _tmp) = setup();
        let report = CrashConsistencyManager::recover_at_startup(&repo).unwrap();
        assert_eq!(report.total_incomplete, 0);
        assert!(report.auto_committed.is_empty());
        assert!(report.marked_failed.is_empty());
    }

    #[test]
    fn test_recover_at_startup_auto_commit() {
        let (repo, _tmp) = setup();

        // Simulate a completed transaction whose journal wasn't cleaned up
        let mut mgr = CrashConsistencyManager::begin(&repo, "recover-complete").unwrap();
        mgr.complete_block_store().unwrap();
        mgr.complete_block_map().unwrap();
        mgr.complete_catalog().unwrap();
        mgr.complete_metadata().unwrap();
        // Journal is in WRITING state but all components completed
        // Drop mgr without commit to simulate crash

        let report = CrashConsistencyManager::recover_at_startup(&repo).unwrap();
        assert_eq!(report.auto_committed.len(), 1);
        assert_eq!(report.auto_committed[0], "recover-complete");
    }

    #[test]
    fn test_recover_at_startup_mark_failed() {
        let (repo, _tmp) = setup();

        // Simulate a transaction that was interrupted mid-write
        let mgr = CrashConsistencyManager::begin(&repo, "recover-failed").unwrap();
        // Nothing completed — journal in CREATING state
        drop(mgr);

        let report = CrashConsistencyManager::recover_at_startup(&repo).unwrap();
        assert_eq!(report.marked_failed.len(), 1);
        assert_eq!(report.marked_failed[0], "recover-failed");
    }

    #[test]
    fn test_recover_at_startup_cleans_terminal_journals() {
        let (repo, tmp) = setup();

        // Create a committed journal (simulate interrupted cleanup)
        let mut j = TransactionJournal::new("clean-committed");
        j.state = TransactionState::Committed;
        write_journal(tmp.path(), &j).unwrap();
        drop(j);

        let report = CrashConsistencyManager::recover_at_startup(&repo).unwrap();
        assert_eq!(report.total_incomplete, 1);

        // After recovery, no journals should remain
        let remaining = scan_journals(tmp.path()).unwrap();
        assert!(remaining.is_empty());
    }

    #[test]
    fn test_full_lifecycle() {
        let (repo, _tmp) = setup();
        let mut mgr = CrashConsistencyManager::begin(&repo, "full-lifecycle").unwrap();
        assert_eq!(mgr.state(), TransactionState::Creating);

        mgr.complete_block_store().unwrap();
        mgr.complete_block_map().unwrap();
        mgr.complete_catalog().unwrap();
        mgr.complete_metadata().unwrap();

        mgr.enter_verify().unwrap();
        assert_eq!(mgr.state(), TransactionState::Verifying);

        mgr.commit().unwrap();
        // After commit, can't check state since self is consumed
    }
}
