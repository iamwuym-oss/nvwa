# Nüwa Backup — Phase 2.5 Current Progress Report

**Document Type:** Current Progress Tracking (NOT a closing report)
**Version:** 2.0
**Date:** 2026-07-09

> ⚠️ This document tracks Phase 2.5 progress. Phase 2.5 is **not closed** because History and Schedule pages were placeholders at draft time (now implemented in 0a8ed93), and T2.5-04D Backup Content Browser was recently committed. See [Phase_2_5_Current_Status.md](./Phase_2_5_Current_Status.md) for the latest state.

## 1. Phase Summary

Phase 2.5 migrated the desktop GUI from **egui + eframe** to **Tauri 2.0 + React + TypeScript + Vite**,
and established the **Application Service Layer** as the bridge between UI and Core Engine.

### What Phase 2.5 Achieved

| Milestone | Deliverable | Status |
|-----------|-------------|:------:|
| GUI technology migration | egui removed, Tauri 2.0 direction confirmed | 鉁?DONE |
| Tauri scaffold | src-tauri/ with commands, AppState, capabilities | 鉁?DONE |
| React frontend | ui/ with 7 page stubs, dark enterprise theme | 鉁?DONE |
| Application Layer | src/app/ with models, services, error types | 鉁?DONE |
| Dashboard real data | DashboardOverview flow from Core -> React | 鉁?DONE |
| Dashboard polish | Skeleton, empty, error, no-history states | 鉁?DONE |
| Documentation sync | AGENTS.md, ENGINEERING_MEMORY, DOC_INDEX updated | 鉁?DONE |

---

## 2. Current Architecture

```
React UI (ui/)
    |
    | Tauri invoke()
    v
Tauri Command Layer (src-tauri/src/commands/)
    | - Parameter validation only
    | - No business logic
    v
Application Service Layer (src/app/services/)
    | - Data aggregation and orchestration
    | - Product-level model mapping
    v
Core Engine (src/)
    | - Backup, restore, verify, storage
    v
File System / SQLite
```

### Layer Count

| Layer | Directory | Language | Files |
|-------|-----------|----------|:-----:|
| Frontend | ui/src/ | TypeScript + React | ~25 |
| Tauri Commands | src-tauri/src/commands/ | Rust | 6 |
| Application Layer | src/app/ | Rust | 15 |
| Core Engine | src/ | Rust | ~15 |
| Tests | tests/ | Rust | 23 integration + 75 unit |

---

## 3. Completed Tasks

| Task | Scope | Status |
|:----:|-------|:------:|
| T2.5-00 | Remove egui GUI, clean Cargo.toml, document migration | 鉁?COMMITTED |
| T2.5-01 | Tauri 2.0 scaffold, command bridge, React frontend | 鉁?COMMITTED |
| T2.5-02 | Dashboard UI architecture (7 pages, mock data, layout) | 鉁?COMMITTED |
| T2.5-03A | Application API Layer (models, services, error, Tauri bridge) | 鉁?COMMITTED |
| T2.5-03A.1 | Dashboard product polish (skeleton, empty/error states) | 鉁?COMMITTED |
| T2.5-03B | Backup Application Service (models, service, Tauri cmd, tests) | 鉁?COMMITTED |
| T2.5-03C | Backup UI Integration (Backup page, job list, run, empty state) | 鉁?COMMITTED |
| T2.5-03D | Restore Service + UI (preview, execute, restore page) | 鉁?COMMITTED |
| T2.5-03D.1 | Restore Safety Hardening (path traversal, rename reject, .gitignore) | 鉁?COMMITTED |
| T2.5-04A | Config Job CRUD Service (part of Settings backend) | 鉁?COMMITTED |
| T2.5-04B | Settings Backup Plan UI (plan list, create form in Settings) | 鉁?COMMITTED |
| T2.5-04C | Native Path Picker (tauri-plugin-dialog, superseded by 04C.1) | 鉁?COMMITTED |
| T2.5-04C.1 | In-App File Browser (replaced OS dialog with FileBrowserModal) | 鉁?COMMITTED |
| T2.5-04D | Backup Content Browser (Restore 3-column layout + BackupTreeView) | 馃攧 UNCOMMITTED |
| T2.5-DOC-01A | Documentation synchronization | 鉁?COMMITTED |
| T2.5-DOC-02 | AGENTS + README final alignment | 鉁?COMMITTED |## 4. Architecture Decisions Made

| ID | Decision | Rationale |
|:--:|----------|-----------|
| ADR-T2.5-001 | Replace egui with Tauri 2.0 + React | egui visual quality ceiling too low for commercial backup product |
| ADL-T25-002 | Introduce src/app/ Application Layer | Prevent UI-Core coupling, enable enterprise model reuse |
| ADL-T25-003 | Return structured data, not UI strings | One model serves Desktop, CLI, and future Server API |

---

## 5. Current Limitations

1. **History page is a placeholder** 鈥?4-line stub, no backend service
2. **Schedule page is a placeholder** 鈥?4-line stub, no backend service
3. **Clone page is disabled** 鈥?12-line "future phase" placeholder
4. **T2.5-04D is uncommitted** 鈥?Restore 3-column layout and BackupTreeView pending commit
5. **Backup Content Browser is UI-only enhancement** 鈥?uses mock data, not a full enterprise catalog browser
6. **No Tauri command integration tests** 鈥?service tests exist but invoke chain is untested
7. **Node.js toolchain required** 鈥?~200MB dependency for frontend development

### Pages That Are Complete

| Page | Lines | Status |
|------|:-----:|:------:|
| Dashboard | 465 | 鉁?Complete 鈥?real data, health status, activity list, storage chart |
| Settings | 764 | 鉁?Complete 鈥?Backup Plan CRUD (list/create/edit/delete) |
| Restore | 628 | 鉁?3-column layout 鈥?plan grouping, file tree, restore form (04D uncommitted) |
| Backup | 421 | 鉁?Complete 鈥?job list, run backup, empty state (depends on Settings plans) |

### Pages Still Placeholder

| Page | Lines | Status |
|------|:-----:|:------:|
| History | 4 | 鉂?Placeholder 鈥?"Backup history coming soon." |
| Schedule | 4 | 鉂?Placeholder 鈥?"Schedule management coming soon." |
| Clone | 12 | 鉂?Disabled 鈥?"planned for a future phase" |## 6. Frozen Core Modules

The following modules are frozen and must NOT be modified:

| Module | File | Status |
|--------|------|--------|
| Backup engine | src/backup.rs | FROZEN |
| Restore engine | src/restore.rs | FROZEN |
| Verification | src/verify.rs | FROZEN |
| Manifest | src/manifest.rs | FROZEN |
| Checksum | src/checksum.rs | FROZEN |
| Storage | src/storage.rs | FROZEN |
| Prune | src/prune.rs | FROZEN |

---

## 7. All Tests Passing

| Suite | Count | Result |
|-------|:-----:|:------:|
| Unit tests (src/lib.rs) | 92 | 鉁?ALL PASS |
| Backup/restore integration (tests/) | 19 | 鉁?ALL PASS |
| Backup service tests | 7 | 鉁?ALL PASS |
| Config service tests | 15 | 鉁?ALL PASS |
| Dashboard service tests | 4 | 鉁?ALL PASS |
| File browser service tests | 9 | 鉁?ALL PASS |
| Restore service tests | 6 | 鉁?ALL PASS |
| **Total** | **152** | **鉁?ALL PASS** |## 8. Current Test State (same as section 7)

| Suite | Count | Result |
|-------|:-----:|:------:|
| Unit tests (src/lib.rs) | 92 | ALL PASS |
| Backup/restore integration | 19 | ALL PASS |
| Backup service tests | 7 | ALL PASS |
| Config service tests | 15 | ALL PASS |
| Dashboard service tests | 4 | ALL PASS |
| File browser service tests | 9 | ALL PASS |
| Restore service tests | 6 | ALL PASS |
| **Total** | **152** | **ALL PASS** |## 9. Tauri Commands Registered

14 commands across 6 modules:
- **dashboard** (1): get_dashboard_overview
- **backup** (3): list_backup_jobs, get_backup_job_detail, run_backup
- **config** (5): list_job_configs, get_job_config, create_job_config, update_job_config, delete_job_config
- **restore** (3): list_restore_points, get_restore_preview, execute_restore
- **file_browser** (2): list_roots, list_directory## 10. Completed Beyond v1.1

| Task | Scope | Status |
|:----:|-------|:------:|
| T2.5-03B | Backup Application Service (models, service, Tauri cmd, tests) | COMMITTED |
| T2.5-03C | Backup UI Integration (Backup page, job list, run, empty state) | COMMITTED |
| T2.5-04A | Config Job CRUD Service (plan list, create, update, delete) | PENDING COMMIT |
| T2.5-04B | Settings Backup Plan UI (plan list, create form in Settings) | PENDING COMMIT |
| T2.5-03D | Restore Service + UI (preview, execute, restore page) | PENDING COMMIT |
| T2.5-03D.1 | Restore Safety Hardening (path traversal, rename reject, .gitignore) | PENDING COMMIT |

## 11. Frozen Core Modules

The following modules are frozen baselines. restore.rs is FROZEN except for approved safety hardening (path traversal protection):

| Module | File | Status |
|--------|------|--------|
| Backup engine | src/backup.rs | FROZEN |
| Restore engine | src/restore.rs | FROZEN (approved safety hardening: path validation) |
| Verification | src/verify.rs | FROZEN |
| Manifest | src/manifest.rs | FROZEN |
| Checksum | src/checksum.rs | FROZEN |
| Storage | src/storage.rs | FROZEN |
| Prune | src/prune.rs | FROZEN |

## 12. Next Phase

### T2.5-03E -- History / Schedule / Remaining Pages

Complete the remaining product pages with real Application Layer integration.

