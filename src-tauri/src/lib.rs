// ============================================================================
// src-tauri/src/lib.rs -- Tauri 2.0 backend entry: registers commands,
// builds the desktop window, and bridges frontend to the nuwa-backup core.
// ============================================================================

use std::sync::Mutex;

mod commands;

// ---------------------------------------------------------------------------
// App state that holds the shared nuwa-backup configuration path
// ---------------------------------------------------------------------------
pub struct AppState {
    pub config_path: Mutex<Option<String>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            config_path: Mutex::new(None),
        }
    }
}

// ---------------------------------------------------------------------------
// Tauri commands -- each wraps a call into the nuwa-backup core library.
//
// Command layer rules (see AGENTS.md / T2.5-03A spec):
//   1. Only receive params, call a service, return result
//   2. No business logic in commands
//   3. Errors must be AppError (not raw NuwaError)
// ---------------------------------------------------------------------------

/// Return the application and core library version strings.
#[tauri::command]
fn get_version() -> String {
    let core = env!("CARGO_PKG_VERSION");
    format!("Nuwa Backup v{} (GUI)", core)
}

/// Return a summary of the current backup jobs from the config file.
#[tauri::command]
fn list_backup_jobs() -> Result<Vec<(String, nuwa_backup::config::JobConfig)>, String> {
    let config = nuwa_backup::config::Config::load().map_err(|e| e.to_string())?;
    Ok(config.job.into_iter().collect::<Vec<_>>())
}

// ---------------------------------------------------------------------------
// Build and run the Tauri application.
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_version,
            list_backup_jobs,
            commands::dashboard::get_dashboard_overview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Nuwa Backup GUI");
}
