# Nüwa Backup (女娲备份)

**Version:** 0.2.0
**Phase:** 2.5 — Tauri Desktop GUI + Application Layer
**Status:** ACTIVE
Nüwa Backup is a **local-first desktop backup and recovery application** for Windows.
It combines a high-performance Rust backup engine with a modern Tauri 2.0 desktop GUI
built with React + TypeScript.

### Current Architecture

```
React UI (TypeScript)
    |
    | Tauri invoke() IPC
    v
Tauri Command Layer (Rust, thin wrapper)
    |
    v
Application Service Layer (Rust, src/app/)
    |
    v
Core Engine (Rust, src/) — backup, restore, verify, storage
    |
    v
File System / SQLite
```

### Phase 1 & 2 (CLOSED)
- File-level backup/restore CLI with SHA-256 verification
- Configuration system with multi-job TOML support
- Backup history with SQLite database
- Windows Task Scheduler integration
- SMB/UNC path support
- Retention policy with count/dry-run/prune

### Phase 2.5 (ACTIVE)
- Tauri 2.0 desktop GUI (replaces egui)
- React + TypeScript + Vite frontend
- Application Service Layer (src/app/)
- Dashboard with real data integration
- Commercial-grade UI with skeleton, empty, error states

---

## CLI Usage

### Global Options

| Flag | Description |
|------|-------------|
| `--help`, `-h` | Show full usage help |
| `--version`, `-V` | Show program version |

### Commands

#### `nuwa backup --source <path> --dest <path> [--compress]`

Perform a complete file backup from source to destination.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--source <path>` | Yes | Source directory to back up |
| `--dest <path>` | Yes | Destination root directory for backup storage |
| `--compress` | No | Enable zstd compression (requires `compress` feature; returns error if feature not compiled in) |

**Safety rules:**
- Source path must exist.
- Source and destination must not be the same path.
- Destination must not be inside source (prevents circular backup).
- Source must not be inside destination.
- Destination disk must have sufficient free space (10% safety margin).

**Example:**
```
nuwa backup --source C:\Users\Me\Documents --dest D:\Backups
nuwa backup --source C:\Data --dest E:\Backup --compress
```

---

#### `nuwa restore --backup <path> --dest <path> [--overwrite]`

Restore files from a backup point.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--backup <path>` | Yes | Path to the backup point directory (containing manifest.json) |
| `--dest <path>` | Yes | Destination directory for restored files |
| `--overwrite` | No | Overwrite existing files at destination |

**Behavior:**
- Without `--overwrite`, existing files at the destination are skipped.
- With `--overwrite`, existing files are overwritten after a lock check.
- Files locked by other processes are skipped with a warning.
- SHA-256 checksum is verified on every restored file.

**Example:**
```
nuwa restore --backup D:\Backups\20260705_143000_Documents --dest C:\Users\Me\Documents
nuwa restore --backup D:\Backups\20260705_143000_Documents --dest C:\Restore --overwrite
```

---

#### `nuwa verify --backup <path>`

Verify the integrity of a backup point.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--backup <path>` | Yes | Path to the backup point directory |

**Verification checks:**
1. Manifest file exists and is valid JSON.
2. Manifest schema version is compatible (major version match).
3. Every file listed in manifest exists in backup storage.
4. SHA-256 checksum matches for every file.
5. `.tmp` residue files are ignored (they do not cause verification failure).

**Example:**
```
nuwa verify --backup D:\Backups\20260705_143000_Documents
```

---

#### `nuwa list --dest <path>`

List all backup points stored at the given destination.

| Parameter | Required | Description |
|-----------|----------|-------------|
| `--dest <path>` | Yes | Backup root directory to scan |

**Example:**
```
nuwa list --dest D:\Backups
```

Output:
```
Backup points in D:\Backups:
  20260705_143000_Documents  (7 files, 5 dirs, 1.2 MB)
  20260705_123000_Photos     (142 files, 8 dirs, 512.3 MB)
```

---

### Exit Codes

| Code | Meaning | When |
|------|---------|------|
| 0 | Success | Operation completed successfully |
| 1 | General failure | Unexpected error |
| 2 | Invalid arguments | Missing or incorrect CLI parameters |
| 3 | I/O error | Disk full, permission denied, file locked, etc. |
| 4 | Checksum failure | File content does not match recorded checksum |
| 5 | Restore validation failure | Restored file checksum verification failed |
| 6 | Safety violation | src = dest, dest inside src, or src inside dest |
| 7 | Manifest error | Missing, corrupted, or incompatible manifest.json |

---

## Backup Storage Layout

Each backup operation creates a timestamped directory under the destination root:

```
<dest_root>/
  YYYYMMDD_HHMMSS_<source_name>/
    manifest.json          # Backup metadata and file index
    files/                 # Flat copy of all backed-up files
      <relative_path_1>
      <relative_path_2>
      ...
```

**Directory naming:** `YYYYMMDD_HHMMSS_<source_dir_name>`
- Timestamp uses local time for human readability.
- If collision occurs (two backups in the same second), a millisecond suffix is appended.

**Atomic writes:**
- Each file is written as `.tmp` first, then renamed to its final name.
- Manifest is written as `manifest.json.tmp` first, then renamed to `manifest.json`.
- If the process crashes mid-backup, existing backup points are not affected.
- Leftover `.tmp` files are silently ignored during verify.

---

## Manifest Schema

**Version:** `1.0` (current, only supported version)

```json
{
  "schema_version": "1.0",
  "backup_id": "uuid-v4-string",
  "created_at": "2026-07-05T14:30:00+00:00",
  "source_root": "C:\\Users\\Me\\Documents",
  "storage_format": "flat-file",
  "compression": {
    "enabled": false,
    "algorithm": null
  },
  "files": [
    {
      "relative_path": "docs/report.txt",
      "size_bytes": 1024,
      "modified_time": "2026-07-04T10:00:00+00:00",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "stored_path": "docs/report.txt"
    }
  ],
  "directories": [
    {
      "relative_path": "docs"
    },
    {
      "relative_path": "docs/reports"
    }
  ],
  "summary": {
    "file_count": 1,
    "directory_count": 2,
    "total_bytes": 1024
  }
}
```

### Schema Stability Rules
- `schema_version` uses `major.minor` format.
- `verify` rejects any manifest whose major version differs from the current version.
- Any schema change must be documented and version-bumped.
- Do not silently change schema fields.

---

## Known Limitations

| Limitation | Status | Details |
|------------|--------|---------|
| 10GB+ large file backup/restore | Manual test only | Not automated; run `--ignored` tests manually |
| Destination disk full during backup | Manual test only | Hard to simulate safely in automated tests |
| Locked file detection | Best-effort | Uses `OpenOptions::write(true).create(false).truncate(false)` to detect locks. Covers `PermissionDenied` and `WouldBlock` on Windows, but does not cover all Windows file lock scenarios (e.g., mapped file locks, transactional locks). |
| Destination space check (non-Windows) | PARTIAL | On Windows, uses `GetDiskFreeSpaceExW` Win32 API. On non-Windows platforms, falls back to a writability check only. True remaining-space check is not implemented. |
| Compression | BUG / TODO | Without `compress` feature, `--compress` returns a clear unsupported-feature error (exit code 2, InvalidArgs). This is the correct behavior. See `backup.rs` `#[cfg(not(feature = "compress"))]` check. |
| Symlinks | Skipped | Symbolic links are silently skipped during backup. |
| Performance optimization | Not started | Phase 1 does not optimize for speed. 1000+ file directories work correctly but may be slow. |

---

## Development

### Prerequisites
- Rust 1.79+ (MSVC toolchain on Windows)
- No network access required for builds (all crates cached)

### Build
```bash
cargo build                          # Debug build
cargo build --release                # Release build
cargo build --features compress      # Build with compression support
```

### Test
```bash
cargo test                           # All tests (offline)
cargo test -- --ignored              # Manual tests (large file, disk full)
```

### Quality Gates
```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo build
cargo test
```

### Project Structure
```
src/
  main.rs          # Program entry and CLI dispatch
  cli.rs           # Manual CLI argument parser (no clap)
  backup.rs        # Backup execution and directory traversal
  restore.rs       # Restore execution and lock detection
  verify.rs        # Backup point verification
  list.rs          # Backup point listing
  manifest.rs      # JSON manifest model (serde)
  checksum.rs      # SHA-256 helpers
  storage.rs       # Flat-file storage layout and atomic writes
  errors.rs        # Error types and exit code mapping
  diskspace.rs     # Win32 GetDiskFreeSpaceExW FFI
  lib.rs           # Module exports

tests/
  backup_restore_tests.rs   # 19 integration tests (3 unit tests in src/)
```

