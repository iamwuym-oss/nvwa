// ============================================================================
// mod.rs — Retention Engine module entry point
// ============================================================================
//
// Phase S S-09: Restore Point lifecycle management (logical deletion only).
// See Architecture v1.0 §12.

pub mod engine;

pub use engine::{
    apply_retention, count_orphan_candidates, list_orphan_candidates, recover_incomplete_deletions,
    OrphanSummary, RetentionPolicy, RetentionResult,
};
