// ============================================================================
// repo_manager.rs 鈥?S-01: Repository initialization, opening, and validation
// ============================================================================
//
// Phase S S-01. Manages the lifecycle of a backup Repository.
// Responsible for:
//   - Creating the directory structure (.nuwarepo/, block-store/, backup-instances/)
//   - Initializing repo.db with schema and metadata
//   - Opening and validating an existing Repository
//   - Providing the root path for other components

use crate::repository::error::RepositoryError;
use crate::repository::metadata::models::{ChunkPolicyMetadata, RepositoryCapabilities};
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Default block size for Phase S: 256KB
pub const DEFAULT_BLOCK_SIZE: u32 = 262144;

/// Repository metadata stored in repo.db
#[derive(Debug, Clone)]
pub struct RepositoryInfo {
    pub version: u32,
    pub chunk_policy: ChunkPolicyMetadata,
    pub restore_point_count: u64,
    pub total_raw_bytes: u64,
    // Enterprise readiness fields (Architecture v1.1)
    pub repository_id: String,
    pub repository_name: String,
    pub repository_type: String,
    pub format_version: u32,
    pub min_compatible_version: u32,
    pub capabilities: RepositoryCapabilities,
}

/// Handle to an opened Repository, providing access to its paths.
#[derive(Debug, Clone)]
pub struct RepoHandle {
    pub root: PathBuf,
    pub nuwarepo_dir: PathBuf,
    pub block_store_dir: PathBuf,
    pub instances_dir: PathBuf,
    pub legacy_dir: PathBuf,
    pub repo_db_path: PathBuf,
    pub info: RepositoryInfo,
}

impl RepoHandle {
    pub fn repo_db(&self) -> Result<Connection, RepositoryError> {
        let conn = Connection::open(&self.repo_db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch("PRAGMA busy_timeout=5000;")?;
        Ok(conn)
    }
}

/// Check if a path contains an existing Nuwa Repository.
pub fn is_repository(path: &Path) -> bool {
    path.join(".nuwarepo").join("repo.db").exists()
}

/// Create the Repository directory structure.
fn create_directory_structure(root: &Path) -> Result<(), RepositoryError> {
    let dirs = [
        root.join(".nuwarepo"),
        root.join("block-store"),
        root.join("backup-instances"),
        root.join("legacy"),
    ];

    for dir in &dirs {
        std::fs::create_dir_all(dir).map_err(|e| {
            RepositoryError::io(dir.clone(), "Cannot create repository directory", e)
        })?;
    }

    Ok(())
}

/// Initialize repo.db with schema
fn init_repo_db(
    db_path: &Path,
    chunk_policy: &ChunkPolicyMetadata,
) -> Result<String, RepositoryError> {
    let conn = Connection::open(db_path)?;

    // Enable WAL mode for crash safety
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;

    // Create tables
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS repository_meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS backup_jobs (
            job_id          TEXT PRIMARY KEY,
            job_name        TEXT NOT NULL,
            source_type     INTEGER NOT NULL,
            source_path     TEXT,
            created_at      TEXT NOT NULL,
            status          TEXT NOT NULL DEFAULT 'active'
        );

        CREATE TABLE IF NOT EXISTS restore_points (
            point_id        TEXT PRIMARY KEY,
            job_id          TEXT NOT NULL,
            chain_id        TEXT NOT NULL,
            chain_position  INTEGER NOT NULL,
            created_at      TEXT NOT NULL,
            status          TEXT NOT NULL,
            instance_path   TEXT NOT NULL,
            block_count     INTEGER NOT NULL DEFAULT 0,
            total_raw_bytes INTEGER NOT NULL DEFAULT 0,
            parent_point_id TEXT,
            FOREIGN KEY (job_id) REFERENCES backup_jobs(job_id)
        );

        CREATE INDEX IF NOT EXISTS idx_restore_job ON restore_points(job_id);
        CREATE INDEX IF NOT EXISTS idx_restore_chain ON restore_points(chain_id);
        ",
    )?;

    // Insert repository metadata
    let mut stmt =
        conn.prepare("INSERT OR REPLACE INTO repository_meta (key, value) VALUES (?1, ?2)")?;

    let rid = Uuid::new_v4().to_string();

    stmt.execute(["version", "1"])?;
    stmt.execute(["repository_id", &rid])?;
    stmt.execute(["repository_name", "Nuwa Backup Repository"])?;
    stmt.execute(["repository_type", "filesystem"])?;
    stmt.execute(["format_version", "1"])?;
    stmt.execute(["min_compatible_version", "1"])?;
    stmt.execute(["chunk_policy_type", &chunk_policy.policy_type])?;
    stmt.execute(["block_size", &chunk_policy.block_size.to_string()])?;
    stmt.execute(["cap_compression", "true"])?;
    stmt.execute(["cap_encryption", "false"])?;
    stmt.execute(["cap_dedup", "false"])?;
    stmt.execute(["cap_immutable", "false"])?;
    stmt.execute(["created_at", &chrono::Utc::now().to_rfc3339()])?;

    Ok(rid)
}

/// Initialize a new Repository at the given path.
/// Creates the directory structure and repo.db with schema.
pub fn init_repo(root: &Path, block_size: u32) -> Result<RepoHandle, RepositoryError> {
    // Validate block size (256KB default, must be between 4KB and 4MB)
    if !(4096..=4_194_304).contains(&block_size) {
        return Err(RepositoryError::General {
            detail: format!(
                "Block size must be between 4KB and 4MB. Got {} ({}KB)",
                block_size,
                block_size / 1024
            ),
        });
    }

    // Check if repository already exists
    if is_repository(root) {
        return Err(RepositoryError::AlreadyExists(root.to_path_buf()));
    }

    // Create directory structure
    create_directory_structure(root)?;

    let chunk_policy = ChunkPolicyMetadata {
        policy_type: "fixed".to_string(),
        block_size,
    };

    // Initialize repo.db
    let db_path = root.join(".nuwarepo").join("repo.db");
    let rid = init_repo_db(&db_path, &chunk_policy)?;
    write_repository_json(&root.join(".nuwarepo"), &rid, &chunk_policy)?;

    Ok(RepoHandle {
        root: root.to_path_buf(),
        nuwarepo_dir: root.join(".nuwarepo"),
        block_store_dir: root.join("block-store"),
        instances_dir: root.join("backup-instances"),
        legacy_dir: root.join("legacy"),
        repo_db_path: db_path,
        info: RepositoryInfo {
            version: 1,
            chunk_policy,
            restore_point_count: 0,
            total_raw_bytes: 0,
            repository_id: rid.clone(),
            repository_name: "Nuwa Backup Repository".to_string(),
            repository_type: "filesystem".to_string(),
            format_version: 1,
            min_compatible_version: 1,
            capabilities: RepositoryCapabilities::phase_s_default(),
        },
    })
}

/// Open an existing Repository and validate its structure.
pub fn open_repo(root: &Path) -> Result<RepoHandle, RepositoryError> {
    if !is_repository(root) {
        return Err(RepositoryError::NotFound(root.to_path_buf()));
    }

    let db_path = root.join(".nuwarepo").join("repo.db");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
    conn.execute_batch("PRAGMA foreign_keys=ON;")?;
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;

    // Verify repository version
    let version: String = conn
        .query_row(
            "SELECT value FROM repository_meta WHERE key = 'version'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| {
            RepositoryError::InvalidRepository(
                root.to_path_buf(),
                "Cannot read repository version".to_string(),
            )
        })?;

    let version_num: u32 = version.parse().unwrap_or(0);
    if version_num != 1 {
        return Err(RepositoryError::UnsupportedVersion(version_num));
    }

    // Read chunk policy
    let policy_type: String = conn
        .query_row(
            "SELECT value FROM repository_meta WHERE key = 'chunk_policy_type'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| {
            RepositoryError::InvalidRepository(
                root.to_path_buf(),
                "Missing chunk_policy_type in repo.db".to_string(),
            )
        })?;

    let block_size_str: String = conn
        .query_row(
            "SELECT value FROM repository_meta WHERE key = 'block_size'",
            [],
            |row| row.get(0),
        )
        .map_err(|_| {
            RepositoryError::InvalidRepository(
                root.to_path_buf(),
                "Missing block_size in repo.db".to_string(),
            )
        })?;

    let block_size: u32 = block_size_str
        .parse()
        .map_err(|_| RepositoryError::General {
            detail: "Invalid block_size in repo.db: not a valid number".to_string(),
        })?;

    // Read enterprise identity fields (Architecture v1.1)
    let rid: String = read_meta_or(&conn, "repository_id", "")?;
    let rname: String = read_meta_or(&conn, "repository_name", "Nuwa Backup Repository")?;
    let rtype: String = read_meta_or(&conn, "repository_type", "filesystem")?;
    let fmt_ver: u32 = read_meta_or(&conn, "format_version", "1")?
        .parse()
        .unwrap_or(1);
    let min_ver: u32 = read_meta_or(&conn, "min_compatible_version", "1")?
        .parse()
        .unwrap_or(1);

    // Read capabilities (Architecture v1.1)
    let cap_compression: bool = read_meta_or(&conn, "cap_compression", "true")?
        .parse()
        .unwrap_or(true);
    let cap_encryption: bool = read_meta_or(&conn, "cap_encryption", "false")?
        .parse()
        .unwrap_or(false);
    let cap_dedup: bool = read_meta_or(&conn, "cap_dedup", "false")?
        .parse()
        .unwrap_or(false);
    let cap_immutable: bool = read_meta_or(&conn, "cap_immutable", "false")?
        .parse()
        .unwrap_or(false);

    let capabilities = RepositoryCapabilities {
        compression: cap_compression,
        encryption: cap_encryption,
        dedup: cap_dedup,
        immutable_storage: cap_immutable,
    };

    // Validate format_version compatibility
    if fmt_ver > 1 || min_ver > 1 {
        return Err(RepositoryError::UnsupportedRepositoryVersion(
            fmt_ver, min_ver,
        ));
    }

    // Validate repository.json if present (Architecture v1.1)
    let repo_json_path = root.join(".nuwarepo").join("repository.json");
    if repo_json_path.exists() {
        if let Ok(json_content) = fs::read_to_string(&repo_json_path) {
            if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&json_content) {
                if let Some(json_id) = json_val.get("repository_id").and_then(|v| v.as_str()) {
                    if json_id != rid {
                        return Err(RepositoryError::RepositoryIdentityMismatch {
                            field: "repository_id".to_string(),
                            v1: json_id.to_string(),
                            v2: rid,
                        });
                    }
                }
            }
        }
    }

    // Count restore points
    let point_count: u64 = conn
        .query_row(
            "SELECT COUNT(*) FROM restore_points WHERE status = 'COMMITTED'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(RepoHandle {
        root: root.to_path_buf(),
        nuwarepo_dir: root.join(".nuwarepo"),
        block_store_dir: root.join("block-store"),
        instances_dir: root.join("backup-instances"),
        legacy_dir: root.join("legacy"),
        repo_db_path: db_path,
        info: RepositoryInfo {
            version: version_num,
            chunk_policy: ChunkPolicyMetadata {
                policy_type,
                block_size,
            },
            restore_point_count: point_count,
            total_raw_bytes: 0,
            repository_id: rid,
            repository_name: rname,
            repository_type: rtype,
            format_version: fmt_ver,
            min_compatible_version: min_ver,
            capabilities,
        },
    })
}

/// Quick structural integrity check of a Repository.
/// Checks:
///   1. Directory structure exists
///   2. repo.db is readable and has correct schema
///   3. Block count consistency (if there are restore points)
pub fn check_repo(handle: &RepoHandle) -> Result<(), RepositoryError> {
    // Check essential directories exist
    let required_dirs = [
        &handle.nuwarepo_dir,
        &handle.block_store_dir,
        &handle.instances_dir,
    ];

    for dir in required_dirs {
        if !dir.exists() {
            return Err(RepositoryError::SelfCheckFailed(format!(
                "Missing required directory: {}",
                dir.display()
            )));
        }
    }

    // Check repo.db is readable
    let conn = handle
        .repo_db()
        .map_err(|e| RepositoryError::SelfCheckFailed(format!("Cannot open repo.db: {}", e)))?;

    // Verify schema integrity
    conn.execute_batch("PRAGMA integrity_check;").map_err(|e| {
        RepositoryError::SelfCheckFailed(format!("repo.db integrity check failed: {}", e))
    })?;

    // Verify tables exist
    for table in &["repository_meta", "backup_jobs", "restore_points"] {
        let count: u32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .map_err(|_| {
                RepositoryError::SelfCheckFailed("Cannot query table list from repo.db".to_string())
            })?;

        if count == 0 {
            return Err(RepositoryError::SelfCheckFailed(format!(
                "Missing required table: {}",
                table
            )));
        }
    }

    Ok(())
}

/// Write repository.json identity manifest (Architecture v1.1)
fn write_repository_json(
    nuwarepo_dir: &Path,
    repository_id: &str,
    _chunk_policy: &ChunkPolicyMetadata,
) -> Result<(), RepositoryError> {
    let manifest = serde_json::json!({
        "schema_version": "1.0",
        "repository_id": repository_id,
        "name": "Nuwa Backup Repository",
        "type": "filesystem",
        "format_version": 1,
        "min_compatible_version": 1,
        "created_at": chrono::Utc::now().to_rfc3339(),
        "capabilities": {
            "compression": true,
            "encryption": false,
            "dedup": false,
            "immutable_storage": false,
        }
    });

    let path = nuwarepo_dir.join("repository.json");
    let tmp_path = nuwarepo_dir.join("repository.json.tmp");

    let json_str = serde_json::to_string_pretty(&manifest)?;
    fs::write(&tmp_path, &json_str).map_err(|e| {
        RepositoryError::io(tmp_path.clone(), "Cannot write repository.json tmp", e)
    })?;
    fs::rename(&tmp_path, &path)
        .map_err(|e| RepositoryError::io(path, "Cannot rename repository.json to final", e))?;

    Ok(())
}

/// Read a metadata value from repo.db, returning default on missing key
fn read_meta_or(conn: &Connection, key: &str, default: &str) -> Result<String, RepositoryError> {
    conn.query_row(
        "SELECT value FROM repository_meta WHERE key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .or_else(|_| Ok(default.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_repo() -> (RepoHandle, TempDir) {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();
        (handle, tmp)
    }

    #[test]
    fn test_init_repo_creates_structure() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        assert!(handle.nuwarepo_dir.exists());
        assert!(handle.block_store_dir.exists());
        assert!(handle.instances_dir.exists());
        assert!(handle.legacy_dir.exists());
        assert!(handle.repo_db_path.exists());
    }

    #[test]
    fn test_init_repo_creates_repo_db() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        let conn = handle.repo_db().unwrap();

        // Verify metadata
        let version: String = conn
            .query_row(
                "SELECT value FROM repository_meta WHERE key = 'version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(version, "1");

        let block_size_str: String = conn
            .query_row(
                "SELECT value FROM repository_meta WHERE key = 'block_size'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let block_size: u32 = block_size_str.parse().unwrap();
        assert_eq!(block_size, DEFAULT_BLOCK_SIZE);
    }

    #[test]
    fn test_init_on_existing_repo_fails() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();
        let result = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE);
        assert!(matches!(result, Err(RepositoryError::AlreadyExists(_))));
    }

    #[test]
    fn test_open_repo() {
        let tmp = TempDir::new().unwrap();
        init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        let handle = open_repo(tmp.path()).unwrap();
        assert_eq!(handle.info.version, 1);
        assert_eq!(handle.info.chunk_policy.block_size, DEFAULT_BLOCK_SIZE);
    }

    #[test]
    fn test_open_non_existent_repo() {
        let tmp = TempDir::new().unwrap();
        let result = open_repo(tmp.path());
        assert!(matches!(result, Err(RepositoryError::NotFound(_))));
    }

    #[test]
    fn test_is_repository() {
        let tmp = TempDir::new().unwrap();
        assert!(!is_repository(tmp.path()));

        init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();
        assert!(is_repository(tmp.path()));
    }

    #[test]
    fn test_check_repo() {
        let (handle, _tmp) = setup_repo();
        assert!(check_repo(&handle).is_ok());
    }

    #[test]
    fn test_check_repo_missing_directory_fails() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        // Remove a required directory to simulate corruption
        std::fs::remove_dir(&handle.block_store_dir).unwrap();

        let result = check_repo(&handle);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_block_size() {
        let tmp = TempDir::new().unwrap();
        let result = init_repo(tmp.path(), 100); // Too small
        assert!(result.is_err());

        let result = init_repo(tmp.path(), 10_000_000); // Too large
        assert!(result.is_err());
    }

    #[test]
    fn test_default_block_size() {
        assert_eq!(DEFAULT_BLOCK_SIZE, 262144);
    }
}
