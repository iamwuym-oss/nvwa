# Phase 1 — Code Map

**Phase:** 1 — Minimal File Backup/Restore CLI

---

## 1. Source Code Files

### `src/main.rs` — Program Entry

- Reads CLI args via `Command::from_args()`.
- Dispatches to `backup::execute_backup`, `restore::execute_restore`,
  `verify::execute_verify`, or `list::execute_list`.
- Maps `NuwaError` → `ExitCode` and calls `process::exit(code)`.
- Prints human-readable output (Chinese error messages).

### `src/cli.rs` — CLI Argument Parser

- Manual parser (no `clap` dependency).
- Parses: `backup`, `restore`, `verify`, `list`, `--help`, `--version`.
- Returns `Command` enum with typed parameter fields.
- On error: returns `NuwaError::InvalidArgument` with usage hint.

### `src/backup.rs` — Backup Execution

- `execute_backup(source, dest_root, compress) -> Result<String>`.
- Safety checks (in order):
  1. Source exists.
  2. Canonicalize source + `canonicalize_partial` for dest (zero side effect).
  3. src=dst, dest-in-src, src-in-dest checks.
  4. Destination space check (GetDiskFreeSpaceExW).
  5. Create dest directory (only after all checks pass).
- `walk_dir(dir, base)`: recursive `fs::read_dir`, skips symlinks, returns
  flat file + directory lists.
- `generate_backup_dir_name(source)`: `YYYYMMDD_HHMMSS_<dirname>`.
- Calculates total size with 10% safety margin.
- `#[cfg(not(feature = "compress"))]` check for --compress error.

### `src/restore.rs` — Restore Execution

- `execute_restore(backup_dir, dest, overwrite) -> Result<RestoreResult>`.
- RestoreResult: `restored_count`, `skipped_count`, `checksum_failures`.
- Reads manifest, recreates directory structure, then restores files.
- Overwrite logic:
  - `!overwrite` + file exists → skip.
  - `overwrite` + file exists → check `is_file_locked` → skip if locked.
- Calls `storage::restore_file` (atomic write).
- Verifies SHA-256 after each restore.
- `is_file_locked(path)`: opens with `write(true).create(false).truncate(false)`.
  Returns `PermissionDenied`/`WouldBlock` → locked.
  **Safe:** does NOT modify file content or size.

### `src/verify.rs` — Backup Verification

- `execute_verify(backup_dir) -> Result<()>`.
- Reads manifest, validates JSON schema.
- For each file: checks existence at `files/<stored_path>`, recomputes SHA-256.
- Ignores `.tmp` residue files.
- Fails if: manifest missing, corrupt, incompatible version, files missing, or
  checksum mismatch.

### `src/list.rs` — Backup Point Listing

- `execute_list(dest_root) -> Result<Vec<BackupPointInfo>>`.
- Scans `dest_root` for subdirectories containing `manifest.json`.
- Reads backup_id, created_at, source_root, file_count, total_bytes from each.
- `print_list()` formats output for human reading.

### `src/manifest.rs` — JSON Manifest Model

- `Manifest` struct with `schema_version`, `backup_id`, `created_at`,
  `source_root`, `storage_format`, `compression`, `files`, `directories`,
  `summary`.
- `FileEntry`: `relative_path`, `size_bytes`, `modified_time`, `sha256`,
  `stored_path`.
- `DirectoryEntry`: `relative_path`.
- `BackupSummary`: `file_count`, `directory_count`, `total_bytes`.
- `to_json_pretty()` → serde serialization.
- `from_file(path)` → serde deserialization + `validate_version()`.

### `src/checksum.rs` — SHA-256 Helpers

- `sha256_file(path) -> Result<String>` — reads file, computes SHA-256,
  returns 64-char lowercase hex.
- `sha256_bytes(data) -> String` — same for in-memory bytes.
- `verify_file_checksum(path, expected) -> Result<bool>` — compute + compare.

### `src/storage.rs` — Flat-File Storage and Atomic Write Engine

- `BackupStorage`: manages `backup_dir/` and `backup_dir/files/`.
- `copy_file_with_atomic_write`: reads source, writes `.tmp`, renames,
  cleans up `.tmp` on failure.
- `write_manifest`: writes `.json.tmp`, renames, cleans up on failure.
- `restore_file(backup_dir, entry, dest_path, was_compressed)`:
  reads from `backup_dir/files/<stored_path>`, writes `.tmp`, renames,
  cleans up on failure.
- `read_manifest(backup_dir)`: reads `manifest.json`.

### `src/errors.rs` — Error Types

- `ExitCode` enum: 8 codes (0-7), maps `NuwaError` → exit code.
- `NuwaError` enum: `InvalidArgument`, `Io`, `ChecksumMismatch`,
  `SafetyViolation`, `ManifestError`, `VerificationFailed`, `General`.
- Each variant has `detail` (Chinese) and `suggestion` (user advice).
- `From<std::io::Error>` conversion with auto-extracted path info.
- Factory methods: `source_not_found()`, `same_source_dest()`,
  `disk_space()`, `manifest_parse()`, `manifest_version()`.

### `src/diskspace.rs` — Win32 Disk Space Query

- `check_disk_space(path, needed_bytes) -> Result<()>`.
- On Windows: calls `GetDiskFreeSpaceExW` via `#[link(name="kernel32")]` FFI.
- On non-Windows: fallback `std::fs::write` writability check.
- Returns `NuwaError::Io` with remaining space info if insufficient.

### `src/lib.rs` — Module Exports

- `pub mod` for all 11 source modules.
- Enables integration tests to access internal functions.

---

## 2. Test Files

### `tests/backup_restore_tests.rs` — 19 Integration Tests

| Test | What it covers |
|------|----------------|
| test_normal_backup | Basic backup success |
| test_normal_restore | Basic restore success |
| test_consistency | Backup → delete → restore → SHA-256 compare |
| test_empty_dirs | Empty directories preserved |
| test_nested | 4-level nested directories |
| test_source_not_found | Missing source → exit 2 |
| test_source_equals_dest | src=dst → exit 6 |
| test_no_overwrite | Existing file protection |
| test_corrupted_manifest | Corrupt JSON → verify fails |
| test_tampered_data | Tampered file → verify fails |
| test_list | Correct backup point count |
| test_100_files | 100 small files |
| test_dest_not_writable | File as dest → fails |
| test_1000_files | 1000 small files |
| test_interrupt_safety | .tmp residue tolerated |
| test_dest_inside_source | dest in src → exit 6 |
| test_source_inside_dest | src in dest → exit 6 |
| test_lock_check_does_not_modify | Lock check preserves file content |
| test_dest_inside_source_no_create | Rejection doesn'\''t create dir |

---

## 3. Architecture Note

This is a **file backup core**. It is NOT:

- A volume backup core.
- A disk image core.
- A system recovery core.

When entering Phase 3, new modules should be added:
- `src/image/` — NTFS volume image and .nwb format.
- `src/block/` — Block-level backup engine.
- `src/platform/windows/` — VSS, PhysicalDrive, WinPE.

Do NOT force VSS, NTFS, PhysicalDrive, or .nwb logic into existing
Phase 1 file modules.