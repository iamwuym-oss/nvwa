// ============================================================================
// sqlite_block_map.rs 鈥?SqliteBlockMap implementation
// ============================================================================
//
// SQLite-backed BlockMapEngine. Each BlockMap is a per-Backup-Instance
// SQLite database file stored at backup-instances/{point_id}/block-map.db.
//
// Schema:
//   CREATE TABLE block_map (
//       logical_offset INTEGER PRIMARY KEY,  -- UNIQUE by PRIMARY KEY
//       block_id       TEXT NOT NULL,          -- SHA-256 hex
//       raw_size       INTEGER NOT NULL        -- uncompressed size
//   );
//
// Key design:
// - PRIMARY KEY (logical_offset) ensures no duplicate offsets
// - No FOREIGN KEY 鈥?BlockMap is independent of catalog
// - WAL mode for crash safety (atomic inserts)

use crate::repository::block_map::engine::{BlockMapEngine, BlockMapEntry};
use crate::repository::block_store::block_id::BlockId;
use crate::repository::error::RepositoryError;
use rusqlite::Connection;
use std::path::PathBuf;
use std::str::FromStr;

/// SQLite-backed BlockMapEngine implementation.
///
/// Each instance is tied to one Restore Point's backup-instances/{point_id}/ directory.
pub struct SqliteBlockMap {
    conn: Connection,
    db_path: PathBuf,
}

impl SqliteBlockMap {
    /// Create or open a block-map.db at the given path.
    ///
    /// Creates the database file and table schema if they don't exist.
    /// Enables WAL mode for write performance and crash safety.
    pub fn open(db_path: PathBuf) -> Result<Self, RepositoryError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RepositoryError::io(parent.to_path_buf(), "Cannot create block-map directory", e)
            })?;
        }

        let conn = Connection::open(&db_path)?;

        // WAL mode for concurrent read safety
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch("PRAGMA busy_timeout=5000;")?;

        // Create schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS block_map (
                logical_offset INTEGER PRIMARY KEY,
                block_id       TEXT NOT NULL,
                raw_size       INTEGER NOT NULL
            );",
        )?;

        Ok(SqliteBlockMap { conn, db_path })
    }

    /// Insert multiple mappings in a single transaction for performance.
    /// This is the preferred method for bulk inserts (e.g., from ChunkEngine results).
    pub fn insert_batch(&mut self, entries: &[BlockMapEntry]) -> Result<(), RepositoryError> {
        let tx = self.conn.transaction()?;

        {
            let mut stmt = tx.prepare(
                "INSERT OR IGNORE INTO block_map (logical_offset, block_id, raw_size)
                 VALUES (?1, ?2, ?3)",
            )?;

            for entry in entries {
                stmt.execute(rusqlite::params![
                    entry.logical_offset,
                    entry.block_id.to_hex(),
                    entry.raw_size,
                ])?;
            }
        } // stmt dropped here, tx borrow released

        tx.commit()?;
        Ok(())
    }

    /// Return the path to the database file
    pub fn path(&self) -> &PathBuf {
        &self.db_path
    }
}

impl BlockMapEngine for SqliteBlockMap {
    fn insert_mapping(
        &mut self,
        logical_offset: u64,
        block_id: &BlockId,
        raw_size: u64,
    ) -> Result<(), RepositoryError> {
        self.conn.execute(
            "INSERT OR IGNORE INTO block_map (logical_offset, block_id, raw_size)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![logical_offset, block_id.to_hex(), raw_size],
        )?;
        Ok(())
    }

    fn get_block(&self, logical_offset: u64) -> Result<Option<BlockMapEntry>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT logical_offset, block_id, raw_size FROM block_map
             WHERE logical_offset = ?1",
        )?;

        let mut rows = stmt.query(rusqlite::params![logical_offset])?;
        match rows.next()? {
            Some(row) => {
                let offset: u64 = row.get(0)?;
                let hex: String = row.get(1)?;
                let size: u64 = row.get(2)?;
                let block_id =
                    BlockId::from_str(&hex).map_err(|e| RepositoryError::BlockMapCorrupted {
                        path: self.db_path.clone(),
                        detail: format!("Invalid block_id hex '{}': {}", hex, e),
                    })?;
                Ok(Some(BlockMapEntry {
                    logical_offset: offset,
                    block_id,
                    raw_size: size,
                }))
            }
            None => Ok(None),
        }
    }

    fn get_range(&self, start: u64, end: u64) -> Result<Vec<BlockMapEntry>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT logical_offset, block_id, raw_size FROM block_map
             WHERE logical_offset >= ?1 AND logical_offset < ?2
             ORDER BY logical_offset ASC",
        )?;

        let rows = stmt.query_map(rusqlite::params![start, end], |row| {
            let offset: u64 = row.get(0)?;
            let hex: String = row.get(1)?;
            let size: u64 = row.get(2)?;
            Ok((offset, hex, size))
        })?;

        let mut entries = Vec::new();
        for row in rows {
            let (offset, hex, size) = row?;
            let block_id =
                BlockId::from_str(&hex).map_err(|e| RepositoryError::BlockMapCorrupted {
                    path: self.db_path.clone(),
                    detail: format!("Invalid block_id hex '{}': {}", hex, e),
                })?;
            entries.push(BlockMapEntry {
                logical_offset: offset,
                block_id,
                raw_size: size,
            });
        }

        Ok(entries)
    }

    fn block_count(&self) -> Result<u64, RepositoryError> {
        let count: u64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM block_map", [], |row| row.get(0))?;
        Ok(count)
    }

    fn close(self: Box<Self>) -> Result<PathBuf, RepositoryError> {
        // Drop the connection, closing the database
        let path = self.db_path.clone();

        // Run integrity check before closing
        let integrity: String = self
            .conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if integrity != "ok" {
            return Err(RepositoryError::BlockMapCorrupted {
                path: path.clone(),
                detail: format!("Integrity check failed: {}", integrity),
            });
        }

        // WAL checkpoint to consolidate
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (SqliteBlockMap, TempDir) {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("block-map.db");
        let engine = SqliteBlockMap::open(db_path).unwrap();
        (engine, tmp)
    }

    fn make_id(data: &[u8]) -> BlockId {
        BlockId::from_raw_data(data)
    }

    #[test]
    fn test_insert_and_get() {
        let (mut engine, _tmp) = setup();
        let id = make_id(b"test data block");

        engine.insert_mapping(0, &id, 100).unwrap();
        let entry = engine.get_block(0).unwrap().unwrap();
        assert_eq!(entry.logical_offset, 0);
        assert_eq!(entry.block_id, id);
        assert_eq!(entry.raw_size, 100);
    }

    #[test]
    fn test_get_nonexistent() {
        let (engine, _tmp) = setup();
        let result = engine.get_block(9999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_range() {
        let (mut engine, _tmp) = setup();
        let id1 = make_id(b"block 1");
        let id2 = make_id(b"block 2");
        let id3 = make_id(b"block 3");

        engine.insert_mapping(0, &id1, 100).unwrap();
        engine.insert_mapping(100, &id2, 200).unwrap();
        engine.insert_mapping(300, &id3, 150).unwrap();

        let entries = engine.get_range(0, 300).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].logical_offset, 0);
        assert_eq!(entries[1].logical_offset, 100);
    }

    #[test]
    fn test_block_count() {
        let (mut engine, _tmp) = setup();
        assert_eq!(engine.block_count().unwrap(), 0);

        let id = make_id(b"some data");
        engine.insert_mapping(0, &id, 50).unwrap();
        engine.insert_mapping(100, &id, 50).unwrap();
        assert_eq!(engine.block_count().unwrap(), 2);
    }

    #[test]
    fn test_insert_batch() {
        let (mut engine, _tmp) = setup();
        let id1 = make_id(b"batch 1");
        let id2 = make_id(b"batch 2");

        let entries = vec![
            BlockMapEntry {
                logical_offset: 0,
                block_id: id1,
                raw_size: 100,
            },
            BlockMapEntry {
                logical_offset: 100,
                block_id: id2,
                raw_size: 200,
            },
            BlockMapEntry {
                logical_offset: 300,
                block_id: id1,
                raw_size: 100,
            },
        ];
        engine.insert_batch(&entries).unwrap();
        assert_eq!(engine.block_count().unwrap(), 3);
    }

    #[test]
    fn test_insert_duplicate_offset() {
        let (mut engine, _tmp) = setup();
        let id1 = make_id(b"first");
        let id2 = make_id(b"second");

        // Insert at same offset 鈥?second should be IGNORE'd
        engine.insert_mapping(0, &id1, 100).unwrap();
        engine.insert_mapping(0, &id2, 200).unwrap(); // Duplicate

        assert_eq!(engine.block_count().unwrap(), 1);
        let entry = engine.get_block(0).unwrap().unwrap();
        assert_eq!(entry.block_id, id1); // First write wins
    }

    #[test]
    fn test_close_integrity() {
        let (mut engine, tmp) = setup();
        let id = make_id(b"close test");
        engine.insert_mapping(0, &id, 50).unwrap();

        let path = Box::new(engine).close().unwrap();
        assert!(path.exists());
        // tmp is still alive here
        let _ = tmp;
    }

    #[test]
    fn test_concurrent_read_write() {
        // Verify that writes are visible to the same engine
        let (mut engine, _tmp) = setup();
        let id = make_id(b"concurrent test");

        engine.insert_mapping(42, &id, 128).unwrap();
        let entry = engine.get_block(42).unwrap().unwrap();
        assert_eq!(entry.block_id, id);
        assert_eq!(entry.raw_size, 128);
    }

    #[test]
    fn test_empty_range() {
        let (mut engine, _tmp) = setup();
        let id = make_id(b"data");
        engine.insert_mapping(1000, &id, 100).unwrap();

        // Query a range with no data
        let entries = engine.get_range(0, 500).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_large_offset_values() {
        let (mut engine, _tmp) = setup();
        let id = make_id(b"large offset");

        // Test with large offsets (TB-scale)
        let large_offset: u64 = 1_099_511_627_776; // 1TB
        engine.insert_mapping(large_offset, &id, 4096).unwrap();

        let entry = engine.get_block(large_offset).unwrap().unwrap();
        assert_eq!(entry.logical_offset, large_offset);
    }
}
