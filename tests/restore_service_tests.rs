// ============================================================================
// restore_service_tests.rs -- Test the restore service layer
//
// Test scenarios:
//   1. list_restore_points with valid config and history
//   2. list_restore_points with no config file (empty)
//   3. list_restore_points with no history (empty)
//   4. get_restore_preview with invalid backup_id (error)
//   5. execute_restore with empty backup_id (validation error)
//   6. execute_restore with empty dest (validation error)
// ============================================================================

use std::fs;
use std::sync::Mutex;

use nuwa_backup::app::models::restore::RestoreRequest;
use nuwa_backup::app::services::restore_service;

static CWD_LOCK: Mutex<()> = Mutex::new(());

fn lock_cwd() -> std::sync::MutexGuard<'static, ()> {
    CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn with_clean_dir<F: FnOnce(&std::path::Path) -> T + std::panic::UnwindSafe, T>(
    name: &str,
    f: F,
) -> T {
    let _lock = lock_cwd();
    let dir = std::env::temp_dir()
        .join("nuwa_test")
        .join("restore_service")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();
    let result = std::panic::catch_unwind(|| f(&dir));
    std::env::set_current_dir(&old).unwrap();
    let _ = fs::remove_dir_all(&dir);
    drop(_lock);
    match result {
        Ok(v) => v,
        Err(e) => std::panic::resume_unwind(e),
    }
}

fn create_empty_config_in(dir: &std::path::Path) {
    fs::write(
        dir.join("nuwa.toml"),
        "# Nuwa Backup Configuration\n\n[job]\n",
    )
    .unwrap();
}

#[test]
fn test_list_restore_points_no_config() {
    with_clean_dir("no_config", |_dir| {
        let points = restore_service::list_restore_points().unwrap();
        assert!(points.is_empty(), "No config should return empty list");
    });
}

#[test]
fn test_list_restore_points_empty_history() {
    with_clean_dir("empty_history", |dir| {
        create_empty_config_in(dir);
        let points = restore_service::list_restore_points().unwrap();
        // No history records means empty list
        assert!(points.is_empty(), "No history should return empty list");
    });
}

#[test]
fn test_get_restore_preview_invalid_backup_id() {
    with_clean_dir("invalid_preview", |dir| {
        create_empty_config_in(dir);
        let err = restore_service::get_restore_preview("nonexistent").unwrap_err();
        assert!(
            err.message.contains("not found"),
            "Should reject invalid backup_id: {}",
            err.message
        );
    });
}

#[test]
fn test_execute_restore_empty_backup_id() {
    with_clean_dir("empty_id", |dir| {
        create_empty_config_in(dir);
        let err = restore_service::execute_restore(RestoreRequest {
            backup_id: "".into(),
            dest: "D:\\Restore".into(),
            overwrite: false,
        })
        .unwrap_err();
        assert!(
            err.message.contains("empty"),
            "Should reject empty backup_id: {}",
            err.message
        );
    });
}

#[test]
fn test_execute_restore_empty_dest() {
    with_clean_dir("empty_dest", |dir| {
        create_empty_config_in(dir);
        let err = restore_service::execute_restore(RestoreRequest {
            backup_id: "test-bp".into(),
            dest: "".into(),
            overwrite: false,
        })
        .unwrap_err();
        assert!(
            err.message.contains("empty"),
            "Should reject empty dest: {}",
            err.message
        );
    });
}

#[test]
fn test_execute_restore_not_found() {
    with_clean_dir("not_found", |dir| {
        create_empty_config_in(dir);
        let err = restore_service::execute_restore(RestoreRequest {
            backup_id: "nonexistent-bp".into(),
            dest: dir.join("restore").to_string_lossy().into_owned(),
            overwrite: false,
        })
        .unwrap_err();
        assert!(
            err.message.contains("not found"),
            "Should reject nonexistent backup: {}",
            err.message
        );
    });
}
