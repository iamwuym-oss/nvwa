# Phase 1 — Closing Report

**Product:** Nüwa Backup (女娲备份)
**Phase:** 1 — Minimal File Backup/Restore CLI
**Closing Date:** 2026-07-05
**Final Closing Status:** CLOSED (with known limitations)

---

## 1. Closing Summary

| Field | Value |
|-------|-------|
| **Phase Name** | Phase 1 — Minimal File Backup/Restore CLI |
| **Phase Goal** | Build a minimal, reliable file-level backup and restore CLI tool for Windows, with SHA-256 integrity verification, atomic writes, and safety guards. |
| **Start Status** | Project initialization from scratch |
| **End Status** | PARTIAL (non-Windows space check is writability-only; accepted known limitation) |
| **Closing Decision** | CLOSED — Phase 1 is frozen as project baseline. No unauthorized changes to Phase 1 file backup core. |

---

## 2. Final Phase 1 Scope

### Implemented and Accepted

- **File-level backup** — Recursive directory traversal (manual `fs::read_dir`, no `walkdir`), skips symlinks
- **File-level restore** — Restore to original or alternate location, with overwrite control
- **`nuwa verify`** — Manifest integrity + per-file SHA-256 verification
- **`nuwa list`** — Backup point listing with metadata
- **JSON manifest v1.0** — Schema `"1.0"` with backup_id, created_at, source_root, files, directories, summary
- **SHA-256 per-file checksum** — Computed at backup, verified at restore and verify
- **Flat-file storage** — Directory + JSON manifest, NOT `.nwb`
- **Atomic write safety** — `.tmp` → rename for all files and manifest; `.tmp` cleanup on failure
- **Path safety checks** — Source exists, src ≠ dest, dest not inside src, src not inside dest
- **Windows destination space check** — `GetDiskFreeSpaceExW` Win32 API via raw FFI, with 10% safety margin
- **Best-effort locked file detection** — `OpenOptions::write(true).create(false).truncate(false)` during restore overwrite
- **CLI E2E restore validation** — 1012 files backup → list → verify → delete source → restore → SHA-256 compare (100% match)
- **22 automated tests** — 3 unit + 19 integration, all passing
- **Manual test plan** — Documented for 10GB+ large file, disk full, locked file

### Explicitly Not Included

| Feature | Status |
|---------|--------|
| `.nwb` image format | Phase 3+ |
| VSS snapshot integration | Phase 3+ |
| Volume-level backup | Phase 3+ |
| Disk-level backup | Phase 5 |
| System volume backup / system restore | Phase 4 |
| WinPE / Alpine recovery media | Phase 4 |
| Disk cloning | Phase 5 |
| Universal restore / heterogeneous restore | Future |
| Differential backup | Phase 6+ |
| Incremental backup | Excluded / Not planned |
| AES encryption | Phase 6+ |
| XOR parity / erasure coding | Future |
| daemon / system service | Phase 2+ |
| IPC | Phase 2+ |
| GUI | Phase 2+ |
| Scheduler / periodic backup | Phase 2 |
| SMB / NFS network target | Phase 2+ |
| Linux / domestic OS support | Phase 6+ |
| Performance optimization | Future |

---

## 3. Evidence Summary

### Quality Gates

| Gate | Result | Evidence |
|------|--------|----------|
| `cargo fmt --check` | PASS | Exit code 0, no formatting issues |
| `cargo clippy --all-targets -- -D warnings` | PASS | Exit code 0, no warnings |
| `cargo build --offline` | PASS | Exit code 0 |
| `cargo test --offline` | PASS (22/22) | 3 unit + 19 integration, all passing |

### Automated Test Results

| Category | Count | Status |
|----------|-------|--------|
| Unit tests (checksum) | 3 | PASS |
| Integration tests (backup_restore) | 19 | PASS |
| **Total** | **22** | **PASS** |

### CLI E2E Restore Validation

| Test | Result |
|------|--------|
| Task 1.2 CLI E2E — 1012 files | PASS |
| backup → list → verify → delete source → restore → SHA-256 compare | PASS — 1012/1012 files matched (100%) |
| Unicode / Chinese / Japanese / emoji / space filename validation | PASS |
| 1000-file test | PASS |
| Manifest corruption detection | PASS |
| Data tampering detection | PASS |
| `.tmp` residue tolerance | PASS |
| Path containment safety tests | PASS |
| Lock check does not modify existing file | PASS |

---

## 4. Accepted Known Limitations

| Limitation | Status | Blocks Phase 1? |
|------------|--------|:----------------:|
| 10GB+ large file backup/restore — manual test only | MANUAL | No |
| Destination disk full during backup — manual test only | MANUAL | No |
| Locked file detection — best-effort, not covering all Windows lock types | BEST-EFFORT | No |
| Non-Windows destination space check — writability-only, no real remaining-space query | PARTIAL | No |
| Symbolic links — explicitly skipped | DESIGN | No |
| No encryption — data at rest is plain text | FUTURE | No |
| No performance optimization — single-threaded, no buffer tuning | FUTURE | No |
| No differential backup | FUTURE | No |
| No incremental backup | EXCLUDED | No |
| No VSS snapshot integration | FUTURE | No |
| No volume/disk/system recovery | FUTURE | No |
| Compression (`--compress` without feature) — returns clear error, exit code 2 | FIXED | No |

---

## 5. P1-09 Destination Space Check — Final Truth

### Real Status

**A. Windows: GetDiskFreeSpaceExW is fully integrated and actively called during backup.**

The function `check_disk_space()` in `src/diskspace.rs` calls `platform::free_space()` which invokes `GetDiskFreeSpaceExW` via `#[link(name = "kernel32")]` FFI on Windows. It returns actual free bytes. The backup flow in `src/backup.rs` calls this before any data is written. This is a REAL remaining-space check on Windows.

**B. Non-Windows: Falls back to writability check (PARTIAL).**

On non-Windows targets, `platform::free_space()` returns `Err`, and `check_disk_space()` degrades to a simple writability test (create a file, delete it). This is NOT a real space check.

### Consistency Check Results

All documents agree:

| Document | Windows Status | Non-Windows Status | Consistent? |
|----------|---------------|--------------------|:-----------:|
| `README.md` | `GetDiskFreeSpaceExW` | PARTIAL (writability only) | ✅ |
| `docs/phase-1/Phase_1_Final_Acceptance_Report.md` | PASS (`GetDiskFreeSpaceExW`) | PARTIAL (writability only) | ✅ |
| `docs/phase-1/Phase_1_Known_Limitations_and_Risks.md` | Full implementation | PARTIAL (writability only) | ✅ |
| `docs/phase-1/Phase_1_Technical_Baseline.md` | `GetDiskFreeSpaceExW` via FFI | Writable check | ✅ |
| `docs/phase-1/Phase_1_Closing_Report.md` (this doc) | `GetDiskFreeSpaceExW` integrated | PARTIAL (writability only) | ✅ |

**Conclusion:** P1-09 status is consistent across all documents. No correction needed for P1-09.

---

## 6. README Compression Description Fix

**Issue found:** `README.md` stated that `--compress` without feature "currently falls back to uncompressed copy. **This is a bug.**" But the actual code (`backup.rs` line 161-166) correctly returns `Err(NuwaError::InvalidArgument { ... })` with exit code 2.

**Fix applied:** Updated README.md to reflect the correct current behavior.

---

## 7. Phase 2 Status

| Question | Answer |
|----------|--------|
| **Phase 2 planning is allowed?** | **YES** |
| **Phase 2 coding is allowed?** | **NO** — Requires explicit user approval after Phase 1 final acceptance |
| **Phase 2 must NOT implement:** | `.nwb`, VSS, volume backup, disk backup, WinPE, system restore, cloning, differential/incremental backup, encryption |

### Recommended Phase 2 Tasks (for planning only)

1. Backup scheduler — timer-based periodic backup trigger
2. Retention policy — auto-delete old backup points by age or count
3. Backup history — log or database of backup operations
4. SMB network target support
5. GUI initial prototype
6. Improved CLI output (progress bar, JSON output)
7. Config file support

---

## 8. Baseline Freeze

The Phase 1 file-level backup/restore CLI is now a **frozen baseline**.

**Codex must not:**
- Rewrite, restructure, or expand the Phase 1 file backup core
- Modify backup, restore, verify, or list core logic
- Implement any Phase 2+ features

**Codex may:**
- Fix bugs in Phase 1 core (with explicit user approval)
- Create Phase 2 planning documents and design artifacts
- Update documentation for accuracy
- Run quality gates

---

## 9. Final Documents

| Document | Location | Authority |
|----------|----------|-----------|
| Phase 1 Closing Report | `docs/phase-1/Phase_1_Closing_Report.md` | AUTHORITATIVE |
| Phase 1 Final Acceptance Report | `docs/phase-1/Phase_1_Final_Acceptance_Report.md` | AUTHORITATIVE |
| Phase 1 Technical Baseline | `docs/phase-1/Phase_1_Technical_Baseline.md` | AUTHORITATIVE |
| Phase 1 Test Evidence | `docs/phase-1/Phase_1_Test_Evidence.md` | AUTHORITATIVE |
| Phase 1 Known Limitations and Risks | `docs/phase-1/Phase_1_Known_Limitations_and_Risks.md` | AUTHORITATIVE |
| Phase 1 Code Map | `docs/phase-1/Phase_1_Code_Map.md` | REFERENCE |
| Phase 1 → Phase 2 Handoff | `docs/phase-1/Phase_1_to_Phase_2_Handoff.md` | AUTHORITATIVE |
| Phase 1 Manual Test Plan | `docs/phase-1/Phase_1_Manual_Test_Plan.md` | REFERENCE |
