// ============================================================================
// sqlite_catalog.rs — SqliteCatalog implementation (P-00C: entry_type, sha256, file_offset)
// ============================================================================
//
// P-00C changes per P-00_File_Backup_Repository_Data_Contract.md v0.7:
// - file_entries: added entry_type TEXT, sha256 TEXT
// - file_extents: added file_offset INTEGER NOT NULL
// - add_file() now accepts sha256: Option<String>
// - add_directory() for recording empty directories
// - get_file() returns entry_type and sha256 fields
//
// Schema additions (via ALTER TABLE migration):
//   ALTER TABLE file_entries ADD COLUMN entry_type TEXT NOT NULL DEFAULT 'file';
//   ALTER TABLE file_entries ADD COLUMN sha256 TEXT;
//   ALTER TABLE file_extents ADD COLUMN file_offset INTEGER NOT NULL DEFAULT 0;
//
// Key design:
// - file_entries.path is UNIQUE — each file appears once per Restore Point
// - file_extents uses file_id as FK for efficient joins
// - WAL mode for crash safety
// - ALTER TABLE migrations for existing databases (additive, backward-compatible)

use crate::repository::catalog::engine::{CatalogEngine, CatalogEntryType, FileEntry, FileExtent};
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
    /// Applies additive migrations for existing databases.
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

        // Create base schema (IF NOT EXISTS for existing databases)
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS file_entries (
                file_id     INTEGER PRIMARY KEY AUTOINCREMENT,
                path        TEXT NOT NULL UNIQUE,
                size        INTEGER NOT NULL,
                modified    TEXT NOT NULL,
                entry_type  TEXT NOT NULL DEFAULT 'file',
                sha256      TEXT
            );

            CREATE TABLE IF NOT EXISTS file_extents (
                id             INTEGER PRIMARY KEY AUTOINCREMENT,
                file_id         INTEGER NOT NULL,
                logical_offset  INTEGER NOT NULL,
                length          INTEGER NOT NULL,
                file_offset     INTEGER NOT NULL DEFAULT 0,
                FOREIGN KEY (file_id) REFERENCES file_entries(file_id) ON DELETE CASCADE
            );

            CREATE INDEX IF NOT EXISTS idx_file_path ON file_entries(path);
            CREATE INDEX IF NOT EXISTS idx_extent_file ON file_extents(file_id);",
        )?;

        // P-00C migration: add entry_type column if missing
        let has_entry_type: bool = conn
            .prepare("SELECT entry_type FROM file_entries LIMIT 0")
            .is_ok();
        if !has_entry_type {
            conn.execute_batch(
                "ALTER TABLE file_entries ADD COLUMN entry_type TEXT NOT NULL DEFAULT 'file';",
            )?;
        }

        // P-00C migration: add sha256 column if missing
        let has_sha256: bool = conn
            .prepare("SELECT sha256 FROM file_entries LIMIT 0")
            .is_ok();
        if !has_sha256 {
            conn.execute_batch("ALTER TABLE file_entries ADD COLUMN sha256 TEXT;")?;
        }

        // P-00C migration: add file_offset column to file_extents if missing
        let has_file_offset: bool = conn
            .prepare("SELECT file_offset FROM file_extents LIMIT 0")
            .is_ok();
        if !has_file_offset {
            conn.execute_batch(
                "ALTER TABLE file_extents ADD COLUMN file_offset INTEGER NOT NULL DEFAULT 0;",
            )?;
        }

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
        sha256: Option<String>,
        extents: Vec<FileExtent>,
    ) -> Result<(), RepositoryError> {
        let tx = self.conn.transaction()?;

        // Insert or replace file entry
        tx.execute(
            "INSERT OR REPLACE INTO file_entries (path, entry_type, size, modified, sha256)
             VALUES (?1, 'file', ?2, ?3, ?4)",
            rusqlite::params![path, size, modified, sha256],
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
                "INSERT INTO file_extents (file_id, file_offset, logical_offset, length)
                 VALUES (?1, ?2, ?3, ?4)",
            )?;

            for extent in &extents {
                stmt.execute(rusqlite::params![
                    file_id,
                    extent.file_offset,
                    extent.logical_offset,
                    extent.length,
                ])?;
            }
        } // stmt dropped here, tx borrow released

        tx.commit()?;
        Ok(())
    }

    fn add_directory(&mut self, path: &str, modified: &str) -> Result<(), RepositoryError> {
        let tx = self.conn.transaction()?;

        // Insert or replace directory entry
        // P-00 §4.5: Directories have size=0, sha256=None, extents=[]
        tx.execute(
            "INSERT OR REPLACE INTO file_entries (path, entry_type, size, modified, sha256)
             VALUES (?1, 'directory', 0, ?2, NULL)",
            rusqlite::params![path, modified],
        )?;

        tx.commit()?;
        Ok(())
    }

    fn get_file(&self, path: &str) -> Result<Option<FileEntry>, RepositoryError> {
        let mut stmt = self.conn.prepare(
            "SELECT file_id, path, entry_type, size, modified, sha256
                 FROM file_entries WHERE path = ?1",
        )?;

        let file_result = stmt.query_row([path], |row| {
            let file_id: i64 = row.get(0)?;
            let path: String = row.get(1)?;
            let entry_type_str: String = row.get(2)?;
            let size: u64 = row.get::<_, i64>(3)? as u64;
            let modified: String = row.get(4)?;
            let sha256: Option<String> = row.get(5)?;

            Ok((file_id, path, entry_type_str, size, modified, sha256))
        });

        match file_result {
            Ok((file_id, path, entry_type_str, size, modified, sha256)) => {
                let entry_type = match entry_type_str.as_str() {
                    "file" => CatalogEntryType::File,
                    "directory" => CatalogEntryType::Directory,
                    other => {
                        return Err(RepositoryError::General {
                            detail: format!("Unknown catalog entry_type: '{}'", other),
                        });
                    }
                };
                // Fetch extents for this file
                let mut ext_stmt = self.conn.prepare(
                    "SELECT file_offset, logical_offset, length
                     FROM file_extents
                     WHERE file_id = ?1
                     ORDER BY file_offset ASC",
                )?;

                let extents: Vec<FileExtent> = ext_stmt
                    .query_map([file_id], |row| {
                        let file_offset: u64 = row.get::<_, i64>(0)? as u64;
                        let logical_offset: u64 = row.get::<_, i64>(1)? as u64;
                        let length: u64 = row.get::<_, i64>(2)? as u64;
                        Ok(FileExtent {
                            file_offset,
                            logical_offset,
                            length,
                        })
                    })?
                    .collect::<Result<Vec<_>, _>>()?;

                Ok(Some(FileEntry {
                    entry_type,
                    path,
                    size,
                    modified,
                    extents,
                    sha256,
                }))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn list_files(&self) -> Result<Vec<String>, RepositoryError> {
        let mut stmt = self
            .conn
            .prepare("SELECT path FROM file_entries ORDER BY path ASC")?;

        let paths: Vec<String> = stmt
            .query_map([], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(paths)
    }

    fn file_count(&self) -> Result<u64, RepositoryError> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM file_entries", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    fn close(self: Box<Self>) -> Result<PathBuf, RepositoryError> {
        // WAL checkpoint before closing - ensures metadata SHA-256 matches
        // on-disk .db content (not just WAL)
        self.conn
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|e| RepositoryError::General {
                detail: format!("Failed to checkpoint catalog database: {}", e),
            })?;
        self.conn
            .close()
            .map_err(|(_conn, e)| RepositoryError::General {
                detail: format!("Failed to close catalog database: {}", e),
            })?;
        Ok(self.db_path)
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
                file_offset: 0,
                logical_offset: 0,
                length: 100,
            },
            FileExtent {
                file_offset: 100,
                logical_offset: 100,
                length: 200,
            },
        ];

        catalog
            .add_file(
                "test/file.txt",
                300,
                "2026-07-10T10:00:00Z",
                Some(
                    "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
                ),
                extents.clone(),
            )
            .unwrap();

        let entry = catalog.get_file("test/file.txt").unwrap().unwrap();
        assert_eq!(entry.entry_type, CatalogEntryType::File);
        assert_eq!(entry.path, "test/file.txt");
        assert_eq!(entry.size, 300);
        assert_eq!(
            entry.sha256,
            Some("abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string())
        );
        assert_eq!(entry.extents, extents);
    }

    #[test]
    fn test_add_directory() {
        let (mut catalog, _tmp) = setup();

        catalog
            .add_directory("empty-dir", "2026-07-10T10:00:00Z")
            .unwrap();

        let entry = catalog.get_file("empty-dir").unwrap().unwrap();
        assert_eq!(entry.entry_type, CatalogEntryType::Directory);
        assert_eq!(entry.size, 0);
        assert_eq!(entry.sha256, None);
        assert!(entry.extents.is_empty());
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
            .add_file(
                "b.txt",
                100,
                "2026-01-01T00:00:00Z",
                Some("b_hash".to_string()),
                vec![],
            )
            .unwrap();
        catalog
            .add_file(
                "a.txt",
                200,
                "2026-01-01T00:00:00Z",
                Some("a_hash".to_string()),
                vec![],
            )
            .unwrap();
        catalog
            .add_file(
                "c.txt",
                300,
                "2026-01-01T00:00:00Z",
                Some("c_hash".to_string()),
                vec![],
            )
            .unwrap();

        let files = catalog.list_files().unwrap();
        assert_eq!(files, vec!["a.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn test_file_count() {
        let (mut catalog, _tmp) = setup();
        assert_eq!(catalog.file_count().unwrap(), 0);

        catalog
            .add_file(
                "f1.txt",
                100,
                "2026-01-01T00:00:00Z",
                Some("h1".to_string()),
                vec![],
            )
            .unwrap();
        catalog
            .add_file(
                "f2.txt",
                200,
                "2026-01-01T00:00:00Z",
                Some("h2".to_string()),
                vec![],
            )
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
                Some("old_hash".to_string()),
                vec![FileExtent {
                    file_offset: 0,
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
                Some("new_hash".to_string()),
                vec![FileExtent {
                    file_offset: 0,
                    logical_offset: 0,
                    length: 200,
                }],
            )
            .unwrap();

        let entry = catalog.get_file("replace.txt").unwrap().unwrap();
        assert_eq!(entry.size, 200);
        assert_eq!(entry.sha256, Some("new_hash".to_string()));
        assert_eq!(entry.extents.len(), 1);
        assert_eq!(entry.extents[0].length, 200);
        assert_eq!(catalog.file_count().unwrap(), 1); // Still one file
    }

    #[test]
    fn test_file_with_no_extents() {
        let (mut catalog, _tmp) = setup();
        catalog
            .add_file(
                "empty.txt",
                0,
                "2026-01-01T00:00:00Z",
                Some("empty_hash".to_string()),
                vec![],
            )
            .unwrap();

        let entry = catalog.get_file("empty.txt").unwrap().unwrap();
        assert_eq!(entry.size, 0);
        assert_eq!(entry.sha256, Some("empty_hash".to_string()));
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
                Some("close_hash".to_string()),
                vec![FileExtent {
                    file_offset: 0,
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
                Some("a_hash".to_string()),
                vec![
                    FileExtent {
                        file_offset: 0,
                        logical_offset: 0,
                        length: 100,
                    },
                    FileExtent {
                        file_offset: 100,
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
                Some("b_hash".to_string()),
                vec![FileExtent {
                    file_offset: 0,
                    logical_offset: 150,
                    length: 75,
                }],
            )
            .unwrap();

        assert_eq!(catalog.file_count().unwrap(), 2);

        let a = catalog.get_file("dir/a.txt").unwrap().unwrap();
        assert_eq!(a.entry_type, CatalogEntryType::File);
        assert_eq!(a.extents.len(), 2);
        assert_eq!(a.extents[0].file_offset, 0);
        assert_eq!(a.extents[1].file_offset, 100);

        let b = catalog.get_file("dir/b.txt").unwrap().unwrap();
        assert_eq!(b.extents.len(), 1);
        assert_eq!(b.extents[0].file_offset, 0);
    }

    #[test]
    fn test_directory_counted_separately() {
        let (mut catalog, _tmp) = setup();

        catalog
            .add_directory("dir1", "2026-01-01T00:00:00Z")
            .unwrap();
        catalog
            .add_directory("dir2", "2026-01-01T00:00:00Z")
            .unwrap();
        catalog
            .add_file(
                "f1.txt",
                10,
                "2026-01-01T00:00:00Z",
                Some("h".to_string()),
                vec![],
            )
            .unwrap();

        assert_eq!(catalog.file_count().unwrap(), 3);
        assert_eq!(catalog.list_files().unwrap().len(), 3);
    }
}
