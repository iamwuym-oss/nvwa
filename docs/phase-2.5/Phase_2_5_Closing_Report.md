# Nüwa Backup — Phase 2.5 Closing Report

**Version:** 1.1
**Date:** 2026-07-08
**Status:** CLOSED — Architecture Established

---

## 1. Phase Summary

Phase 2.5 migrated the desktop GUI from **egui + eframe** to **Tauri 2.0 + React + TypeScript + Vite**,
and established the **Application Service Layer** as the bridge between UI and Core Engine.

### What Phase 2.5 Achieved

| Milestone | Deliverable | Status |
|-----------|-------------|:------:|
| GUI technology migration | egui removed, Tauri 2.0 direction confirmed | ✅ DONE |
| Tauri scaffold | src-tauri/ with commands, AppState, capabilities | ✅ DONE |
| React frontend | ui/ with 7 page stubs, dark enterprise theme | ✅ DONE |
| Application Layer | src/app/ with models, services, error types | ✅ DONE |
| Dashboard real data | DashboardOverview flow from Core -> React | ✅ DONE |
| Dashboard polish | Skeleton, empty, error, no-history states | ✅ DONE |
| Documentation sync | AGENTS.md, ENGINEERING_MEMORY, DOC_INDEX updated | ✅ DONE |

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
| Tauri Commands | src-tauri/src/commands/ | Rust | 2 |
| Application Layer | src/app/ | Rust | 7 |
| Core Engine | src/ | Rust | ~15 |
| Tests | tests/ | Rust | 23 integration + 75 unit |

---

## 3. Completed Tasks

| Task | Scope | Result |
|:----:|-------|:------:|
| T2.5-00 | Remove egui GUI, clean Cargo.toml, document migration | ✅ PASS |
| T2.5-01 | Tauri 2.0 scaffold, command bridge, React frontend | ✅ PASS |
| T2.5-02 | Dashboard UI architecture (7 pages, mock data, layout) | ✅ PASS |
| T2.5-03A | Application API Layer (models, services, error, Tauri bridge) | ✅ PASS |
| T2.5-03A.1 | Dashboard product polish (skeleton, empty/error states) | ✅ PASS |
| T2.5-DOC-01 | Documentation synchronization | ✅ PASS |

---

## 4. Architecture Decisions Made

| ID | Decision | Rationale |
|:--:|----------|-----------|
| ADR-T2.5-001 | Replace egui with Tauri 2.0 + React | egui visual quality ceiling too low for commercial backup product |
| ADL-T25-002 | Introduce src/app/ Application Layer | Prevent UI-Core coupling, enable enterprise model reuse |
| ADL-T25-003 | Return structured data, not UI strings | One model serves Desktop, CLI, and future Server API |

---

## 5. Current Limitations

1. **Only Dashboard page is fully implemented** — Backup, Restore, History, Schedule, Settings pages exist as stubs only
2. **Chart data is static** — hardcoded in Dashboard.tsx, not from backend
3. **No Tauri command integration tests** — dashboard_service tests exist but Tauri invoke chain is untested
4. **CTA buttons in empty state are disabled** — "Create Backup" and "Import Configuration" are visual-only
5. **Node.js toolchain required** — ~200MB dependency for frontend development

---

## 6. Frozen Core Modules

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
| Unit tests (src/lib.rs) | 75 | ✅ ALL PASS |
| Backup/restore integration | 19 | ✅ ALL PASS |
| Backup service tests | 7 | ✅ ALL PASS |
| Dashboard service tests | 4 | ✅ ALL PASS |
| **Total** | **105** | **✅ ALL PASS** |

---

## 8. Current Test State

| Suite | Count | Result |
|-------|:-----:|:------:|
| Unit tests (src/lib.rs) | 92 | ALL PASS |
| Backup/restore integration | 19 | ALL PASS |
| Backup service tests | 7 | ALL PASS |
| Config service tests | 15 | ALL PASS |
| Dashboard service tests | 4 | ALL PASS |
| Restore service tests | 6 | ALL PASS |
| **Total** | **143** | **ALL PASS** |

## 9. Tauri Commands Registered

13 commands across 4 modules:
- dashboard: get_dashboard_overview
- backup: list_backup_jobs, get_backup_job_detail, run_backup  (3 commands)
- config: list_job_configs, get_job_config, create_job_config, update_job_config, delete_job_config  (5 commands)
- restore: list_restore_points, get_restore_preview, execute_restore  (3 commands)

## 10. Completed Beyond v1.1

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
