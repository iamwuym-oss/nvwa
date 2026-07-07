# Nüwa Backup — Phase 2.5 T2.5-03A: Application API Layer Foundation

**Version:** 1.0
**Date:** 2026-07-07
**Status:** DONE / PASS

---

## 1. Task Summary

Build the **Application Service Layer** (`src/app/`) between the Rust Core and Tauri commands,
so the React Dashboard gets real data instead of mockData.

### Architecture Goal

```
React Dashboard
    | invoke("get_dashboard_overview")
    v
Tauri Command (src-tauri/src/commands/dashboard.rs) -- thin wrapper
    |
    v
dashboard_service::get_overview() (src/app/services/dashboard_service.rs)
    |
    +-- config::Config::load()
    +-- history::HistoryDb::query()
    +-- scheduler::list_tasks()
    +-- diskspace::free_space()
    |
    v
DashboardOverview -> serialized -> React render
```

---

## 2. Files Created

### New: src/app/
| File | Purpose |
|------|---------|
| `src/app/mod.rs` | Module root |
| `src/app/error.rs` | AppError with 5 categories (Config/History/Storage/Permission/Internal) |
| `src/app/models/mod.rs` | Models module root |
| `src/app/models/common.rs` | ProtectionStatus, HealthStatus, JobStatus, OperationType enums |
| `src/app/models/dashboard.rs` | DashboardOverview, BackupSummary, StorageStatus, ActivityRecord |
| `src/app/services/mod.rs` | Services module root |
| `src/app/services/dashboard_service.rs` | get_overview() aggregation logic (read-only) |

### New: src-tauri/src/commands/
| File | Purpose |
|------|---------|
| `commands/mod.rs` | Module root |
| `commands/dashboard.rs` | get_dashboard_overview() — thin wrapper, calls service |

### New: tests/
| File | Purpose |
|------|---------|
| `tests/dashboard_service_tests.rs` | 4 test scenarios |

### New: ui/src/api/
| File | Purpose |
|------|---------|
| `dashboardApi.ts` | getDashboard() — mock toggle via VITE_MOCK_DATA env var |

### Modified
| File | Change |
|------|--------|
| `src/lib.rs` | Added pub mod app |
| `src/diskspace.rs` | Added pub fn free_space() (read-only query interface) |
| `src-tauri/src/lib.rs` | Registered get_dashboard_overview command |
| `ui/src/pages/Dashboard/Dashboard.tsx` | Changed from mockData import to getDashboard() API call |
| `ui/.env` | Added VITE_MOCK_DATA env var |

---

## 3. Design Decisions

### AppError Categories

| Category | Meaning | React Use |
|----------|---------|-----------|
| Config | Configuration missing or corrupt | Show setup guide |
| History | History database unavailable | Show "no history" state |
| Storage | Storage drive inaccessible | Show storage error |
| Permission | Permission denied | Show permission tip |
| Internal | Unexpected error | Show generic error with retry |

### DashboardService is READ-ONLY

The service only aggregates data. It never creates, modifies, or deletes:
- Configuration
- History records
- Scheduled tasks
- Backup data

### Tauri Command Layer is a THIN WRAPPER

Commands only:
1. Receive parameters
2. Call the corresponding service
3. Convert errors
4. Return results

No business logic in commands.

---

## 4. Quality Gates

| Gate | Result |
|------|:------:|
| cargo fmt --check | ✅ PASS |
| cargo clippy --all-targets -- -D warnings | ✅ PASS |
| cargo build | ✅ PASS |
| cargo test (98 tests) | ✅ ALL PASS |
| npm run build (ui/) | ✅ PASS |

### Test Scenarios (4 passing)

| # | Scenario | Expected |
|:-:|----------|----------|
| 1 | With jobs + with history | ProtectionStatus::Protected |
| 2 | With jobs + no history | ProtectionStatus::Critical |
| 3 | No config | AppError category = Config |
| 4 | Corrupt config | AppError category = Config |

---

## 5. Forbidden Scope Check

- Not modified: backup.rs, restore.rs, verify.rs, prune.rs, manifest.rs, checksum.rs, storage.rs
- No new features beyond Dashboard data aggregation
- DashboardService is read-only
- Mock data only via VITE_MOCK_DATA env toggle

---

## 6. Known Limitations

1. Tauri command integration test not yet implemented (deferred to T2.5-03B+)
2. Chart data in Dashboard is still static (hardcoded in Dashboard.tsx)
3. Dashboard tests require --test-threads=1 due to CWD-based config search

---

## 7. Next Recommended Task

**T2.5-03A.1 — Dashboard Product Polish**
