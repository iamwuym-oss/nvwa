// ============================================================================
// gui_main.rs — Nuwa Backup GUI binary entry point
//
// This binary is only built when the "gui" feature is enabled.
// cargo build --features gui  →  builds nuwa-gui.exe
// cargo build (default)       →  builds nuwa-backup.exe (CLI only)
// ============================================================================

#[cfg(feature = "gui")]
fn main() {
    nuwa_backup::gui::run();
}

#[cfg(not(feature = "gui"))]
fn main() {
    eprintln!("GUI feature is not enabled.");
    eprintln!("Build with: cargo build --features gui");
    std::process::exit(1);
}
