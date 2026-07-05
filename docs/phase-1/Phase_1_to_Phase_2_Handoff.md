# Phase 1 → Phase 2 Handoff

**From:** Phase 1 — Minimal File Backup/Restore CLI
**To:** Phase 2
**Date:** 2026-07-05
**Phase 1 Status:** PARTIAL

---

## 1. What Phase 1 Completed

### CLI Commands
- `nuwa backup --source <path> --dest <path> [--compress]`
- `nuwa restore --backup <path> --dest <path> [--overwrite]`
- `nuwa verify --backup <path>`
- `nuwa list --dest <path>`

### Core Capabilities
- SHA-256 per-file checksum at backup and restore.
- Atomic write engine (.tmp → rename, cleanup on failure).
- JSON manifest schema v1.0.
- Recursive directory traversal (skips symlinks).
- Path safety checks (src=dst, dest-in-src, src-in-dest).
- Destination space check (Windows: GetDiskFreeSpaceExW; others: writable check).
- Locked file detection (best-effort) during restore.
- Error code system (8 exit codes).
- 22 passing tests (3 unit + 19 integration).

### Quality Gates
- `cargo fmt --check` — PASS
- `cargo clippy --all-targets -- -D warnings` — PASS
- `cargo build` — PASS
- `cargo test` — PASS (22/22)

---

## 2. What Phase 2 Can Reuse

| Capability | Reusable? | Notes |
|------------|-----------|-------|
| CLI argument parser | YES | Manual parser in `cli.rs`, extendable with new subcommands |
| Error type system | YES | `NuwaError` + `ExitCode` mapping covers 8 codes |
| Manifest model | YES | `Manifest` struct can be extended with new fields |
| SHA-256 helpers | YES | `sha256_file`, `verify_file_checksum` |
| Atomic write engine | YES | `copy_file_with_atomic_write`, `restore_file` |
| Recursive walk | YES | `walk_dir` in `backup.rs` |
| Path safety checks | YES | `canonicalize_partial`, containment checks |
| Disk space check | YES | Win32 FFI in `diskspace.rs` |
| Module structure | YES | `lib.rs` exports all modules for testing |

---

## 3. Recommended First Tasks for Phase 2

Phase 2 can consider (not exhaustive, not mandatory):

1. **Backup scheduler** — Timer-based or cron-like trigger for periodic backup.
2. **Retention policy** — Auto-delete old backup points by age or count.
3. **Backup history** — Log or database of backup operations.
4. **SMB network target** — Support `--dest \\server\share\path`.
5. **GUI initial prototype** — Simple TUI or minimal desktop UI.
6. **Improved CLI output** — Progress bar, summary table, JSON output.
7. **Config file** — Persistent configuration for default source/dest.

---

## 4. Phase 2 Boundaries

### Allowed
- Scheduler / periodic backup triggers.
- Retention policies (keep N days, keep N backups).
- Backup history tracking.
- SMB network share as destination (requires auth handling).
- GUI or TUI (initial prototype, not full-featured).
- Configuration file.
- Improved error handling and user feedback.

### NOT Allowed (Phase 3+ or later)
- `.nwb` image format (Phase 3).
- VSS snapshot integration (Phase 3).
- Volume-level backup (Phase 3).
- Block-level backup (Phase 3).
- WinPE recovery media (Phase 4).
- System volume backup / system restore (Phase 4).
- BCD repair (Phase 4).
- Disk cloning (Phase 5).
- Differential backup (Phase 6+).
- Incremental backup (Excluded / Not planned).
- Heterogeneous restore / cross-platform (Phase 6+).
- AES encryption (Phase 6+).
- Linux / domestic OS support (Phase 6+).

---

## 5. Known Issues Phase 2 Should Address

| Issue | Phase 1 Status | Recommendation |
|-------|---------------|----------------|
| Non-Windows space check | PARTIAL | Consider portable disk-space crate or Win32 FFI fallback |
| Locked file detection | Best-effort | Improve coverage or add retry/warning |
| 10GB+ large file test | Manual only | Add automated large-file test |
| `--compress` error | Fixed | Keep the fix; add compress feature properly |
| Performance | Not optimized | Phase 2 may start basic optimization (parallel hashing, buffer tuning) |

---

## 6. Summary

Phase 1 delivers a working, verified file-level backup/restore CLI with
SHA-256 integrity protection, atomic write safety, and basic path safety
checks. Phase 2 can build directly on this foundation for scheduling,
retention, network targets, and user interface.

**Phase 1 overall status: PARTIAL** (non-Windows space check limitation).
**Phase 2 planning is allowed.** Phase 2 coding requires explicit user approval after Phase 1 final acceptance. Phase 2 boundaries must be respected.