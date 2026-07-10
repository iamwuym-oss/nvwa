// ============================================================================
// config_service_tests.rs -- Test the config job CRUD service layer
// ============================================================================

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use nuwa_backup::app::models::config_job::JobConfigRequest;
use nuwa_backup::app::services::config_service;

static CWD_LOCK: Mutex<()> = Mutex::new(());

fn lock_cwd() -> std::sync::MutexGuard<'static, ()> {
    CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn create_empty_config_in(dir: &Path) {
    let content = "# Nuwa Backup Configuration\n\n[job]\n";
    fs::write(dir.join("nuwa.toml"), content).unwrap();
}

fn create_config_with_jobs_in(dir: &Path) {
    let toml = r#"
[job.documents]
source = "C:/Users/Test/Documents"
dest = "D:/Backup/Documents"
compress = true
retention = { keep_count = 7, keep_days = 30 }

[job.photos]
source = "C:/Users/Test/Pictures"
dest = "D:/Backup/Photos"
compress = false
"#;
    fs::write(dir.join("nuwa.toml"), toml).unwrap();
}

fn with_clean_dir<F: FnOnce(&Path) -> T + std::panic::UnwindSafe, T>(name: &str, f: F) -> T {
    let _lock = lock_cwd();
    let dir = std::env::temp_dir()
        .join("nuwa_test")
        .join("config_service")
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

#[test]
fn test_list_job_configs_empty() {
    with_clean_dir("list_empty", |dir| {
        create_empty_config_in(dir);
        let result = config_service::list_job_configs().unwrap();
        assert!(result.is_empty(), "Empty config should return empty list");
    });
}

#[test]
fn test_list_job_configs_with_jobs() {
    with_clean_dir("list_jobs", |dir| {
        create_config_with_jobs_in(dir);
        let result = config_service::list_job_configs().unwrap();
        assert_eq!(result.len(), 2, "Should have 2 job configs");
        assert_eq!(result[0].name, "documents");
        assert_eq!(result[1].name, "photos");
        assert_eq!(result[0].source, "C:/Users/Test/Documents");
        assert!(result[0].compress);
        assert_eq!(result[0].retention_keep_count, Some(7));
        assert_eq!(result[0].retention_keep_days, Some(30));
    });
}

#[test]
fn test_list_job_configs_no_config_file() {
    with_clean_dir("no_config", |_dir| {
        let result = config_service::list_job_configs().unwrap();
        assert!(result.is_empty(), "No config file should return empty list");
    });
}

#[test]
fn test_create_job_config() {
    with_clean_dir("create", |dir| {
        create_empty_config_in(dir);
        let request = JobConfigRequest {
            name: "my-backup".into(),
            source: "C:/Users/Test".into(),
            dest: "D:/Backup".into(),
            compress: true,
            retention_keep_count: Some(10),
            retention_keep_days: None,
            schedule_id: None,
            storage_type: None,
            repository_id: None,
        };
        let view = config_service::create_job_config(&request).unwrap();
        assert_eq!(view.name, "my-backup");
        assert_eq!(view.source, "C:/Users/Test");
        assert!(view.compress);
        assert_eq!(view.retention_keep_count, Some(10));
        let jobs = config_service::list_job_configs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "my-backup");
    });
}

#[test]
fn test_create_duplicate_job_name_fails() {
    with_clean_dir("create_dup", |dir| {
        create_empty_config_in(dir);
        let request = JobConfigRequest {
            name: "my-backup".into(),
            source: "C:/Users/Test".into(),
            dest: "D:/Backup".into(),
            compress: true,
            retention_keep_count: None,
            retention_keep_days: None,
            schedule_id: None,
            storage_type: None,
            repository_id: None,
        };
        config_service::create_job_config(&request).unwrap();
        let err = config_service::create_job_config(&request).unwrap_err();
        assert!(
            err.message.contains("already exists"),
            "Should reject duplicate: {}",
            err.message
        );
    });
}

#[test]
fn test_get_job_config_detail() {
    with_clean_dir("get_detail", |dir| {
        create_config_with_jobs_in(dir);
        let view = config_service::get_job_config("documents").unwrap();
        assert_eq!(view.name, "documents");
        assert_eq!(view.source, "C:/Users/Test/Documents");
        assert_eq!(view.dest, "D:/Backup/Documents");
        let err = config_service::get_job_config("nonexistent").unwrap_err();
        assert!(err.message.contains("not found"));
    });
}

#[test]
fn test_update_job_config() {
    with_clean_dir("update", |dir| {
        create_config_with_jobs_in(dir);
        let request = JobConfigRequest {
            name: "documents".into(),
            source: "C:/Users/Test/Documents/Work".into(),
            dest: "D:/Backup/WorkDocs".into(),
            compress: false,
            retention_keep_count: Some(14),
            retention_keep_days: Some(60),
            schedule_id: None,
            storage_type: None,
            repository_id: None,
        };
        let view = config_service::update_job_config("documents", &request).unwrap();
        assert_eq!(view.source, "C:/Users/Test/Documents/Work");
        assert_eq!(view.dest, "D:/Backup/WorkDocs");
        assert!(!view.compress);
        assert_eq!(view.retention_keep_count, Some(14));
        let jobs = config_service::list_job_configs().unwrap();
        let docs = jobs.iter().find(|j| j.name == "documents").unwrap();
        assert_eq!(docs.source, "C:/Users/Test/Documents/Work");
    });
}

#[test]
fn test_update_missing_job_fails() {
    with_clean_dir("update_missing", |dir| {
        create_empty_config_in(dir);
        let request = JobConfigRequest {
            name: "ghost".into(),
            source: "C:/Source".into(),
            dest: "D:/Dest".into(),
            compress: false,
            retention_keep_count: None,
            retention_keep_days: None,
            schedule_id: None,
            storage_type: None,
            repository_id: None,
        };
        let err = config_service::update_job_config("ghost", &request).unwrap_err();
        assert!(
            err.message.contains("not found"),
            "Should reject update of nonexistent job: {}",
            err.message
        );
    });
}

#[test]
fn test_update_job_config_rejects_rename() {
    with_clean_dir("rename_reject", |dir| {
        create_config_with_jobs_in(dir);
        let request = JobConfigRequest {
            name: "new_name".into(),
            source: "C:/Source".into(),
            dest: "D:/Dest".into(),
            compress: false,
            retention_keep_count: None,
            retention_keep_days: None,
            schedule_id: None,
            storage_type: None,
            repository_id: None,
        };
        let err = config_service::update_job_config("documents", &request).unwrap_err();
        assert!(
            err.message.contains("Cannot rename"),
            "Should reject rename attempt: {}",
            err.message
        );
    });
}

#[test]
fn test_delete_job_config() {
    with_clean_dir("delete", |dir| {
        create_config_with_jobs_in(dir);
        config_service::delete_job_config("photos").unwrap();
        let jobs = config_service::list_job_configs().unwrap();
        assert_eq!(jobs.len(), 1, "Only documents should remain");
        assert_eq!(jobs[0].name, "documents");
    });
}

#[test]
fn test_delete_missing_job_fails() {
    with_clean_dir("delete_missing", |dir| {
        create_empty_config_in(dir);
        let err = config_service::delete_job_config("nonexistent").unwrap_err();
        assert!(
            err.message.contains("not found"),
            "Should reject delete of nonexistent job: {}",
            err.message
        );
    });
}

#[test]
fn test_create_job_empty_name_fails() {
    with_clean_dir("empty_name", |dir| {
        create_empty_config_in(dir);
        let err = config_service::create_job_config(&JobConfigRequest {
            name: "".into(),
            source: "C:/Test".into(),
            dest: "D:/Backup".into(),
            compress: false,
            retention_keep_count: None,
            retention_keep_days: None,
            schedule_id: None,
            storage_type: None,
            repository_id: None,
        })
        .unwrap_err();
        assert!(
            err.message.contains("empty"),
            "Should reject empty name: {}",
            err.message
        );
    });
}

#[test]
fn test_create_job_empty_source_fails() {
    with_clean_dir("empty_source", |dir| {
        create_empty_config_in(dir);
        let err = config_service::create_job_config(&JobConfigRequest {
            name: "test".into(),
            source: "".into(),
            dest: "D:/Backup".into(),
            compress: false,
            retention_keep_count: None,
            retention_keep_days: None,
            schedule_id: None,
            storage_type: None,
            repository_id: None,
        })
        .unwrap_err();
        assert!(
            err.message.contains("empty"),
            "Should reject empty source: {}",
            err.message
        );
    });
}

#[test]
fn test_create_job_empty_dest_fails() {
    with_clean_dir("empty_dest", |dir| {
        create_empty_config_in(dir);
        let err = config_service::create_job_config(&JobConfigRequest {
            name: "test".into(),
            source: "C:/Test".into(),
            dest: "".into(),
            compress: false,
            retention_keep_count: None,
            retention_keep_days: None,
            schedule_id: None,
            storage_type: None,
            repository_id: None,
        })
        .unwrap_err();
        assert!(
            err.message.contains("empty"),
            "Should reject empty dest: {}",
            err.message
        );
    });
}

// ---------------------------------------------------------------------------
// Backward compatibility tests
// ---------------------------------------------------------------------------

#[test]
fn test_old_config_without_schedules_field_loads_gracefully() {
    // Simulate a config from Phase 1 / early Phase 2 that has no [schedules] section.
    // The #[serde(default)] attribute on Config.schedules should default to empty map.
    with_clean_dir("backward_no_schedules", |dir| {
        let toml = r#"
[job.documents]
source = "C:/Users/Test/Documents"
dest = "D:/Backup/Documents"
compress = true
"#;
        fs::write(dir.join("nuwa.toml"), toml).unwrap();
        let jobs = config_service::list_job_configs().unwrap();
        assert_eq!(jobs.len(), 1, "Should parse config without schedules field");
        assert_eq!(jobs[0].name, "documents");
        assert_eq!(
            jobs[0].schedule_id, None,
            "Missing schedule_id should default to None"
        );
    });
}

#[test]
fn test_old_config_without_schedule_id_loads_gracefully() {
    // Simulate a job config from Phase 1 that omitted the schedule_id field entirely.
    with_clean_dir("backward_no_schedule_id", |dir| {
        let toml = r#"
[job.work]
source = "C:/Projects"
dest = "E:/Backup/Projects"
compress = true
retention = { keep_count = 10, keep_days = 30 }
"#;
        fs::write(dir.join("nuwa.toml"), toml).unwrap();
        let jobs = config_service::list_job_configs().unwrap();
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].name, "work");
        assert_eq!(
            jobs[0].schedule_id, None,
            "Missing schedule_id should default to None"
        );
    });
}

#[test]
fn test_save_and_reload_preserves_defaults() {
    // Create a job, save, reload, and verify defaults are preserved correctly.
    with_clean_dir("save_reload_defaults", |_dir| {
        let request = JobConfigRequest {
            name: "defaults-test".into(),
            source: "C:/Test".into(),
            dest: "D:/Backup".into(),
            compress: false,
            retention_keep_count: None,
            retention_keep_days: None,
            schedule_id: None,
            storage_type: None,
            repository_id: None,
        };
        config_service::create_job_config(&request).unwrap();
        let loaded = config_service::get_job_config("defaults-test").unwrap();
        assert_eq!(
            loaded.schedule_id, None,
            "schedule_id should remain None after save-reload"
        );
    });
}

#[test]
fn test_empty_schedules_list_does_not_break_services() {
    // Verify that an empty schedules list (but valid jobs) doesn't break any service.
    with_clean_dir("empty_schedules_ok", |dir| {
        let toml = r#"
[job.test1]
source = "C:/Test1"
dest = "D:/Backup1"
compress = false

[job.test2]
source = "C:/Test2"
dest = "D:/Backup2"
compress = true
"#;
        fs::write(dir.join("nuwa.toml"), toml).unwrap();
        let jobs = config_service::list_job_configs().unwrap();
        assert_eq!(jobs.len(), 2);
        // Both should have schedule_id = None since no schedules section exists
        assert_eq!(jobs[0].schedule_id, None);
        assert_eq!(jobs[1].schedule_id, None);
    });
}
#[test]
fn test_created_config_reflected_by_backup_service() {
    with_clean_dir("backup_reflect", |dir| {
        create_empty_config_in(dir);
        let request = JobConfigRequest {
            name: "integration-test".into(),
            source: "C:/Users/Test".into(),
            dest: "D:/Backup".into(),
            compress: true,
            retention_keep_count: Some(5),
            retention_keep_days: None,
            schedule_id: None,
            storage_type: None,
            repository_id: None,
        };
        config_service::create_job_config(&request).unwrap();
        let jobs = nuwa_backup::app::services::backup_service::list_jobs().unwrap();
        assert!(
            jobs.iter().any(|j| j.name == "integration-test"),
            "BackupService should reflect newly created config job"
        );
    });
}
