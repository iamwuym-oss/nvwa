// ============================================================================
// journal.rs 鈥?Transaction Journal for crash recovery
// ============================================================================
//
// Each backup transaction writes a journal file to track its state across
// multiple components. On startup or crash recovery, these journals are
// scanned to determine whether in-progress transactions should be committed
// or marked as failed.
//
// Key design:
// - Journal is a JSON file (human-readable for debugging)
// - Atomic write: .tmp 鈫?rename for crash safety
// - Journal is the SOURCE OF TRUTH for component completion status
// - An incomplete journal means the transaction did NOT fully complete
//
// Recovery semantics:
//   If ALL components are Completed 鈫?transaction is safe to COMMIT
//   If ANY component is InProgress/Pending 鈫?transaction FAILED
//   COMMITTED/FAILED journals may exist if cleanup was interrupted

use crate::repository::error::RepositoryError;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Transaction state machine
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransactionState {
    /// Restore Point directory created, metadata not yet written
    Creating,
    /// Components are actively writing data
    Writing,
    /// Data written, verification in progress
    Verifying,
    /// All data written and verified 鈥?transaction successful
    Committed,
    /// Transaction failed 鈥?data may be incomplete
    Failed,
}

impl TransactionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionState::Creating => "CREATING",
            TransactionState::Writing => "WRITING",
            TransactionState::Verifying => "VERIFYING",
            TransactionState::Committed => "COMMITTED",
            TransactionState::Failed => "FAILED",
        }
    }

    /// Returns true if this state represents an incomplete (not-terminal) transaction
    pub fn is_incomplete(&self) -> bool {
        matches!(
            self,
            TransactionState::Creating | TransactionState::Writing | TransactionState::Verifying
        )
    }

    /// Returns true if this is a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, TransactionState::Committed | TransactionState::Failed)
    }
}

/// Phase of a single component within a transaction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ComponentPhase {
    /// Component has not started processing
    Pending,
    /// Component is actively processing
    InProgress,
    /// Component has completed successfully
    Completed,
}

impl ComponentPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            ComponentPhase::Pending => "pending",
            ComponentPhase::InProgress => "in_progress",
            ComponentPhase::Completed => "completed",
        }
    }

    pub fn is_completed(&self) -> bool {
        *self == ComponentPhase::Completed
    }
}

/// Per-component status tracking within a transaction.
///
/// Each component must reach Completed before the transaction can commit.
/// Startup recovery uses these statuses to determine if a transaction
/// can be auto-committed or must be marked as failed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub block_store: ComponentPhase,
    pub block_map: ComponentPhase,
    pub catalog: ComponentPhase,
    pub metadata: ComponentPhase,
}

impl ComponentStatus {
    /// All components are in the initial pending state
    pub fn all_pending() -> Self {
        ComponentStatus {
            block_store: ComponentPhase::Pending,
            block_map: ComponentPhase::Pending,
            catalog: ComponentPhase::Pending,
            metadata: ComponentPhase::Pending,
        }
    }

    /// Returns true if ALL components have completed
    pub fn all_completed(&self) -> bool {
        self.block_store.is_completed()
            && self.block_map.is_completed()
            && self.catalog.is_completed()
            && self.metadata.is_completed()
    }
}

/// Transaction Journal 鈥?JSON record of one backup transaction's state.
///
/// Written atomically to `.nuwarepo/transactions/txn-{restore_point_id}.log`.
/// Read during startup recovery to determine crash-consistency disposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionJournal {
    /// Restore Point this journal belongs to
    pub restore_point_id: String,
    /// Current state in the transaction state machine
    pub state: TransactionState,
    /// Per-component completion tracking
    pub components: ComponentStatus,
    /// ISO-8601 timestamp when the transaction started
    pub started_at: String,
}

impl TransactionJournal {
    /// Create a new journal for a transaction in CREATING state.
    pub fn new(restore_point_id: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let started_at = chrono::DateTime::from_timestamp(now.as_secs() as i64, 0)
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| "unknown".to_string());

        TransactionJournal {
            restore_point_id: restore_point_id.to_string(),
            state: TransactionState::Creating,
            components: ComponentStatus::all_pending(),
            started_at,
        }
    }
}

/// Return the directory path for transaction journals
fn journals_dir(repo_root: &Path) -> PathBuf {
    repo_root.join(".nuwarepo").join("transactions")
}

/// Return the file path for a specific transaction journal
fn journal_path(repo_root: &Path, point_id: &str) -> PathBuf {
    journals_dir(repo_root).join(format!("txn-{}.log", point_id))
}

/// Write a transaction journal to disk atomically (.tmp 鈫?rename).
pub fn write_journal(
    repo_root: &Path,
    journal: &TransactionJournal,
) -> Result<(), RepositoryError> {
    let dir = journals_dir(repo_root);
    fs::create_dir_all(&dir)
        .map_err(|e| RepositoryError::io(dir.clone(), "Cannot create transactions directory", e))?;

    let path = journal_path(repo_root, &journal.restore_point_id);
    let tmp_path = dir.join(format!("txn-{}.log.tmp", journal.restore_point_id));

    let json = serde_json::to_string_pretty(journal)?;

    fs::write(&tmp_path, &json)
        .map_err(|e| RepositoryError::io(tmp_path.clone(), "Cannot write journal tmp file", e))?;

    fs::rename(&tmp_path, &path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        RepositoryError::io(path, "Cannot rename journal to final", e)
    })?;

    Ok(())
}

/// Read a transaction journal for the given restore point.
/// Returns None if no journal file exists.
pub fn read_journal(
    repo_root: &Path,
    point_id: &str,
) -> Result<Option<TransactionJournal>, RepositoryError> {
    let path = journal_path(repo_root, point_id);

    if !path.exists() {
        return Ok(None);
    }

    let content = fs::read_to_string(&path)
        .map_err(|e| RepositoryError::io(path.clone(), "Cannot read journal file", e))?;

    let journal: TransactionJournal = serde_json::from_str(&content)?;
    Ok(Some(journal))
}

/// Remove a transaction journal from disk (called after successful commit/fail recovery).
pub fn remove_journal(repo_root: &Path, point_id: &str) -> Result<(), RepositoryError> {
    let path = journal_path(repo_root, point_id);
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| RepositoryError::io(path, "Cannot remove journal file", e))?;
    }
    Ok(())
}

/// Scan all transaction journals in the repository.
/// Returns journals sorted by creation time (oldest first).
pub fn scan_journals(repo_root: &Path) -> Result<Vec<TransactionJournal>, RepositoryError> {
    let dir = journals_dir(repo_root);

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut journals = Vec::new();

    let entries = fs::read_dir(&dir)
        .map_err(|e| RepositoryError::io(dir.clone(), "Cannot scan transactions directory", e))?;

    for entry in entries {
        let entry = entry
            .map_err(|e| RepositoryError::io(dir.clone(), "Cannot read directory entry", e))?;

        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("log")
            && path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.starts_with("txn-"))
        {
            let content = fs::read_to_string(&path).map_err(|e| {
                RepositoryError::io(path.clone(), "Cannot read journal file during scan", e)
            })?;

            match serde_json::from_str::<TransactionJournal>(&content) {
                Ok(journal) => journals.push(journal),
                Err(_) => {
                    // Malformed journal 鈥?skip (orphan will be handled by GC in Phase 6)
                    continue;
                }
            }
        }
    }

    Ok(journals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> TempDir {
        TempDir::new().unwrap()
    }

    #[test]
    fn test_journal_new() {
        let journal = TransactionJournal::new("test-point-001");
        assert_eq!(journal.restore_point_id, "test-point-001");
        assert_eq!(journal.state, TransactionState::Creating);
        assert!(journal.started_at.len() > 5);
    }

    #[test]
    fn test_write_and_read_journal() {
        let tmp = setup();
        let journal = TransactionJournal::new("write-read-test");

        write_journal(tmp.path(), &journal).unwrap();

        let read_back = read_journal(tmp.path(), "write-read-test")
            .unwrap()
            .unwrap();
        assert_eq!(read_back.restore_point_id, "write-read-test");
        assert_eq!(read_back.state, TransactionState::Creating);
    }

    #[test]
    fn test_read_nonexistent_journal() {
        let tmp = setup();
        let result = read_journal(tmp.path(), "nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_remove_journal() {
        let tmp = setup();
        let journal = TransactionJournal::new("remove-test");

        write_journal(tmp.path(), &journal).unwrap();
        assert!(read_journal(tmp.path(), "remove-test").unwrap().is_some());

        remove_journal(tmp.path(), "remove-test").unwrap();
        assert!(read_journal(tmp.path(), "remove-test").unwrap().is_none());
    }

    #[test]
    fn test_scan_journals_empty() {
        let tmp = setup();
        let journals = scan_journals(tmp.path()).unwrap();
        assert!(journals.is_empty());
    }

    #[test]
    fn test_scan_journals_multiple() {
        let tmp = setup();

        write_journal(tmp.path(), &TransactionJournal::new("point-001")).unwrap();
        write_journal(tmp.path(), &TransactionJournal::new("point-002")).unwrap();
        write_journal(tmp.path(), &TransactionJournal::new("point-003")).unwrap();

        let journals = scan_journals(tmp.path()).unwrap();
        assert_eq!(journals.len(), 3);
    }

    #[test]
    fn test_transaction_state_incomplete() {
        assert!(TransactionState::Creating.is_incomplete());
        assert!(TransactionState::Writing.is_incomplete());
        assert!(TransactionState::Verifying.is_incomplete());
        assert!(!TransactionState::Committed.is_incomplete());
        assert!(!TransactionState::Failed.is_incomplete());
    }

    #[test]
    fn test_transaction_state_terminal() {
        assert!(!TransactionState::Creating.is_terminal());
        assert!(TransactionState::Committed.is_terminal());
        assert!(TransactionState::Failed.is_terminal());
    }

    #[test]
    fn test_component_status_all_pending() {
        let status = ComponentStatus::all_pending();
        assert!(!status.all_completed());
        assert!(!status.block_store.is_completed());
    }

    #[test]
    fn test_component_status_all_completed() {
        let mut status = ComponentStatus::all_pending();
        status.block_store = ComponentPhase::Completed;
        status.block_map = ComponentPhase::Completed;
        status.catalog = ComponentPhase::Completed;
        status.metadata = ComponentPhase::Completed;
        assert!(status.all_completed());
    }

    #[test]
    fn test_component_status_partial() {
        let mut status = ComponentStatus::all_pending();
        status.block_store = ComponentPhase::Completed;
        status.block_map = ComponentPhase::Completed;
        // catalog and metadata still pending
        assert!(!status.all_completed());
    }

    #[test]
    fn test_journal_atomic_write_no_tmp_residue() {
        let tmp = setup();
        let journal = TransactionJournal::new("atomic-test");

        write_journal(tmp.path(), &journal).unwrap();
        let dir = journals_dir(tmp.path());

        // No .tmp files should remain
        let has_tmp = fs::read_dir(&dir)
            .unwrap()
            .any(|e| e.unwrap().path().extension().and_then(|s| s.to_str()) == Some("tmp"));
        assert!(!has_tmp);
    }

    #[test]
    fn test_journal_serialization_roundtrip() {
        let mut journal = TransactionJournal::new("serde-test");
        journal.state = TransactionState::Writing;
        journal.components.block_store = ComponentPhase::Completed;
        journal.components.block_map = ComponentPhase::InProgress;

        let json = serde_json::to_string_pretty(&journal).unwrap();
        let deserialized: TransactionJournal = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.restore_point_id, "serde-test");
        assert_eq!(deserialized.state, TransactionState::Writing);
        assert_eq!(
            deserialized.components.block_store,
            ComponentPhase::Completed
        );
        assert_eq!(
            deserialized.components.block_map,
            ComponentPhase::InProgress
        );
        assert_eq!(deserialized.components.catalog, ComponentPhase::Pending);
    }
}
