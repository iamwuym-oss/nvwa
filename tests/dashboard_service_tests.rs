// ============================================================================
// dashboard_service_tests.rs -- Test the dashboard data aggregation service
//
// Test scenarios (T2.5-03A spec):
//   1. With jobs + with history       -> Protected, Healthy
//   2. With jobs + no history          -> Critical, Critical
//   3. No config                       -> error (Config category)
//   4. Corrupt config                  -> error (Config category)
// ============================================================================

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use nuwa_backup::app::models::common::ProtectionStatus;
use nuwa_backup::app::services::dashboard_service;

// Tests that call with_clean_dir() must be serialized because they
// change the global process CWD (Config::load() uses relative path).
// Parallel CWD changes cause tests to read the wrong nuwa.toml.
static CWD_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a minimal valid config that lives in a real temp directory.
/// Returns the config path.
fn create_minimal_config_in(dir: &Path) {
    // Use real paths within the temp dir so we can actually create files there
    let src_1 = dir
        .join("source_docs")
        .to_string_lossy()
        .replace("\\", "\\\\");
    let dst_1 = dir
        .join("dest_docs")
        .to_string_lossy()
        .replace("\\", "\\\\");
    let src_2 = dir
        .join("source_proj")
        .to_string_lossy()
        .replace("\\", "\\\\");
    let dst_2 = dir
        .join("dest_proj")
        .to_string_lossy()
        .replace("\\", "\\\\");

    let toml = format!(
        r#"
[job.Documents]
source = "{}"
dest = "{}"

[job.Projects]
source = "{}"
dest = "{}"
"#,
        src_1, dst_1, src_2, dst_2
    );

    fs::write(dir.join("nuwa.toml"), toml).unwrap();
}

fn create_corrupt_config_in(dir: &Path) {
    fs::write(dir.join("nuwa.toml"), "[[[ invalid toml [[[").unwrap();
}

fn create_history_with_success(db_path: &Path) {
    use nuwa_backup::history::{HistoryDb, OperationRecord};
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let db = HistoryDb::open_or_create(db_path).unwrap();
    db.record_operation(&OperationRecord {
        backup_id: "test-bp-001".into(),
        operation: "backup".into(),
        timestamp: "2026-07-07T10:00:00Z".into(),
        source_root: "C:\\source".into(),
        dest_path: db_path.parent().unwrap().to_string_lossy().into_owned(),
        job_name: Some("Documents".into()),
        file_count: 100,
        total_bytes: 1048576,
        duration_ms: 5000,
        exit_code: 0,
        status: "success".into(),
    })
    .unwrap();
}

fn with_clean_dir<F: FnOnce(&Path) -> T, T>(name: &str, f: F) -> T {
    // Serialize CWD-dependent tests (Config::load() uses relative path)
    // so parallel test threads don't trample each other's working directory.
    let _lock = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let dir = std::env::temp_dir().join("nuwa_test").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    // Set CWD to this dir so config::load() picks up nuwa.toml
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();

    let result = f(&dir);

    std::env::set_current_dir(&old).unwrap();
    let _ = fs::remove_dir_all(&dir);
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn test_overview_with_jobs_and_history() {
    with_clean_dir("with_jobs_and_history", |dir| {
        create_minimal_config_in(dir);

        // Create a real dest dir and seed a history DB with a success record
        let dest = dir.join("dest_docs");
        let history_path = nuwa_backup::history::HistoryDb::history_db_path(&dest);
        create_history_with_success(&history_path);

        match dashboard_service::get_overview() {
            Ok(overview) => {
                assert_eq!(
                    overview.protection_status,
                    ProtectionStatus::Protected,
                    "With a successful backup, status should be Protected"
                );
                assert_eq!(overview.total_jobs, 2, "Config has 2 jobs");
                assert!(
                    overview.last_backup.is_some(),
                    "Should have a last backup record"
                );
                let lb = overview.last_backup.as_ref().unwrap();
                assert_eq!(lb.status, "success", "Last backup status should be success");
                assert!(
                    !overview.recent_activity.is_empty(),
                    "Should have recent activity"
                );
            }
            Err(e) => panic!("Expected Ok, got AppError({}): {}", e.category, e.message),
        }
    });
}

#[test]
fn test_overview_with_jobs_no_history() {
    with_clean_dir("with_jobs_no_history", |dir| {
        create_minimal_config_in(dir);

        match dashboard_service::get_overview() {
            Ok(overview) => {
                assert_eq!(
                    overview.protection_status,
                    ProtectionStatus::Critical,
                    "With no history, status should be Critical"
                );
                assert_eq!(overview.total_jobs, 2, "Config has 2 jobs");
                assert!(overview.last_backup.is_none(), "Should have no last backup");
                assert!(
                    overview.recent_activity.is_empty(),
                    "Should have no activity"
                );
            }
            Err(e) => panic!("Expected Ok, got AppError({}): {}", e.category, e.message),
        }
    });
}

#[test]
fn test_overview_no_config() {
    with_clean_dir("no_config", |_dir| {
        match dashboard_service::get_overview() {
            Ok(_) => panic!("Expected error when no config exists"),
            Err(e) => {
                assert_eq!(
                    e.category, "Config",
                    "Error category should be Config, got: {}",
                    e.category
                );
            }
        }
    });
}

#[test]
fn test_overview_corrupt_config() {
    with_clean_dir("corrupt_config", |dir| {
        create_corrupt_config_in(dir);

        match dashboard_service::get_overview() {
            Ok(_) => panic!("Expected error when config is corrupt"),
            Err(e) => {
                assert_eq!(
                    e.category, "Config",
                    "Error category should be Config for corrupt config, got: {}",
                    e.category
                );
            }
        }
    });
}
