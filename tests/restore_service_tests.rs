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

#[cfg(feature = "repository")]
#[allow(unused_imports)]
use rusqlite::params;
#[allow(unused_imports)]
use sha2::{Digest, Sha256};
#[allow(unused_imports)]
use std::collections::HashMap;
#[allow(unused_imports)]
use std::io::Read;
use tempfile::TempDir;

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

#[cfg(feature = "repository")]
#[test]
fn test_repo_restore_end_to_end_via_service() {
    let _lock = lock_cwd();

    // Create temp root and set up environment
    let root = TempDir::new().expect("temp dir for e2e test");
    let old_cwd = std::env::current_dir().unwrap();
    let old_home = std::env::var("USERPROFILE").ok();

    std::env::set_current_dir(root.path()).unwrap();
    std::env::set_var("USERPROFILE", root.path());

    // Ensure cleanup even on panic
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // ---- Phase 1: Create source files ----
        let src_dir = root.path().join("source");
        fs::create_dir_all(&src_dir).expect("create source dir");

        let file1_content: &[u8] = b"Hello from Nuwa Backup end-to-end test!";
        let file2_content: &[u8] = b"Nested file content for SHA-256 verification";

        fs::write(src_dir.join("hello.txt"), file1_content).expect("write hello.txt");
        fs::create_dir_all(src_dir.join("nested").join("deep")).expect("create nested dirs");
        fs::write(
            src_dir.join("nested").join("deep").join("data.bin"),
            file2_content,
        )
        .expect("write data.bin");

        // Compute expected SHA-256
        use sha2::{Digest, Sha256};
        let expected_sha1 = {
            let mut h = Sha256::new();
            h.update(file1_content);
            hex::encode(h.finalize())
        };
        let expected_sha2 = {
            let mut h = Sha256::new();
            h.update(file2_content);
            hex::encode(h.finalize())
        };

        // ---- Phase 2: Initialize Repository ----
        let repo_dir = root.path().join("repo");
        let handle = nuwa_backup::repository::init_repo(
            &repo_dir,
            nuwa_backup::repository::DEFAULT_BLOCK_SIZE,
        )
        .expect("init repo");
        let repo_id = handle.info.repository_id.clone();

        // ---- Phase 3: Register Repository in RepoRegistry ----
        {
            let mut registry = nuwa_backup::app::services::repo_registry::RepoRegistry::load();
            registry
                .register(nuwa_backup::app::models::repo::RepoRecord {
                    id: repo_id.clone(),
                    name: "e2e-test-repo".into(),
                    path: repo_dir.to_string_lossy().into_owned(),
                    created_at: "2026-07-12T00:00:00Z".into(),
                    last_opened: "2026-07-12T00:00:00Z".into(),
                    status: "active".into(),
                })
                .expect("register repo");
        }

        // ---- Phase 4: Create Job Config (nuwa.toml) ----
        {
            let config = nuwa_backup::config::Config {
                job: HashMap::from([(
                    "e2e-test-job".into(),
                    nuwa_backup::config::JobConfig {
                        source: src_dir.clone(),
                        dest: repo_dir.clone(),
                        compress: false,
                        retention: None,
                        schedule_id: None,
                        storage_type: Some("repository".into()),
                        repository_id: Some(repo_id.clone()),
                    },
                )]),
                schedules: HashMap::new(),
            };
            config
                .save_to(&root.path().join("nuwa.toml"))
                .expect("save config");
        }

        // ---- Phase 5: Run Backup via RepositoryBackupWriter ----
        let point_id = {
            // Need to register job in repo.db first
            let conn = handle.repo_db().expect("open repo_db");
            conn.execute(
                "INSERT OR IGNORE INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)
                 VALUES (?1, ?2, 0, '', datetime('now'), 'Active')",
                rusqlite::params!["e2e-test-job", "e2e-test-job"],
            )
            .expect("insert job");

            let mut writer = nuwa_backup::repository::RepositoryBackupWriter::new(
                &handle,
                "e2e-test-job",
                false,
            )
            .expect("create backup writer");
            writer.begin().expect("begin transaction");
            writer
                .write_file(
                    &src_dir.join("hello.txt"),
                    "hello.txt",
                    "2026-07-12T00:00:00Z",
                )
                .expect("write hello.txt");
            writer
                .write_file(
                    &src_dir.join("nested").join("deep").join("data.bin"),
                    "nested/deep/data.bin",
                    "2026-07-12T00:00:00Z",
                )
                .expect("write data.bin");
            let result = writer.finalize().expect("finalize backup");
            result.point_id
        };

        eprintln!("[TEST] Created restore point: {}", point_id);

        // ---- Phase 6: Restore via Application Service Layer ----
        let dest_dir = root.path().join("restored");
        fs::create_dir_all(&dest_dir).expect("create dest_dir");
        let req = RestoreRequest {
            backup_id: point_id.clone(),
            dest: dest_dir.to_string_lossy().into_owned(),
            overwrite: false,
        };
        let outcome = restore_service::execute_restore(req).expect("restore must succeed");
        eprintln!(
            "[TEST] Restore result: restored_count={}, checksum_failures={}, status={}",
            outcome.restored_count, outcome.checksum_failures, outcome.status
        );

        // ---- Phase 7: Verify Results ----
        assert_eq!(outcome.restored_count, 2, "Must restore exactly 2 files");
        assert_eq!(
            outcome.checksum_failures, 0,
            "Zero checksum failures expected"
        );
        assert_eq!(outcome.status, "success", "Status must be 'success'");

        // Verify file existence
        assert!(dest_dir.join("hello.txt").exists(), "hello.txt must exist");
        assert!(
            dest_dir
                .join("nested")
                .join("deep")
                .join("data.bin")
                .exists(),
            "nested/deep/data.bin must exist"
        );

        // Verify file contents via SHA-256
        {
            let mut f = fs::File::open(dest_dir.join("hello.txt")).expect("open hello.txt");
            let mut actual = Vec::new();
            f.read_to_end(&mut actual).expect("read hello.txt");
            let mut h = Sha256::new();
            h.update(&actual);
            let actual_sha = hex::encode(h.finalize());
            assert_eq!(actual_sha, expected_sha1, "hello.txt SHA-256 must match");

            let mut f = fs::File::open(dest_dir.join("nested").join("deep").join("data.bin"))
                .expect("open data.bin");
            let mut actual = Vec::new();
            f.read_to_end(&mut actual).expect("read data.bin");
            let mut h = Sha256::new();
            h.update(&actual);
            let actual_sha = hex::encode(h.finalize());
            assert_eq!(actual_sha, expected_sha2, "data.bin SHA-256 must match");
        }

        eprintln!("[TEST] All assertions passed!");
    }));

    // ---- Phase 8: Cleanup ----
    if let Some(old) = old_home {
        std::env::set_var("USERPROFILE", old);
    } else {
        std::env::remove_var("USERPROFILE");
    }
    std::env::set_current_dir(&old_cwd).unwrap();
    let _ = fs::remove_dir_all(root.path());
    drop(_lock);

    match result {
        Ok(()) => (),
        Err(e) => std::panic::resume_unwind(e),
    }
}

#[cfg(feature = "repository")]
#[test]
fn test_backup_service_repository_e2e() {
    let _lock = lock_cwd();

    let root = TempDir::new().expect("temp dir for backup service e2e test");
    let old_cwd = std::env::current_dir().unwrap();
    let old_home = std::env::var("USERPROFILE").ok();

    std::env::set_current_dir(root.path().join("cwd")).unwrap_or_else(|_| {
        std::fs::create_dir_all(root.path().join("cwd")).unwrap();
        std::env::set_current_dir(root.path().join("cwd")).unwrap();
    });

    let home_dir = root.path().join("home");
    std::env::set_var("USERPROFILE", &home_dir);

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        // ---- Phase 1: Create source files ----
        let src_dir = root.path().join("source");
        fs::create_dir_all(&src_dir).expect("create src_dir");
        fs::write(src_dir.join("hello.txt"), b"Hello, Repository Backup!")
            .expect("write hello.txt");

        fs::create_dir_all(src_dir.join("nested").join("deep")).expect("create nested");
        let bin_data: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();
        fs::write(
            src_dir.join("nested").join("deep").join("data.bin"),
            &bin_data,
        )
        .expect("write data.bin");

        let mut h1 = Sha256::new();
        h1.update(b"Hello, Repository Backup!");
        let expected_sha1 = hex::encode(h1.finalize());

        let mut h2 = Sha256::new();
        h2.update(&bin_data);
        let expected_sha2 = hex::encode(h2.finalize());

        // ---- Phase 2: Initialize repository ----
        let repo_dir = root.path().join("repo");
        let handle = nuwa_backup::repository::init_repo(&repo_dir, 262144).expect("init repo");

        let repo_id = handle.info.repository_id.clone();

        // ---- Phase 3: Register repo in the registry ----
        {
            let mut registry = nuwa_backup::app::services::repo_registry::RepoRegistry::load();
            registry
                .register(nuwa_backup::app::models::repo::RepoRecord {
                    id: repo_id.clone(),
                    name: "E2E Test Repo".into(),
                    path: repo_dir.to_string_lossy().into_owned(),
                    created_at: "2026-07-12T00:00:00Z".into(),
                    last_opened: "2026-07-12T00:00:00Z".into(),
                    status: "active".into(),
                })
                .expect("register repo");
        }

        // ---- Phase 4: Create config with repository job ----
        {
            use std::collections::HashMap;
            let config = nuwa_backup::config::Config {
                job: HashMap::from([(
                    "backup-e2e".into(),
                    nuwa_backup::config::JobConfig {
                        source: src_dir.clone(),
                        dest: repo_dir.clone(),
                        compress: false,
                        retention: None,
                        schedule_id: None,
                        storage_type: Some("repository".into()),
                        repository_id: Some(repo_id.clone()),
                    },
                )]),
                schedules: HashMap::new(),
            };
            config.save().expect("save config");
        }

        // ---- Phase 5: Run backup via backup_service ----
        let backup_result = nuwa_backup::app::services::backup_service::run_backup("backup-e2e")
            .expect("backup must succeed");

        eprintln!(
            "[TEST] Backup result: point_id={}, file_count={}, total_bytes={}, duration_ms={}",
            backup_result.backup_id,
            backup_result.file_count,
            backup_result.total_bytes,
            backup_result.duration_ms
        );

        assert_eq!(backup_result.file_count, 2, "Must back up exactly 2 files");
        assert!(backup_result.total_bytes > 0, "Total bytes must be > 0");
        assert_eq!(backup_result.status, "success", "Status must be 'success'");

        // ---- Phase 6: Restore via restore_service ----
        let dest_dir = root.path().join("restored");
        fs::create_dir_all(&dest_dir).expect("create dest_dir");

        let req = nuwa_backup::app::models::restore::RestoreRequest {
            backup_id: backup_result.backup_id.clone(),
            dest: dest_dir.to_string_lossy().into_owned(),
            overwrite: false,
        };
        let outcome = nuwa_backup::app::services::restore_service::execute_restore(req)
            .expect("restore must succeed");

        eprintln!(
            "[TEST] Restore result: restored_count={}, checksum_failures={}, status={}",
            outcome.restored_count, outcome.checksum_failures, outcome.status
        );

        // ---- Phase 7: Verify Results ----
        assert_eq!(outcome.restored_count, 2, "Must restore exactly 2 files");
        assert_eq!(
            outcome.checksum_failures, 0,
            "Zero checksum failures expected"
        );
        assert_eq!(outcome.status, "success", "Status must be 'success'");

        assert!(dest_dir.join("hello.txt").exists(), "hello.txt must exist");
        assert!(
            dest_dir
                .join("nested")
                .join("deep")
                .join("data.bin")
                .exists(),
            "nested/deep/data.bin must exist"
        );

        // Verify file contents via SHA-256
        {
            let mut f = fs::File::open(dest_dir.join("hello.txt")).expect("open hello.txt");
            let mut actual = Vec::new();
            f.read_to_end(&mut actual).expect("read hello.txt");
            let mut h = Sha256::new();
            h.update(&actual);
            let actual_sha = hex::encode(h.finalize());
            assert_eq!(actual_sha, expected_sha1, "hello.txt SHA-256 must match");

            let mut f = fs::File::open(dest_dir.join("nested").join("deep").join("data.bin"))
                .expect("open data.bin");
            let mut actual = Vec::new();
            f.read_to_end(&mut actual).expect("read data.bin");
            let mut h = Sha256::new();
            h.update(&actual);
            let actual_sha = hex::encode(h.finalize());
            assert_eq!(actual_sha, expected_sha2, "data.bin SHA-256 must match");
        }

        eprintln!("[TEST] Backup service e2e: All assertions passed!");
    }));

    // ---- Cleanup ----
    if let Some(old) = old_home {
        std::env::set_var("USERPROFILE", old);
    } else {
        std::env::remove_var("USERPROFILE");
    }
    std::env::set_current_dir(&old_cwd).unwrap();
    let _ = fs::remove_dir_all(root.path());
    drop(_lock);

    match result {
        Ok(()) => (),
        Err(e) => std::panic::resume_unwind(e),
    }
}
