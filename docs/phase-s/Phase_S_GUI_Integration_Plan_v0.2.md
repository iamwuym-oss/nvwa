## 1. Background and Goals

Phase S (Repository Engine) completed 13/13 tasks, 337 tests passed, 5 Audit gates all PASS.
It is now formally CLOSED baseline.

Current Nuwa has two storage methods:

| Storage Type | Status | Use Case |
|-------------|--------|----------|
| Flat-File | **Existing** - Phase 1/2 legacy | Directory + JSON manifest traditional backup |
| Repository Engine (Phase S) | **New** - This plan's target | Enterprise-grade block storage backup |

### 1.1 This Integration Goal

Enable users to:

1. **Create Repository** - In Settings page, choose path, create enterprise storage repository
2. **Manage Repository** - View list, details, execute verify
3. **Use Repository for backup** - Select Repository as target storage when creating backup strategy

### 1.2 Explicitly Excluded (Deferred to Future Versions)

| Feature | Reason for Deferral | Target Phase |
|---------|--------------------|-------------|
| Retention UI operations | Users may mistakenly think it frees disk space | After GC is complete |
| rebuild_repo | Disaster recovery tool, not daily operation | CLI / Advanced Tools |
| backup instance explorer | Repository internal management | CLI / Advanced Tools |
| Block Size / Compression selection | Phase S already frozen fixed parameters | Read-only display |

## 2. Confirmed Design Decisions (Frozen)

### 2.1 Repository Entry Point

**Repository management lives in Settings page.** No new sidebar menu.

- Settings page adds "Repository Management" area
- Contains: create, list, details, verify operations
- Same page level as global config, but separate section

### 2.2 Repository in Backup Strategy Creation Flow

| Scenario | Behavior |
|----------|----------|
| No Repository exists | Show "Create Repository" button, guide user to create first |
| Repository exists | Dropdown to select existing Repository, or create new |
| Switch storage type | Add storage type selector when creating strategy: Flat File / Repository |

### 2.3 Restore Page - New RestoreProvider

**Restore page UI layout unchanged, but backend needs RestoreProvider mechanism.**

v0.2 correction (actual code confirmed):

Current restore_service.rs is entirely based on manifest.json:
- let manifest_path = backup_dir.join("manifest.json")
- let manifest = storage::read_manifest(&backup_dir)

Repository backup sets do NOT generate manifest.json, so Restore needs a new adapter layer.

Solution: New RestoreProvider trait unifying both sources:
- FlatFileRestoreProvider - existing logic, reads manifest.json
- RepositoryRestoreProvider - new, via Repository Engine recovery API

restore_service.rs public API stays the same, internally routes to correct Provider.

### 2.4 Retention Boundary

**First version: UI shows Retention status info only, no action buttons.**

- Show: active Restore Point count, deleted count, Orphan Candidate count
- No: Execute Retention, set policy buttons
- Clear note: "Deleting restore points does not free disk space"
- Full Retention UI after Phase 6+ GC is ready

### 2.5 Block Size and Compression

**All Phase S frozen parameters are read-only in UI.**

From Phase S baseline:
- Block Size: display "256 KiB (Fixed)"
- Compression: display "zstd (default)"
- No dropdown or configuration

## 3. Repository Registry Design

### 3.1 Why a Registry?

Repository's stable identity is UUID, not filesystem path.

Scenario: User moves Repository from D: to E: (e.g., disk replacement). Path changes, UUID stays.

If Service layer uses path as identity:
- All backup strategy references break after move
- Cannot auto-discover Repository's new location

### 3.2 Registry Design

File location: ~/.nuwa/repositories.json

`json
{
  "version": 1,
  "repositories": [
    {
      "id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
      "name": "Main Backup",
      "path": "D:\\NuwaRepo\\main",
      "created_at": "2026-07-09T14:30:00Z",
      "last_opened": "2026-07-10T10:00:00Z",
      "status": "active"
    }
  ]
}
`

### 3.3 Registry Operations

| Operation | Description |
|-----------|-------------|
| register_repo(id, name, path) | Register after creation |
| unregister_repo(id) | User-initiated removal (does not delete data) |
| list_repos() | List all registered Repositories |
| get_repo(id) | Query by UUID |
| update_repo_path(id, new_path) | Update after path change |
| resolve_repo_path(id) | Resolve UUID to actual path |

Upper Service layer always operates by UUID; path changes only update registry record.

## 4. New Backend Components

### 4.1 Architecture Overview

`
+--------------------------------------------------------------------+
|                     React UI                                        |
|  Settings Page    Backup Page      Restore Page (UI unchanged)      |
|  (Repo Mgmt)     (Strategy Config) (RestoreProvider added)         |
+--------+------------+-------------------------------+----------------+
         |            |                               |
    invoke(repo_*)  invoke(backup_*)                  invoke(restore_*)
         |            |                               |
         v            v                               v
+--------------------------------------------------------------------+
|              Tauri Command Layer (thin wrapper)                     |
|  repo.rs    backup.rs    config.rs    restore.rs (no change)        |
+--------+------------+-------------------------------+----------------+
         |            |                               |
         v            v                               v
+--------------------------------------------------------------------+
|           Application Service Layer                                  |
|                                                                     |
|  RepoService         BackupService           RestoreService         |
|  (registry+core)     (match+TODO trait)      (+ RestoreProvider)    |
|                                                                     |
|  +-------------+     +------------------+    +------------------+   |
|  | RepoRegistry|     | run_backup()     |    | FlatFileProvider |   |
|  | (JSON file) |     |   flat-file path |    | RepoProvider     |   |
|  |            |     |   repo path(TODO)|    | (NEW)            |   |
|  +-------------+     +------------------+    +------------------+   |
+--------+------------+-------------------------------+----------------+
         |            |                               |
         v            v                               v
+--------------------------------------------------------------------+
|                  Core Engine Layer                                   |
|  repository/ (Phase S FROZEN)    backup.rs  restore.rs  storage.rs |
+--------------------------------------------------------------------+
`

### 4.2 RepoRegistry - src/app/services/repo_registry.rs (NEW)

| Method | Description |
|--------|-------------|
| load() -> Self | Load ~/.nuwa/repositories.json |
| save(&self) | Persist to disk |
| list() -> Vec<RepoRecord> | Return all registered Repositories |
| get(id) -> Option<RepoRecord> | Query by UUID |
| register(record) | Register new Repository |
| unregister(id) | Remove registration (does not delete data) |
| update_path(id, new_path) | Update path |
| resolve_path(id) -> Result<PathBuf> | Resolve UUID to path (check existence) |

### 4.3 RepoService - src/app/services/repo_service.rs (NEW)

Phase 1 exposes 4 methods only:

| Method | Underlying Call | Description |
|--------|----------------|-------------|
| create_repo(name, path) -> RepoInfo | repo_manager::init_repo() + RepoRegistry::register() | Create and register |
| list_repos() -> Vec<RepoInfo> | RepoRegistry::list() + open each | Get all repos info |
| get_repo_info(id) -> RepoInfo | RepoRegistry::get() + open_repo() | Get detailed info |
| verify_repo(id, options) -> VerifyResult | resolve_path() + verify::verify_repo() | Verify integrity |

#### Future Phase (NOT NOW)

`
// Phase 2: Retention management
pub fn apply_retention(id, policy) -> RetentionResult
pub fn get_retention_status(id) -> RetentionStatus

// CLI / Advanced Tools only:
pub fn rebuild_repo(id) -> RecoveryResult
pub fn list_backup_instances(id) -> Vec<BackupInstanceSummary>
pub fn get_backup_instance(id, instance_id) -> BackupInstanceDetail
`

### 4.4 Repo API Models - src/app/models/repo.rs (NEW)

`ust
#[derive(Serialize, Deserialize)]
pub struct RepoRecord {
    pub id: String,           // UUID
    pub name: String,
    pub path: String,
    pub created_at: String,
    pub last_opened: String,
    pub status: String,       // "active" | "missing"
}

#[derive(Serialize)]
pub struct RepoInfoResponse {
    pub id: String,
    pub name: String,
    pub path: String,
    pub uuid: String,             // Repository Engine internal UUID
    pub format_version: u32,
    pub created_at: String,
    pub capabilities: Vec<String>,
    pub total_chunks: u64,
    pub total_size_bytes: u64,
    pub instance_count: u32,
    pub block_size: String,       // "256 KiB (Fixed)"
    pub compression: String,      // "zstd"
    pub retention_status: RetentionStatusSummary,
}

#[derive(Serialize)]
pub struct RetentionStatusSummary {
    pub total_restore_points: u32,
    pub active_restore_points: u32,
    pub deleted_restore_points: u32,
    pub orphan_candidates: u64,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub passed: bool,
    pub checked_instances: u32,
    pub checked_blocks: u64,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Serialize, Deserialize)]
pub struct CreateRepoRequest {
    pub name: String,
    pub path: String,
}
`

Note: CreateRepoRequest does NOT include block_size, compression etc.

### 4.5 Repo Tauri Commands - src-tauri/src/commands/repo.rs (NEW)

4 Tauri Commands only:

`ust
#[tauri::command]
pub async fn create_repo(name: String, path: String) -> Result<RepoInfoResponse, String>

#[tauri::command]
pub async fn list_repos() -> Result<Vec<RepoInfoResponse>, String>

#[tauri::command]
pub async fn get_repo_info(id: String) -> Result<RepoInfoResponse, String>

#[tauri::command]
pub async fn verify_repo(id: String, quick: bool) -> Result<VerifyResponse, String>
`

Requirements:
- Each Command returns Result<T, String> (AppError to user-friendly messages)
- Pure forwarding to RepoService, no business logic

### 4.6 JobConfig Field Extension - src/app/models/config_job.rs (MODIFY)

`ust
pub struct JobConfigView {
    // ... existing fields unchanged
    pub storage_type: String,              // "flat-file" | "repository"
    pub repository_id: Option<String>,     // Repository UUID
}

pub struct JobConfigRequest {
    // ... existing fields unchanged
    pub storage_type: String,
    pub repository_id: Option<String>,
}
`

### 4.7 BackupService Branch - src/app/services/backup_service.rs (MODIFY)

`ust
pub fn run_backup(job_name: &str) -> Result<BackupResult, AppError> {
    let config = load_job_config(job_name)?;
    match config.storage_type.as_str() {
        "flat-file" => run_flat_file_backup(config),
        "repository" => {
            // TODO: Extract to BackupBackend trait (before Phase 3 Volume Backup)
            run_repository_backup(config)
        }
        _ => Err(AppError::invalid_config("Unknown storage type")),
    }
}

fn run_repository_backup(config: JobConfig) -> Result<BackupResult, AppError> {
    // 1. repo_registry.resolve_path(config.repository_id)
    // 2. repo_manager::open_repo(path)
    // 3. Traverse source files, chunk via ChunkEngine, write to BlockStore
    // 4. Record BlockMap + Catalog
    // 5. TransactionManager: CREATING -> WRITING -> VERIFYING -> COMMITTED
    // 6. Write backup instance metadata
    // 7. Write history record to HistoryDb (for Restore discovery)
}
`

Key: Repository backup MUST write a history record to HistoryDb so list_restore_points() can find it.

### 4.8 RestoreProvider - src/app/services/restore_provider.rs (NEW)

`ust
pub trait RestoreProvider {
    fn list_restore_points() -> Result<Vec<RestorePointView>, AppError>;
    fn get_preview(backup_id: &str) -> Result<RestorePreview, AppError>;
    fn execute_restore(request: &RestoreRequest) -> Result<RestoreOperationResult, AppError>;
    fn delete_backup_set(backup_id: &str) -> Result<(), AppError>;
}

pub struct FlatFileRestoreProvider;    // Existing logic, reads manifest.json
pub struct RepositoryRestoreProvider;  // New, via Repository recovery/reader API
`

restore_service.rs public API unchanged, internally dispatches to correct Provider:

`ust
pub fn list_restore_points() -> Result<Vec<RestorePointView>, AppError> {
    let mut points = FlatFileRestoreProvider::list_restore_points()?;
    points.extend(RepositoryRestoreProvider::list_restore_points()?);
    points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(points)
}
`

## 5. New UI Components

### 5.1 CreateRepoModal - ui/src/components/repo/CreateRepoModal.tsx (NEW)

Repository creation dialog. Reused in Settings and Backup pages.

Contains:
- Repository name input
- Storage path selector (reuse FileBrowserModal)
- Block Size read-only display: "256 KiB (Fixed)"
- Compression read-only display: "zstd"

Interaction states:
- **Loading:** Button shows "Creating..." spinner
- **Error:** Inline red message (path exists, path unwritable, permission denied)
- **Success:** Close Modal, refresh repository list
- **Validation:** Name not empty, path not empty, valid path

### 5.2 RepoListPanel - ui/src/components/repo/RepoListPanel.tsx (NEW)

Repository list panel showing all registered repositories.

`
+--------------------------------------------------------------+
|  Repository Management                          [+ New Repo] |
|                                                              |
|  +-- Main Backup -----------------------------------------+ |
|  |  Path: D:\\NuwaRepo\\main                                | |
|  |  UUID: a1b2...   Size: 298.5 GB                         | |
|  |  Restore Points: 12   Status: Active                    | |
|  |  [Verify] [Details] [Remove from List]                  | |
|  +--------------------------------------------------------+ |
+--------------------------------------------------------------+
`

Interaction states:
- **Loading:** Skeleton cards (same style as Backup page)
- **Error:** ErrorState + retry button
- **Empty:** EmptyState + "Create Repository" CTA button
- **Repository Missing:** status = missing, orange warning

### 5.3 RepoDetailPanel - ui/src/components/repo/RepoDetailPanel.tsx (NEW)

Repository detail panel (click Details or select repo).

Contains:
- Basic info: UUID, Format version, creation time, Block Size, Compression
- Capability list (Fixed Block / SHA-256 / Transaction Log / CDC N/A / Encryption N/A)
- Statistics: Total Chunks, Total Size, Instance Count
- Retention status (read-only): Active RP / Deleted RP / Orphan Candidates
- Action buttons: [Verify Integrity] [Remove from List]

**Does NOT include (deferred):** [Rebuild] [Apply Retention] [Browse Instances]

### 5.4 StorageTypeSelector - ui/src/components/repo/StorageTypeSelector.tsx (NEW)

Storage type selector for Backup page strategy creation/editing.

`
Storage Type:
  [● Repository (recommended)]
  [○ Flat File (legacy)]

Repository:
  [Main Backup (298.5 GB)  v]  [+ New]
  Remaining: 500.2 GB / Total 1 TB
`

Interaction logic:
1. Default storage_type = "flat-file" (backward compatible)
2. Switch to epository: dest field hidden, show Repository dropdown
3. No Repository: dropdown shows "No repositories yet" + [+ New] button

## 6. Page Modification Plan

### 6.1 Settings Page - ui/src/pages/Settings.tsx (MODIFY)

From placeholder to dual-area layout:
1. Repository Management area (RepoListPanel + CreateRepoModal + RepoDetailPanel)
2. Global Settings area (future: theme, language, etc.)

State chain:
- **No Repository:** EmptyState -> CreateRepoModal -> Refresh
- **Has Repository:** RepoListPanel -> Details -> RepoDetailPanel
- **Loading:** Skeleton
- **Error:** ErrorState + Retry

### 6.2 Backup Page - ui/src/pages/Backup.tsx (MODIFY)

Add storage type selection to PlanForm area.

Changes:
1. StorageTypeSelector at top of PlanForm
2. Submit form with storage_type + repository_id
3. Echo storage method when editing existing strategy

Existing fields (source, dest, compress, retention) remain unchanged.

### 6.3 Restore Page - ui/src/pages/Restore.tsx (NO UI CHANGE)

UI layout unchanged. Backend restore_service.rs adds RestoreProvider dispatch.

## 7. Frontend API Layer

### 7.1 repoApi - ui/src/api/repoApi.ts (NEW)

`	ypescript
export async function createRepo(name: string, path: string): Promise<RepoInfoResponse>
export async function listRepos(): Promise<RepoInfoResponse[]>
export async function getRepoInfo(id: string): Promise<RepoInfoResponse>
export async function verifyRepo(id: string, quick?: boolean): Promise<VerifyResponse>
`

Requirements:
- Each API function includes try...catch...finally (loading/error state)
- Error messages in user-friendly Chinese

## 8. File Change List

### New Files (10)

| # | File Path | Content |
|---|-----------|---------|
| 1 | src/app/services/repo_registry.rs | Repository registry |
| 2 | src/app/services/repo_service.rs | Repo management service (4 APIs) |
| 3 | src/app/services/restore_provider.rs | RestoreProvider trait + implementations |
| 4 | src/app/models/repo.rs | Repository API models |
| 5 | src-tauri/src/commands/repo.rs | 4 Tauri Commands |
| 6 | ui/src/api/repoApi.ts | Frontend API layer |
| 7 | ui/src/components/repo/CreateRepoModal.tsx | Create dialog |
| 8 | ui/src/components/repo/RepoListPanel.tsx | Repository list panel |
| 9 | ui/src/components/repo/RepoDetailPanel.tsx | Detail panel |
| 10 | ui/src/components/repo/StorageTypeSelector.tsx | Storage type selector |

### Modified Files (7)

| # | File Path | Changes |
|---|-----------|---------|
| 1 | src/app/models/config_job.rs | Add storage_type + repository_id fields |
| 2 | src/app/services/backup_service.rs | Add repository branch + TODO in run_backup() |
| 3 | src/app/services/restore_service.rs | Switch to RestoreProvider dispatch |
| 4 | src/app/services/mod.rs | Register new modules |
| 5 | src-tauri/src/commands/mod.rs | Register repo module |
| 6 | ui/src/pages/Settings.tsx | From placeholder to Repository management |
| 7 | ui/src/pages/Backup.tsx | Add storage type selection in PlanForm |

### Untouched Files (Confirmed Safe)

| File | Reason |
|------|--------|
| src/repository/** | Phase S core FROZEN |
| src/app/services/dashboard_service.rs | No change needed |
| src/app/services/history_service.rs | No change needed |
| src/app/services/schedule_service.rs | No change needed |
| src-tauri/src/commands/restore.rs | Restore command unchanged |
| src-tauri/src/commands/backup.rs | Unchanged (BackupService internal) |
| ui/src/pages/Restore.tsx | UI unchanged |
| ui/src/pages/Dashboard/** | No change needed |
| ui/src/pages/History.tsx | No change needed |
| ui/src/pages/Schedule.tsx | No change needed |
| ui/src/pages/Clone.tsx | No change needed |

## 9. Implementation Order

`
Step 1:  repo.rs (models)
Step 2:  repo_registry.rs
Step 3:  repo_service.rs (4 core methods)
Step 4:  commands/repo.rs (4 tauri commands)
Step 5:  Modify backup_service.rs (+repository branch + TODO)
Step 6:  restore_provider.rs (trait + FlatFile + Repository)
Step 7:  Modify restore_service.rs (route to providers)
Step 8:  repoApi.ts
Step 9:  CreateRepoModal.tsx
Step 10: RepoListPanel.tsx
Step 11: RepoDetailPanel.tsx
Step 12: StorageTypeSelector.tsx
Step 13: Modify Settings.tsx
Step 14: Modify Backup.tsx (PlanForm)
Step 15: Build + fmt + clippy + test
Step 16: Audit: UI -> Service -> Repository boundary test
`

## 10. Test Strategy

### Unit Tests

| Test | Type |
|------|------|
| test_repo_registry_create_and_list | Unit |
| test_repo_registry_persistence | Unit (file I/O) |
| test_repo_registry_update_path | Unit |
| test_repo_service_create | Unit |
| test_repo_service_verify | Unit |
| test_repo_service_list | Unit |
| test_restore_provider_flat_file | Unit (existing logic preserved) |
| test_repository_backup_flow | Integration |
| test_restore_provider_repository | Integration |

### Boundary Tests (New)

| Test | Description |
|------|-------------|
| test_flat_file_and_repo_restore_coexist | Mixed restore points from both storage types |
| test_repo_missing_path_handling | Graceful degradation when path not found |
| test_repo_ui_core_boundary | Simulated Tauri invoke -> Service -> Repository full chain |

### Manual UI Tests

| Test | Description |
|------|-------------|
| Settings first open -> guidance shown | Empty state chain |
| Create Repository -> verify success | Full flow |
| Verify Repository integrity | Verify operation |
| Backup page select Repository type | Form binding |
| Repository backup -> Run -> Restore verify | Full data chain |
| Restore page shows both source types | RestoreProvider verification |

## 11. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Repository handle management complexity | Multiple open/close affects performance | RepoService internal LRU cache |
| Path change breaks references | User misoperation | Registry mechanism + status detection |
| Flat-file existing data compatibility | Upgrade experience | Both methods coexist, no migration required |
| Tauri Command async vs Repository sync conflict | Blocking operations | Use spawn_blocking |
| RepoRegistry JSON file corruption | Registry loss | Auto-rebuild from Repository internal metadata |

## 12. GUI Integration v1 Scope Boundary

`
Phase 2.5-T2.5-05 Repository Integration v1

IN SCOPE:
  [Backend]
    - RepoRegistry (repositories.json)
    - RepoService: create, list, info, verify
    - JobConfig: storage_type, repository_id
    - BackupService: repository branch (+ TODO for future trait)
    - RestoreProvider: trait + FlatFileProvider + RepositoryProvider

  [UI]
    - Settings: RepoListPanel + CreateRepoModal + RepoDetailPanel
    - Backup: StorageTypeSelector + PlanForm integration
    - Restore: NO UI CHANGE (backend only)

OUT OF SCOPE (deferred):
    - Retention operations (read-only status only)
    - rebuild_repo (CLI only)
    - Backup instance explorer (CLI only)
    - Block size / compression selection (read-only display)
    - Volume Backup (Phase 3)
    - GC / physical block deletion (Phase 6+)
`

## Appendix A: Retention Notice Text

In Repository detail panel, below retention status area, display:

> **Note:** Currently deleting restore points only marks them as deleted, does NOT free disk space.
> Physical space reclamation (Garbage Collection) is planned for a future version.

## Appendix B: Relationship with Phase_S_Known_Limitations_and_Roadmap.md

This plan's retention limitations, deferred GC, RestoreProvider design, and other architecture decisions are tracked in Phase_S_Known_Limitations_and_Roadmap.md. Any new limitations discovered during implementation must be synced to that document.

