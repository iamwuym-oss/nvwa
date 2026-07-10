// ============================================================================
// sqlite_catalog.rs 鈥?SqliteCatalog implementation
// ============================================================================
//
// SQLite-backed CatalogEngine. Each Catalog is a per-Backup-Instance
// SQLite database file stored at backup-instances/{point_id}/catalog.db.
//
// Schema:
//   CREATE TABLE file_entries (
//       file_id  INTEGER PRIMARY KEY AUTOINCREMENT,
//       path     TEXT NOT NULL UNIQUE,
//       size     INTEGER NOT NULL,
//       modified TEXT NOT NULL
//   );
//
//   CREATE TABLE file_extents (
//       id              INTEGER PRIMARY KEY AUTOINCREMENT,
//       file_id         INTEGER NOT NULL,
//       logical_offset  INTEGER NOT NULL,
//       length          INTEGER NOT NULL,
//       FOREIGN KEY (file_id) REFERENCES file_entries(file_id)
//   );
//
// Key design:
// - file_entries.path is UNIQUE 鈥?each file appears once per Restore Point
// - file_extents uses file_id as FK for efficient joins
// - WAL mode for crash safety

use crate::repository::catalog::engine::{CatalogEngine, FileEntry, FileExtent};
use crate::repository::error::RepositoryError;
use rusqlite::Connection;
use std::path::PathBuf;

/// SQLite-backed CatalogEngine implementation.
///
/// Each instance is tied to one Restore Point's backup-instances/{point_id}/ directory.
pub struct SqliteCatalog {
    conn: Connection,
    db_path: PathBuf,
}

impl SqliteCatalog {
    /// Create or open a catalog.db at the given path.
    ///
    /// Creates the database file and table schema if they don't exist.
    /// Enables WAL mode for write performance and crash safety.
    pub fn open(db_path: PathBuf) -> Result<Self, RepositoryError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                RepositoryError::io(parent.to_path_buf(), "Cannot create catalog directory", e)
            })?;
        }

        let conn = Connection::open(&db_path)?;

        // WAL mode for crash safety
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch("PRAGMA busy_timeout=5000;")?;

        // Create schema
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS file_entries (
                file_id  INTEGER PRIMARY KEY AUTOINCREMENT,
                path     TEXT NOT NULL UNIQUE,
                size     INTEGER NOT NULL,
                modified TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS file_extents (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id         INTEGER NOT NULL,
                logical_offset  INTEGER NOT NULL,
                length          INTEGER NOT NULL,
                FOREIGN KEY (file_id) REFERENCES file_entries(file_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_file_path ON file_entries(path);
            CREATE INDEX IF NOT EXISTS idx_extent_file ON file_extents(file_id);",
        )?;

        Ok(SqliteCatalog { conn, db_path })
    }

    /// Return the path to the database file
    pub fn path(&self) -> &PathBuf {
        &self.db_path
    }
}

impl CatalogEngine for SqliteCatalog {
    fn add_file(
        &mut self,
        path: &str,
        size: u64,
        modified: &str,
        extents: Vec<FileExtent>,
    ) -> Result<(), RepositoryError> {
        let tx = self.conn.transaction()?;

        // Insert or replace file entry
        tx.execute(
            "INSERT OR REPLACE INTO file_entries (path, size, modified)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![path, size, modified],
        )?;

        // Get the file_id
        let file_id: i64 = tx.query_row(
            "SELECT file_id FROM file_entries WHERE path = ?1",
            [path],
            |row| row.get(0),
        )?;

        // Remove old extents (in case of REPLACE)
        tx.execute("DELETE FROM file_extents WHERE file_id = ?1", [file_id])?;

        // Insert new extents in a block scope so stmt is dropped before tx.commit
        {
            let mut stmt = tx.prepare(
                "INSERT INTO file_extents (file_id, logical_offset, length)
                 VALUES (?1, ?2, ?3)",
            )?;

            for extent in &extents {
                stmt.execute(rusqlite::params![
                    file_id,
                    extent.logical_offset,
                    extent.length,
                ])?;
            }
        } // stmt dropped here, tx borrow released

        tx.commit()?;
        Ok(())
    }

    fn get_file(&self, path: &str) -> Result<Option<FileEntry>, RepositoryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT file_id, path, size, modified FROM file_entries WHERE path = ?1")?;

        let mut rows = stmt.query(rusqlite::params![path])?;
        match rows.next()? {
            Some(row) => {
                let file_id: i64 = row.get(0)?;
                let file_path: String = row.get(1)?;
                let size: u64 = row.get(2)?;
                let modified: String = row.get(3)?;

                // Fetch extents for this file
                let mut extent_stmt = self.conn.prepare(
                    "SELECT logical_offset, length FROM file_extents
                     WHERE file_id = ?1 ORDER BY logical_offset ASC",
                )?;

                let extent_rows = extent_stmt.query_map([file_id], |row| {
                    let offset: u64 = row.get(0)?;
                    let length: u64 = row.get(1)?;
                    Ok(FileExtent {
                        logical_offset: offset,
                        length,
                    })
                })?;

                let mut extents = Vec::new();
                for extent in extent_rows {
                    extents.push(extent?);
                }

                Ok(Some(FileEntry {
                    path: file_path,
                    size,
                    modified,
                    extents,
                }))
            }
            None => Ok(None),
        }
    }

    fn list_files(&self) -> Result<Vec<String>, RepositoryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT path FROM file_entries ORDER BY path ASC")?;

        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut paths = Vec::new();
        for row in rows {
            paths.push(row?);
        }
        Ok(paths)
    }

    fn file_count(&self) -> Result<u64, RepositoryError> {
        let count: u64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM file_entries", [], |row| row.get(0))?;
        Ok(count)
    }

    fn close(self: Box<Self>) -> Result<PathBuf, RepositoryError> {
        let path = self.db_path.clone();

        // Run integrity check
        let integrity: String = self
            .conn
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if integrity != "ok" {
            return Err(RepositoryError::CatalogCorrupted {
                path: path.clone(),
                detail: format!("Integrity check failed: {}", integrity),
            });
        }

        // WAL checkpoint
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;

        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup() -> (SqliteCatalog, TempDir) {
        let tmp = TempDir::new().unwrap();
        let db_path = tmp.path().join("catalog.db");
        let catalog = SqliteCatalog::open(db_path).unwrap();
        (catalog, tmp)
    }

    #[test]
    fn test_add_and_get_file() {
        let (mut catalog, _tmp) = setup();

        let extents = vec![
            FileExtent {
                logical_offset: 0,
                length: 100,
            },
            FileExtent {
                logical_offset: 100,
                length: 200,
            },
        ];

        catalog
            .add_file(
                "test/file.txt",
                300,
                "2026-07-10T10:00:00Z",
                extents.clone(),
            )
            .unwrap();

        let entry = catalog.get_file("test/file.txt").unwrap().unwrap();
        assert_eq!(entry.path, "test/file.txt");
        assert_eq!(entry.size, 300);
        assert_eq!(entry.extents, extents);
    }

    #[test]
    fn test_get_nonexistent_file() {
        let (catalog, _tmp) = setup();
        let result = catalog.get_file("nonexistent.txt").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_files() {
        let (mut catalog, _tmp) = setup();

        catalog
            .add_file("b.txt", 100, "2026-01-01T00:00:00Z", vec![])
            .unwrap();
        catalog
            .add_file("a.txt", 200, "2026-01-01T00:00:00Z", vec![])
            .unwrap();
        catalog
            .add_file("c.txt", 300, "2026-01-01T00:00:00Z", vec![])
            .unwrap();

        let files = catalog.list_files().unwrap();
        assert_eq!(files, vec!["a.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn test_file_count() {
        let (mut catalog, _tmp) = setup();
        assert_eq!(catalog.file_count().unwrap(), 0);

        catalog
            .add_file("f1.txt", 100, "2026-01-01T00:00:00Z", vec![])
            .unwrap();
        catalog
            .add_file("f2.txt", 200, "2026-01-01T00:00:00Z", vec![])
            .unwrap();
        assert_eq!(catalog.file_count().unwrap(), 2);
    }

    #[test]
    fn test_replace_file() {
        let (mut catalog, _tmp) = setup();

        // Add file with initial extents
        catalog
            .add_file(
                "replace.txt",
                100,
                "2026-01-01T00:00:00Z",
                vec![FileExtent {
                    logical_offset: 0,
                    length: 100,
                }],
            )
            .unwrap();

        // Replace with new extents
        catalog
            .add_file(
                "replace.txt",
                200,
                "2026-01-02T00:00:00Z",
                vec![FileExtent {
                    logical_offset: 0,
                    length: 200,
                }],
            )
            .unwrap();

        let entry = catalog.get_file("replace.txt").unwrap().unwrap();
        assert_eq!(entry.size, 200);
        assert_eq!(entry.extents.len(), 1);
        assert_eq!(entry.extents[0].length, 200);
        assert_eq!(catalog.file_count().unwrap(), 1); // Still one file
    }

    #[test]
    fn test_file_with_no_extents() {
        let (mut catalog, _tmp) = setup();
        catalog
            .add_file("empty.txt", 0, "2026-01-01T00:00:00Z", vec![])
            .unwrap();

        let entry = catalog.get_file("empty.txt").unwrap().unwrap();
        assert_eq!(entry.size, 0);
        assert!(entry.extents.is_empty());
    }

    #[test]
    fn test_close_integrity() {
        let (mut catalog, tmp) = setup();
        catalog
            .add_file(
                "close.txt",
                50,
                "2026-01-01T00:00:00Z",
                vec![FileExtent {
                    logical_offset: 0,
                    length: 50,
                }],
            )
            .unwrap();

        let path = Box::new(catalog).close().unwrap();
        assert!(path.exists());
        let _ = tmp;
    }

    #[test]
    fn test_multiple_files_with_extents() {
        let (mut catalog, _tmp) = setup();

        catalog
            .add_file(
                "dir/a.txt",
                150,
                "2026-01-01T00:00:00Z",
                vec![
                    FileExtent {
                        logical_offset: 0,
                        length: 100,
                    },
                    FileExtent {
                        logical_offset: 100,
                        length: 50,
                    },
                ],
            )
            .unwrap();

        catalog
            .add_file(
                "dir/b.txt",
                75,
                "2026-01-01T00:00:00Z",
                vec![FileExtent {
                    logical_offset: 150,
                    length: 75,
                }],
            )
            .unwrap();

        assert_eq!(catalog.file_count().unwrap(), 2);

        let a = catalog.get_file("dir/a.txt").unwrap().unwrap();
        assert_eq!(a.extents.len(), 2);

        let b = catalog.get_file("dir/b.txt").unwrap().unwrap();
        assert_eq!(b.extents.len(), 1);
    }
}
