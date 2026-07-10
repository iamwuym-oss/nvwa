// ============================================================================
// mod.rs — Verify Engine module entry point
// ============================================================================
//
// Phase S Wave 4 (S-08): Three-level repository verification.
// See Architecture v1.0 §12.

pub mod engine;

pub use engine::{verify_repo, VerifyLevel, VerifyReport};
