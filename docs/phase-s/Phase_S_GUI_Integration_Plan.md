# Phase S — GUI Integration Plan

**Date:** 2026-07-10
**Phase S Status:** CLOSED BASELINE (commit `0e624e5`)
**Target:** Desktop GUI (Tauri 2.0 + React + TypeScript + Vite)

---

## 一、背景与目标

Phase S（Repository Engine）已完成 13/13 个任务、337 项测试通过、5 项 Audit 全部 PASS，正式进入 CLOSED 基线冻结。

当前 Nuwa 的备份系统有两种存储方式并存：

| 存储方式 | 状态 | 适用场景 |
|---------|------|---------|
| 扁平文件（Flat-File） | **现有** — Phase 1/2 遗留 | 目录 + JSON manifest 的传统备份方式 |
| Repository Engine（Phase S） | **新开发** — 本计划目标 | 企业级块存储备份方式 |

本计划定义如何将 **Repository Engine 的后端能力** 通过 **Tauri Command + Application Service** 暴露给 React UI。

## 二、已确认的设计决策（冻结，不再讨论）

以下决策来自用户与 GPT 的交叉审查，已冻结：

### 2.1 Repository 入口位置

**Repository 管理放置在 Settings 页面内。** 不新增左侧大菜单页面。

- Settings 页面增加「Repository 管理」区域
- 包含创建、打开、验证、查看详情、Retention 操作
- 不与全局配置混淆，但同属一个页面层级

### 2.2 备份策略创建流程中的 Repository

| 场景 | 行为 |
|------|------|
| 无 Repository 存在 | 显示「+ 创建 Repository」按钮，引导用户先创建 |
| 有 Repository 存在 | 下拉选择已有 Repository，或新建 |
| 切换存储方式 | 策略创建时增加「存储方式」选择器：扁平文件 | Repository |

### 2.3 恢复页面不修改

**Restore 页面保持不变。** 备份集（Backup Set）在备份完成后自然出现，用户不需要在 Restore 页面选择 Repository。

- 当前 Restore 页面的三栏布局（还原点列表 | 文件预览 | 恢复表单）保持不变
- Restore Service 自动识别备份集类型（flat-file vs repository），对 UI 透明

### 2.4 Retention 边界

**Retention 不释放磁盘空间。**

- 删除 Restore Point 仅将其标记为 DELETED，不删除物理块
- UI 中必须明确提示：「删除备份点不释放磁盘空间」
- 物理 GC 属于未来 Phase 6+ 功能

### 2.5 数据流架构

```
+----------------------------------------------------------------+
|                     React UI                                    |
|  Settings Page    Backup Page    Restore Page (NO CHANGE)       |
|  (Repo管理)      (策略配置)      (恢复操作)                     |
+--------+------------+---------------------------+---------------+
         |            |                           |
    invoke(repo_*)  invoke(backup_*)            invoke(restore_*)
         |            |                           |
         v            v                           v
+----------------------------------------------------------------+
|              Tauri Command Layer (thin wrapper)                 |
|  repo.rs    backup.rs    config.rs    restore.rs                |
+--------+------------+---------------------------+---------------+
         |            |                           |
         v            v                           v
+----------------------------------------------------------------+
|           Application Service Layer (src/app/services/)         |
|                                                                 |
|  RepoService     BackupService     RestoreService               |
|  (NEW)           (modified)        (NO CHANGE)                  |
|                                                                 |
|       +---------------------------+                             |
|       |    ConfigService          |                             |
|       |  (+ repository_path)      |                             |
|       +---------------------------+                             |
+--------+------------+---------------------------+---------------+
         |            |                           |
         v            v                           v
+----------------------------------------------------------------+
|                  Core Engine Layer                               |
|                                                                 |
|  backup.rs   restore.rs   repository/ (Phase S FROZEN)          |
|                                                                 |
|  Flat-File Path           Repository Path                       |
|  manifest.json            Block Store + Metadata                |
+----------------------------------------------------------------+
```

## 三、当前代码状态分析

### 3.1 现有页面

| 页面 | 文件 | 功能 | 本计划涉及的修改 |
|------|------|------|----------------|
| Settings | `ui/src/pages/Settings.tsx` | 全局设置（当前为空占位）| **新增** Repository 管理区域 |
| Backup | `ui/src/pages/Backup.tsx` | 三栏布局：策略列表 | 配置面板 + 表单（右栏）| **修改** — 增加存储方式选择 |
| Restore | `ui/src/pages/Restore.tsx` | 三栏布局：还原点列表 | 文件预览 | 恢复表单 | **无修改** |
| Dashboard | `ui/src/pages/Dashboard/` | 概览卡片 | **无修改** |
| History | `ui/src/pages/History.tsx` | 历史记录列表 | **无修改** |
| Schedule | `ui/src/pages/Schedule.tsx` | 调度配置 | **无修改** |
| Clone | `ui/src/pages/Clone.tsx` | 禁用占位 | **无修改** |

### 3.2 现有 Application Services

| Service | 文件 | 功能 | 本计划涉及的修改 |
|---------|------|------|----------------|
| backup_service | `src/app/services/backup_service.rs` | 执行备份、列表、状态查询 | **修改** — 支持 Repository 备份 |
| config_service | `src/app/services/config_service.rs` | 策略 CRUD | **修改** — 新增 repository_path 字段 |
| restore_service | `src/app/services/restore_service.rs` | 恢复操作、还原点查询 | **无修改**（已自动适配）|
| dashboard_service | `src/app/services/dashboard_service.rs` | 概览统计 | **无修改** |
| file_browser_service | `src/app/services/file_browser_service.rs` | 文件浏览 | **可复用**（Repository 路径选择）|
| history_service | `src/app/services/history_service.rs` | 历史记录 | **无修改** |
| schedule_service | `src/app/services/schedule_service.rs` | 调度配置 | **无修改** |

### 3.3 Rpository Engine 核心模块（FROZEN）

```
src/repository/
+-- mod.rs              # Repository pub interface
+-- config.rs           # RepositoryConfig, init_repo(), open_repo()
+-- block_store.rs      # BlockStore trait + fs implementation
+-- chunk_engine.rs     # ChunkPolicy, FixedChunkPolicy
+-- metadata.rs         # RepoMetadata, BackupInstanceStatus
+-- block_map.rs        # BlockMapEngine trait
+-- catalog.rs          # CatalogEngine trait
+-- transaction.rs      # TransactionManager
+-- verify.rs           # VerifyEngine
+-- retention.rs        # RetentionEngine
+-- recovery.rs         # RepositoryRecovery
+-- legacy.rs           # LegacyCompatibility
+-- types.rs            # 核心数据类型
+-- error.rs            # RepositoryError
```

## 四、新增后端组件

### 4.1 RepoService — `src/app/services/repo_service.rs` (NEW)

Repository 管理的应用层封装。职责：

| 方法 | 对应 Repository Core | 用途 |
|------|---------------------|------|
| `init_repo(path, config) -> RepoInfoResponse` | `repository::init_repo()` | 创建新 Repository |
| `open_repo(path) -> RepoInfoResponse` | `repository::open_repo()` | 打开已有 Repository |
| `close_repo(path)` | - | 释放 Repository 句柄 |
| `get_repo_info(path) -> RepoInfoResponse` | `repository::repo_info()` | 获取详细状态 |
| `verify_repo(path, options) -> VerifyResponse` | `repository::verify_repo()` | 完整验证 |
| `check_integrity(path) -> IntegrityResponse` | `repository::check_integrity()` | 快速完整性检查 |
| `rebuild_repo(path) -> RecoveryResult` | `repository::rebuild_repo()` | 重建元数据 |
| `apply_retention(path, policy) -> RetentionResult` | `repository::apply_retention()` | 执行 Retention |
| `get_retention_status(path) -> RetentionStatus` | `repository::retention_status()` | Retention 状态 |
| `list_backup_instances(path) -> Vec<BackupInstanceSummary>` | `repository::list_backup_instances()` | 实例列表 |
| `get_backup_instance(path, id) -> BackupInstanceDetail` | `repository::get_backup_instance()` | 实例详情 |

设计要点：
- 所有操作以 `path`（Repository 存储路径）为标识
- 内部管理句柄缓存或每次操作临时打开（取决于性能要求）
- 返回类型必须是 `Serialize`，直接映射到 JSON

### 4.2 Repo API Models — `src/app/models/repo.rs` (NEW)

包含 RepoInfoResponse、VerifyResponse、IntegrityResponse、RetentionResult、RetentionStatus、CreateRepoRequest 等结构体，全部派生 Serialize/Deserialize。

### 4.3 Repo Tauri Commands — `src-tauri/src/commands/repo.rs` (NEW)

10-12 个 Tauri Command，与 RepoService 的方法一一对应。

要求：
- 每个 Command 使用 `Result<T, String>` 返回（AppError 转为用户友好的中文错误消息）
- 不包含业务逻辑（纯转发到 RepoService）

### 4.4 JobConfig 字段扩展 — `src/app/models/config_job.rs` (MODIFY)

向 JobConfigView 和 JobConfigRequest 增加：

```rust
pub struct JobConfigView {
    // ... 现有字段
    /// 存储类型: "flat-file" | "repository"
    pub storage_type: String,
    /// Repository 路径（当 storage_type = "repository" 时有效）
    pub repository_path: Option<String>,
}
```

### 4.5 BackupService 分支 — `src/app/services/backup_service.rs` (MODIFY)

`run_backup()` 方法增加分支逻辑：

```rust
pub fn run_backup(job_name: &str) -> Result<BackupResult, AppError> {
    let config = load_job_config(job_name)?;
    match config.storage_type.as_str() {
        "flat-file" => run_flat_file_backup(config),
        "repository" => run_repository_backup(config),  // 新增
        _ => Err(AppError::invalid_config("Unknown storage type")),
    }
}
```

## 五、新增 UI 组件

### 5.1 CreateRepoModal — `ui/src/components/repo/CreateRepoModal.tsx` (NEW)

Repository 创建对话框。在 Settings 页面和 Backup 页面复用。

包含：存储路径选择、Block Size 选择（64/256/1024 KiB）、压缩方式选择。

交互状态：
- **Loading：** 按钮显示 spinner
- **Error：** 行内红色提示（路径已存在、权限不足）
- **Success：** 关闭 Modal，刷新列表
- **Validation：** 路径不能为空、不含非法字符

### 5.2 RepoDetailPanel — `ui/src/components/repo/RepoDetailPanel.tsx` (NEW)

显示已打开的 Repository 的详细信息。

包含：UUID、Format 版本、创建时间、Block Size、压缩方式、Capability 列表、统计信息、操作按钮。

交互状态：
- **Loading：** Skeleton 卡片（与 Backup 页面风格一致）
- **Error：** ErrorState 组件 + retry 按钮
- **Empty：** 引导创建新的 Repository

### 5.3 RetentionPanel — `ui/src/components/repo/RetentionPanel.tsx` (NEW)

Repository 的 Retention 策略配置和操作面板。

包含：keep_days 输入、keep_count 输入、执行按钮、当前状态显示。

交互状态：
- **Loading：** 按钮显示 spinner
- **Success：** 显示被删除的 Restore Point 数量
- **Error：** 行内错误消息
- **Empty：** 无 Restore Point 时显示提示

### 5.4 StorageTypeSelector — `ui/src/components/repo/StorageTypeSelector.tsx` (NEW)

在 Backup 页面创建/编辑策略时选择存储方式。

包含：单选框（Repository / Flat File）、Repository 下拉选择器、空间信息。

## 六、页面修改方案

### 6.1 Settings 页面修改

**文件：** `ui/src/pages/Settings.tsx`

从空状态占位改为 Repository 管理区域。页面分区：

1. **Repository Management** — CreateRepoModal / RepoDetailPanel / RetentionPanel
2. **Global Settings (Future)** — 主题、语言等

交互状态：
- **No Repository：** EmptyState + 「+ 创建 Repository」大按钮
- **Repository Loaded：** RepoDetailPanel（可折叠/展开）
- **Loading：** Skeleton 卡片
- **Error：** ErrorState + 重试按钮

### 6.2 Backup 页面修改

**文件：** `ui/src/pages/Backup.tsx` — PlanForm 区域

在 PlanForm 中增加存储方式选择区域。

新增字段：`storage_type`, `repository_path`

交互逻辑：
1. 默认 `storage_type = "flat-file"`（向下兼容）
2. 切换到 `repository`：dest 字段变为只读或隐藏，显示 Repository 下拉选择器
3. 无 Repository：显示「+ Create Repository」引导按钮

## 七、Frontend API 层新增

### 7.1 repoApi — `ui/src/api/repoApi.ts` (NEW)

```typescript
export async function initRepo(path: string, config: CreateRepoRequest): Promise<RepoInfoResponse>
export async function openRepo(path: string): Promise<RepoInfoResponse>
export async function getRepoInfo(path: string): Promise<RepoInfoResponse>
export async function verifyRepo(path: string, options: VerifyOptionsRequest): Promise<VerifyResponse>
export async function checkIntegrity(path: string): Promise<IntegrityResponse>
export async function rebuildRepo(path: string): Promise<RecoveryResult>
export async function applyRetention(path: string, policy: RetentionPolicyRequest): Promise<RetentionResult>
export async function getRetentionStatus(path: string): Promise<RetentionStatus>
```

要求：每个 API 函数包含 `try...catch...finally`，错误消息为用户友好的中文。

## 八、文件修改清单（完整）

### 新增文件

| # | 文件路径 | 内容 |
|---|---------|------|
| 1 | `src/app/services/repo_service.rs` | Repository 管理服务 |
| 2 | `src/app/models/repo.rs` | Repository API 模型 |
| 3 | `src-tauri/src/commands/repo.rs` | Repository Tauri 命令 |
| 4 | `ui/src/components/repo/CreateRepoModal.tsx` | 创建 Repository 对话框 |
| 5 | `ui/src/components/repo/RepoDetailPanel.tsx` | Repository 详情面板 |
| 6 | `ui/src/components/repo/RetentionPanel.tsx` | Retention 操作面板 |
| 7 | `ui/src/components/repo/StorageTypeSelector.tsx` | 存储方式选择器 |
| 8 | `ui/src/api/repoApi.ts` | Repository API 调用层 |

### 修改文件

| # | 文件路径 | 修改内容 |
|---|---------|---------|
| 1 | `src/app/models/config_job.rs` | 增加 storage_type 和 repository_path 字段 |
| 2 | `src/app/services/backup_service.rs` | run_backup() 增加 repository 分支 |
| 3 | `src/app/services/mod.rs` | 增加 pub mod repo_service; |
| 4 | `src-tauri/src/commands/mod.rs` | 增加 pub mod repo; |
| 5 | `ui/src/pages/Settings.tsx` | 从空占位改为 Repository 管理区域 |
| 6 | `ui/src/pages/Backup.tsx` | PlanForm 增加存储方式选择 |

### 不修改的文件

| 文件 | 原因 |
|------|------|
| `src/repository/**` | Phase S 核心冻结 |
| `src/app/services/restore_service.rs` | 对 Repository 透明 |
| `src/app/services/dashboard_service.rs` | 无需修改 |
| `src/app/services/history_service.rs` | 无需修改 |
| `src/app/services/schedule_service.rs` | 无需修改 |
| `src-tauri/src/commands/restore.rs` | 恢复逻辑不涉及 Repository 选择 |
| `src-tauri/src/commands/backup.rs` | 不变（BackupService 内部处理分支）|
| `ui/src/pages/Restore.tsx` | 恢复页面不感知存储方式 |
| `ui/src/pages/Dashboard/**` | 无需修改 |
| `ui/src/pages/History.tsx` | 无需修改 |
| `ui/src/pages/Schedule.tsx` | 无需修改 |
| `ui/src/pages/Clone.tsx` | 无需修改 |

## 九、实现顺序建议

```
Phase 1 (Core Backend):
  Step 1: repo.rs (models)
  Step 2: repo_service.rs
  Step 3: commands/repo.rs
  Step 4: Modify backup_service.rs (repository branch)

Phase 2 (UI):
  Step 5: repoApi.ts
  Step 6: CreateRepoModal.tsx
  Step 7: RepoDetailPanel.tsx
  Step 8: RetentionPanel.tsx
  Step 9: StorageTypeSelector.tsx

Phase 3 (Page Integration):
  Step 10: Modify Settings.tsx
  Step 11: Modify Backup.tsx (PlanForm)

Phase 4 (QA):
  Step 12: Build & test
  Step 13: Audit
```

## 十、测试策略

### 单元测试（后端）

| 测试 | 说明 |
|------|------|
| `test_repo_service_init_and_open` | 创建和打开 Repository |
| `test_repo_service_verify` | 验证 Repository 完整性 |
| `test_repo_service_retention` | 执行 Retention 策略 |
| `test_backup_service_repository_branch` | Repository 类型备份流程 |
| `test_repo_service_invalid_path` | 无效路径错误处理 |
| `test_repo_service_rebuild` | 重建元数据 |

### 集成测试（后端）

| 测试 | 说明 |
|------|------|
| `test_repo_backup_restore_cycle` | Repository 备份恢复完整链路 |
| `test_flat_file_and_repo_coexist` | 两种存储方式互不影响 |

### UI 测试（手动）

| 测试 | 说明 |
|------|------|
| Settings 页面首次打开显示引导 | 空状态 |
| 创建 Repository 验证成功 | 完整流程 |
| 验证 Repository 状态 | 信息展示 |
| 执行 Retention 确认结果 | Retention 操作 |
| Backup 页面选择 Repository 类型 | 表单联动 |
| Backup Run Restore 验证 | 完整链路 |

## 十一、风险评估

| 风险 | 影响 | 缓解措施 |
|------|------|---------|
| Repository 句柄管理复杂 | 多次打开/关闭影响性能 | RepoService 内部维护 LRU 缓存 |
| Repository 路径变更导致数据不可用 | 用户误操作 | 创建后路径不可修改；UI 明确提醒 |
| Flat-file 到 Repository 迁移 | 用户存量数据 | Phase S 不要求数据迁移，两种方式共存 |
| Tauri Command 异步与 Repository 同步冲突 | Blocking 操作 | 使用 spawn_blocking |
| Repository 不存在时的降级体验 | 用户困惑 | 清晰的错误引导 + 创建按钮 |

## 十二、与 Phase 3 Volume Backup 的关系

本集成计划仅覆盖 **文件级备份的 Repository 支持**。

Volume Backup（Phase 3）将直接使用 Repository Engine：

```
Volume Backup (Phase 3)
    |
    +-- Volume Discovery
    +-- VSS Snapshot
    +-- Volume -> RawDataStream -> ChunkEngine -> BlockStore
    +-- 复用现有 Restore Point / Catalog / BlockMap
```

当前 GUI 集成中设计的 RepoService 和 RepoDetailPanel 在 Volume Backup 阶段直接复用。

## 附录 A：Repository UI 中 Retention 的提示文字

在所有涉及 Retention 操作的 UI 位置，必须显示以下提示：

> **注意：Retention 说明**
> 当前删除备份点仅标记为已删除，不释放磁盘空间。
> 物理空间回收（Garbage Collection）将在未来版本中实现。

## 附录 B：与 Phase_S_Known_Limitations_and_Roadmap.md 的关联

本计划中的 Retention 限制、GC 未实现等内容已记录在 Phase_S_Known_Limitations_and_Roadmap.md 中。
如果在 UI 集成过程中发现新的限制，必须同步更新该文档。
