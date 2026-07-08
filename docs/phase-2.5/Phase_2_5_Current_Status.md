# Nüwa Backup — Phase 2.5 Current Status

**Last Updated:** 2026-07-09
**Status:** IN PROGRESS
**Latest Commit:** 4962910 — fix: replace native dialog with in-app file browser

---

## 1. What Phase 2.5 Is

Phase 2.5 migrates the desktop GUI from egui + eframe to **Tauri 2.0 + React + TypeScript + Vite** and establishes the **Application Service Layer** (src/app/) as the bridge between UI and Core Engine.

Phase 2.5 is **NOT closed**. History and Schedule pages remain placeholders.

---

## 2. Real Architecture

```
React UI (ui/src/pages/)
    | invoke()
    v
Tauri Command Layer (src-tauri/src/commands/) — thin wrapper, no business logic
    |
    v
Application Service Layer (src/app/services/) — business orchestration, data aggregation
    |
    v
Core Engine (src/ — backup/restore/verify/manifest/checksum/storage/prune)
```

### File Counts

| Layer | Directory | Files |
|-------|-----------|:-----:|
| Rust Core Engine | src/ | 33 .rs |
| Application Layer | src/app/ | 15 .rs (models:6 + services:5 + error + mod) |
| Tauri Commands | src-tauri/src/commands/ | 6 .rs |
| Frontend Pages | ui/src/pages/ | 7 .tsx + 1 subdir |
| Components | ui/src/components/ | 18 .tsx/.ts |
| API Bridge | ui/src/api/ | 5 .ts |

---

## 3. Real Application Layer

### Models (src/app/models/)

| File | Lines | Contents |
|------|:-----:|----------|
| backup.rs | 75 | BackupJobView, BackupRequest, BackupResult, BackupJobStatus |
| common.rs | 53 | ProtectionStatus, HealthStatus, StorageUsage |
| config_job.rs | 50 | JobConfig, JobConfigView, CreateJobConfigRequest |
| dashboard.rs | 78 | DashboardOverview, BackupSummary, ActivityEntry |
| restore.rs | 88 | RestorePointView, RestorePreview, RestoreRequest, RestoreFileEntry |
| task.rs | 98 | TaskType, TaskState, TaskProgress, TaskInfo (unified task model) |

### Services (src/app/services/)

| File | Lines | API |
|------|:-----:|-----|
| backup_service.rs | 271 | list_jobs, get_job_detail, run_backup, run_backup_dry |
| config_service.rs | 219 | list_jobs, create_job, update_job, delete_job, get_job_detail |
| dashboard_service.rs | 199 | get_overview |
| file_browser_service.rs | 191 | list_roots, list_directory |
| restore_service.rs | 264 | list_restore_points, get_restore_preview, execute_restore |

### Tauri Commands (src-tauri/src/commands/)

| File | Lines | Commands |
|------|:-----:|----------|
| backup.rs | 32 | list_backup_jobs, get_backup_job_detail, run_backup |
| config.rs | 44 | list_job_configs, get_job_config, create_job_config, update_job_config, delete_job_config |
| dashboard.rs | 24 | get_dashboard_overview |
| file_browser.rs | 19 | list_roots, list_directory |
| restore.rs | 34 | list_restore_points, get_restore_preview, execute_restore |

**Total: 14 commands across 5 modules**

---

## 4. Real UI Page State

| Page | Lines | Status | Notes |
|------|:-----:|:------:|-------|
| Dashboard | 465 | ✅ Complete | Real data flow: Core → Service → Command → React |
| Settings | 764 | ✅ Complete | Backup Plan CRUD (list/create/edit/delete via config_service) |
| Restore | 628 | ✅ 3-column | Plan grouping + BackupTreeView + restore form (04D uncommitted) |
| Backup | 421 | ✅ Complete | Job list, run backup, empty state. Reads from Settings-created plans |
| History | 4 | ❌ Placeholder | "Backup history coming soon." — no backend service |
| Schedule | 4 | ❌ Placeholder | "Schedule management coming soon." — no backend service |
| Clone | 12 | ❌ Disabled | "Planned for a future phase" — visual-only |

---

## 5. Real Test State

| Suite | Count | Result |
|-------|:-----:|:------:|
| Unit tests (core) | 92 | ✅ ALL PASS |
| Backup/restore integration | 19 | ✅ ALL PASS |
| Backup service tests | 7 | ✅ ALL PASS |
| Config service tests | 15 | ✅ ALL PASS |
| Dashboard service tests | 4 | ✅ ALL PASS |
| File browser service tests | 9 | ✅ ALL PASS |
| Restore service tests | 6 | ✅ ALL PASS |
| **Total Rust** | **152** | **✅ ALL PASS** |
| TypeScript | — | **0 errors** |
| Vite build | — | **pass** |

---

## 6. Current Uncommitted Work

T2.5-04D — Backup Content Browser (3 files):

| File | Change |
|------|--------|
| ui/src/pages/Restore.tsx | Single-column → 3-column layout with plan grouping |
| ui/src/api/restoreApi.ts | Mock data enhanced: 3 plans, multiple versions, nested directories |
| ui/src/components/common/BackupTreeView.tsx | NEW: flat file list → expandable directory tree |

**Not yet committed.** These files are in the working tree.

---

## 7. Current Shortboards

1. **History page** is a 4-line placeholder — no backend service
2. **Schedule page** is a 4-line placeholder — no backend service
3. **Backup Content Browser** is frontend-only enhancement (mock data) — not a full enterprise catalog
4. **No Tauri command integration tests** — only service-level tests exist
5. **T2.5-04D is uncommitted**

---

## 8. What NOT to Write in Future

- Phase 2.5 is NOT closed
- History/Schedule are NOT implemented
- Clone is NOT implemented
- Backup Content Browser is NOT 100% complete — it's a frontend tree on mock data
- There is NO BackupPlan abstraction — JobConfig is the factual model
- OS native dialog (tauri-plugin-dialog) has been REMOVED — the in-app FileBrowserModal is the current solution
- Restore destination is NOT restricted by a system directory blacklist
- Core Engine (backup/restore/etc.) is NOT being refactored
