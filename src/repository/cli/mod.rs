// ============================================================================
// mod.rs -- Repository CLI module entry
// ============================================================================

pub mod commands;

pub use commands::{
    cmd_repo_check, cmd_repo_init, cmd_repo_list, cmd_repo_orphans, cmd_repo_rebuild,
    cmd_repo_verify, handle_repo_command,
};
