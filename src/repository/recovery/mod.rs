// ============================================================================
// mod.rs — Recovery module entry point
// ============================================================================
//
// Phase S S-13: Repository Recovery
// See Architecture v1.0 §13.
//
// Provides:
//   rebuild.rs         — repo.db rebuild from backup-instance metadata
//   integrity_check.rs — Cross-component consistency check

pub mod integrity_check;
pub mod rebuild;

pub use integrity_check::{
    check_integrity, CheckDetail, CheckResult, CheckStatus, IntegrityReport, IntegritySummary,
};
pub use rebuild::rebuild_repo;
