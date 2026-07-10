// ============================================================================
// rebuild.rs — S-13: Repository rebuild from backup-instance metadata
// ============================================================================
//
// Rebuilds repo.db by scanning backup-instances/ for backup-metadata.json.
// See Architecture v1.0 §13.2.
//
// Recovery boundaries (Architecture v1.0 §13.3):
//   repo.db         → rebuildable
//   block-map.db    → NOT rebuildable (lost = unrecoverable)
//   catalog.db      → NOT rebuildable (lost = full-point restore only)
//
// These boundaries are enforced in code.

use crate::repository::error::RepositoryError;
use crate::repository::repo_manager::{open_repo, RepoHandle};
use crate::repository::retention::engine::recover_incomplete_deletions;
use rusqlite::Connection;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Rebuild repo.db by scanning backup-instances/ for backup-metadata.json.
///
/// This function:
///   1. Opens the repository
///   2. Scans backup-instances/*/backup-metadata.json
///   3. Rebuilds restore_points table
///   4. Rebuilds backup_jobs table
///   5. Marks CREATING/WRITING/VERIFYING points as FAILED
///   6. Delegates DELETING recovery to Retention Engine
///
/// # Safety
/// Does NOT modify block-store, block-map, or catalog data.
pub fn rebuild_repo(repo_path: &Path) -> Result<(), RepositoryError> {
    if !repo_path.exists() {
        return Err(RepositoryError::NotFound(repo_path.to_path_buf()));
    }

    let handle = open_repo(repo_path)?;
    let conn = handle.repo_db()?;

    conn.execute_batch("BEGIN TRANSACTION;")?;
    let result = rebuild_tables_inner(&handle, &conn);
    match result {
        Ok(()) => conn.execute_batch("COMMIT;")?,
        Err(e) => {
            let _ = conn.execute_batch("ROLLBACK;");
            return Err(e);
        }
    }

    // Handle DELETING states via Retention Engine
    recover_incomplete_deletions(&handle)?;

    Ok(())
}

/// Inner rebuild logic (inside a DB transaction).
fn rebuild_tables_inner(handle: &RepoHandle, conn: &Connection) -> Result<(), RepositoryError> {
    // Clear existing dynamic data (keep repository_meta)
    conn.execute_batch(
        "DELETE FROM backup_jobs;
         DELETE FROM restore_points;",
    )?;

    let instances_dir = &handle.instances_dir;
    if !instances_dir.exists() {
        fs::create_dir_all(instances_dir)?;
        return Ok(());
    }

    let mut discovered_jobs: HashSet<String> = HashSet::new();

    let entries = fs::read_dir(instances_dir).map_err(|e| {
        RepositoryError::io(
            instances_dir.clone(),
            "Cannot scan backup-instances directory",
            e,
        )
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            RepositoryError::io(instances_dir.clone(), "Cannot read directory entry", e)
        })?;

        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        let metadata_path = dir_path.join("backup-metadata.json");
        if !metadata_path.exists() {
            continue;
        }

        let content = fs::read_to_string(&metadata_path).map_err(|e| {
            RepositoryError::io(metadata_path.clone(), "Cannot read backup-metadata.json", e)
        })?;

        let meta: serde_json::Value = serde_json::from_str(&content)?;

        let point_id = meta["restore_point_id"]
            .as_str()
            .unwrap_or("unknown")
            .to_string();
        let status_str = meta["status"].as_str().unwrap_or("UNKNOWN");
        let job_id = meta["job_id"].as_str().unwrap_or("unknown").to_string();
        let job_name = meta
            .get("job_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&job_id)
            .to_string();
        let source_type = meta
            .get("source_type")
            .and_then(|v| v.as_str())
            .unwrap_or("File");
        let source_type_int = match source_type {
            "Volume" => 1,
            "Disk" => 2,
            _ => 0,
        };
        let created_at = meta["created_at"]
            .as_str()
            .unwrap_or("2026-01-01T00:00:00Z")
            .to_string();

        // Determine restore status
        let restore_status = match status_str {
            "COMMITTED" | "FAILED" | "DELETED" | "DELETING" => status_str,
            _ => "FAILED",
        };

        // Insert backup_job FIRST (FK must exist before restore_point)
        if !discovered_jobs.contains(&job_id) {
            conn.execute(
                "INSERT OR IGNORE INTO backup_jobs
                 (job_id, job_name, source_type, source_path, created_at, status)
                 VALUES (?1, ?2, ?3, '', ?4, 'active')",
                rusqlite::params![job_id, job_name, source_type_int, created_at],
            )?;
            discovered_jobs.insert(job_id.clone());
        }

        let instance_path_str = dir_path.to_string_lossy().to_string();
        let total_raw_bytes: u64 = meta["summary"]["total_raw_bytes"].as_u64().unwrap_or(0);
        let chain_id = format!("chain-{}", job_id);

        conn.execute(
            "INSERT OR REPLACE INTO restore_points
             (point_id, job_id, chain_id, chain_position, created_at, status,
              instance_path, block_count, total_raw_bytes, parent_point_id)
             VALUES (?1, ?2, ?3, 0, ?4, ?5, ?6, 0, ?7, NULL)",
            rusqlite::params![
                point_id,
                job_id,
                chain_id,
                created_at,
                restore_status,
                instance_path_str,
                total_raw_bytes,
            ],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::repo_manager::{init_repo, DEFAULT_BLOCK_SIZE};
    use tempfile::TempDir;

    fn setup_repo_with_points() -> (RepoHandle, TempDir) {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        for point_id in &["point-a", "point-b"] {
            let dir = handle.instances_dir.join(point_id);
            fs::create_dir_all(&dir).unwrap();
            let meta = serde_json::json!({
                "schema_version": "1.0",
                "restore_point_id": point_id,
                "job_id": "job-001",
                "job_name": "test-job",
                "source_type": "File",
                "source_description": "test",
                "created_at": "2025-06-01T00:00:00Z",
                "status": "COMMITTED",
                "block_chunk_policy": {
                    "policy_type": "fixed",
                    "block_size": 262144
                },
                "block_map": {
                    "database": "block-map.db",
                    "block_count": 10,
                    "sha256": "abc",
                    "first_offset": 0,
                    "last_offset": 2621440
                },
                "summary": {
                    "total_raw_bytes": 5000,
                    "file_count": 5
                }
            });
            let content = serde_json::to_string_pretty(&meta).unwrap();
            fs::write(dir.join("backup-metadata.json"), &content).unwrap();
        }

        (handle, tmp)
    }

    #[test]
    fn test_rebuild_from_instance_metadata() {
        let (handle, tmp) = setup_repo_with_points();

        let conn = handle.repo_db().unwrap();
        conn.execute_batch("DELETE FROM restore_points; DELETE FROM backup_jobs;")
            .unwrap();
        drop(conn);
        drop(handle);

        rebuild_repo(tmp.path()).unwrap();

        let handle = open_repo(tmp.path()).unwrap();
        let conn = handle.repo_db().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM restore_points", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2, "Should restore both restore points");

        let job_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM backup_jobs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(job_count, 1, "Should restore the backup job");

        let status: String = conn
            .query_row(
                "SELECT status FROM restore_points WHERE point_id = 'point-a'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(status, "COMMITTED");
    }

    #[test]
    fn test_rebuild_handles_incomplete_points() {
        let (handle, tmp) = setup_repo_with_points();

        // Add an incomplete point (WRITING status)
        let dir = handle.instances_dir.join("point-incomplete");
        fs::create_dir_all(&dir).unwrap();
        let meta = serde_json::json!({
            "schema_version": "1.0",
            "restore_point_id": "point-incomplete",
            "job_id": "job-001",
            "job_name": "test-job",
            "source_type": "File",
            "created_at": "2025-07-01T00:00:00Z",
            "status": "WRITING",
            "block_chunk_policy": {
                "policy_type": "fixed",
                "block_size": 262144
            },
            "block_map": {
                "database": "block-map.db",
                "block_count": 3,
                "sha256": "def",
                "first_offset": 0,
                "last_offset": 786432
            },
            "summary": {
                "total_raw_bytes": 1500,
                "file_count": 2
            }
        });
        let content = serde_json::to_string_pretty(&meta).unwrap();
        fs::write(dir.join("backup-metadata.json"), &content).unwrap();

        let conn = handle.repo_db().unwrap();
        conn.execute_batch("DELETE FROM restore_points; DELETE FROM backup_jobs;")
            .unwrap();
        drop(conn);
        drop(handle);

        rebuild_repo(tmp.path()).unwrap();

        let handle = open_repo(tmp.path()).unwrap();
        let conn = handle.repo_db().unwrap();
        let status: String = conn
            .query_row(
                "SELECT status FROM restore_points WHERE point_id = 'point-incomplete'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(
            status, "FAILED",
            "Incomplete points should be marked FAILED"
        );
    }
}
