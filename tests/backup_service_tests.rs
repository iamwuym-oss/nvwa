// ============================================================================
// backup_service_tests.rs -- Test the backup service layer
//
// Test scenarios:
//   1. list_jobs with valid config
//   2. list_jobs with missing config (error)
//   3. get_job_detail with valid job name
//   4. get_job_detail with invalid job name (error)
//   5. run_backup_dry 鈥?validates job exists
//   6. Misconfigured job (source not found)
// ============================================================================

use std::fs;
use std::path::Path;
use std::sync::Mutex;

use nuwa_backup::app::models::backup::BackupJobStatus;
use nuwa_backup::app::services::backup_service;

// Tests that call with_clean_dir() must be serialized because they
// change the global process CWD (Config::load() uses relative path).
static CWD_LOCK: Mutex<()> = Mutex::new(());

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn create_valid_config_in(dir: &Path) {
    let src = dir.join("source").to_string_lossy().replace("\\", "\\\\");
    let dst = dir.join("dest").to_string_lossy().replace("\\", "\\\\");

    // Create source dir so it exists
    fs::create_dir_all(dir.join("source")).unwrap();

    let toml = format!(
        r#"
[job.TestJob]
source = "{}"
dest = "{}"
compress = true

[job.AnotherJob]
source = "{}"
dest = "{}"
compress = false
"#,
        src, dst, src, dst
    );
    fs::write(dir.join("nuwa.toml"), &toml).unwrap();
}

fn create_misconfigured_config_in(dir: &Path) {
    // Source path does not exist
    let src = dir
        .join("nonexistent_source")
        .to_string_lossy()
        .replace("\\", "\\\\");
    let dst = dir.join("dest").to_string_lossy().replace("\\", "\\\\");

    let toml = format!(
        r#"
[job.BrokenJob]
source = "{}"
dest = "{}"
compress = false
"#,
        src, dst
    );
    fs::write(dir.join("nuwa.toml"), &toml).unwrap();
}

fn with_clean_dir<F: FnOnce(&Path) -> T, T>(name: &str, f: F) -> T {
    let _lock = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let dir = std::env::temp_dir().join("nuwa_test").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

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
fn test_list_jobs_with_valid_config() {
    with_clean_dir("list_jobs_valid", |dir| {
        create_valid_config_in(dir);

        let jobs = backup_service::list_jobs().expect("Should list jobs successfully");
        assert_eq!(jobs.len(), 2, "Config has 2 jobs");

        let test_job = jobs.iter().find(|j| j.name == "TestJob").unwrap();
        assert_eq!(
            test_job.source,
            dir.join("source").to_string_lossy().to_string()
        );
        assert!(test_job.compress, "TestJob should have compression enabled");
        assert_eq!(test_job.status, BackupJobStatus::NeverRun, "No history yet");

        let another = jobs.iter().find(|j| j.name == "AnotherJob").unwrap();
        assert!(!another.compress, "AnotherJob should not have compression");
    });
}

#[test]
fn test_list_jobs_no_config() {
    with_clean_dir("list_jobs_no_config", |_dir| {
        let result = backup_service::list_jobs();
        assert!(result.is_err(), "Should error when no config exists");
    });
}

#[test]
fn test_get_job_detail_valid() {
    with_clean_dir("get_job_detail", |dir| {
        create_valid_config_in(dir);

        let job = backup_service::get_job_detail("TestJob").expect("Should find TestJob");
        assert_eq!(job.name, "TestJob");
        assert!(job.compress);
        assert_eq!(job.status, BackupJobStatus::NeverRun);
    });
}

#[test]
fn test_get_job_detail_not_found() {
    with_clean_dir("get_job_not_found", |dir| {
        create_valid_config_in(dir);

        let result = backup_service::get_job_detail("NonExistentJob");
        assert!(result.is_err(), "Should error for non-existent job");
    });
}

#[test]
fn test_run_backup_dry_success() {
    with_clean_dir("run_dry", |dir| {
        create_valid_config_in(dir);

        let result = backup_service::run_backup_dry("TestJob").expect("Dry run should succeed");
        assert_eq!(result.status, "dry_run");
        assert!(result.error.is_none());
    });
}

#[test]
fn test_run_backup_dry_job_not_found() {
    with_clean_dir("run_dry_not_found", |dir| {
        create_valid_config_in(dir);

        let result = backup_service::run_backup_dry("NonExistent");
        assert!(result.is_err(), "Should error for non-existent job");
    });
}

#[test]
fn test_run_backup_source_missing() {
    with_clean_dir("run_source_missing", |dir| {
        let toml = format!(
            "\n[job.TestJob]\nsource = \"{}\"\ndest = \"{}\"\n",
            dir.join("missing_source")
                .to_string_lossy()
                .replace("\\", "\\\\"),
            dir.join("dest").to_string_lossy().replace("\\", "\\\\")
        );
        fs::write(dir.join("nuwa.toml"), &toml).unwrap();
        let result = backup_service::run_backup("TestJob");
        assert!(
            result.is_err(),
            "run_backup with missing source should error"
        );
    });
}

#[test]
fn test_list_jobs_with_corrupt_config() {
    with_clean_dir("corrupt_config_jobs", |dir| {
        fs::write(dir.join("nuwa.toml"), "[[[ invalid toml [[[").unwrap();
        let result = backup_service::list_jobs();
        if let Ok(jobs) = result {
            assert!(
                jobs.is_empty(),
                "Corrupt config should return empty list or error"
            );
        }
    });
}

#[test]
fn test_misconfigured_job_source_missing() {
    with_clean_dir("misconfigured", |dir| {
        create_misconfigured_config_in(dir);

        let jobs = backup_service::list_jobs().expect("Should list jobs");
        let broken = jobs.iter().find(|j| j.name == "BrokenJob").unwrap();
        assert_eq!(
            broken.status,
            BackupJobStatus::Misconfigured,
            "Job with missing source should be Misconfigured"
        );
    });
}
