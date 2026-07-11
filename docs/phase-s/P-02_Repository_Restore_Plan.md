# P-02: Repository File Restore Reader — Implementation Plan

**Status:** PLANNED
**Phase:** P-02 (Repository Restore)
**Contract:** P-00_File_Backup_Repository_Data_Contract.md §10 Gate 2–3
**Dependency:** P-01 Writer (RepositoryBackupWriter) — must be formally signed off via Gate 1 before P-02a starts
**File:** src/repository/file_restore_reader.rs (new)
**Old code:** src/restore.rs (Flat File, kept as read-only reference until Gate 6)

---

## Overview

P-02 implements the **Repository Restore Reader**: a component that reads from a verified, COMMITTED Restore Point and reconstructs files streamingly with integrity verification.

### Core Design Principle

> Only COMMITTED Restore Points are eligible for restore. Any CREATING / WRITING / VERIFYING / FAILED / DELETED / DELETING state is rejected.

### Restore Algorithm (Approved)

`
Catalog FileEntry
→ sort extents by file_offset (ascending)
→ validate: no overlap, no holes, checked_add for overflow
→ for each extent:
    block_map.get_range(logical_offset, logical_offset + length)
    validate_block_coverage() — no gaps, exact boundary matches
    for each block mapping:
        get_block(block_id) — SHA-256 verified internally by BlockStore
        compute range intersection (file bytes to take from this block)
        write_all() to .tmp file at correct file_offset
→ compute SHA-256 of .tmp file
→ compare to catalog.sha256
→ atomic rename (or Windows ReplaceFileW for overwrite)
`

### Error Semantics

| Category | Behavior |
|----------|----------|
| Integrity errors (path escape, metadata mismatch, hash mismatch, block corruption) | Err — hard stop |
| Single-file I/O errors (disk full, permission denied, file locked) | RestoreOutcome::Partial — soft fail, continues |
| RestoreOutcome | Complete(RestoreSummary) or Partial(RestoreSummary) |

---

## Execution Plan

### Phase 0: P-01 Writer Bug Fixes (Pre-requisite, DEFERRED)

Two bugs in P-01 Writer must be fixed before P-02 can be validated against real Writer output:

| ID | Bug | File | Fix |
|----|-----|------|-----|
| F1 | SqliteCatalog::close() missing WAL checkpoint | src/repository/catalog/sqlite_catalog.rs | Add PRAGMA wal_checkpoint(TRUNCATE) before conn.close() |
| F2 | CatalogEntryType unknown value defaults to File | src/repository/catalog/sqlite_catalog.rs | "file" => File, "directory" => Directory, _ => Err(...) |

**Note:** These are P-01 bugs, not P-02 scope. They are listed here as pre-requisites only. The user has deferred these for now.

---

### Phase 1: New Shared Modules (M1, M2)

These modules are shared between Writer and Reader, created before P-02a.

#### M1: src/repository/path_security.rs — Path Security Module

Provides path validation and safe directory creation shared by Writer and Reader.

`ust
pub fn validate_catalog_relative_path(path: &str) -> Result<String, RepositoryError>
pub fn prepare_restore_file_target(dest_root: &Path, relative_path: &str) -> Result<PathBuf>
pub fn prepare_restore_directory(dest_root: &Path, relative_path: &str) -> Result<PathBuf>
`

**Validation rules (P-00 §1.4):**
- Reject empty path, absolute paths (/ or \\ or X:), .., ., UNC (\\\\)
- Normalize all separators to forward slash /
- Strip trailing slash, reject consecutive separators
- Per-ancestor symlink_metadata() check — reject symlink, junction, reparse point
- Create dirs one-by-one (NOT create_dir_all)
- prepare_restore_file_target: create parent dirs only (not last component)
- prepare_restore_directory: create all components including last

#### M2: RepoHandle::get_restore_point() — in src/repository/repo_manager.rs

`ust
pub struct RestorePointRecord {
    pub point_id: String,
    pub job_id: String,
    pub status: String,
    pub instance_path: String,
    pub block_count: i64,
    pub total_raw_bytes: i64,
}

pub fn get_restore_point(&self, point_id: &str)
    -> Result<Option<RestorePointRecord>, RepositoryError>
`

---

### Phase 2: P-02 Implementation Sub-tasks

#### P-02a: Repository Preflight + Restore Point Identity & Integrity Validation (14-step)

**File:** src/repository/file_restore_reader.rs — FileRestoreReader::open()

| Step | Action | Fail |
|------|--------|------|
| 1 | check_repo() — basic repository integrity | Err |
| 2 | get_restore_point(point_id) — read from repo.db | Err if not found |
| 3 | Verify status == "COMMITTED"; reject anything else | Err |
| 4 | Resolve instance_path — validate relative, join instances_dir, canonicalize | Err |
| 5 | Read backup-metadata.json from instance_path | Err |
| 6 | Verify metadata.restore_point_id == point_id | Err |
| 7 | Verify metadata.job_id == repo.db job_id | Err |
| 8 | Verify metadata.asset_type == "file", source_type matches | Err |
| 9 | Recompute SHA-256 of catalog.db == metadata.integrity.catalog.sha256 | Err |
| 10 | Recompute SHA-256 of block-map.db == metadata.integrity.block_map.sha256 | Err |
| 11 | Open Catalog (SqliteCatalog::open), open Block Map (SqliteBlockMap::open) | Err |
| 12 | Compare actual Block Map row count == metadata.block_count == repo.db.block_count | Err |
| 13 | Compare repo.db.total_raw_bytes == metadata.summary.total_raw_bytes == sum(Catalog file sizes) | Err |
| 14 | Verify restore root dest_root is not symlink/junction/reparse point | Err |

#### P-02b: Catalog Entry Validation — list_entries(), get_entry()

- Read file entries from Catalog
- Validate entry_type: "file" and "directory" only; unknown = Err
- Validate sha256: must be Some(...) for files; None = Err
- Validate path via validate_catalog_relative_path()
- Validate extents: sort by file_offset, no overlap, no holes, full coverage, checked_add
- Non-empty files: reject zero-length extents

#### P-02c: Streaming File Restore — restore_file()

- Empty file (0 bytes, no extents) → create empty, return
- Non-empty: create .tmp with create_new, stream extents via block_map → block_store
- Sequential write_all (no write_at, no sparse files)
- Verify total .tmp size == file_size
- Compute SHA-256 of .tmp → compare to catalog.sha256
- Atomic rename or Windows ReplaceFileW

#### P-02d: Directory Restore — restore_directory()

- Use prepare_restore_directory() for full guard + creation
- Directory already exists → ok (idempotent)
- Target exists as file → Err

#### P-02e: Full & Selective Restore + RestoreOutcome

`ust
pub enum RestoreOutcome { Complete(RestoreSummary), Partial(RestoreSummary) }
`

- restore_all() — restore every catalog entry
- restore_selected(paths) — selective by path, directory = recursive
- overwrite=false + file exists → Err (not skip)
- overwrite=true → Windows ReplaceFileW
- Integrity errors = hard Err; I/O errors = recorded in failed_files

#### P-02f: Gate 2 & Gate 3 Tests (27 test cases)

**Gate 2 — Single-File Restore:**
1. 300KB cross-2-block, close+reopen repo, restore, SHA-256 match
2. Empty file (0 bytes) restore + SHA-256 verify
3. Atomic write .tmp → rename correctness
4. overwrite=false on existing file → Err
5. overwrite=true on existing file → success + SHA-256 match

**Gate 3 — Directory & Boundary:**
6. Multi-level nested dir (depth >= 5) — all files restored
7. Empty directory — verify exists
8. Directory tree with only empty dirs
9. Mixed empty dirs + files + nested dirs
10. Selective restore of one file from multi-file backup
11. Selective restore of a directory (recursive)
12. Restore to non-existent deep directory (creates parents)

**Security & Integrity:**
13. Catalog path with .. → Err
14. Catalog absolute path → Err
15. Restore root is symlink → Err
16. Parent directory is symlink → Err
17. Block corrupted → get_block() returns Err
18. metadata.catalog.sha256 wrong → preflight reject
19. metadata.block_map.sha256 wrong → preflight reject
20. metadata.restore_point_id mismatch → preflight reject
21. metadata.block_count mismatch → preflight reject
22. metadata.total_raw_bytes mismatch → preflight reject
23. Restore Point not COMMITTED (WRITING/VERIFYING/FAILED/DELETED) → reject
24. Extent overlaps/gaps in Catalog → Err
25. Temp file cleaned up on restore failure
26. Restore failure leaves no final half-file
27. Restored file byte-identical to original

---

### Key Design Decisions

| Decision | Detail | Source |
|----------|--------|--------|
| Only COMMITTED | Any other status → reject | P-00 §3.2 |
| Preflight before data | 14-step validation before any read | P-02a |
| 4-way metadata compare | repo.db ↔ metadata ↔ actual rows ↔ file size sum | P-02a |
| WAL hash guarantee | Writer must checkpoint before hashing | F1 |
| Sequential write | write_all not write_at for cross-platform | P-02c |
| Windows atomic replace | ReplaceFileW for overwrite | P-02c |
| Per-ancestor dir guard | No create_dir_all, check symlink at every level | M1 |
| Integrity = hard Err | Path escape, hash mismatch, block corruption | Error model |
| IO = soft fail | Single file failure in summary, continue | Error model |
| RestoreOutcome | Complete vs Partial prevents misinterpretation | P-02e |
| Unknown entry_type | Strict Err not silent File | P-02b |
| Extent checked_add | Prevent overflow from corrupt data | P-02b |
| Shared path module | Writer and Reader use same path security | M1 |

### Files to Change

| File | Change | Phase |
|------|--------|-------|
| src/repository/path_security.rs | NEW | M1 |
| src/repository/repo_manager.rs | ADD RestorePointRecord + get_restore_point() | M2 |
| src/repository/mod.rs | ADD pub mod path_security, pub mod file_restore_reader | M1, P-02 |
| src/repository/file_restore_reader.rs | NEW — FileRestoreReader | P-02a–e |
| tests/repository_integration_tests.rs | ADD — 27 Gate 2–3 tests | P-02f |

### Not In Scope

- src/restore.rs (Flat File) — kept as read-only reference
- Verify Engine integration (P-03)
- Application Layer / Tauri UI (Gate 5)
- Flat File code removal (Gate 6)
- Block-level restore
- System/volume/disk restore
- Retention or pruning

### Dependencies

| ID | Dependency | Status |
|----|-----------|--------|
| F1 | SqliteCatalog WAL checkpoint | DEFERRED |
| F2 | CatalogEntryType strict parsing | DEFERRED |
| M1 | path_security.rs | NOT STARTED |
| M2 | RepoHandle::get_restore_point() | NOT STARTED |

**Note:** P-02 Phase 2 implementation must not start until F1 and F2 are fixed AND P-01 Gate 1 is formally signed off based on current real source code.
