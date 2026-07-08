# Nüwa Backup — T2.5-03B: Backup Application Service Foundation

**Version:** 1.0
**Date:** 2026-07-08
**Status:** DONE / PASS

---

## 1. Task Summary

Established the Backup Application Service Layer as the second business domain (after Dashboard), proving that the Application Layer architecture is reusable and extensible.

### Goal

Build `BackupService` with full data contracts, Tauri command bridge, React API layer, and unit tests — **without** modifying frozen core modules (`backup.rs`, `restore.rs`, etc.).

---

## 2. Files Changed

### New Files

| File | Purpose |
|------|---------|
| `src/app/models/task.rs` | Unified Task model: `TaskType`, `TaskState`, `TaskProgress`, `TaskResult`, `TaskInfo` |
| `src/app/models/backup.rs` | Backup domain models: `BackupJobView`, `BackupRequest`, `BackupResult`, `BackupJobStatus` |
| `src/app/services/backup_service.rs` | BackupService: `list_jobs()`, `get_job_detail()`, `run_backup()`, `run_backup_dry()` |
| `src-tauri/src/commands/backup.rs` | Tauri command layer (thin wrapper): 3 commands |
| `ui/src/api/backupApi.ts` | React API layer: `listJobs()`, `runBackup()`, `getJobDetail()` + mock fallback |
| `tests/backup_service_tests.rs` | 7 unit tests covering all backup service scenarios |

### Modified Files

| File | Change |
|------|--------|
| `src/app/models/mod.rs` | Added `task`, `backup` module exports |
| `src/app/services/mod.rs` | Added `backup_service` module export |
| `src-tauri/src/commands/mod.rs` | Added `backup` command module |
| `src-tauri/src/lib.rs` | Registered 3 new Tauri commands; old raw commands marked `DEPRECATED` |

---

## 3. Unified Task Model

Created `src/app/models/task.rs` as a shared model for all operation types:

```rust
pub enum TaskType {
    Backup,
    Restore,
    Verify,
    Prune,
    Schedule,
}

pub enum TaskState {
    Idle,
    Running,
    Completed,
    Failed { reason: String },
    Cancelled,
}

pub struct TaskProgress {
    pub percent: f64,
    pub processed_bytes: u64,
    pub total_bytes: u64,
    pub current_file: String,
}

pub struct TaskResult {
    pub task_id: String,
    pub task_type: TaskType,
    pub state: TaskState,
    pub progress: Option<TaskProgress>,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub error_message: Option<String>,
}

pub struct TaskInfo {
    pub task_id: String,
    pub task_type: TaskType,
    pub state: TaskState,
    pub source: String,
    pub dest: String,
    pub created_at: i64,
    pub updated_at: i64,
}
```

This model is designed to be reused by Restore, Verify, Prune, and Schedule services in future tasks.

---

## 4. Backup Models

```rust
pub enum BackupJobStatus {
    Idle,
    Running,
    Completed,
    Failed,
    Scheduled,
}

pub struct BackupJobView {
    pub name: String,
    pub job_name: String,
    pub source: String,
    pub dest: String,
    pub schedule: Option<String>,
    pub compress: bool,
    pub last_run: Option<i64>,
    pub last_status: BackupJobStatus,
    pub total_backups: u32,
}

pub struct BackupRequest {
    pub name: String,
    pub source: String,
    pub dest: String,
    pub compress: bool,
}

pub struct BackupResult {
    pub success: bool,
    pub backup_id: String,
    pub files_backed_up: u64,
    pub total_bytes: u64,
    pub duration_secs: f64,
    pub error_message: Option<String>,
}
```

---

## 5. BackupService API

```rust
impl BackupService {
    pub fn list_jobs(config_path: &Path) -> Result<Vec<BackupJobView>, AppError>
    pub fn get_job_detail(config_path: &Path, name: &str) -> Result<BackupJobView, AppError>
    pub fn run_backup(config_path: &Path, request: &BackupRequest) -> Result<BackupResult, AppError>
    pub fn run_backup_dry(config_path: &Path, request: &BackupRequest) -> Result<BackupResult, AppError>
}
```

### Architecture

```
Backup.tsx (React)
    |
    | invoke("list_backup_jobs") / invoke("run_backup") / invoke("get_backup_job_detail")
    v
commands/backup.rs (Tauri Command — thin wrapper)
    |
    | calls BackupService
    v
services/backup_service.rs (Application Service — orchestration)
    |
    | reads config, calls backup::create_backup_job / backup::run_backup
    v
Core Engine (backup.rs — FROZEN, not modified)
```

---

## 6. Tauri Commands

```rust
#[tauri::command]
fn list_backup_jobs(app_state: State<AppState>) -> Result<Vec<BackupJobView>, String>

#[tauri::command]
fn run_backup(app_state: State<AppState>, request: BackupRequest) -> Result<BackupResult, String>

#[tauri::command]
fn get_backup_job_detail(app_state: State<AppState>, name: String) -> Result<BackupJobView, String>
```

Each command is a thin wrapper — parameter validation + error conversion only.

---

## 7. React API Layer (backupApi.ts)

```typescript
export async function listJobs(): Promise<BackupJobView[]>
export async function runBackup(request: BackupRequest): Promise<BackupResult>
export async function getJobDetail(jobId: string): Promise<BackupJobView>
```

Each function:
1. Calls `invoke()` for real data
2. Falls back to mock data if `VITE_MOCK_DATA=true` or invoke fails
3. Returns typed responses matching Rust models

---

## 8. Test Results

### 7 Backup Service Tests

| Test | Scenario | Result |
|------|----------|:------:|
| `test_list_jobs_with_config` | Config with multiple jobs | ✅ PASS |
| `test_list_jobs_no_config` | No config file exists | ✅ PASS |
| `test_get_job_detail_found` | Job exists by ID | ✅ PASS |
| `test_get_job_detail_not_found` | Job ID does not exist | ✅ PASS |
| `test_run_backup_success` | Full backup run with result | ✅ PASS |
| `test_run_backup_dry_run` | Dry run returns expected result | ✅ PASS |
| `test_run_backup_config_missing` | Backup with missing config | ✅ PASS |

### Overall Test Suite

| Suite | Count | Result |
|-------|:-----:|:------:|
| Unit tests | 75 | ✅ ALL PASS |
| Integration tests | 19 | ✅ ALL PASS |
| Backup service tests (new) | 7 | ✅ ALL PASS |
| Dashboard service tests | 4 | ✅ ALL PASS |
| **Total** | **105** | **✅ ALL PASS** |

### Quality Gates

| Gate | Result |
|------|:------:|
| `cargo fmt --check` | ✅ PASS |
| `cargo clippy --all-targets -- -D warnings` | ✅ PASS |
| `cargo build` | ✅ PASS |
| `cargo test` | ✅ PASS (105) |
| `pnpm tsc` | ✅ ZERO errors |
| `pnpm vite build` | ✅ PASS |
| Forbidden scope audit | ✅ No frozen core modules modified |
| Task model reusability | ✅ Restore/Verify/Prune/Schedule can reuse |

---

## 9. Architecture Verification

### What was proved

1. **Application Layer is extensible** — Adding a new business domain (Backup) required only new Service + Model files, no architecture changes
2. **Task model is shared** — `TaskType`, `TaskState`, `TaskProgress`, `TaskResult` are designed for all operation types
3. **Dashboard pattern is reusable** — Backup follows the exact same pattern: `React -> invoke() -> Command -> Service -> Core`
4. **Core remains protected** — `backup.rs`, `restore.rs`, etc. were NOT modified

### Data Flow (Verified)

```
React Backup.tsx
    |
    invoke("list_backup_jobs" | "run_backup" | "get_backup_job_detail")
    |
Tauri Command (commands/backup.rs)
    |
BackupService (services/backup_service.rs)
    |
Core Engine (backup.rs) — FROZEN, not touched
```

---

## 10. Known Limitations

1. `run_backup()` creates a blocking call — no real-time progress streaming yet (Tauri event system available for future enhancement)
2. No backup cancellation support
3. `run_backup_dry()` validates config but does not test destination disk space (deferred)
4. Task model's `Running` state is not yet wired to actual progress events

---

## 11. Next Recommended Task

**T2.5-03C — Backup UI Integration**

Reuse the existing `Dashboard.tsx` component pattern, `backupApi.ts`, and built commands to create a full Backup page with:
- Job list table
- Run backup action
- Status display
- Empty state
- Error state
