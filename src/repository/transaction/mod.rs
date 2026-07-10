// ============================================================================
// mod.rs — Crash Consistency Manager module entry point
// ============================================================================
//
// Phase S Wave 3 (S-07): Transaction state machine + journal for crash-safe
// backup operations. See Architecture v1.0 §11.

pub mod journal;
pub mod manager;

pub use journal::{ComponentPhase, ComponentStatus, TransactionJournal, TransactionState};
pub use manager::{CrashConsistencyManager, RecoveryReport};
