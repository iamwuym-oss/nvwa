// ============================================================================
// history.rs -- SQLite backup history module
//
// Responsibilities:
// 1. Create and maintain SQLite history database (.nuwa_history.db)
// 2. Record backup / restore / verify operation history
// 3. Query history records by conditions
// 4. Rebuild history index from manifests
//
// Authority relationship (strictly enforced):
//   manifest.json  鈫?Sole authoritative credential for each backup point (irreplaceable)
//        鈫?
//   .nuwa_history.db  鈫?History index, rebuildable by scanning manifests (deletable, rebuildable)
//
// If SQLite conflicts with manifest, manifest takes precedence.
// ============================================================================

use crate::errors::NuwaError;
use std::path::{Path, PathBuf};

/// Operation history record
#[derive(Debug, Clone)]
pub struct OperationRecord {
    pub backup_id: String,
    pub operation: String, // "backup" | "restore" | "verify"
    pub timestamp: String, // ISO 8601
    pub source_root: String,
    pub dest_path: String,
    pub job_name: Option<String>,
    pub file_count: u64,
    pub total_bytes: u64,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub status: String, // "success" | "failure" | "partial"
}

/// SQLite history database
pub struct HistoryDb {
    db_path: PathBuf,
}

impl HistoryDb {
    /// Open or create the history database at the given path
    ///
    /// If the database file does not exist, the table schema is automatically created.
    pub fn open_or_create(db_path: &Path) -> Result<Self, NuwaError> {
        let conn = Self::open_connection(db_path)?;

        // Create table structure (IF NOT EXISTS is idempotent)
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS operations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                backup_id TEXT NOT NULL,
                operation TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                source_root TEXT NOT NULL,
                dest_path TEXT NOT NULL,
                job_name TEXT,
                file_count INTEGER NOT NULL DEFAULT 0,
                total_bytes INTEGER NOT NULL DEFAULT 0,
                duration_ms INTEGER NOT NULL DEFAULT 0,
                exit_code INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'success'
            );

            CREATE INDEX IF NOT EXISTS idx_operations_timestamp
                ON operations(timestamp DESC);
            CREATE INDEX IF NOT EXISTS idx_operations_operation
                ON operations(operation);
            CREATE INDEX IF NOT EXISTS idx_operations_status
                ON operations(status);
            CREATE INDEX IF NOT EXISTS idx_operations_backup_id
                ON operations(backup_id);
            ",
        )
        .map_err(|e| Self::db_error(db_path, e))?;

        Ok(HistoryDb {
            db_path: db_path.to_path_buf(),
        })
    }

    /// Record one operation history entry
    pub fn record_operation(&self, record: &OperationRecord) -> Result<(), NuwaError> {
        let conn = Self::open_connection(&self.db_path)?;

        conn.execute(
            "INSERT INTO operations (backup_id, operation, timestamp, source_root, dest_path, job_name, file_count, total_bytes, duration_ms, exit_code, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                record.backup_id,
                record.operation,
                record.timestamp,
                record.source_root,
                record.dest_path,
                record.job_name,
                record.file_count as i64,
                record.total_bytes as i64,
                record.duration_ms as i64,
                record.exit_code,
                record.status,
            ],
        )
        .map_err(|e| Self::db_error(&self.db_path, e))?;

        Ok(())
    }

    /// Delete operation records by backup_id
    pub fn delete_operation_by_backup_id(&self, backup_id: &str) -> Result<u32, NuwaError> {
        let conn = Self::open_connection(&self.db_path)?;
        let count = conn
            .execute(
                "DELETE FROM operations WHERE backup_id = ?1",
                rusqlite::params![backup_id],
            )
            .map_err(|e| Self::db_error(&self.db_path, e))?;
        Ok(count as u32)
    }
    /// Query history records
    ///
    /// - limit: Maximum records to return (default 10)
    /// - operation: Optional filter ("backup" | "restore" | "verify")
    pub fn query_history(
        &self,
        limit: u32,
        operation: Option<&str>,
    ) -> Result<Vec<OperationRecord>, NuwaError> {
        let conn = Self::open_connection(&self.db_path)?;

        let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = match operation {
            Some(op) => (
                "SELECT backup_id, operation, timestamp, source_root, dest_path, job_name, file_count, total_bytes, duration_ms, exit_code, status
                 FROM operations WHERE operation = ?1 ORDER BY timestamp DESC LIMIT ?2"
                    .to_string(),
                vec![Box::new(op.to_string()), Box::new(limit as i64)],
            ),
            None => (
                "SELECT backup_id, operation, timestamp, source_root, dest_path, job_name, file_count, total_bytes, duration_ms, exit_code, status
                 FROM operations ORDER BY timestamp DESC LIMIT ?1"
                    .to_string(),
                vec![Box::new(limit as i64)],
            ),
        };

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| Self::db_error(&self.db_path, e))?;

        let param_refs: Vec<&dyn rusqlite::types::ToSql> =
            params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok(OperationRecord {
                    backup_id: row.get(0)?,
                    operation: row.get(1)?,
                    timestamp: row.get(2)?,
                    source_root: row.get(3)?,
                    dest_path: row.get(4)?,
                    job_name: row.get(5)?,
                    file_count: row.get::<_, i64>(6)? as u64,
                    total_bytes: row.get::<_, i64>(7)? as u64,
                    duration_ms: row.get::<_, i64>(8)? as u64,
                    exit_code: row.get(9)?,
                    status: row.get(10)?,
                })
            })
            .map_err(|e| Self::db_error(&self.db_path, e))?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row.map_err(|e| Self::db_error(&self.db_path, e))?);
        }
        Ok(records)
    }

    /// Rebuild history database from backup destination directory
    ///
    /// Scans <dest> for all backup point directories, reads manifest.json from each,
    /// and writes valid backup points into the SQLite history database.
    ///
    /// Backup points with corrupted manifests will NOT be recorded as successful history.
    /// If a record already exists in SQLite, it will not be duplicated (deduplicated by backup_id).
    pub fn rebuild_from_manifest(dest_root: &Path) -> Result<u32, NuwaError> {
        if !dest_root.exists() {
            return Err(NuwaError::InvalidArgument {
                detail: format!(
                    "Backup destination path does not exist '{}'",
                    dest_root.display()
                ),
                suggestion: "Please verify the backup destination path".to_string(),
            });
        }

        // Build history database path
        let db_path = Self::history_db_path(dest_root);

        // Delete old .nuwa_history.db (rebuild mode)
        if db_path.exists() {
            std::fs::remove_file(&db_path).map_err(|e| NuwaError::Io {
                source: Some(e),
                path: Some(db_path.clone()),
                detail: format!("Cannot delete old history database '{}'", db_path.display()),
                suggestion: "Check file permissions".to_string(),
            })?;
        }

        // Create new database
        let db = Self::open_or_create(&db_path)?;

        // Scan destination directory for backup point directories
        let entries = std::fs::read_dir(dest_root).map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(dest_root.to_path_buf()),
            detail: "Failed to read directory entries".to_string(),
            suggestion: "Check directory permissions".to_string(),
        })?;

        let mut imported_count = 0u32;

        for entry in entries {
            let entry = entry.map_err(|e| NuwaError::Io {
                source: Some(e),
                path: Some(dest_root.to_path_buf()),
                detail: "Failed to read directory entry".to_string(),
                suggestion: "Check directory permissions".to_string(),
            })?;

            let dir_path = entry.path();
            if !dir_path.is_dir() {
                continue;
            }

            // Skip hidden directories (like .nuwa_history.db itself)
            let dir_name = dir_path.file_name().unwrap_or_default().to_string_lossy();
            if dir_name.starts_with('.') {
                continue;
            }

            let manifest_path = dir_path.join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            // Read manifest.json
            let content = match std::fs::read_to_string(&manifest_path) {
                Ok(c) => c,
                Err(_) => continue, // Cannot read manifest -- skip this backup point
            };

            let manifest: serde_json::Value = match serde_json::from_str(&content) {
                Ok(v) => v,
                Err(_) => continue, // Corrupted manifest -- do not record as valid history
            };

            let backup_id = manifest
                .get("backup_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let created_at = manifest
                .get("created_at")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let source_root = manifest
                .get("source_root")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let file_count = manifest
                .get("summary")
                .and_then(|s| s.get("file_count"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let total_bytes = manifest
                .get("summary")
                .and_then(|s| s.get("total_bytes"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            // Record as backup operation
            // Check if backup_id already exists
            if db.check_exists(backup_id).unwrap_or(false) {
                continue; // Already exists -- skip duplicate
            }

            let record = OperationRecord {
                backup_id: backup_id.to_string(),
                operation: "backup".to_string(),
                timestamp: created_at.to_string(),
                source_root: source_root.to_string(),
                dest_path: dest_root.to_string_lossy().to_string(),
                job_name: None,
                file_count,
                total_bytes,
                duration_ms: 0, // Original duration is unknown during rebuild
                exit_code: 0,
                status: "success".to_string(),
            };

            if db.record_operation(&record).is_ok() {
                imported_count += 1;
            }
        }

        Ok(imported_count)
    }

    /// Check if a backup_id already exists (for rebuild deduplication)
    fn check_exists(&self, backup_id: &str) -> Result<bool, NuwaError> {
        let conn = Self::open_connection(&self.db_path)?;
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM operations WHERE backup_id = ?1")
            .map_err(|e| Self::db_error(&self.db_path, e))?;
        let count: i64 = stmt
            .query_row(rusqlite::params![backup_id], |row| row.get(0))
            .map_err(|e| Self::db_error(&self.db_path, e))?;
        Ok(count > 0)
    }

    /// Get total number of history records
    pub fn count(&self) -> Result<u32, NuwaError> {
        let conn = Self::open_connection(&self.db_path)?;
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM operations")
            .map_err(|e| Self::db_error(&self.db_path, e))?;
        let count: i64 = stmt
            .query_row([], |row| row.get(0))
            .map_err(|e| Self::db_error(&self.db_path, e))?;
        Ok(count as u32)
    }

    /// Get the history database path
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Establish SQLite database connection
    fn open_connection(db_path: &Path) -> Result<rusqlite::Connection, NuwaError> {
        rusqlite::Connection::open(db_path).map_err(|e| NuwaError::General {
            detail: format!(
                "Cannot open history database '{}': {}",
                db_path.display(),
                e
            ),
            suggestion: "Check file permissions or disk space".to_string(),
        })
    }

    /// Convert rusqlite error to NuwaError
    fn db_error(db_path: &Path, e: rusqlite::Error) -> NuwaError {
        NuwaError::General {
            detail: format!("History database operation failed '{}': {}", db_path.display(), e),
            suggestion: "If the problem persists, try deleting .nuwa_history.db and running 'nuwa history --rebuild' to recreate it".to_string(),
        }
    }

    /// Build history database path from backup destination
    pub fn history_db_path(dest_root: &Path) -> PathBuf {
        dest_root.join(".nuwa_history.db")
    }
}

/// Print history records to console
pub fn print_history(records: &[OperationRecord]) {
    if records.is_empty() {
        println!("No history records.");
        return;
    }

    println!("Operation history ({} records):\n", records.len());
    println!(
        "{:<4} {:<12} {:<22} {:<10} {:<10} {:<12} {:<10}",
        "ID", "Operation", "Time", "Status", "Files", "Size(MB)", "Time(s)"
    );
    println!("{}", "-".repeat(80));

    for record in records {
        let id_display = if record.backup_id.len() > 8 {
            &record.backup_id[..8]
        } else {
            &record.backup_id
        };
        let status_display = match record.status.as_str() {
            "success" => "OK",
            "failure" => "FAIL",
            "partial" => "PARTIAL",
            _ => &record.status,
        };
        let size_mb = record.total_bytes as f64 / (1024.0 * 1024.0);
        let duration_s = record.duration_ms as f64 / 1000.0;

        println!(
            "{:<4} {:<12} {:<22} {:<10} {:<10} {:<12.2} {:<10.2}",
            id_display,
            record.operation,
            record.timestamp,
            status_display,
            record.file_count,
            size_mb,
            duration_s,
        );
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_test_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("nuwa_history_test_{}_{}", std::process::id(), n))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    fn create_test_db() -> (HistoryDb, PathBuf) {
        let dir = unique_test_dir();
        fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join(".nuwa_history.db");
        let db = HistoryDb::open_or_create(&db_path).unwrap();
        (db, db_path)
    }

    #[test]
    fn test_create_history_db() {
        let (db, db_path) = create_test_db();
        assert!(db_path.exists());
        assert_eq!(db.count().unwrap(), 0);
        cleanup(db_path.parent().unwrap());
    }

    #[test]
    fn test_record_backup_operation() {
        let (db, db_path) = create_test_db();
        let record = OperationRecord {
            backup_id: "test-uuid-0001".to_string(),
            operation: "backup".to_string(),
            timestamp: "2026-07-05T10:00:00+08:00".to_string(),
            source_root: "C:\\Users\\Test".to_string(),
            dest_path: db_path.parent().unwrap().to_string_lossy().to_string(),
            job_name: None,
            file_count: 100,
            total_bytes: 1048576,
            duration_ms: 5000,
            exit_code: 0,
            status: "success".to_string(),
        };
        db.record_operation(&record).unwrap();
        assert_eq!(db.count().unwrap(), 1);
        cleanup(db_path.parent().unwrap());
    }

    #[test]
    fn test_record_restore_operation() {
        let (db, db_path) = create_test_db();
        let record = OperationRecord {
            backup_id: "test-uuid-0002".to_string(),
            operation: "restore".to_string(),
            timestamp: "2026-07-05T11:00:00+08:00".to_string(),
            source_root: "C:\\Users\\Test".to_string(),
            dest_path: "C:\\Restore".to_string(),
            job_name: None,
            file_count: 50,
            total_bytes: 512000,
            duration_ms: 3000,
            exit_code: 0,
            status: "success".to_string(),
        };
        db.record_operation(&record).unwrap();
        let records = db.query_history(10, Some("restore")).unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].operation, "restore");
        cleanup(db_path.parent().unwrap());
    }

    #[test]
    fn test_record_verify_operation() {
        let (db, db_path) = create_test_db();
        let record = OperationRecord {
            backup_id: "test-uuid-0003".to_string(),
            operation: "verify".to_string(),
            timestamp: "2026-07-05T12:00:00+08:00".to_string(),
            source_root: "C:\\Users\\Test".to_string(),
            dest_path: db_path.parent().unwrap().to_string_lossy().to_string(),
            job_name: None,
            file_count: 0,
            total_bytes: 0,
            duration_ms: 1000,
            exit_code: 0,
            status: "success".to_string(),
        };
        db.record_operation(&record).unwrap();
        assert_eq!(db.count().unwrap(), 1);
        cleanup(db_path.parent().unwrap());
    }

    #[test]
    fn test_query_limit() {
        let (db, db_path) = create_test_db();
        for i in 0..5 {
            let record = OperationRecord {
                backup_id: format!("limit-test-{:04}", i),
                operation: "backup".to_string(),
                timestamp: format!("2026-07-05T10:00:0{}+08:00", i),
                source_root: "C:\\Users\\Test".to_string(),
                dest_path: db_path.parent().unwrap().to_string_lossy().to_string(),
                job_name: None,
                file_count: 10,
                total_bytes: 102400,
                duration_ms: 1000,
                exit_code: 0,
                status: "success".to_string(),
            };
            db.record_operation(&record).unwrap();
        }
        let records = db.query_history(3, None).unwrap();
        assert_eq!(records.len(), 3);
        cleanup(db_path.parent().unwrap());
    }

    #[test]
    fn test_query_by_operation() {
        let (db, db_path) = create_test_db();
        for i in 0..3 {
            let record = OperationRecord {
                backup_id: format!("backup-op-{:04}", i),
                operation: "backup".to_string(),
                timestamp: format!("2026-07-05T10:00:0{}+08:00", i),
                source_root: "C:\\Users\\Test".to_string(),
                dest_path: db_path.parent().unwrap().to_string_lossy().to_string(),
                job_name: None,
                file_count: 10,
                total_bytes: 102400,
                duration_ms: 1000,
                exit_code: 0,
                status: "success".to_string(),
            };
            db.record_operation(&record).unwrap();
        }

        let restore_record = OperationRecord {
            backup_id: "restore-op-0001".to_string(),
            operation: "restore".to_string(),
            timestamp: "2026-07-05T11:00:00+08:00".to_string(),
            source_root: "C:\\Users\\Test".to_string(),
            dest_path: "C:\\Restore".to_string(),
            job_name: None,
            file_count: 100,
            total_bytes: 1048576,
            duration_ms: 3000,
            exit_code: 0,
            status: "success".to_string(),
        };
        db.record_operation(&restore_record).unwrap();

        let backups = db.query_history(10, Some("backup")).unwrap();
        assert_eq!(backups.len(), 3);
        assert!(backups.iter().all(|r| r.operation == "backup"));

        let restores = db.query_history(10, Some("restore")).unwrap();
        assert_eq!(restores.len(), 1);
        cleanup(db_path.parent().unwrap());
    }

    #[test]
    fn test_rebuild_from_manifest() {
        let dest_dir = unique_test_dir();

        for i in 0..3 {
            let backup_dir_name = format!("20260705_10000{}_Test", i);
            let backup_dir = dest_dir.join(&backup_dir_name);
            fs::create_dir_all(&backup_dir).unwrap();

            let manifest = serde_json::json!({
                "schema_version": "1.0",
                "backup_id": format!("rebuild-uuid-{:04}", i),
                "created_at": format!("2026-07-05T10:00:0{}+08:00", i),
                "source_root": "C:\\Users\\Test",
                "storage_format": "flat-file",
                "compression": { "enabled": false, "algorithm": null },
                "files": [],
                "directories": [],
                "summary": { "file_count": 10, "directory_count": 2, "total_bytes": 102400 }
            });
            fs::write(
                backup_dir.join("manifest.json"),
                serde_json::to_string_pretty(&manifest).unwrap(),
            )
            .unwrap();
        }

        let count = HistoryDb::rebuild_from_manifest(&dest_dir).unwrap();
        assert_eq!(count, 3);

        let db_path = dest_dir.join(".nuwa_history.db");
        let db = HistoryDb::open_or_create(&db_path).unwrap();
        assert_eq!(db.count().unwrap(), 3);

        let count2 = HistoryDb::rebuild_from_manifest(&dest_dir).unwrap();
        assert_eq!(count2, 3); // Rebuild deletes old DB and rescans

        drop(db);
        let _ = fs::remove_dir_all(&dest_dir);
    }

    #[test]
    fn test_rebuild_skips_corrupted_manifest() {
        let dest_dir = unique_test_dir();

        let valid_dir = dest_dir.join("20260705_100000_Valid");
        fs::create_dir_all(&valid_dir).unwrap();
        let manifest = serde_json::json!({
            "schema_version": "1.0",
            "backup_id": "valid-uuid-0001",
            "created_at": "2026-07-05T10:00:00+08:00",
            "source_root": "C:\\Users\\Test",
            "storage_format": "flat-file",
            "compression": { "enabled": false, "algorithm": null },
            "files": [],
            "directories": [],
            "summary": { "file_count": 10, "directory_count": 2, "total_bytes": 102400 }
        });
        fs::write(
            valid_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let corrupt_dir = dest_dir.join("20260705_100001_Corrupt");
        fs::create_dir_all(&corrupt_dir).unwrap();
        fs::write(corrupt_dir.join("manifest.json"), "This is not valid JSON").unwrap();

        let count = HistoryDb::rebuild_from_manifest(&dest_dir).unwrap();
        assert_eq!(count, 1);

        let _ = fs::remove_dir_all(&dest_dir);
    }

    #[test]
    fn test_delete_and_rebuild() {
        let dest_dir = unique_test_dir();

        let backup_dir = dest_dir.join("20260705_100000_Test");
        fs::create_dir_all(&backup_dir).unwrap();
        let manifest = serde_json::json!({
            "schema_version": "1.0",
            "backup_id": "rebuild-test-uuid-0001",
            "created_at": "2026-07-05T10:00:00+08:00",
            "source_root": "C:\\Users\\Test",
            "storage_format": "flat-file",
            "compression": { "enabled": false, "algorithm": null },
            "files": [],
            "directories": [],
            "summary": { "file_count": 10, "directory_count": 2, "total_bytes": 102400 }
        });
        fs::write(
            backup_dir.join("manifest.json"),
            serde_json::to_string_pretty(&manifest).unwrap(),
        )
        .unwrap();

        let count = HistoryDb::rebuild_from_manifest(&dest_dir).unwrap();
        assert_eq!(count, 1);

        let db_path = dest_dir.join(".nuwa_history.db");
        assert!(db_path.exists());
        fs::remove_file(&db_path).unwrap();

        let count2 = HistoryDb::rebuild_from_manifest(&dest_dir).unwrap();
        assert_eq!(count2, 1);

        let db = HistoryDb::open_or_create(&db_path).unwrap();
        assert_eq!(db.count().unwrap(), 1);

        drop(db);
        let _ = fs::remove_dir_all(&dest_dir);
    }
}
