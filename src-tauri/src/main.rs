// ============================================================================
// src-tauri/src/main.rs -- Tauri 2.0 binary entry point
// ============================================================================

// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    nuwa_tauri_lib::run()
}
