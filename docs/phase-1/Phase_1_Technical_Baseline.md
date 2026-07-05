# Phase 1 — Technical Baseline

**Phase:** 1 — Minimal File Backup/Restore CLI
**Version:** 0.1.0

---

## 1. Module Structure

```
src/
  main.rs          # Program entry, CLI dispatch, exit code mapping
  cli.rs           # Manual CLI argument parser (no clap)
  backup.rs        # Backup execution, recursive directory traversal
  restore.rs       # Restore execution, locked file detection
  verify.rs        # Backup point integrity verification
  list.rs          # Backup point listing
  manifest.rs      # JSON manifest model (serde), v1.0 schema
  checksum.rs      # SHA-256 file and byte helpers
  storage.rs       # Flat-file storage layout, atomic write engine
  errors.rs        # Error types, exit code mapping, user-facing messages
  diskspace.rs     # Win32 GetDiskFreeSpaceExW FFI
  lib.rs           # Module exports

tests/
  backup_restore_tests.rs   # 19 integration tests
```

**Important:** This is a file backup core. It is NOT a volume backup core,
disk image core, or system recovery core. Future phases (Phase 3+ ) should
add new modules (image/, block/, platform/, windows/) rather than force
VSS/NTFS/PhysicalDrive/.nwb logic into existing file modules.

---

## 2. Data Flow

### Backup Flow

```
CLI args → cli.rs (parse)
         → backup.rs (execute_backup)
              → check source exists [safety]
              → canonicalize source + canonicalize_partial dest [zero-side-effect]
              → check src=dst, dest-in-src, src-in-dest [safety]
              → check disk space (GetDiskFreeSpaceExW on Windows)
              → create dest directory [only after all safety checks pass]
              → walk_dir (recursive fs::read_dir, skips symlinks)
              → for each file:
                    sha256_file (SHA-256)
                    copy_file_with_atomic_write (.tmp → rename)
                    [optional zstd compression]
              → write_manifest (JSON .json.tmp → rename)
```

### Restore Flow

```
CLI args → cli.rs (parse)
         → restore.rs (execute_restore)
              → read_manifest (validate schema v1.0)
              → for each file in manifest:
                    if exists + !overwrite → skip
                    if exists + overwrite → is_file_locked?
                         if locked → skip (best-effort)
                    storage::restore_file (.tmp → rename, atomic)
                    verify_file_checksum (SHA-256 compare)
```

### Verify Flow

```
CLI args → cli.rs (parse)
         → verify.rs (execute_verify)
              → read_manifest
              → for each file entry:
                    verify file exists
                    verify SHA-256 matches
              → report result
```

### List Flow

```
CLI args → cli.rs (parse)
         → list.rs (execute_list)
              → scan dest dir for subdirs with manifest.json
              → read each manifest for metadata
              → return sorted list
```

---

## 3. Manifest Schema v1.0

```json
{
  "schema_version": "1.0",
  "backup_id": "uuid-v4",
  "created_at": "ISO-8601 UTC",
  "source_root": "original source path",
  "storage_format": "flat-file",
  "compression": { "enabled": false, "algorithm": null },
  "files": [{ "relative_path", "size_bytes", "modified_time", "sha256", "stored_path" }],
  "directories": [{ "relative_path" }],
  "summary": { "file_count", "directory_count", "total_bytes" }
}
```

**Rules:**
- `schema_version` uses `major.minor`. Verify rejects major version mismatch.
- Any schema change must be documented and version-bumped.
- Do not store absolute paths, passwords, tokens, or credentials.

---

## 4. Error Code System

| Code | Name | Meaning |
|------|------|---------|
| 0 | Success | Operation completed successfully |
| 1 | GeneralFailure | Unexpected error |
| 2 | InvalidArgs | Missing or incorrect CLI parameters |
| 3 | IoError | Disk full, permission denied, file locked |
| 4 | ChecksumFailure | File content does not match recorded checksum |
| 5 | RestoreFailure | Restored file checksum verification failed |
| 6 | SafetyViolation | src=dst, dest-in-src, or src-in-dest |
| 7 | ManifestError | Missing, corrupted, or incompatible manifest |

---

## 5. Path Safety Rules

1. Source path must exist before any operation.
2. Source and destination must not be the same path (exact match).
3. Destination must not be inside source (prevents circular backup).
4. Source must not be inside destination (prevents self-referencing).
5. All path comparisons use `canonicalize_partial` which resolves 8.3 short
   names and `\\?\` prefix without creating any files or directories.
6. Destination directory is created only after all safety checks pass.

---

## 6. Atomic Write Rules

1. Every file is written to `.tmp` first, then renamed to final name.
2. Manifest is written to `.json.tmp` first, then renamed to `manifest.json`.
3. If rename fails, the `.tmp` file is immediately deleted.
4. Verify ignores `.tmp` residue files (they do not cause failure).
5. Mid-crash: existing valid backup points are never corrupted.

---

## 7. SHA-256 Checksum Rules

1. Every file is checksummed at backup time.
2. Checksum is stored in manifest per-file.
3. Verify re-checksums every file and compares with manifest.
4. Restore verifies checksum after atomic write completes.
5. If checksum mismatch: file is reported with error, backup/restore continues
   with failure count.

---

## 8. Disk Space Check

- **Windows:** Uses `GetDiskFreeSpaceExW` Win32 API via raw FFI.
- **Non-Windows:** Falls back to writability check (attempts to create a
  file; if it succeeds, space is assumed sufficient).
- 10% safety margin is added to estimated backup size.
- Space check happens before any data is written.
- **Known limitation:** Non-Windows check is PARTIAL.

---

## 9. Locked File Detection

- **Mechanism:** `OpenOptions::new().write(true).create(false).truncate(false)`
- When restore with `--overwrite` encounters an existing file, it first checks
  if the file is locked by another process.
- **Status:** Best-effort.
  - Covers: `PermissionDenied` (Windows sharing violation),
    `WouldBlock` (POSIX advisory lock).
  - Does NOT cover: mapped file locks, transactional NTFS locks, SMB locks.
- Locked files are skipped with a user-visible warning.

---

## 10. Automated Tests

### Unit Tests (3)
| Test | Scope |
|------|-------|
| `test_sha256_empty_file` | SHA-256 on empty content |
| `test_sha256_bytes_consistency` | SHA-256 consistency across calls |
| `test_sha256_format` | SHA-256 output format (64 hex chars) |

### Integration Tests (19)
| Test | Type | Scenario |
|------|------|----------|
| test_normal_backup | Unit | Normal file backup |
| test_normal_restore | Unit | Normal file restore |
| test_consistency | Integration | Backup → restore → SHA-256 compare |
| test_empty_dirs | Unit | Empty directory preservation |
| test_nested | Unit | 4-level nested directories |
| test_source_not_found | Error | Missing source path, exit code 2 |
| test_source_equals_dest | Safety | src=dst, exit code 6 |
| test_no_overwrite | Safety | Existing file protection |
| test_corrupted_manifest | Verification | Corrupt JSON → verify fails |
| test_tampered_data | Verification | Tampered file → verify fails |
| test_list | CLI | Correct backup point count |
| test_100_files | Integration | 100 small files |
| test_dest_not_writable | Error | File path as dest → fails |
| test_1000_files | Integration | 1000 small files |
| test_interrupt_safety | Recovery | .tmp residue tolerated |
| test_dest_inside_source | Safety | dest in src, exit code 6 |
| test_source_inside_dest | Safety | src in dest, exit code 6 |
| test_lock_check_does_not_modify | Safety | Lock check preserves file |
| test_dest_inside_source_no_create | Safety | Rejection does not create dir |