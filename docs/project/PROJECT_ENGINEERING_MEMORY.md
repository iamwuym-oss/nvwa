# Nüwa Backup — Project Engineering Memory

**Version:** 0.6.0
**Last Updated:** 2026-07-13 (All Repository references removed)
**Current Phase:** Storage Engine Redesign

---

## 1. Purpose

This document records the essential project state — what phase we are in, what has been done, what is allowed and forbidden.

---

## 2. Project Identity

Nüwa Backup is a local-first, single-machine backup and disaster recovery product for Windows Workstation, Windows Server, power users, small offices, PC repair shops, and edge nodes.

Core principles:
- Data recoverability is more important than development speed
- Backup is a user-visible file (like Acronis .tib), not a directory tree
- Local-first, single-machine focus
- Staged delivery: file-level CLI → desktop GUI → volume backup → system recovery

---

## 3. Completed Baselines

| Phase | Scope | Status |
|-------|-------|--------|
| Phase 1 | File-level backup/restore CLI (Rust) | CLOSED |
| Phase 2 | CLI usability: config, history, scheduler, SMB, UNC, egui GUI | CLOSED |
| Phase 2.5 | Tauri 2.0 desktop GUI, React frontend, Application Layer | CLOSED |

Phase 1 core modules (backup.rs, restore.rs, verify.rs, manifest.rs, checksum.rs, storage.rs, prune.rs) are frozen baselines but backup/restore operations are disconnected pending storage engine redesign.

---

## 4. Current Architecture

```
React UI
    |
Tauri invoke() IPC
    |
Tauri Command Layer
    |
Application Service Layer (src/app/)
    |
New Storage Engine (under design)
```

### Source Code Structure

```
src/
  ├── main.rs           — CLI entry (init, history, schedule only)
  ├── lib.rs            — Library root
  ├── cli.rs            — CLI argument parsing
  ├── checksum.rs       — SHA-256 helpers
  ├── config.rs         — Configuration management
  ├── diskspace.rs      — Disk space queries
  ├── errors.rs         — Error types
  ├── history.rs        — Backup history SQLite DB
  ├── path_support.rs   — Path validation
  ├── scheduler.rs      — Windows Task Scheduler integration
  ├── cli_output.rs     — CLI output formatting
  ├── app/
  │   ├── error.rs
  │   ├── models/       — API models for Tauri/React
  │   └── services/     — Application services (stubs during redesign)
  │
src-tauri/              — Tauri command layer
ui/                     — React frontend
```

---

## 5. Phase Boundaries

| Phase | Scope | Status |
|-------|-------|--------|
| Phase 0 | Project setup, MVP boundary, guardrails | COMPLETE |
| Phase 1 | File-level backup/restore CLI | CLOSED |
| Phase 2 | CLI usability + egui GUI | CLOSED |
| Phase 2.5 | Tauri desktop GUI + Application Layer | CLOSED |
| Phase 3 | NTFS non-system volume image, VSS, block backup | PLANNING |
| Phase 4 | WinPE recovery media, system restore, BCD repair | NOT AUTHORIZED |
| Phase 5 | Disk cloning | NOT AUTHORIZED |
| Phase 6+ | Differential backup, encryption, cross-platform | NOT AUTHORIZED |

### Permanent Exclusions

- Cloud backup, cloud sync, antivirus, ransomware protection
- AI threat detection, enterprise centralized management
- Multi-device dashboard, SaaS account system

---

## 6. Build Status

| Command | Status |
|---------|--------|
| cargo build (nuwa-backup) | PASS |
| cargo build (nuwa-tauri) | PASS |
| cargo fmt --check | PASS |
| cargo clippy --all-targets -- -D warnings | PASS |

Test count reduced from ~370 to approximately 50 (repository tests removed). Remaining tests cover config, history, scheduler, and app service layer.

---

## 7. Known Limitations

- Backup and restore operations are disconnected pending storage engine redesign
- Phase 1 core modules (backup.rs, restore.rs, verify.rs, manifest.rs, storage.rs) are frozen but operations are disconnected — they need reconnection to new engine
- No volume backup or system recovery capability yet
- Windows-only (schtasks, path conventions)
