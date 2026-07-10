// ============================================================================

// repository_integration_tests.rs 閳?Phase S Repository Engine Integration Tests

// ============================================================================

//

// Tests the complete Repository Engine data flow across all components.

// These tests verify that BlockStore, BlockMap, Catalog, Transaction,

// Verify, Retention, and Recovery work correctly together.

//

// Run with: cargo test --features repository --test repository_integration_tests -- --test-threads=1

use std::fs;

use tempfile::TempDir;

// Trait imports for calling trait methods

use nuwa_backup::repository::BlockMapEngine;

use nuwa_backup::repository::BlockStore;

use nuwa_backup::repository::CatalogEngine;

// ============================================================================

// Helper: initialize a repo and return handle + tmpdir

// ============================================================================

fn init_repo(tmp: &TempDir) -> nuwa_backup::repository::RepoHandle {
    nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
        .expect("init_repo should succeed")
}

// ============================================================================

// Test 1: Repository Lifecycle

// ============================================================================

#[test]

fn test_repository_lifecycle() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    let repo_path = tmp.path();

    // Step 1: Init

    let handle = init_repo(&tmp);

    assert!(handle.nuwarepo_dir.exists(), ".nuwarepo dir should exist");

    assert!(
        handle.block_store_dir.exists(),
        "block-store dir should exist"
    );

    assert!(
        handle.instances_dir.exists(),
        "backup-instances dir should exist"
    );

    assert!(handle.repo_db_path.exists(), "repo.db should exist");

    assert_eq!(handle.info.version, 1, "repo version should be 1");

    assert_eq!(handle.info.format_version, 1, "format version should be 1");

    assert_eq!(
        handle.info.min_compatible_version, 1,
        "min compatible version should be 1"
    );

    assert!(
        !handle.info.repository_id.is_empty(),
        "repository_id should be set"
    );

    assert!(
        handle.info.capabilities.compression,
        "compression should be enabled"
    );

    // Step 2: is_repository

    assert!(
        nuwa_backup::repository::is_repository(repo_path),
        "is_repository should return true"
    );

    // Step 3: Open

    let handle2 = nuwa_backup::repository::open_repo(repo_path).expect("open_repo should succeed");

    assert_eq!(handle2.info.version, 1);

    assert_eq!(
        handle2.info.repository_id, handle.info.repository_id,
        "repository_id should persist across open"
    );

    // Step 4: repo_db() returns a connection with PRAGMAs configured

    let conn = handle2.repo_db().expect("repo_db() should succeed");

    conn.execute_batch("SELECT 1")
        .expect("connection should be usable");

    drop(conn);

    // Step 5: check_repo

    nuwa_backup::repository::check_repo(&handle2).expect("check_repo should pass on valid repo");

    // Step 6: Verify empty repo

    let store = nuwa_backup::repository::block_store::store::LocalFsBlockStore::new(
        handle2.block_store_dir.clone(),
    );

    let report = nuwa_backup::repository::verify_repo(
        &handle2,
        &store,
        nuwa_backup::repository::VerifyLevel::Metadata,
    )
    .expect("verify_repo should succeed");

    assert_eq!(
        report.total_restore_points, 0,
        "empty repo should have 0 restore points"
    );

    assert!(
        !report.data_loss_detected,
        "empty repo should have no data loss"
    );

    // Step 7: repository.json exists

    let repo_json_path = handle2.nuwarepo_dir.join("repository.json");

    assert!(repo_json_path.exists(), "repository.json should exist");

    let content = fs::read_to_string(&repo_json_path).expect("should read repository.json");

    assert!(
        content.contains(&handle2.info.repository_id),
        "repository.json should contain the repo ID"
    );

    // Step 8: Init on existing repo should fail

    let result =
        nuwa_backup::repository::init_repo(repo_path, nuwa_backup::repository::DEFAULT_BLOCK_SIZE);

    assert!(result.is_err(), "init on existing repo should fail");
}

// ============================================================================

// Test 2: Backup Instance Flow

// ============================================================================

#[test]

fn test_backup_instance_flow() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    let _repo_path = tmp.path();

    let handle = init_repo(&tmp);

    let point_id = "test-point-001";

    let instance_dir = handle.instances_dir.join(point_id);

    fs::create_dir_all(&instance_dir).expect("should create instance dir");

    // Step 1: Write blocks to BlockStore

    use nuwa_backup::repository::block_store::block_header::{BlockHeader, Compression};

    use nuwa_backup::repository::block_store::store::LocalFsBlockStore;

    let store = LocalFsBlockStore::new(handle.block_store_dir.clone());

    let test_data = b"Hello, Nuwa Backup Integration Test!";

    let test_data2 = b"Second file content for multi-block verification";

    let block_id_1 = {
        let header = BlockHeader::new(
            Compression::None,
            test_data.len() as u64,
            test_data.len() as u64,
        );

        let block = nuwa_backup::repository::Block {
            header,

            data: test_data.to_vec(),
        };

        store.put_block(&block).expect("put_block should succeed")
    };

    let block_id_2 = {
        let header = BlockHeader::new(
            Compression::None,
            test_data2.len() as u64,
            test_data2.len() as u64,
        );

        let block = nuwa_backup::repository::Block {
            header,

            data: test_data2.to_vec(),
        };

        store.put_block(&block).expect("put_block should succeed")
    };

    assert_ne!(
        block_id_1, block_id_2,
        "different data should produce different block IDs"
    );

    assert!(store.exists(&block_id_1).expect("block exists check"));

    assert!(store.exists(&block_id_2).expect("block exists check"));

    // Step 2: Create BlockMap

    let bm_path = instance_dir.join("block-map.db");

    let mut block_map = nuwa_backup::repository::SqliteBlockMap::open(bm_path.clone())
        .expect("should create block-map.db");

    block_map
        .insert_mapping(0, &block_id_1, test_data.len() as u64)
        .expect("insert mapping should succeed");

    block_map
        .insert_mapping(100, &block_id_2, test_data2.len() as u64)
        .expect("insert mapping should succeed");

    assert_eq!(
        block_map.block_count().expect("block count"),
        2,
        "should have 2 mappings"
    );

    let _bm_path = Box::new(block_map).close().expect("close block map");

    // Step 3: Create Catalog

    let cat_path = instance_dir.join("catalog.db");

    let mut catalog = nuwa_backup::repository::SqliteCatalog::open(cat_path.clone())
        .expect("should create catalog.db");

    catalog
        .add_file(
            "test/file1.txt",
            test_data.len() as u64,
            "2026-07-10T00:00:00Z",
            vec![nuwa_backup::repository::FileExtent {
                logical_offset: 0,

                length: test_data.len() as u64,
            }],
        )
        .expect("add_file should succeed");

    catalog
        .add_file(
            "test/file2.txt",
            test_data2.len() as u64,
            "2026-07-10T00:00:00Z",
            vec![nuwa_backup::repository::FileExtent {
                logical_offset: 100,

                length: test_data2.len() as u64,
            }],
        )
        .expect("add_file should succeed");

    assert_eq!(
        catalog.file_count().expect("file count"),
        2,
        "should have 2 files"
    );

    let _cat_path = Box::new(catalog).close().expect("close catalog");

    // Step 4: Write backup-metadata.json

    let meta = serde_json::json!({

        "schema_version": "1.0",

        "restore_point_id": point_id,

        "job_id": "job-001",

        "job_name": "test-job",

        "source_type": "File",

        "source_description": "test data",

        "asset_id": "",

        "asset_type": "",

        "created_at": "2026-07-10T00:00:00Z",

        "status": "COMMITTED",

        "block_chunk_policy": {

            "policy_type": "fixed",

            "block_size": nuwa_backup::repository::DEFAULT_BLOCK_SIZE,

        },

        "block_map": {

            "database": "block-map.db",

            "block_count": 2,

            "sha256": "placeholder",

            "first_offset": 0,

            "last_offset": 200,

        },

        "summary": {

            "total_raw_bytes": (test_data.len() + test_data2.len()) as u64,

            "file_count": 2,

        },

    });

    let meta_content = serde_json::to_string_pretty(&meta).expect("serialize metadata");

    fs::write(instance_dir.join("backup-metadata.json"), &meta_content)
        .expect("write backup-metadata.json");

    // Step 5: Insert into repo.db

    let conn = handle.repo_db().expect("repo_db should succeed");

    conn.execute(
        "INSERT INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)

         VALUES ('job-001', 'test-job', 0, '/test/path', '2026-07-10T00:00:00Z', 'active')",
        [],
    )
    .expect("insert job should succeed");

    let instance_path_str = instance_dir.to_string_lossy().to_string();

    conn.execute(

        "INSERT INTO restore_points (point_id, job_id, chain_id, chain_position, created_at, status, instance_path, block_count, total_raw_bytes)

         VALUES (?1, 'job-001', 'chain-001', 0, '2026-07-10T00:00:00Z', 'COMMITTED', ?2, 2, ?3)",

        rusqlite::params![point_id, instance_path_str, (test_data.len() + test_data2.len()) as u64],

    ).expect("insert restore point should succeed");

    drop(conn);

    // Step 6: Verify with verify_repo

    let report = nuwa_backup::repository::verify_repo(
        &handle,
        &store,
        nuwa_backup::repository::VerifyLevel::Metadata,
    )
    .expect("verify_repo should succeed");

    assert_eq!(
        report.total_restore_points, 1,
        "should find 1 restore point"
    );

    assert!(!report.data_loss_detected, "should have no data loss");

    // Step 7: Full verification

    let full_report = nuwa_backup::repository::verify_repo(
        &handle,
        &store,
        nuwa_backup::repository::VerifyLevel::Full,
    )
    .expect("full verify should succeed");

    assert_eq!(full_report.total_blocks, 2, "should find 2 blocks");

    assert_eq!(full_report.verified_blocks, 2, "should verify 2 blocks");

    assert_eq!(full_report.failed_blocks, 0, "0 blocks should fail");
}

// ============================================================================

// Test 3: Retention Flow

// ============================================================================

#[test]

fn test_retention_flow() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    let _repo_path = tmp.path();

    let handle = init_repo(&tmp);

    let conn = handle.repo_db().expect("repo_db should succeed");

    // Insert backup job

    conn.execute(
        "INSERT INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)

         VALUES ('job-ret', 'retention-job', 0, '/test', '2026-01-01T00:00:00Z', 'active')",
        [],
    )
    .expect("insert job should succeed");

    // Create RP1 (old, should be deleted)

    let rp1_dir = handle.instances_dir.join("rp-old");

    fs::create_dir_all(&rp1_dir).expect("create rp1 dir");

    conn.execute(

        "INSERT INTO restore_points (point_id, job_id, chain_id, chain_position, created_at, status, instance_path, block_count, total_raw_bytes)

         VALUES ('rp-old', 'job-ret', 'chain-ret', 0, '2025-01-01T00:00:00Z', 'COMMITTED', ?1, 100, 10000)",

        rusqlite::params![rp1_dir.to_string_lossy().to_string()],

    ).expect("insert rp1 should succeed");

    // Create RP2 (recent, should be kept)

    let rp2_dir = handle.instances_dir.join("rp-recent");

    fs::create_dir_all(&rp2_dir).expect("create rp2 dir");

    conn.execute(

        "INSERT INTO restore_points (point_id, job_id, chain_id, chain_position, created_at, status, instance_path, block_count, total_raw_bytes)

         VALUES ('rp-recent', 'job-ret', 'chain-ret', 0, '2026-07-10T00:00:00Z', 'COMMITTED', ?1, 50, 5000)",

        rusqlite::params![rp2_dir.to_string_lossy().to_string()],

    ).expect("insert rp2 should succeed");

    drop(conn);

    // Verify initial state

    let conn = handle.repo_db().expect("repo_db should succeed");

    let initial_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM restore_points WHERE status = 'COMMITTED'",
            [],
            |row| row.get(0),
        )
        .expect("count committed");

    assert_eq!(initial_count, 2, "should start with 2 committed points");

    drop(conn);

    // Apply retention: keep 1 day, protect 0 full backups

    let policy = nuwa_backup::repository::RetentionPolicy::new(1, 0);

    let result = nuwa_backup::repository::apply_retention(&handle, &policy)
        .expect("apply_retention should succeed");

    assert_eq!(result.deleted_points.len(), 1, "1 point should be deleted");

    assert_eq!(
        result.deleted_points[0], "rp-old",
        "rp-old should be deleted"
    );

    // rp1 directory should be gone, rp2 should remain

    assert!(
        !rp1_dir.exists(),
        "rp1 instance directory should be deleted"
    );

    assert!(rp2_dir.exists(), "rp2 instance directory should remain");

    // Verify state in DB

    let conn = handle.repo_db().expect("repo_db should succeed");

    let remaining: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM restore_points WHERE status = 'COMMITTED'",
            [],
            |row| row.get(0),
        )
        .expect("count committed after retention");

    assert_eq!(remaining, 1, "1 committed point should remain");

    let deleted_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM restore_points WHERE status = 'DELETED'",
            [],
            |row| row.get(0),
        )
        .expect("count deleted");

    assert_eq!(deleted_count, 1, "1 point should be DELETED");

    drop(conn);

    // Verify orphan candidates

    let orphan_count =
        nuwa_backup::repository::count_orphan_candidates(&handle).expect("count orphan candidates");

    assert_eq!(orphan_count, 100, "100 orphan blocks from rp-old");

    // Verify re-running retention doesn't delete more

    let result2 = nuwa_backup::repository::apply_retention(&handle, &policy)
        .expect("second retention should succeed");

    assert!(
        result2.deleted_points.is_empty(),
        "second run should delete nothing"
    );
}

// ============================================================================

// Test 4: Recovery Flow

// ============================================================================

#[test]

fn test_recovery_flow() {
    let tmp = TempDir::new().expect("Failed to create temp dir");

    let repo_path = tmp.path();

    let handle = init_repo(&tmp);

    let point_id = "recovery-test-point";

    let instance_dir = handle.instances_dir.join(point_id);

    fs::create_dir_all(&instance_dir).expect("create instance dir");

    // Write backup-metadata.json

    let meta = serde_json::json!({

        "schema_version": "1.0",

        "restore_point_id": point_id,

        "job_id": "job-recovery",

        "job_name": "recovery-job",

        "source_type": "File",

        "source_description": "recovery test",

        "asset_id": "",

        "asset_type": "",

        "created_at": "2026-07-10T00:00:00Z",

        "status": "COMMITTED",

        "block_chunk_policy": {

            "policy_type": "fixed",

            "block_size": nuwa_backup::repository::DEFAULT_BLOCK_SIZE,

        },

        "block_map": {

            "database": "block-map.db",

            "block_count": 5,

            "sha256": "abc123",

            "first_offset": 0,

            "last_offset": 5000,

        },

        "summary": {

            "total_raw_bytes": 5000,

            "file_count": 3,

        },

    });

    let meta_content = serde_json::to_string_pretty(&meta).expect("serialize");

    fs::write(instance_dir.join("backup-metadata.json"), &meta_content).expect("write metadata");

    // Insert into repo.db

    let conn = handle.repo_db().expect("repo_db");

    conn.execute(
        "INSERT INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)

         VALUES ('job-recovery', 'recovery-job', 0, '/test', '2026-07-10T00:00:00Z', 'active')",
        [],
    )
    .expect("insert job");

    let istr = instance_dir.to_string_lossy().to_string();

    conn.execute(

        "INSERT INTO restore_points (point_id, job_id, chain_id, chain_position, created_at, status, instance_path, block_count, total_raw_bytes)

         VALUES ('recovery-test-point', 'job-recovery', 'chain-rec', 0, '2026-07-10T00:00:00Z', 'COMMITTED', ?1, 5, 5000)",

        rusqlite::params![istr],

    ).expect("insert point");

    drop(conn);

    // Verify healthy state before damage

    let report = nuwa_backup::repository::check_integrity(&handle);

    assert!(report.is_healthy(), "repo should be healthy before damage");

    assert_eq!(report.summary.errors, 0, "0 errors before damage");

    // Damage: delete restore_points and backup_jobs from repo.db

    let conn = handle.repo_db().expect("repo_db");

    conn.execute_batch("DELETE FROM restore_points; DELETE FROM backup_jobs;")
        .expect("clear restore_points");

    drop(conn);

    // Verify damaged state (but don't assert 閳?check_integrity doesn't fail, just reports)

    let _report_damaged = nuwa_backup::repository::check_integrity(&handle);

    // Rebuild

    nuwa_backup::repository::rebuild_repo(repo_path).expect("rebuild_repo should succeed");

    // Verify rebuilt state

    let handle_after = nuwa_backup::repository::open_repo(repo_path).expect("open rebuilt repo");

    let report_after = nuwa_backup::repository::check_integrity(&handle_after);

    assert!(
        report_after.is_healthy(),
        "repo should be healthy after rebuild"
    );

    // Verify restore point was recovered

    let conn = handle_after.repo_db().expect("repo_db");

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM restore_points", [], |row| row.get(0))
        .expect("count restore points");

    assert_eq!(count, 1, "1 restore point should be recovered");

    let status: String = conn
        .query_row(
            "SELECT status FROM restore_points WHERE point_id = 'recovery-test-point'",
            [],
            |row| row.get(0),
        )
        .expect("get status");

    assert_eq!(status, "COMMITTED", "recovered point should be COMMITTED");
}

// ============================================================================

// A-07: Retention Integrity Tests

// ============================================================================

use nuwa_backup::repository::block_store::block_header::{BlockHeader, Compression};

use nuwa_backup::repository::block_store::store::LocalFsBlockStore;

/// Helper: insert a restore point directly into repo.db
fn insert_restore_point(
    handle: &nuwa_backup::repository::RepoHandle,

    point_id: &str,

    job_id: &str,

    created_at: &str,

    block_count: u64,

    total_bytes: u64,

    status: &str,
) {
    let dir = handle.instances_dir.join(point_id);

    std::fs::create_dir_all(&dir).expect("create instance dir");

    let conn = handle.repo_db().expect("repo_db");

    conn.execute(

        "INSERT INTO restore_points (point_id, job_id, chain_id, chain_position, created_at, status, instance_path, block_count, total_raw_bytes)

         VALUES (?1, ?2, 'chain-a', 0, ?3, ?4, ?5, ?6, ?7)",

        rusqlite::params![point_id, job_id, created_at, status, dir.to_string_lossy().to_string(), block_count, total_bytes],

    ).expect("insert restore point");
}

/// A-07-01: Retention does NOT modify block-store

#[test]

fn test_retention_does_not_touch_block_store() {
    let tmp = TempDir::new().expect("temp dir");

    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let store = LocalFsBlockStore::new(handle.block_store_dir.clone());

    // Write blocks to block-store

    let block_data = b"retention should not delete this block";

    let header = BlockHeader::new(
        Compression::None,
        block_data.len() as u64,
        block_data.len() as u64,
    );

    let block = nuwa_backup::repository::Block {
        header,

        data: block_data.to_vec(),
    };

    let block_id = store.put_block(&block).expect("put block");

    // Verify block exists before retention

    assert!(store
        .exists(&block_id)
        .expect("block should exist before retention"));

    // Configure a restore point referencing this block

    let conn = handle.repo_db().expect("repo_db");

    conn.execute(
        "INSERT INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)

         VALUES ('job-block-test', 'block-test', 0, '/test', '2025-01-01T00:00:00Z', 'active')",
        [],
    )
    .expect("insert job");

    drop(conn);

    insert_restore_point(
        &handle,
        "rp-block-test",
        "job-block-test",
        "2025-01-01T00:00:00Z",
        1,
        100,
        "COMMITTED",
    );

    // Run retention aggressively (keep 0 days, keep 0 full)

    let policy = nuwa_backup::repository::RetentionPolicy::new(1, 0);

    let result = nuwa_backup::repository::apply_retention(&handle, &policy)
        .expect("retention should succeed");

    assert_eq!(
        result.deleted_points.len(),
        1,
        "should delete the old point"
    );

    // CRITICAL: block must still exist in block-store

    assert!(
        store
            .exists(&block_id)
            .expect("block should still exist after retention"),
        "Retention must NOT delete blocks from block-store"
    );
}

/// A-07-02: Non-COMMITTED restore points are never touched by retention

#[test]

fn test_retention_never_touches_non_committed() {
    let tmp = TempDir::new().expect("temp dir");

    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let conn = handle.repo_db().expect("repo_db");

    conn.execute(
        "INSERT INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)

         VALUES ('job-nc', 'non-committed', 0, '/test', '2026-01-01T00:00:00Z', 'active')",
        [],
    )
    .expect("insert job");

    drop(conn);

    // Create restore points in various non-COMMITTED states

    insert_restore_point(
        &handle,
        "rp-creating",
        "job-nc",
        "2025-01-01T00:00:00Z",
        0,
        0,
        "CREATING",
    );

    insert_restore_point(
        &handle,
        "rp-writing",
        "job-nc",
        "2025-01-01T00:00:00Z",
        0,
        0,
        "WRITING",
    );

    insert_restore_point(
        &handle,
        "rp-verifying",
        "job-nc",
        "2025-01-01T00:00:00Z",
        0,
        0,
        "VERIFYING",
    );

    insert_restore_point(
        &handle,
        "rp-failed",
        "job-nc",
        "2025-01-01T00:00:00Z",
        0,
        0,
        "FAILED",
    );

    insert_restore_point(
        &handle,
        "rp-committed",
        "job-nc",
        "2025-01-01T00:00:00Z",
        0,
        0,
        "COMMITTED",
    );

    // Aggressive retention: keep 0 days

    let policy = nuwa_backup::repository::RetentionPolicy::new(1, 0);

    let result = nuwa_backup::repository::apply_retention(&handle, &policy)
        .expect("retention should succeed");

    // Only COMMITTED points should be touched

    assert_eq!(
        result.deleted_points.len(),
        1,
        "only committed points should be deleted"
    );

    assert_eq!(
        result.deleted_points[0], "rp-committed",
        "only rp-committed should be deleted"
    );

    // Verify non-COMMITTED points remain

    let conn = handle.repo_db().expect("repo_db");

    for status in &["CREATING", "WRITING", "VERIFYING", "FAILED"] {
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM restore_points WHERE status = ?1",
                [status],
                |row| row.get(0),
            )
            .expect("count by status");

        assert_eq!(count, 1, "{} point should remain untouched", status);
    }
}

/// A-07-03: Orphan candidates are correctly calculated across multiple RPs

#[test]

fn test_orphan_candidate_calculation() {
    let tmp = TempDir::new().expect("temp dir");

    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let conn = handle.repo_db().expect("repo_db");

    conn.execute(
        "INSERT INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)

         VALUES ('job-orphan', 'orphan-test', 0, '/test', '2026-01-01T00:00:00Z', 'active')",
        [],
    )
    .expect("insert job");

    drop(conn);

    // Three RPs with different block counts

    insert_restore_point(
        &handle,
        "rp-a",
        "job-orphan",
        "2025-01-01T00:00:00Z",
        100,
        10000,
        "COMMITTED",
    );

    insert_restore_point(
        &handle,
        "rp-b",
        "job-orphan",
        "2025-02-01T00:00:00Z",
        200,
        20000,
        "COMMITTED",
    );

    insert_restore_point(
        &handle,
        "rp-c",
        "job-orphan",
        "2025-03-01T00:00:00Z",
        300,
        30000,
        "COMMITTED",
    );

    // Also add a FAILED point (should not count toward orphans)

    insert_restore_point(
        &handle,
        "rp-failed",
        "job-orphan",
        "2025-01-01T00:00:00Z",
        500,
        50000,
        "FAILED",
    );

    // Apply retention: keep points < 500 days, no full backup protection

    let policy = nuwa_backup::repository::RetentionPolicy::new(500, 0);

    let result = nuwa_backup::repository::apply_retention(&handle, &policy)
        .expect("retention should succeed");

    // protect 1 full backup 鈥?only 2 should be deleted

    assert_eq!(
        result.deleted_points.len(),
        2,
        "2 of 3 committed points should be deleted"
    );

    // Orphan count: 100(rp-a) + 200(rp-b) = 300, FAILED(500) never counted, rp-c(300) still COMMITTED

    assert_eq!(
        result.orphan_candidate_count, 300,
        "orphan count should sum deleted blocks"
    );

    // Verify FAILED points are NOT included in orphans

    let summary = nuwa_backup::repository::list_orphan_candidates(&handle).expect("list orphans");

    assert_eq!(summary.deleted_points.len(), 2, "only 2 deleted points");

    assert_eq!(summary.total_orphan_blocks, 300, "300 total orphan blocks");
}

/// A-07-04: Orphan candidates are never deleted from block-store

#[test]

fn test_orphan_candidates_never_delete_blocks() {
    let tmp = TempDir::new().expect("temp dir");

    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let store = LocalFsBlockStore::new(handle.block_store_dir.clone());

    // Write a real block

    let raw = b"orphan block data that must never be deleted";

    let header = BlockHeader::new(Compression::None, raw.len() as u64, raw.len() as u64);

    let block = nuwa_backup::repository::Block {
        header,

        data: raw.to_vec(),
    };

    let bid = store.put_block(&block).expect("put block");

    // Configure a restore point using this block

    let conn = handle.repo_db().expect("repo_db");

    conn.execute(
        "INSERT INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)

         VALUES ('job-orphan-block', 'orphan-block', 0, '/test', '2026-01-01T00:00:00Z', 'active')",
        [],
    )
    .expect("insert job");

    drop(conn);

    insert_restore_point(
        &handle,
        "rp-orphan-block",
        "job-orphan-block",
        "2025-01-01T00:00:00Z",
        1,
        100,
        "COMMITTED",
    );

    // Delete via retention

    let policy = nuwa_backup::repository::RetentionPolicy::new(1, 0);

    nuwa_backup::repository::apply_retention(&handle, &policy).expect("retention");

    // Verify orphan candidate count

    let count = nuwa_backup::repository::count_orphan_candidates(&handle).expect("count orphans");

    assert_eq!(count, 1, "1 orphan candidate block");

    // CRITICAL: block must still exist in block-store

    assert!(
        store.exists(&bid).expect("block exists check"),
        "Orphan candidate blocks must NEVER be deleted from block-store"
    );
}

/// A-07-05: DELETING state recovery after crash

#[test]

fn test_deleting_state_recovery() {
    let tmp = TempDir::new().expect("temp dir");

    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let conn = handle.repo_db().expect("repo_db");

    conn.execute(
        "INSERT INTO backup_jobs (job_id, job_name, source_type, source_path, created_at, status)

         VALUES ('job-del-rec', 'del-recovery', 0, '/test', '2026-01-01T00:00:00Z', 'active')",
        [],
    )
    .expect("insert job");

    drop(conn);

    // Create a restore point

    insert_restore_point(
        &handle,
        "rp-del-crash",
        "job-del-rec",
        "2025-01-01T00:00:00Z",
        50,
        5000,
        "COMMITTED",
    );

    let dir_path = handle.instances_dir.join("rp-del-crash");

    assert!(dir_path.exists(), "instance dir should exist before crash");

    // Simulate crash after Phase 1: manually set status to DELETING

    let conn = handle.repo_db().expect("repo_db");

    conn.execute(
        "UPDATE restore_points SET status = 'DELETING' WHERE point_id = 'rp-del-crash'",
        [],
    )
    .expect("set DELETING");

    drop(conn);

    // Directory should still exist (Phase 2 wasn't executed)

    assert!(dir_path.exists(), "dir should exist after Phase 1 crash");

    // Run retention (which includes recover_incomplete_deletions)

    let policy = nuwa_backup::repository::RetentionPolicy::new(30, 0);

    let _result = nuwa_backup::repository::apply_retention(&handle, &policy)
        .expect("retention should recover and complete deletion");

    // Directory should now be deleted

    assert!(!dir_path.exists(), "dir should be deleted after recovery");

    // Status should be DELETED

    let conn = handle.repo_db().expect("repo_db");

    let status: String = conn
        .query_row(
            "SELECT status FROM restore_points WHERE point_id = 'rp-del-crash'",
            [],
            |row| row.get(0),
        )
        .expect("get status");

    assert_eq!(status, "DELETED", "should be DELETED after recovery");
}

// ============================================================================
// A-06: Crash Recovery Tests
// ============================================================================

/// Helper: count journal files in the repo
fn count_journals(handle: &nuwa_backup::repository::RepoHandle) -> usize {
    nuwa_backup::repository::transaction::journal::scan_journals(&handle.root)
        .expect("scan journals")
        .len()
}

/// A-06-01: Crash during CREATING state (no components completed)
#[test]
fn test_crash_during_creating() {
    let tmp = TempDir::new().expect("temp dir");
    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let journal = nuwa_backup::repository::TransactionJournal {
        restore_point_id: "crash-creating".to_string(),
        state: nuwa_backup::repository::TransactionState::Creating,
        components: nuwa_backup::repository::ComponentStatus::all_pending(),
        started_at: "2026-07-10T10:00:00Z".to_string(),
    };
    nuwa_backup::repository::transaction::journal::write_journal(&handle.root, &journal)
        .expect("write journal");

    assert_eq!(
        count_journals(&handle),
        1,
        "journal should exist before recovery"
    );

    let report = nuwa_backup::repository::CrashConsistencyManager::recover_at_startup(&handle)
        .expect("recover");

    assert_eq!(report.total_incomplete, 1, "1 incomplete journal");
    assert_eq!(report.marked_failed.len(), 1, "incomplete mark FAILED");
    assert_eq!(report.auto_committed.len(), 0, "0 auto-committed");
    assert!(report.errors.is_empty(), "no errors");
    assert_eq!(count_journals(&handle), 0, "journals cleaned up");
}

/// A-06-02: Crash during WRITING state (partial components completed)
#[test]
fn test_crash_during_writing_partial() {
    let tmp = TempDir::new().expect("temp dir");
    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let mut components = nuwa_backup::repository::ComponentStatus::all_pending();
    components.block_store = nuwa_backup::repository::ComponentPhase::Completed;

    let journal = nuwa_backup::repository::TransactionJournal {
        restore_point_id: "crash-partial".to_string(),
        state: nuwa_backup::repository::TransactionState::Writing,
        components,
        started_at: "2026-07-10T10:00:00Z".to_string(),
    };
    nuwa_backup::repository::transaction::journal::write_journal(&handle.root, &journal)
        .expect("write journal");

    let report = nuwa_backup::repository::CrashConsistencyManager::recover_at_startup(&handle)
        .expect("recover");

    assert_eq!(report.total_incomplete, 1);
    assert_eq!(
        report.marked_failed.len(),
        1,
        "partial write should be FAILED"
    );
    assert_eq!(report.auto_committed.len(), 0);
    assert!(report.errors.is_empty());
}

/// A-06-03: Crash after all components completed (data is safe)
#[test]
fn test_crash_after_all_components_completed() {
    let tmp = TempDir::new().expect("temp dir");
    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let mut components = nuwa_backup::repository::ComponentStatus::all_pending();
    components.block_store = nuwa_backup::repository::ComponentPhase::Completed;
    components.block_map = nuwa_backup::repository::ComponentPhase::Completed;
    components.catalog = nuwa_backup::repository::ComponentPhase::Completed;
    components.metadata = nuwa_backup::repository::ComponentPhase::Completed;

    let journal = nuwa_backup::repository::TransactionJournal {
        restore_point_id: "crash-verify-complete".to_string(),
        state: nuwa_backup::repository::TransactionState::Verifying,
        components,
        started_at: "2026-07-10T10:00:00Z".to_string(),
    };
    nuwa_backup::repository::transaction::journal::write_journal(&handle.root, &journal)
        .expect("write journal");

    let report = nuwa_backup::repository::CrashConsistencyManager::recover_at_startup(&handle)
        .expect("recover");

    assert_eq!(report.total_incomplete, 1);
    assert_eq!(
        report.auto_committed.len(),
        1,
        "all components done auto-commit"
    );
    assert_eq!(report.marked_failed.len(), 0);
    assert!(report.errors.is_empty());
}

/// A-06-04: COMMITTED journal (terminal, just cleanup)
#[test]
fn test_crash_committed_journal_cleanup() {
    let tmp = TempDir::new().expect("temp dir");
    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let mut components = nuwa_backup::repository::ComponentStatus::all_pending();
    components.block_store = nuwa_backup::repository::ComponentPhase::Completed;
    components.block_map = nuwa_backup::repository::ComponentPhase::Completed;
    components.catalog = nuwa_backup::repository::ComponentPhase::Completed;
    components.metadata = nuwa_backup::repository::ComponentPhase::Completed;

    let journal = nuwa_backup::repository::TransactionJournal {
        restore_point_id: "committed-cleanup".to_string(),
        state: nuwa_backup::repository::TransactionState::Committed,
        components,
        started_at: "2026-07-10T10:00:00Z".to_string(),
    };
    nuwa_backup::repository::transaction::journal::write_journal(&handle.root, &journal)
        .expect("write journal");

    assert_eq!(count_journals(&handle), 1);

    let report = nuwa_backup::repository::CrashConsistencyManager::recover_at_startup(&handle)
        .expect("recover");

    assert_eq!(report.total_incomplete, 1);
    assert_eq!(report.auto_committed.len(), 0);
    assert_eq!(report.marked_failed.len(), 0);
    assert_eq!(count_journals(&handle), 0, "journal cleaned up");
}

/// A-06-05: FAILED journal (terminal, just cleanup)
#[test]
fn test_crash_failed_journal_cleanup() {
    let tmp = TempDir::new().expect("temp dir");
    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let journal = nuwa_backup::repository::TransactionJournal {
        restore_point_id: "failed-cleanup".to_string(),
        state: nuwa_backup::repository::TransactionState::Failed,
        components: nuwa_backup::repository::ComponentStatus::all_pending(),
        started_at: "2026-07-10T10:00:00Z".to_string(),
    };
    nuwa_backup::repository::transaction::journal::write_journal(&handle.root, &journal)
        .expect("write journal");

    let report = nuwa_backup::repository::CrashConsistencyManager::recover_at_startup(&handle)
        .expect("recover");

    assert_eq!(report.total_incomplete, 1);
    assert_eq!(report.auto_committed.len(), 0);
    assert_eq!(report.marked_failed.len(), 0, "already FAILED");
    assert_eq!(count_journals(&handle), 0, "journal cleaned up");
}

/// A-06-06: Recovery with no journals
#[test]
fn test_crash_recovery_empty_repo() {
    let tmp = TempDir::new().expect("temp dir");
    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let report = nuwa_backup::repository::CrashConsistencyManager::recover_at_startup(&handle)
        .expect("recover");

    assert_eq!(report.total_incomplete, 0, "no journals");
    assert_eq!(report.auto_committed.len(), 0);
    assert_eq!(report.marked_failed.len(), 0);
    assert!(report.errors.is_empty());
}

/// A-06-07: Multiple journals in different states
#[test]
fn test_crash_multiple_journals() {
    let tmp = TempDir::new().expect("temp dir");
    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let j1 = nuwa_backup::repository::TransactionJournal {
        restore_point_id: "multi-creating".to_string(),
        state: nuwa_backup::repository::TransactionState::Creating,
        components: nuwa_backup::repository::ComponentStatus::all_pending(),
        started_at: "2026-07-10T10:00:00Z".to_string(),
    };
    nuwa_backup::repository::transaction::journal::write_journal(&handle.root, &j1)
        .expect("write j1");

    let mut all_done = nuwa_backup::repository::ComponentStatus::all_pending();
    all_done.block_store = nuwa_backup::repository::ComponentPhase::Completed;
    all_done.block_map = nuwa_backup::repository::ComponentPhase::Completed;
    all_done.catalog = nuwa_backup::repository::ComponentPhase::Completed;
    all_done.metadata = nuwa_backup::repository::ComponentPhase::Completed;
    let j2 = nuwa_backup::repository::TransactionJournal {
        restore_point_id: "multi-verify-done".to_string(),
        state: nuwa_backup::repository::TransactionState::Verifying,
        components: all_done,
        started_at: "2026-07-10T10:00:00Z".to_string(),
    };
    nuwa_backup::repository::transaction::journal::write_journal(&handle.root, &j2)
        .expect("write j2");

    let mut all_done2 = nuwa_backup::repository::ComponentStatus::all_pending();
    all_done2.block_store = nuwa_backup::repository::ComponentPhase::Completed;
    all_done2.block_map = nuwa_backup::repository::ComponentPhase::Completed;
    all_done2.catalog = nuwa_backup::repository::ComponentPhase::Completed;
    all_done2.metadata = nuwa_backup::repository::ComponentPhase::Completed;
    let j3 = nuwa_backup::repository::TransactionJournal {
        restore_point_id: "multi-committed".to_string(),
        state: nuwa_backup::repository::TransactionState::Committed,
        components: all_done2,
        started_at: "2026-07-10T10:00:00Z".to_string(),
    };
    nuwa_backup::repository::transaction::journal::write_journal(&handle.root, &j3)
        .expect("write j3");

    assert_eq!(count_journals(&handle), 3);

    let report = nuwa_backup::repository::CrashConsistencyManager::recover_at_startup(&handle)
        .expect("recover");

    assert_eq!(report.total_incomplete, 3);
    assert_eq!(report.marked_failed.len(), 1, "1 marked FAILED");
    assert_eq!(report.auto_committed.len(), 1, "1 auto-committed");
    assert!(report.errors.is_empty());
    assert_eq!(count_journals(&handle), 0, "all cleaned up");
}

/// A-06-08: Normal backup flow leaves no journals after commit
#[test]
fn test_crash_no_journal_after_normal_flow() {
    let tmp = TempDir::new().expect("temp dir");
    let handle =
        nuwa_backup::repository::init_repo(tmp.path(), nuwa_backup::repository::DEFAULT_BLOCK_SIZE)
            .expect("init");

    let mut mgr = nuwa_backup::repository::CrashConsistencyManager::begin(&handle, "normal-flow")
        .expect("begin");
    mgr.complete_block_store().expect("store");
    mgr.complete_block_map().expect("map");
    mgr.complete_catalog().expect("catalog");
    mgr.complete_metadata().expect("meta");
    mgr.commit().expect("commit");

    assert_eq!(
        count_journals(&handle),
        0,
        "no lingering journals after commit"
    );
}
