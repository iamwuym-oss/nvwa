# P-00 — File Backup Repository Data Contract

**Product:** Nüwa Backup (女娲备份)
**Date:** 2026-07-11
**Status:** DRAFT — awaiting review before coding
**Governs:** File backup integration with Phase S Repository Engine
**Relation to Phase S:** Defines the data contract between file backup pipeline (FileCollector) and Repository Engine. Phase S architecture v1.1 remains authoritative; this document specifies the file backup mapping within that architecture.

---

## Table of Contents

1. Data Model Mapping
2. Storage Layout
3. Restore Point State Machine (Dual: Journal + repo.db)
4. Controlled Modifications to Phase S Frozen Types
5. P-00C — Crash Recovery Commit Safety Fix
6. Asset Identity Model
7. History Model
8. First Version Scope
9. Unsupported File System Semantics
10. Gate Definitions
11. References

---

## 1. Data Model Mapping

### 1.1 Entity Hierarchy

```
BackupJob (user config)
  +-- source_path: Path          (what to back up)
  +-- repository_id: String      (where to store -- references RepoRegistry)
  +-- asset_id: String           (stable identity of this data source)
  +-- asset_type: "file"         (fixed for file backup)

Run (one execution)
  +-- RestorePoint
        +-- point_id: String     (UUID v4, unique per run)
        +-- chain_id: String     (= point_id for Full; groups Inc points)
        +-- chain_position: 0    (Full only in v1)
        +-- parent_point_id: NULL
        +-- status: PointStatus  (CREATING -> WRITING -> VERIFYING -> COMMITTED | FAILED)
        +-- instance_path: Path  (backup-instances/{point_id}/)

RestorePoint
  +-- BackupInstance (1:1 with RestorePoint in v1)
        +-- instance_id: String        (= point_id, explicit v1 constraint -- NOT accidental reuse)
        +-- restore_point_id: String   (= instance_id)
        +-- job_id: String
        +-- source_type: "file"
        +-- source_description: String (source_path display string)
        +-- asset_id: String            (same as Job asset_id)
        +-- asset_type: "file"
        +-- 1 SqliteCatalog (catalog.db)
        |     +-- per-file entries with extents
        +-- 1 SqliteBlockMap (block-map.db)
        |     +-- per-logical-offset mappings to block_id
        +-- N Blocks in block-store/
        +-- backup-metadata.json (integrity manifests + summary)
```

### 1.2 Rationale for 1:1 RestorePoint:BackupInstance

In v1, each backup run processes exactly one source (a single directory tree). Each run produces one set of blocks, one catalog, and one block map. This simplifies crash recovery and makes the RestorePoint-to-BackupInstance mapping unambiguous.

Future incremental backup or multi-source jobs may introduce 1:N mapping, but v1 is strictly 1:1.

### 1.3 Job Config Changes (from current config.rs)

Current:
```rust
pub struct JobConfig {
    pub source: PathBuf,
    pub dest: PathBuf,              // DELETE (flat-file legacy)
    pub compress: bool,
    pub retention: Option<RetentionPolicy>,
    pub schedule_id: Option<String>,
    pub storage_type: Option<String>,  // DELETE (replaced by repository_id)
    pub repository_id: Option<String>,
}
```

New:
```rust
pub struct JobConfig {
    pub source: PathBuf,             // what to back up
    pub repository_id: String,       // where to store (lookup in RepoRegistry)
    pub asset_id: String,            // stable source identity
    pub asset_type: String,          // "file"
    pub compress: bool,              // default: true
    pub retention: Option<RetentionPolicy>,
    pub schedule_id: Option<String>,
}
```

Deleted: `dest`, `storage_type`.
Migration: storage_type removed entirely (not default-changed); dest replaced by repository_id; asset_id added as required.

---

### 1.4 Catalog Path Security Contract

The `path` field in every Catalog entry (both File and Directory entry types) is subject to the following mandatory rules at BOTH write time (backup pipeline inserts into Catalog) AND restore time (before any file/directory is written to disk).

#### Write-time rules (before insertion into Catalog):
- `path` MUST be a relative path from the backup source root.
- Absolute paths, Windows drive letters (`C:\...`), UNC roots (`\\server\share`), and URL-form paths are STRICTLY FORBIDDEN.
- Path segments containing `.` (current directory) or `..` (parent directory) are STRICTLY FORBIDDEN. If any segment equals `".."`, the entire entry MUST be rejected and the backup MUST fail fast.
- The path MUST be normalized before insertion: all separators unified to forward slash (`/`), consecutive separators collapsed, trailing separator removed.
- The normalized path when joined with the backup source root MUST NOT resolve outside the source root after canonicalization.

#### Restore-time rules (before writing any file or directory):

The restore target path is constructed by joining the Catalog relative path with the restore destination root. Since the target file typically does NOT exist yet at restore time, `canonicalize()` cannot be called on the full target path. The following rules account for this:

1. **Canonicalize the restore root first.** Ensure the restore destination root exists and obtain its canonical path via `canonicalize()`. This establishes the trusted boundary. If the restore root itself is a symbolic link, junction, or reparse point, the restore MUST fail immediately.

2. **Semantic validation of the Catalog relative path.** The path MUST pass all of the following, or the restore MUST FAIL immediately:
   - Not absolute (must not start with `/`, `\`, or a Windows drive letter).
   - No path segment equals `.` or `..`.
   - Not a Windows root (`C:\`), UNC root (`\\host\share\`), or URL form.
   - After normalization (unified separators, collapsed consecutive separators, no trailing separator), no `.` or `..` fragments remain.

3. **Construct the target path.** Use `Path::join` to combine the verified relative path with the canonical restore root. The resulting path is the candidate target. At this point the target file or directory may or may not exist; existence is NOT required.

4. **Component-by-component directory walk with symlink/junction/reparse point guard.** Do NOT use `create_dir_all` on the full derived path (it follows symlinks during the creation process, potentially creating directories outside the restore root before detection).
   - Split the Catalog relative path into individual components.
   - Determine which components are directories based on `entry_type`:
     * For `File` entries: all components EXCEPT the last one are parent directories. The last component is the filename and MUST NOT be treated as a directory.
     * For `Directory` entries: ALL components are directories. Every component must be created as a directory.
   - Walk from the canonical restore root, one component at a time, considering only the directory components:
     a. Use `symlink_metadata` (not `metadata`, not `canonicalize`) on the current accumulated path. `symlink_metadata` reads the on-disk entry without following links.
     b. If the entry is a symbolic link, junction, or any reparse point -> RESTORE MUST FAIL IMMEDIATELY. Do NOT check whether the link target resolves inside the restore root. ANY reparse point on the restore path is unconditionally rejected.
     c. If the entry does not exist, create the directory explicitly via `std::fs::create_dir` (NOT `create_dir_all`).
     d. After creation, call `canonicalize()` on this directory. Verify the canonical result still starts_with the canonical restore root.
   - This walk ensures every ancestor directory is a real, verified-on-disk directory before any file is written. The final filename component (for File entries) is never created as a directory.

5. **Final check before writing.** After all ancestor directories pass the walk, write the restored file or directory to the target path. The target path itself is NOT canonicalized -- it is a constructed path from trusted components.

6. **Atomic write pattern for files.** Write file content to a `.tmp` file adjacent to the final target, then atomically rename (using `std::fs::rename`) to the target name. This prevents partial writes from producing a corrupted file at the final path.

7. **Denied patterns (MUST cause immediate FAILURE):**
   - Catalog path with absolute, `..`, or root prefix (caught by step 2).
   - Any ancestor is a symlink/junction/reparse point (caught by step 4b).
   - Any ancestor canonicalizes to a path outside the restore root (caught by step 4d).
   - The target path, after construction and parent-directory traversal, equals the restore root itself (prevents root overwrite).

#### Rationale:
Path traversal through backup metadata is a known attack vector in backup/restore software. A corrupt, malformed, or malicious Catalog could otherwise write files outside the intended restore boundary. The component-by-component walk detects symlink-based escapes at the exact point of traversal -- before any child directories are created outside the trusted boundary. The atomic write pattern ensures that even if the write is interrupted, no corrupted file remains at the final target path.

#### Implementation notes:
- Use `fs::symlink_metadata` on Windows to detect reparse points without following them, then inspect the reparse tag to identify junctions (`IO_REPARSE_TAG_MOUNT_POINT`) and symlinks (`IO_REPARSE_TAG_SYMLINK`).
- Normalize UNC long-path prefixes (`\\?\`) and 8.3 short names before comparison.
- The Catalog MUST NOT store any path that could be used directly as an absolute filesystem path without joining with a restore root.
- For the component-by-component walk, use `Path::components()` to decompose the relative path and `Path::join()` to build each accumulated ancestor.

## 2. Storage Layout

### 2.1 Directory Structure (Repository Engine Phase S, unchanged)

```
{repo_root}/
  .nuwarepo/
    repo.db                  -- Repository metadata + restore_points table
    repository.json          -- Identity + capabilities manifest
    transactions/            -- Crash consistency journals
      txn-{point_id}.log
  block-store/
    {prefix1}/{prefix2}/{hash}.block   -- Content-addressed blocks
  backup-instances/
    {point_id}/
      catalog.db             -- SqliteCatalog (per-instance)
      block-map.db           -- SqliteBlockMap (per-instance)
      backup-metadata.json   -- Instance metadata + integrity manifests
```

### 2.2 repo.db Schema (existing, unchanged)

```sql
CREATE TABLE restore_points (
    point_id        TEXT PRIMARY KEY,
    job_id          TEXT NOT NULL,
    chain_id        TEXT NOT NULL,
    chain_position  INTEGER NOT NULL,
    created_at      TEXT NOT NULL,
    status          TEXT NOT NULL,   -- PointStatus as string
    instance_path   TEXT NOT NULL,   -- "backup-instances/{point_id}/"
    block_count     INTEGER NOT NULL DEFAULT 0,
    total_raw_bytes INTEGER NOT NULL DEFAULT 0,
    parent_point_id TEXT,
    FOREIGN KEY (job_id) REFERENCES backup_jobs(job_id)
);
```

### 2.3 backup-instances/{point_id}/ Directory

Created at the start of each backup run (before any data is written). Contains:
- catalog.db -- owned by SqliteCatalog
- block-map.db -- owned by SqliteBlockMap
- backup-metadata.json -- written by metadata_store::write_metadata()

### 2.4 Journal File (.nuwarepo/transactions/txn-{point_id}.log)

Managed by CrashConsistencyManager. Contains:
- point_id
- TransactionState (CREATING / WRITING / VERIFYING / COMMITTED / FAILED)
- ComponentStatus for: block_store, block_map, catalog, metadata

---

## 3. Restore Point State Machine (Dual States)

### 3.1 Two Independent State Machines

The system maintains two state machines with the same enum values but DIFFERENT update timing:

| Entity | Location | Created at | Source of truth for |
|--------|----------|-----------|---------------------|
| Transaction Journal | transactions/txn-{id}.log | Step 2 | Crash recovery entry point |
| repo.db restore_points | repo.db table row | Step 1 | Restore point visibility to user |

### 3.2 Canonical State Sequence

```
Step | repo.db restore_points.status | Transaction Journal | Action
-----|------------------------------|--------------------|-------
  1  | CREATING                     | (not created)      | INSERT restore_points, create instance dir
  2  | WRITING                      | CREATING           | CrashConsistencyManager::begin()
  3  | WRITING                      | WRITING            | Write blocks/block_map/catalog/metadata
  4  | WRITING                      | WRITING            | Components completed one by one
  5  | VERIFYING                    | VERIFYING          | enter_verify()
  6  | VERIFYING -> COMMITTED       | (still exists)     | UPDATE repo.db SET status=COMMITTED (SQL txn)
  7  | COMMITTED                    | (updating)         | Write journal with state=COMMITTED
  8  | COMMITTED                    | (confirm)          | Verify repo.db status is COMMITTED
  9  | COMMITTED                    | (deleted)          | Remove journal file

FAILURE PATH (any step 1-5):
  X  | FAILED                       | FAILED             | UPDATE repo.db + journal.fail()

> **Step 1 crash note:** If the crash occurs after the repo.db INSERT but before journal creation (Step 1 boundary), no journal file exists. In this case, startup recovery still detects the CREATING entry via repo.db scan and marks it FAILED. The FAILED action targets whichever state records exist. See Section 3.3 recovery table (missing) | CREATING/WRITING/VERIFYING row.
```

### 3.3 Crash Recovery Strategy (post P-00C fix)

| Journal state | repo.db state | Recovery action |
|--------------|--------------|-----------------|
| COMMITTED | COMMITTED | Clean up journal (normal completion) |
| (exists) | COMMITTED | Clean up journal only. Interrupted after repo.db commit but before journal cleanup. This is safe because repo.db is the terminal truth. |
| COMMITTED | anything != COMMITTED | STATE MISMATCH. Flag as error, not user-visible. Requires human investigation. This combination violates the commit order contract. |
| CREATING | CREATING/WRITING | Mark repo.db FAILED (nothing verified) |
| WRITING | WRITING | Mark repo.db FAILED (unverified data) |
| VERIFYING | VERIFYING | Mark repo.db FAILED (verification incomplete) |
| FAILED | FAILED | Keep for orphan tracking (no action) |
| (missing) | CREATING/WRITING/VERIFYING | Orphan: mark FAILED (journal lost) |

**Key rules:**
- Only COMMITTED in repo.db is the terminal success state.
- Non-terminal states (CREATING, WRITING, VERIFYING) always resolve to FAILED upon recovery, regardless of component completion flags.
- **Recovery must scan BOTH sources: journals AND repo.db non-terminal restore_points.** A restore point with CREATING/WRITING/VERIFYING status in repo.db but NO journal file is still an orphan and must be marked FAILED.
- The absence of a journal is NOT proof of a clean state. repo.db non-terminal entries without journals indicate a crash before journal creation.

---

## 4. Controlled Modifications to Phase S Frozen Types

The following changes are necessary due to real integration gaps discovered during TP-00 audit. Each is a minimal, backward-compatible extension.

### 4.1 CatalogEngine::FileEntry -- Add sha256

Current:
```rust
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub modified: String,
    pub extents: Vec<FileExtent>,
}
```

New:
```rust
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub modified: String,
    pub sha256: Option<String>,    // NEW: file-level SHA-256 for restore-time verification. v1 new file writes REQUIRE Some(valid_sha256). None only for backward-compatible catalog opens.
    pub extents: Vec<FileExtent>,
}
```

Rationale: Without stored expected_sha256, restore-time SHA-256 computation has no comparison target. Block-level verification alone cannot detect extent ordering errors or data assembly bugs.

Schema change (SqliteCatalog):
```sql
-- Current:
CREATE TABLE file_entries (
    file_id  INTEGER PRIMARY KEY AUTOINCREMENT,
    path     TEXT NOT NULL UNIQUE,
    size     INTEGER NOT NULL,
    modified TEXT NOT NULL
);

-- New:
CREATE TABLE file_entries (
    file_id  INTEGER PRIMARY KEY AUTOINCREMENT,
    path     TEXT NOT NULL UNIQUE,
    size     INTEGER NOT NULL,
    modified TEXT NOT NULL,
    sha256   TEXT              -- NULL allowed (backward compatible)
);
```

Trait change:
```rust
// Current:
fn add_file(&mut self, path: &str, size: u64, modified: &str, extents: Vec<FileExtent>)
    -> Result<(), RepositoryError>;

// New:
fn add_file(&mut self, path: &str, size: u64, modified: &str,
             sha256: Option<&str>, extents: Vec<FileExtent>)
    -> Result<(), RepositoryError>;
```
v1 requirement: For ALL new v1 file writes, sha256 MUST be Some(valid_sha256). Empty files must store sha256 of empty content (e3b0c442...). None is only valid when opening an older catalog that predates this field.
Backward compatibility: sha256 defaults to None. SqliteCatalog writes NULL for None. Existing tests pass None.

### 4.2 FileExtent -- Add file_offset

Current:
```rust
pub struct FileExtent {
    pub logical_offset: u64,   // position in Backup Instance logical address space
    pub length: u64,
}
```

New:
```rust
pub struct FileExtent {
    pub file_offset: u64,       // byte offset within the file (NEW, required)
    pub logical_offset: u64,    // position in Backup Instance logical address space
    pub length: u64,            // number of bytes
}
```

Rationale: Explicit file_offset enables independent verification that:
- Extents within a file are contiguous (no gaps), covering [0, file_size)
- No extents overlap within a file
- The last extent of each file ends at exactly the file size
- Extent order in the catalog matches file_offset order
- This prevents silent corruption from extent ordering bugs

Constraints (v1):
- file_offset + length must not exceed file_size for each file
- Extents within a file must be non-overlapping and ordered by file_offset
- Extents must cover the full range [0, file_size) for non-empty files
- Empty files have zero extents (extents = [])
- Holes (sparse files) are NOT supported in v1 -- any byte range [0, file_size)
  must be covered by exactly one extent

### 4.3 logical_offset Semantics

- logical_offset is the position in the Backup Instance global logical address space
- It does NOT reset to 0 for each file
- It is the key into SqliteBlockMap (BlockMapEngine.get_range())
- Each file data occupies a contiguous range in this address space
- The mapping from file_offset to logical_offset is: logical_offset = base + file_offset
  where base is the cumulative size of all previously written files in this Backup Instance

### 4.4 Summary of Changes

| Type | File | Change | Risk | Test Impact |
|------|------|--------|------|-------------|
| Struct | catalog/engine.rs: FileEntry | +sha256: Option<String> | Low (optional field; v1 new writes REQUIRE Some) | None existing break; new tests for Some validation needed |
| Struct | catalog/engine.rs: FileExtent | +file_offset: u64 | Low (new required field) | All existing FileExtent constructors must be updated |
| Trait | catalog/engine.rs: CatalogEngine | add_file signature: +sha256: Option<&str> | Medium (trait change) | All implementations (SqliteCatalog) must update |
| Schema | catalog/sqlite_catalog.rs | file_entries +sha256 TEXT | Low (NULL-compatible) | Existing catalog.db unaffected (NULL) |
| Tests | sqlite_catalog.rs | 7 tests need file_offset + sha256 | Low | Mechanical update |

---

### 4.5 CatalogEngine::FileEntry -- Add entry_type

Current (P-00 v0.2 proposal, sha256 only):
```rust
pub struct FileEntry {
    pub path: String,
    pub size: u64,
    pub modified: String,
    pub sha256: Option<String>,
    pub extents: Vec<FileExtent>,
}
```

New:
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CatalogEntryType {
    File,
    Directory,
}

pub struct FileEntry {
    pub path: String,
    pub entry_type: CatalogEntryType,  // NEW: distinguishes files from directories
    pub size: u64,
    pub modified: String,
    pub sha256: Option<String>,        // File: Some(valid_sha256), Directory: None
    pub extents: Vec<FileExtent>,
}
```

Rationale: Without entry_type, empty files (extents=[], sha256=Some(empty_hash)) and empty directories (extents=[], sha256=None, size=0) are indistinguishable in the Catalog. The old Phase 1 flat-file format correctly separated files from directories via distinct manifest arrays; the new Catalog must preserve this distinction.

Entry type constraints:

| Entry type | SHA-256 | Extents | Restore behavior |
|------------|---------|---------|------------------|
| File | REQUIRED (empty file: e3b0c442...) | MAY be empty | Write file content |
| Directory | None (None) | MUST be empty ([]) | Create via: symlink_metadata -> reject reparse point -> create_dir (one component at a time) -> canonicalize + starts_with verify (see \u00a71.4 step 4) |

Trait change:
```rust
// Existing (per 4.1):
fn add_file(&mut self, path: &str, size: u64, modified: &str,
             sha256: Option<&str>, extents: Vec<FileExtent>)
    -> Result<(), RepositoryError>;

// NEW:
fn add_directory(&mut self, path: &str) -> Result<(), RepositoryError>;

// Modified: get_file now returns entry_type alongside sha256:
fn get_file(&self, path: &str) -> Result<Option<FileEntry>, RepositoryError>;
```

Schema change (SqliteCatalog):
```sql
-- New file_entries table:
CREATE TABLE file_entries (
    file_id     INTEGER PRIMARY KEY AUTOINCREMENT,
    path        TEXT NOT NULL UNIQUE,
    entry_type  TEXT NOT NULL DEFAULT 'file',  -- NEW: 'file' | 'directory'
    size        INTEGER NOT NULL,
    modified    TEXT NOT NULL,
    sha256      TEXT,
    FOREIGN KEY...
);
```

Updated Summary of Changes (amend 4.4):

| Type | File | Change | Risk | Test Impact |
|------|------|--------|------|-------------|
| Enum | catalog/engine.rs | +CatalogEntryType {File, Directory} | Low | New tests for add_directory |
| Struct | catalog/engine.rs: FileEntry | +entry_type: CatalogEntryType | Low (new required field) | All constructors must be updated |
| Trait | catalog/engine.rs: CatalogEngine | +add_directory(); get_file returns entry_type | Low (backward-incompatible trait extension; all impls must update) | SqliteCatalog must impl add_directory |
| Schema | catalog/sqlite_catalog.rs | file_entries +entry_type TEXT DEFAULT 'file' | Low (NULL -> 'file' for backward compat) | Existing catalog.db files unaffected |
| Tests | sqlite_catalog.rs | +empty directory round-trip; +empty file vs empty dir distinction | Low | 3-4 new tests |

Additionally, Gate 3 must add:
- [ ] Directory tree containing ONLY empty directories (all Catalog entries are Directory type; no extents; no blocks) -- restore and verify every directory exists
- [ ] Empty file (entry_type=File, extents=[], sha256=Some(e3b0c442...)) -- verify file exists with zero bytes after restore
- [ ] Mixed empty dirs + files + nested dirs in single restore

## 5. P-00C -- Crash Recovery Commit Safety Fix

### 5.1 Defect Location

File: src/repository/transaction/manager.rs lines 214-221

```
// CURRENT (BUGGY):
_ => {
    if journal.components.all_completed() {
        // All components completed data is safe auto-commit
        report.auto_committed.push(point_id.clone());
    } else {
        report.marked_failed.push(point_id.clone());
    }
}
```

### 5.2 Defect Description

When CrashConsistencyManager::recover_at_startup() encounters a journal in non-terminal state (CREATING, WRITING, or VERIFYING), it checks component completion flags. If all 4 components (block_store, block_map, catalog, metadata) are marked Completed, it auto-commits the transaction.

**This is unsafe because:**
1. Component completion only means "the write function returned Ok()"
2. It does not mean the written data has been verified for consistency
3. The enter_verify() step was never called or confirmed
4. repo.db restore_points.status has never been updated to COMMITTED
5. The journal state variable itself is still non-terminal

### 5.3 Fix Requirements

1. **recover_at_startup() must not auto-commit from non-terminal states**
   - CREATING -> mark FAILED (no data was written)
   - WRITING -> mark FAILED (components may be complete but data is unverified)
   - VERIFYING -> mark FAILED (verification was interrupted)
   - COMMITTED -> only terminal state where journal cleanup is safe
   - FAILED -> preserve for orphan tracking

2. **commit() sequence must be reordered**
   Current: update_state(COMMITTED) -> remove_journal()
   New: repo.db UPDATE restore_points SET status=COMMITTED -> write_journal(COMMITTED) -> confirm -> remove_journal()

3. **commit() must use a Repository Metadata API, not raw SQL inside Transaction Manager**
   Current: commit(self) -- no access to repo.db. Anti-pattern: passing RepoHandle into commit() and writing raw SQL inside Transaction Manager
   Design target: introduce a dedicated RepositoryMetadata API for restore point CRUD. Transaction Manager owns journal/crash state; Metadata API owns repo.db state. Exact trait shape decided in P-00C.

4. **Tests to add:**
   - Journal in WRITING with all components completed -> recover -> marked FAILED (not auto-committed)
   - Journal in VERIFYING with all components completed -> recover -> marked FAILED
   - Journal in COMMITTED state -> recovery cleans up journal
   - repo.db shows COMMITTED + journal exists -> recovery only cleans up journal
   - Full Phase S regression after fix

### 5.4 Priority

**BLOCKING**: P-01 must not begin until P-00C passes all gates.

---

## 6. Asset Identity Model

### 6.1 asset_id

Each backup source (in v1: each file directory) must have a stable asset_id.

```
Generation: UUID v4, generated once when Job is first created
Persistence: Stored in JobConfig.asset_id (TOML) and BackupInstanceMetadata.asset_id
Scope: Globally stable UUID tied to the Job, NOT scoped to a single Repository
Purpose: Future cross-asset features (volume/disk, remote nodes, Repository migration)
Stability: asset_id does not change when Repository path changes or when Job is moved to a different Repository
```

Rationale (contra empty-string): An empty asset_id makes it impossible to correlate backup instances across restores, Repository migrations, or future multi-asset tracking. Generating one UUID per Job adds zero runtime cost. The asset_id must be globally stable -- not tied to a single Repository -- so that a Job moved to a new Repository retains its asset identity.

### 6.2 asset_type

Fixed to "file" for v1 file backup.
Future values: "volume", "disk", "system".

---

## 7. History Model

### 7.1 Design Decision: Independent SQLite

History (operation log) remains in its own SQLite database, NOT in the Repository.

Rationale:
- History is operational metadata (when did backup run, duration, status)
- Repository holds backup data (blocks, catalogs, metadata)
- Different lifecycle: history is append-only and can be safely truncated
- History should be queryable even when Repository is offline
- History CANNOT be fully rebuilt from Repository metadata alone (operation duration, failure details, user cancellations are lost)

### 7.2 History Record Fields (current, with repository_id addition)

```rust
pub struct OperationRecord {
    pub backup_id: String,       // Restore Point UUID
    pub operation: String,       // "backup" | "restore" | "verify"
    pub timestamp: String,       // ISO 8601
    pub source_root: String,     // source_path from Job
    pub repository_path_snapshot: Option<String>,  // Snapshot at time of operation (informational, NOT for lookup)
    pub restore_point_id: Option<String>, // Restore Point UUID from this operation
    pub backup_instance_id: Option<String>, // Backup Instance UUID
    pub job_name: Option<String>,
    pub repository_id: String,       // Primary association key -- links history to Repository
    pub file_count: u64,
    pub total_bytes: u64,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub status: String,          // "success" | "failure" | "partial"
}
```

### 7.3 Location

History DB path: ~/.nuwa/history.db (single location, shared across Repositories)

---

## 8. First Version Scope

### 8.1 Included

| Feature | Status |
|---------|--------|
| Source file consistency during backup | Yes -- best-effort consistent read based on pre/post-read metadata check. Detects size or mtime changes during read -> retry or fail. | Without VSS/snapshot, no atomic point-in-time guarantee for active or database files. Whatever is written to Repository is verified by stored SHA-256 on restore. |
| Repository-only storage | Yes |
| File source (single directory) | Yes |
| Full backup only (chain_position=0, parent=NULL) | Yes |
| 256KB fixed block | Yes |
| zstd compression per-block (default enabled) | Yes |
| Single-file restore (by path) | Yes |
| Full-directory restore | Yes |
| Restore to original or alternate destination | Yes |
| Restore-time SHA-256 verification (against stored expected_sha256 in Catalog) | Yes |
| Retention (logical deletion, no GC) | Yes |
| Repo-level verify (3-level: catalog integrity, block-map integrity, block hash) | Yes |

### 8.2 Deferred

| Feature | Reason | Future Phase |
|---------|--------|-------------|
| Incremental backup | Chain model is designed but not implemented | Phase 6+ |
| Small-file packing (multiple files per block) | Optimization, not correctness | Future |
| Physical GC / dedup | Explicitly excluded by Phase S contract | Phase 6+ |
| Encryption | Explicitly excluded by Phase S contract | Phase 6+ |
| Object storage backend | Product scope boundary | Enterprise |

---

## 9. Unsupported File System Semantics

These are inherited from Phase 1 limitations and explicitly documented:

| Feature | Behavior | Risk |
|---------|----------|------|
| Symbolic links | Causes job FAILURE unless user explicitly configures exclusion rules. Symlinks are listed in results with clear warnings. | Data loss risk eliminated by fail-safe default |
| Junction points (Windows) | Causes job FAILURE unless user explicitly configures exclusion rules. Listed in results with clear warnings. | Same risk as symlinks |
| Reparse points | Causes job FAILURE unless user explicitly configures exclusion rules. Listed in results with clear warnings. | Same risk as symlinks |
| Hard links | Each hardlink backed up as independent file | Space waste on restore |
| ACL / permissions | NOT backed up or restored | Security context lost on restore |
| Alternate Data Streams (ADS, Windows) | NOT backed up | Data loss for ADS-dependent files |
| Sparse files | Restored as full (non-sparse) files. Behavior documented in results. | Space differs from source; data content preserved correctly |
| Encrypted files (EFS) | Backed up as decrypted bytes (if readable) | Same as source |
| Compressed files (NTFS) | Backed up as decompressed bytes | Same |
| Locked files | Read consistency check before and after read. Retry limited times on inconsistency. Persistent inconsistency causes backup FAILURE. Never produces COMMITTED RestorePoint with partial data. | Silent partial data eliminated |

---

## 10. Gate Definitions

Each Gate must pass before the next phase of work begins.

### Gate 1: Single-File Write Reliability

Single cross-block file (~300 KB) backup through complete pipeline:

- [ ] Generate test file with known content
- [ ] Initialize Repository
- [ ] Write file via: walk -> ChunkEngine -> BlockStore -> BlockMap -> Catalog -> Transaction
- [ ] Close all handles (drop SqliteCatalog, SqliteBlockMap, RepoHandle)
- [ ] Re-open Repository
- [ ] Verify RepoHandle integrity (check_repo())
- [ ] Read RestorePoint from repo.db -- status must be COMMITTED
- [ ] Verify Catalog has exactly 1 file entry with expected sha256
- [ ] Verify BlockMap has exactly 2 entries for this 300KB file
- [ ] Verify each block via BlockStore::verify_block()
- [ ] Verify no orphan journals remain (scan_journals returns empty)

### Gate 2: Single-File Restore Correctness

- [ ] Re-open Repository from Gate 1
- [ ] Read file entry from Catalog
- [ ] For each extent (sorted by file_offset): lookup logical_offset in Catalog -> look up block_ids from BlockMap.get_range(logical_offset, logical_offset + length) -> read blocks from BlockStore
- [ ] Write data to temp file: each extent writes exactly length bytes at file_offset position
- [ ] Verify extents: no gaps, no overlap, covers [0, file_size)
- [ ] Atomic write to destination (.tmp -> rename)
- [ ] Compute SHA-256 of restored file
- [ ] Compare against expected_sha256 from Catalog -- must match
- [ ] Confirm source file and restored file are byte-identical
- [ ] Test: restore to same path without --overwrite -> rejected (safety)
- [ ] Test: restore to same path with --overwrite -> succeeds + SHA-256 match

### Gate 3: Directory & Boundary Semantics

- [ ] Multi-level nested directory (depth >= 5)
- [ ] Empty directory (catalog entry but no file extents)
- [ ] Empty file (extents = []; block count = 0; sha256 of empty data)
- [ ] Files with Unicode names (Chinese, Japanese, emoji)
- [ ] Files with spaces in names
- [ ] Files with identical content (content-addressed blocks should match)
- [ ] File > 1 block (cross-block boundary verified)
- [ ] 1000+ small files in one backup run
- [ ] Source file changes between backup runs (new, modified, deleted)
- [ ] Source file changes DURING a single backup run -- must trigger retry or failure, never silent COMMITTED
- [ ] Restore to original path (safety check: path traversal prevention)
- [ ] Restore to alternate path (explicit --dest)
- [ ] Restore a nested file whose parent directories do not exist (e.g., D:\Restore\a\b\c\d\report.docx where none of a/b/c/d exist) -- only parent directories are created; the final component (report.docx) is NOT created as a directory; SHA-256 of restored file matches expected
- [ ] Restore root contains a pre-existing malicious symlink/junction pointing outside restore root -- restore must FAIL
- [ ] Recovery path has a symlink/junction/reparse point as an intermediate ancestor directory -- must FAIL, and NO directories or files must be created outside the restore root
- [ ] Catalog path contains `..` segment -- restore must FAIL
- [ ] Catalog path is absolute -- restore must FAIL
- [ ] Directory tree containing ONLY empty directories (all Catalog entries are Directory type; no extents; no blocks) -- restore and verify every directory exists
- [ ] Empty file (entry_type=File, extents=[], sha256=Some(e3b0c442...)) -- verify file exists with zero bytes after restore
- [ ] Mixed empty dirs + files + nested dirs in single restore

### Gate 4: Fault Injection

- [ ] Crash during RestorePoint creation (step 1) -- repo.db has CREATING entry, no journal yet. Recovery scan marks it FAILED.
- [ ] Crash during BlockStore writes (step 3) -- WRITING state -> recover -> FAILED
- [ ] Crash during BlockMap writes -- same -> FAILED
- [ ] Crash during Catalog writes -- same -> FAILED
- [ ] Crash after metadata write but before commit -- VERIFYING -> FAILED
- [ ] Crash after repo.db COMMITTED but before journal removal -- recovery cleans journal
- [ ] Repository disk full during write -- clear error, no silent degradation
- [ ] User interrupts (Ctrl+C) during backup -- transaction FAILED, blocks remain as orphan candidates
- [ ] Corrupted block in block-store -- verify detects mismatch
- [ ] Corrupted catalog.db -- verify detects mismatched sha256 (backup-metadata.json has CatalogIntegrity)

### Gate 5: Application Layer + CLI + Tauri UI Integration

- [ ] nuwa backup --source <path> --repo <path> completes and returns success
- [ ] nuwa list --repo <path> shows the new RestorePoint
- [ ] nuwa restore --repo <path> --point <id> --dest <path> restores and passes SHA-256
- [ ] nuwa verify --repo <path> passes (uses VerifyEngine)
- [ ] nuwa repo check passes (full repository integrity)
- [ ] Tauri list_restore_points returns repository-based points (no FlatFileProvider)
- [ ] Tauri execute_restore uses RepositoryRestoreProvider
- [ ] Tauri run_backup uses new backup pipeline
- [ ] Settings page: Job config shows repository_id (not dest, not storage_type)
- [ ] Settings page: storage_type field and UI removed

### Gate 6: Flat File Removal and Residual Scan

- [ ] FlatFileRestoreProvider deleted; no code references remain
- [ ] storage_type deleted from all JobConfig (Rust, TOML, View, Request)
- [ ] dest field removed from JobConfig (replaced by repository_id)
- [ ] backup.rs, restore.rs, storage.rs, manifest.rs, list.rs (old) deleted
- [ ] No Flat File test remains (backup_restore_tests.rs deleted or migrated)
- [ ] TOML config template updated (no --dest, no storage_type)
- [ ] CLI help text updated (--repo, --point, --dest for restore only)
- [ ] UI TypeScript types no longer reference storage_type or flat-file fields
- [ ] All old Flat File .rs imports removed from lib.rs
- [ ] grep -r "flat.file|FlatFile|storage_type|manifest.json|BackupStorage" reveals zero hits

---

## 11. References

| Document | Relation |
|----------|----------|
| Phase S Architecture v1.0 | Repository Engine design baseline |
| Phase S Architecture v1.1 (Enterprise Readiness) | Asset identity, capabilities, version migration |
| Phase S API Freeze Report | Frozen types this document modifies (Section 4) |
| Phase S Closing Report | Phase S baseline freeze status |
| Phase 1 Technical Baseline | Original file backup safety semantics |
| Phase 1 Known Limitations | Inherited FS semantics (Section 9) |
| AGENTS.md Section 7 | Current phase confirmation |

---

## Revision History

| Version | Date | Change |
|---------|------|--------|
| v0.7 | 2026-07-11 | Fixed \u00a74.5 Directory restore behavior to reference \u00a71.4 guarded walk (not create_dir_all); fixed \u00a71.4 formatting: step 5 title detached from step 4 text: File entries skip last component (filename is not a directory); Directory entries traverse all components; Gate 3 nested-file test clarified for parent-only creation + symlink_metadata guard per ancestor (\u00a71.4 step 4); added intermediate-ancestor symlink escape test to Gate 3; atomic write pattern for files (step 6) |
| v0.4 | 2026-07-11 | Fixed restore-time path rules: canonicalize-on-target replaced with parent-chain traversal guard; added Gate 3 path traversal tests and empty-directory/empty-file scenarios; unified Catalog entry terminology |
| v0.3 | 2026-07-11 | Added CatalogEntryType (File|Directory) with add_directory() (4.5); added Catalog Path Security Contract (1.4); repaired UTF-8 encoding corruption |
| v0.2 | 2026-07-11 | Fixed stray control character; added Step 1 crash footnote; 5+1 audit items verified as correctly reflected |
| v0.1 | 2026-07-11 | Initial draft -- P-00 Data Contract Freeze |
