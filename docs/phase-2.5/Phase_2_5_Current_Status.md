# Nüwa Backup — Phase 2.5 Current Status

**Last Updated:** 2026-07-09 (Updated with Schedule/History completion)
**Status:** IN PROGRESS
**Latest Commit:** fa60dd9 — feat: add backup content browser with three-column restore layout

---

## 1. What Phase 2.5 Is

Phase 2.5 migrates the desktop GUI from egui + eframe to **Tauri 2.0 + React + TypeScript + Vite** and establishes the **Application Service Layer** (src/app/) as the bridge between UI and Core Engine.


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
| Application Layer | src/app/ | 17 .rs (models:8 + services:7 + error + mod) |
| Tauri Commands | src-tauri/src/commands/ | 8 .rs |
| Frontend Pages | ui/src/pages/ | 7 .tsx |
| Components | ui/src/components/ | 18 .tsx/.ts |
| API Bridge | ui/src/api/ | 7 .ts |

---

## 3. Real Application Layer

### Models (src/app/models/)

| File | Lines | Contents |
|------|:-----:|----------|
| backup.rs | 75 | BackupJobView, BackupRequest, BackupResult, BackupJobStatus |
| common.rs | 53 | ProtectionStatus, HealthStatus, StorageUsage |
| config_job.rs | 50 | JobConfig, JobConfigView, CreateJobConfigRequest |
| dashboard.rs | 78 | DashboardOverview, BackupSummary, ActivityEntry |
| history.rs | ? | OperationRecord, HistoryQuery (HistoryDb integration) |
| restore.rs | 88 | RestorePointView, RestorePreview, RestoreRequest, RestoreFileEntry |
| schedule.rs | 80 | ScheduleProfileView, ScheduleProfileRequest, ScheduleDeleteResult, TriggerType, TaskSyncStatus |
| task.rs | 98 | TaskType, TaskState, TaskProgress, TaskInfo (unified task model) |

### Services (src/app/services/)

| File | Lines | API |
|------|:-----:|-----|
| backup_service.rs | 271 | list_jobs, get_job_detail, run_backup, run_backup_dry |
| config_service.rs | 219 | list_jobs, create_job, update_job, delete_job, get_job_detail |
| dashboard_service.rs | 199 | get_overview |
| file_browser_service.rs | 191 | list_roots, list_directory |
| history_service.rs | ? | get_history, query_by_date, query_by_job (HistoryDb wrapper) |
| restore_service.rs | 264 | list_restore_points, get_restore_preview, execute_restore |
| schedule_service.rs | 500+ | list_schedules, get_schedule, create/update/delete/enable/disable, sync_create/remove_tasks, sync_schedule_trigger_update |

### Tauri Commands (src-tauri/src/commands/)

| File | Lines | Commands |
|------|:-----:|----------|
| backup.rs | 32 | list_backup_jobs, get_backup_job_detail, run_backup |
| config.rs | 44 | list_job_configs, get_job_config, create_job_config, update_job_config, delete_job_config |
| dashboard.rs | 24 | get_dashboard_overview |
| file_browser.rs | 19 | list_roots, list_directory |
| history.rs | ? | get_history, query_history_by_date, query_history_by_job |
| restore.rs | 34 | list_restore_points, get_restore_preview, execute_restore |
| schedule.rs | 80+ | list_schedules, get_schedule, create/update/delete_schedule, enable/disable_schedule |

**Total: 23+ commands across 8 modules**

---

## 4. Real UI Page State

| Page | Lines | Status | Notes |
|------|:-----:|:------:|-------|
| Dashboard | 465 | ✅ Complete | Real data flow: Core → Service → Command → React |
| Settings | 764 | ✅ Complete | Backup Plan CRUD (list/create/edit/delete via config_service) |
| Restore | 628 | ✅ 3-column | Plan grouping + BackupTreeView + restore form (committed 0a8ed93) |
| Backup | 421 | ✅ Complete | Job list, run backup, empty state. Reads from Settings-created plans |
| History | 400+ | ✅ Complete | Real data flow: HistoryDb → Service → Command → React |
| Schedule | 700+ | ✅ Complete | ScheduleProfile CRUD, two-phase delete, schtasks sync, Schedule assignment in Settings |
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

## 6. Current Shortboards

1. Schedule schtasks sync is best-effort (errors logged, not UI-visible)
2. Schedule task_sync_status field exists but not displayed in Schedule page UI yet
3. **Backup Content Browser** is frontend-only enhancement (mock data) — not a full enterprise catalog
4. **No Tauri command integration tests** — only service-level tests exist
5. **T2.5-04D is uncommitted**

---

## 7. What NOT to Write in Future

- Phase 2.5 is NOT closed
- History/Schedule are NOW IMPLEMENTED (committed in 0a8ed93)
- Clone is NOT implemented
- Backup Content Browser is NOT 100% complete — it's a frontend tree on mock data
- There is NO BackupPlan abstraction — JobConfig is the factual model
- OS native dialog (tauri-plugin-dialog) has been REMOVED — the in-app FileBrowserModal is the current solution
- Restore destination is NOT restricted by a system directory blacklist
- Core Engine (backup/restore/etc.) is NOT being refactored
