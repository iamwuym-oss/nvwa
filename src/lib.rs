// ============================================================================
// lib.rs -- Nuwa Backup library entry point
// ============================================================================

pub mod app;
pub mod checksum;
pub mod cli;
pub mod cli_output;
pub mod config;
pub mod diskspace;
pub mod errors;
pub mod history;
pub mod path_support;
pub mod scheduler;

// ============================================================================
// Phase S — Repository Engine
// ============================================================================
#[cfg(feature = "repository")]
pub mod repository;
