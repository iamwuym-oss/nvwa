# Nüwa Backup — T2.5-03C: Backup UI Integration

**Version:** 1.0
**Date:** 2026-07-08
**Status:** DONE / PASS

---

## 1. Task Summary

Integrated the Backup UI page with the real Application Service Layer, proving that the full React → Tauri → Service → Core data flow works for a second business domain (Backup).

### Goal

Replace the Backup page stub with a fully functional page connected to `backupApi.ts`, `backup_service.rs`, and the Tauri command bridge — reusing components from Dashboard polish.

---

## 2. Files Changed

### New Files

| File | Purpose |
|------|---------|
| `ui/src/pages/Backup.tsx` | Full Backup page with job list, run backup, status indicators |

### Modified Files

| File | Change |
|------|--------|
| `src-tauri/src/commands/mod.rs` | (already updated in T2.5-03B) |
| `src-tauri/src/lib.rs` | (already updated in T2.5-03B) |

No frozen core modules were touched.

---

## 3. Backup Page Features

### Job List
- Table view of all configured backup jobs from `list_backup_jobs()`
- Columns: Job Name, Source, Destination, Schedule, Last Run, Last Status
- Status badges with color coding (green=Completed, red=Failed, yellow=Running, gray=Idle/Scheduled)

### Run Backup Action
- Each job row has a "Run Backup" button
- Calls `run_backup()` via `backupApi.ts` → Tauri invoke → BackupService → Core
- Displays result toast with file count, total bytes, duration

### States Implemented
- **Loading**: Skeleton cards while fetching (reuses Dashboard pattern)
- **Empty**: "No backup jobs configured" with CTA guidance
- **Error**: Unified error card with Retry button
- **Normal**: Job list table with last run status
- **Success toast**: After backup completes

### UI Architecture

```
Backup.tsx
    |
    useDashboardData() pattern (similar to Dashboard)
    |
    backupApi.ts
    |    |
    |    +-- invoke("list_backup_jobs") — real data
    |    +-- invoke("run_backup") — real data
    |    +-- mock fallback when VITE_MOCK_DATA=true
    |
    Tauri Command (commands/backup.rs)
    |
    BackupService (services/backup_service.rs)
    |
    Core Engine (backup.rs) — FROZEN
```

---

## 4. Component Reuse

The Backup page reuses the following patterns established in T2.5-03A.1:

| Component/Pattern | Source | Reused For |
|-------------------|--------|------------|
| Skeleton loading | MetricCard skeleton | Backup job cards loading |
| Empty state pattern | Dashboard empty state | "No backup jobs" state |
| Error card pattern | Dashboard error card | Backup API errors |
| Status badge colors | Dashboard status indicators | Backup job status display |
| API layer pattern | `dashboardApi.ts` | `backupApi.ts` structure |
| VITE_MOCK_DATA guard | `dashboardApi.ts` | `backupApi.ts` mock fallback |

---

## 5. Quality Gates

| Gate | Result |
|------|:------:|
| `cargo fmt --check` | ✅ PASS |
| `cargo clippy --all-targets -- -D warnings` | ✅ PASS |
| `cargo build` | ✅ PASS |
| `cargo test` | ✅ PASS (105) |
| `pnpm tsc` | ✅ ZERO errors |
| `pnpm vite build` | ✅ PASS |
| Forbidden scope audit | ✅ No frozen core modules modified |

---

## 6. Full Test Suite (105 tests ALL PASS)

```
test result: ok. 75 passed (unit tests)
test result: ok. 19 passed (integration tests)
test result: ok. 7 passed (backup service tests)
test result: ok. 4 passed (dashboard service tests)
```

---

## 7. Architecture Verification

The following data flows are now proven working:

| Flow | Status |
|------|:------:|
| Dashboard: React → invoke() → dashboard_command → dashboard_service → config/history/scheduler/diskspace | ✅ PROVEN (T2.5-03A) |
| Backup: React → invoke() → backup_command → backup_service → backup.rs | ✅ PROVEN (T2.5-03C) |
| Task model shared across domains | ✅ VERIFIED |
| Component library reusable across pages | ✅ VERIFIED |

---

## 8. Known Limitations

1. No real-time progress streaming for running backups (blocking call)
2. No cancellation support for in-progress backups
3. Backup page does not yet include restore integration (future T2.5-03D)
4. Table pagination not implemented (assumes reasonable job count)

---

## 9. Next Recommended Task

**T2.5-03D — Restore Application Service Layer + UI**

Goal: Complete the Restore page following the same pattern:
1. `src/app/models/restore.rs`
2. `src/app/services/restore_service.rs`
3. `src-tauri/src/commands/restore.rs`
4. `ui/src/api/restoreApi.ts`
5. `ui/src/pages/Restore.tsx`
6. Unit tests
