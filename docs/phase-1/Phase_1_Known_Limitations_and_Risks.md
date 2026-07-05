# Phase 1 — Known Limitations and Risks

**Phase:** 1 — Minimal File Backup/Restore CLI
**Last Updated:** 2026-07-05

---

## 1. Overview

This document lists all known limitations, risks, and unfinished items from
Phase 1. Phase 1 can proceed to acceptance with these documented limitations.

---

## 2. Technical Limitations

### 2.1 Destination Space Check (non-Windows)

| Field | Value |
|-------|-------|
| **Status** | PARTIAL |
| **Severity** | Medium |
| **Windows** | Full implementation using `GetDiskFreeSpaceExW` Win32 API via raw FFI. Returns exact available bytes. |
| **Non-Windows** | Falls back to a writability check: attempts to create a file at the destination. If it succeeds, space is assumed sufficient. This is NOT a real remaining-space check. |
| **Impact** | On Linux or other non-Windows platforms, backup may start and then fail mid-way due to disk full. |
| **Planned Fix** | Future phase when cross-platform support is implemented. |

### 2.2 Locked File Detection

| Field | Value |
|-------|-------|
| **Status** | Best-effort |
| **Severity** | Low |
| **Detection method** | `OpenOptions::new().write(true).create(false).truncate(false)` |
| **Covers** | `PermissionDenied` (Windows sharing violation), `WouldBlock` (POSIX advisory lock) |
| **Does NOT cover** | Mapped file locks, transactional NTFS locks, SMB/network file locks, some antivirus scanner locks |
| **Behavior on locked file** | File is skipped; user is warned; restore continues |
| **Impact** | In rare cases, a locked file may be reported as unlocked and overwritten during restore with --overwrite. |
| **Mitigation** | User must ensure files are not in use before restore. |

### 2.3 Large File Handling

| Field | Value |
|-------|-------|
| **Status** | Manual test only |
| **Severity** | Low |
| **Automated test** | Not included (destructive/time-consuming) |
| **Expected behavior** | Code uses streaming I/O (64 KB buffer), no file-size limit. Should handle any file size the filesystem supports. |
| **Manual test procedure** | See `Phase_1_Manual_Test_Plan.md` |

### 2.4 Destination Disk Full

| Field | Value |
|-------|-------|
| **Status** | Manual test only |
| **Severity** | Low |
| **Automated test** | Not included (destructive/hard to simulate) |
| **Expected behavior** | Backup will fail with I/O error; existing backup points are not corrupted; .tmp files are cleaned up. |
| **Manual test procedure** | See `Phase_1_Manual_Test_Plan.md` |

---

## 3. Design Limitations

### 3.1 Symbolic Links

| Field | Value |
|-------|-------|
| **Status** | Explicitly skipped |
| **Rationale** | Symlinks can point outside the backup scope. Phase 1 design decision. |
| **Future** | May be added in Phase 2+ as opt-in. |

### 3.2 Compression (`--compress` flag without feature)

| Field | Value |
|-------|-------|
| **Status** | BUG (fixed in Task 1.4) |
| **Previous behavior** | `--compress` was silently ignored when `feature = "compress"` was not enabled. |
| **Current behavior** | Returns clear error: "压缩功能未启用" (exit code 2, InvalidArgs) |
| **Verification** | Tested manually: `cargo run -- backup --source <dir> --dest <dir> --compress` → exit 2, Chinese error message |

### 3.3 Performance

| Field | Value |
|-------|-------|
| **Status** | Not optimized |
| **Rationale** | Phase 1 explicitly excludes performance optimization. |
| **Expected** | 1000 files backup completes in ~seconds. 10000+ files may be slower. Single-threaded I/O. |

---

## 4. Risks

| Risk | Level | Description | Mitigation |
|------|-------|-------------|------------|
| Space check PARTIAL on non-Windows | MEDIUM | Backup may fail mid-way on Linux | Phase 1 targets Windows; document as Known Limitation |
| Lock detection best-effort | LOW | Rare lock types not detected | Document limitation; user must close files before restore |
| 10GB+ file not tested | LOW | Large file edge case not covered | Manual test procedure documented |
| Symlinks silently skipped | LOW | User may not notice missing data | Documented design decision |
| No encryption | LOW | Data at rest is plain | Phase 3+ scope |
| 8.3 short name on Windows | LOW | `canonicalize_partial` resolves short names; but path comparison may behave unexpectedly with very unusual paths | Handled by `canonicalize_partial` function |