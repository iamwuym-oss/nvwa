# Phase 2 — Technical Design Document

**Product:** Nüwa Backup (女娲备份)
**Phase:** 2 — 可用文件级备份
**Version:** v1.0
**Date:** 2026-07-05
**Status:** DRAFT — 待用户审阅后批准编码

---

## 1. Architecture Overview

### 1.1 Phase 2 Architecture

`
┌──────────────────────────────────────────────────────────────┐
│                        用户界面层                              │
│  ┌──────────────────────┐      ┌──────────────────────────┐  │
│  │   CLI (cli.rs)      │      │   GUI (gui.rs / egui)   │  │
│  │   • nuwa backup     │      │   • Dashboard            │  │
│  │   • nuwa restore    │      │   • Backup / Restore     │  │
│  │   • nuwa history    │      │   • History / Schedule   │  │
│  │   • nuwa prune      │      │   • Clone (placeholder)  │  │
│  │   • nuwa schedule   │      │   • Settings             │  │
│  │   • nuwa init       │      │                          │  │
│  └──────────────────────┘      └──────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
             │                              │
             │                              │
┌──────────────────────────────────────────────────────────────┐
│                      核心库 (lib.rs)                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │
│  │ backup   │ │ restore  │ │ verify   │ │  list        │   │
│  │ .rs      │ │ .rs      │ │ .rs      │ │  .rs         │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────────┘   │
│       │            │            │                 │          │
│       │            │            │                 │          │
│  ┌───────────────────────────────────────────────────────┐  │
│  │  共享基础设施层                                         │  │
│  │  manifest.rs / checksum.rs / storage.rs / errors.rs   │  │
│  │  diskspace.rs                                          │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                                │
│  ┌──────────────── ┌──────────────── ┌────────────────────┐  │
│  │ config.rs      │ history.rs     │ scheduler.rs        │  │
│  │ (TOML配置)     │ (SQLite索引)   │ (Task Scheduler)    │  │
│  └──────────────── └──────────────── └────────────────────┘  │
│  ┌──────────────── ┌────────────────                         │
│  │ prune.rs       │ cli_output     │                         │
│  │ (保留策略)     │ .rs (JSON)     │                         │
│  └──────────────── └────────────────                         │
└──────────────────────────────────────────────────────────────┘
`

### 1.2 Design Principles

| 原则 | 说明 |
|------|------|
| 双界面共享核心 | CLI 和 GUI 都通过 lib.rs 调用核心逻辑 |
| 向后兼容 | Phase 2 不修改 Phase 1 core (backup/restore/verify/list) |
| CLI 优先稳定 | Phase 1 的 CLI 命令接口不变 |
| 离线构建 | 所有新依赖必须支持 cargo build --offline |
| GUI 可选编译 | GUI 通过 Cargo feature 控制，不影响 CLI 构建 |

---

## 2. Module Structure
### 2.1 Phase 2 新增模块

`
src/
  ├── main.rs              # 程序入口：CLI 或 GUI 模式切换
  ├── lib.rs               # 公共 API，Phase 2 新模块通过此处暴露
  ├── cli.rs               # CLI 参数解析（扩展支持新子命令）
  ├── config.rs            # [NEW] TOML 配置文件读/写/校验
  ├── history.rs           # [NEW] SQLite 备份历史记录
  ├── prune.rs             # [NEW] 保留策略执行
  ├── scheduler.rs         # [NEW] Windows Task Scheduler 集成
  ├── cli_output.rs        # [NEW] JSON 输出格式化和进度显示
  │
  ├── backup.rs            # Phase 1 core — 冻结不修改
  ├── restore.rs           # Phase 1 core — 冻结不修改
  ├── verify.rs            # Phase 1 core — 冻结不修改
  ├── list.rs              # Phase 1 core — 冻结不修改
  ├── manifest.rs          # Phase 1 core — 冻结不修改
  ├── checksum.rs          # Phase 1 core — 冻结不修改
  ├── storage.rs           # Phase 1 core — 冻结不修改
  ├── errors.rs            # Phase 1 core — 冻结不修改
  ├── diskspace.rs         # Phase 1 core — 冻结不修改
  │
  ├── gui/                 # [NEW] GUI 模块（Phase 2 单独 feature）
  │   ├── mod.rs           # GUI 入口和启动
  │   ├── app.rs           # egui 应用主状态
  │   ├── task.rs          # [NEW] 后台 worker thread 管理
  │   ├── pages/
  │   │   ├── mod.rs
  │   │   ├── dashboard.rs
  │   │   ├── backup.rs
  │   │   ├── restore.rs
  │   │   ├── history.rs
  │   │   ├── schedule.rs
  │   │   ├── clone.rs     # [PLACEHOLDER] 仅 Coming Soon 提示
  │   │   └── settings.rs
  │   ├── widgets/         # 可复用 UI 组件
  │   │   ├── mod.rs
  │   │   ├── sidebar.rs
  │   │   ├── topbar.rs
  │   │   ├── card.rs
  │   │   ├── status_indicator.rs
  │   │   └── progress.rs
  │   └── theme.rs         # 深色科技风主题

tests/
  ├── backup_restore_tests.rs   # Phase 1 tests — 全部 22 个保留
  ├── config_tests.rs           # [NEW] 配置文件测试
  ├── history_tests.rs          # [NEW] 历史记录测试
  ├── prune_tests.rs            # [NEW] 保留策略测试
  └── scheduler_tests.rs        # [NEW] 计划任务测试
`

### 2.2 模块依赖关系

`
config.rs     → lib.rs (errors.rs for error types)
              → serde, toml (new dependencies)

history.rs    → lib.rs (manifest.rs, errors.rs)
              → rusqlite (new dependency, part of history feature)

prune.rs      → lib.rs (manifest.rs, storage.rs, errors.rs, history.rs)
              → config.rs (to read job retention policy)

scheduler.rs  → std::process::Command (schtasks.exe)
              → config.rs (to read job definitions)

cli_output.rs → lib.rs (manifest.rs types for JSON serialization)

gui/          → lib.rs (all core modules)
              → egui, eframe (new dependencies, part of gui feature)
              → std::sync::mpsc (worker thread communication)
`

---

## 3. Configuration System

### 3.1 Cargo Feature Design

`	oml
[features]
default = ["history"]
history = ["rusqlite"]
gui = ["egui", "eframe"]
`

### 3.2 Feature 设计说明

| Feature | 包含 | 说明 |
|---------|------|------|
| default | history | 默认启用历史记录（含 SQLite） |
| history | 
usqlite | 备份历史索引。rusqlite 不属于 gui feature，因为 CLI 也需要 history |
| gui | egui, eframe | GUI 桌面界面。仅含 GUI 框架依赖，不包含 rusqlite |

**关键设计决策：** rusqlite 只放在 history feature 中，**不**放在 gui feature 中。
这样 CLI 模式（cargo build）也能使用 SQLite 历史，GUI 模式（cargo build --features gui）额外引入 GUI 框架。

### 3.3 Config File Design

`	oml
# 默认配置示例
[job.default]
source = "C:\\Users"
dest = "D:\\Backup"
compress = false
retention = { keep_count = 10, keep_days = 30 }

[job.documents]
source = "C:\\Users\\Tony\\Documents"
dest = "\\\\nas\\backup\\docs"
compress = true
retention = { keep_count = 20 }

[job.projects]
source = "C:\\Projects"
dest = "D:\\Backup\\Projects"
compress = true
`

### 3.4 Config File Location

| 平台 | 路径 |
|------|------|
| Windows | %APPDATA%/nuwa/config.toml |
| 回退路径 | ./nuwa.toml（当前目录） |

### 3.5 Config API

`
ust
// config.rs — 公开 API
pub struct Config {
    pub jobs: HashMap<String, JobConfig>,
}

pub struct JobConfig {
    pub source: PathBuf,
    pub dest: PathBuf,
    pub compress: bool,
    pub retention: Option<RetentionPolicy>,
}

pub struct RetentionPolicy {
    pub keep_count: Option<u32>,
    pub keep_days: Option<u64>,
}

pub fn load() -> Result<Config, ConfigError>;
pub fn init(path: Option<&Path>) -> Result<PathBuf, ConfigError>;
pub fn validate(config: &Config) -> Result<(), ConfigError>;
`

---

## 4. History System

### 4.1 SQLite Schema

`sql
CREATE TABLE IF NOT EXISTS operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    backup_id TEXT NOT NULL,
    operation TEXT NOT NULL,        -- 'backup' | 'restore' | 'verify'
    timestamp TEXT NOT NULL,         -- ISO 8601
    source_root TEXT NOT NULL,
    dest_path TEXT NOT NULL,
    job_name TEXT,
    file_count INTEGER NOT NULL,
    total_bytes INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,
    exit_code INTEGER NOT NULL,
    status TEXT NOT NULL             -- 'success' | 'failure' | 'partial'
);

CREATE INDEX idx_operations_timestamp ON operations(timestamp DESC);
CREATE INDEX idx_operations_operation ON operations(operation);
CREATE INDEX idx_operations_status ON operations(status);
`

### 4.2 History API

`
ust
// history.rs — 公开 API
pub struct OperationRecord {
    pub id: i64,
    pub backup_id: String,
    pub operation: String,
    pub timestamp: String,
    pub source_root: String,
    pub dest_path: String,
    pub job_name: Option<String>,
    pub file_count: u64,
    pub total_bytes: u64,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub status: String,
}

pub fn open_or_create(db_path: &Path) -> Result<HistoryDb, HistoryError>;
pub fn record_operation(db: &HistoryDb, record: &OperationRecord) -> Result<(), HistoryError>;
pub fn query_history(db: &HistoryDb, limit: u32, operation: Option<&str>) -> Result<Vec<OperationRecord>, HistoryError>;
pub fn rebuild_from_manifest(db: &HistoryDb, dest_root: &Path) -> Result<u32, HistoryError>;
`

### 4.3 Manifest vs SQLite Relationship

`
manifest.json  ← 每个备份点的唯一权威凭证（不可替代）
     ↑
.nuwa_history.db  ← 历史索引，可通过扫描 manifest 重建（可删除、可重建）
`

---

## 5. Prune (Retention Policy)

### 5.1 Algorithm

`
ust
pub fn prune(
    dest: &Path,
    keep_count: Option<u32>,
    keep_days: Option<u64>,
    dry_run: bool,
) -> Result<PruneReport, PruneError>;
`

1. 扫描 dest 目录下所有备份点文件夹
2. 按 created_at 降序排列
3. 如果同时指定 --keep-count 和 --keep-days，使用 Union 规则：满足任意条件即保留（安全优先）
4. 跳过最后一个可用的备份点（即使超过保留数量）
5. 删除前验证备份点的 manifest.json 存在且可读
6. 如果 dry_run == true，只报告不删除

### 5.2 Safety Rules

- 从不删除最后一个可用的备份点
- 删除前必须读取并验证 manifest.json
- manifest 损坏的备份点计入数量但标记为 damaged
- 删除操作使用 atomic-write 风格的目录删除（先重命名再删除）

## 5.3 Phase 2 Retention Boundary

Phase 2 prune operates only on **independent full file-level backup points**.
Each backup point is self-contained with its own manifest.json.

The following are **out of scope for Phase 2** and must not be implemented:
- Backup chains (parent-child backup point dependencies)
- Differential backup retention
- Incremental backup retention
- Chain-aware retention (e.g., must-keep-all-chains-if-any-member-exists)
- Dependency tracking between backup points

Phase 2 retention treats every backup point as independent. If the chain-aware
retention feature is added in a future phase, the prune module must be
redesigned or a separate chain-prune module must be created. Phase 2 prune
must never be extended to handle backup chain dependencies.

## 6. GUI Architecture — Local Desktop GUI (egui + eframe)

### 6.1 Technology Decision

| 维度 | egui + eframe | Web GUI (FastAPI+JS) | Qt6 (qmeta) |
|------|:------------:|:-------------------:|:-----------:|
| 语言 | Rust | Rust (CLI) + Python (Backend) + JS (Frontend) | Rust (bindings) |
| 运行时依赖 | 无 | Python runtime, Node.js(?) | Qt6 DLL (≈200MB) |
| 离线构建 | ✅ | ❌ | ✅ |
| 与 CLI core 共享代码 | ✅ 直接 | ❌ HTTP/IPC | ✅ |
| 二进制体积 | ≈15MB | N/A | ≈50MB+ |
| 开发者体验 | 纯 Rust，一致 | 三栈切换 | C++ bindings |
| Windows 集成 | ✅ | ✅ | ✅ |

**结论：egui + eframe 是最适合 Phase 2 GUI 的技术方案。**

### 6.2 Dependency Strategy

`	oml
[features]
default = ["history"]
history = ["rusqlite"]
gui = ["egui", "eframe"]

[dependencies]
# 核心依赖（始终启用）
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
sha2 = "0.10"
zstd = { version = "0.13", optional = true }

# history feature — rusqlite 不属于 gui feature
rusqlite = { version = "0.31", features = ["bundled"], optional = true }

# gui feature — 只包含 GUI 框架本身
egui = { version = "0.27", optional = true }
eframe = { version = "0.27", optional = true }
`

**重要：** 
usqlite 的 undled feature 会静态编译 SQLite C 库，无需系统安装 SQLite。

### 6.3 Application State Model

`
ust
// gui/app.rs — 应用主状态
pub struct NuwaApp {
    // 导航
    current_page: Page,

    // 共享数据
    config: Option<Config>,
    history: Vec<OperationRecord>,
    backup_points: Vec<BackupPoint>,

    // 后台任务
    task_channel: Option<(Sender<TaskCommand>, JoinHandle<()>)>,
    task_status: Option<TaskStatus>,
    logs: Vec<String>,

    // UI 状态
    theme: Theme,
    notifications: Vec<Notification>,
}

enum Page {
    Dashboard,
    Backup,
    Restore,
    History,
    Schedule,
    Clone,       // Coming Soon placeholder
    Settings,
}

enum TaskStatus {
    Idle,
    Running { progress: f32, current_file: String },
    Completed { summary: String },
    Failed { error: String },
}
`

### 6.4 Worker Thread Model

GUI 使用后台 worker thread 执行长时间任务，主线程仅负责 UI 渲染。

`
ust
// gui/task.rs — 后台任务管理

use std::sync::mpsc;

pub enum TaskCommand {
    Backup {
        source: PathBuf,
        dest: PathBuf,
        job_name: Option<String>,
        compress: bool,
    },
    Restore {
        backup_id: String,
        dest: PathBuf,
        overwrite: bool,
    },
    Verify {
        backup_id: String,
    },
    Prune {
        dest: PathBuf,
        keep_count: Option<u32>,
        keep_days: Option<u64>,
        dry_run: bool,
    },
    Cancel,
}

pub enum TaskMessage {
    Progress { percent: f32, current_file: String },
    Completed { summary: String },
    Failed { error: String },
    Log { message: String },
}

pub fn start_backup_task(
    cmd: TaskCommand,
    tx: mpsc::Sender<TaskMessage>,
) -> JoinHandle<()> {
    std::thread::spawn(move || {
        // 1. 解析命令
        // 2. 调用 lib.rs 备份 API
        // 3. 通过 tx 发送进度
        // 4. 完成后发送 Completed 或 Failed
    })
}
`

### 6.5 后台任务分类

| 任务 | Worker 函数 | 说明 |
|------|-------------|------|
| backup | start_backup_task() | 调用 lib.rs backup API |
| restore | start_restore_task() | 调用 lib.rs restore API |
| verify | start_verify_task() | 调用 lib.rs verify API |
| prune | start_prune_task() | 调用 prune.rs 策略引擎 |

### 6.6 线程模型

`
egui 主线程                    Worker Thread
    │                              │
    │ 点击 "Start Backup"          │
    │ ──────────────────────────►  │
    │                              │ 开始备份
    │    ← Progress { 45% }  ───── │
    │    ← Log { "File: x.mp4" } ─ │
    │    ↑ UI 更新进度条            │ 继续工作
    │    ↑ 不阻塞界面              │
    │                              │
    │    ← Completed { ... }  ──── │
    │                              │ thread 退出
    │ 显示完成摘要                  │
`

- worker thread 和 egui 主线程之间使用 mpsc::channel 通信
- egui 主线程不执行任何 I/O 操作
- worker thread 不接触任何 UI 状态
- GUI 关闭时，使用 CancellationToken 通知 worker thread 优雅退出
- 使用 	ry_recv() 而非 
ecv() 避免 UI 卡顿

---

## 7. CLI JSON Output & Purpose Clarification

CLI 的 --json 输出用于以下场景：

| 场景 | 说明 |
|------|------|
| 脚本自动化 | PowerShell 脚本：
uwa backup --source X --dest Y --json | ConvertFrom-Json 做后续处理 |
| 外部工具集成 | 其他工具或 CI 系统解析 nuwa CLI 的 JSON 输出 |
| 日志记录 | 将 JSON 输出重定向到文件做审计日志 |
| **不作为 GUI 通信方式** | GUI 不解析 CLI JSON 输出。GUI 直接调用 lib.rs API |

### 关系图

`
nuwa CLI  ──→  lib.rs  ←──  nuwa GUI
  │                          │
  └── JSON output ──→ 脚本/自动化/外部工具
`

CLI 和 GUI 都通过 lib.rs 调用核心功能。
JSON 输出是 CLI 的附加功能，不是 GUI 的通信链路。

---

## 8. Testing Strategy

### 8.1 Phase 1 回归测试

`
cargo test                          # 全部 22/22 测试通过
cargo clippy --all-targets -- -D warnings  # 无警告
cargo build --offline               # 离线构建成功
`

### 8.2 Phase 2 新增测试

| 测试文件 | 测试内容 | 类型 |
|---------|---------|------|
| config_tests.rs | TOML 解析验证、非法 TOML 错误、job 覆盖逻辑、多 job 执行 | Unit |
| history_tests.rs | 记录写入/查询、按操作类型过滤、重建逻辑（从 manifest 重建 SQLite 索引） | Unit |
| prune_tests.rs | keep-count 逻辑、keep-days 逻辑、dry-run 不删除、manifest 损坏时处理、不删除最后一个备份 | Unit |
| scheduler_tests.rs | schtasks XML 模板生成、参数验证、已存在任务检查 | Unit (mock schtasks) |
| smb_tests.rs | UNC 路径解析、path safety 验证、UNC 不可达时错误处理 | Unit |

### 8.3 CLI E2E 测试

| 场景 | 验证点 |
|---------|------|
| 初始化 + 备份 + 历史 | 
uwa init → 配置 job → 
uwa backup --job test → 
uwa history 显示记录 |
| 保留策略 + 确认 | 创建 10 个备份 → 
uwa prune --keep-count 5 → 仅保留 5 个 |
| 调度 + 任务 + 验证 | 创建计划任务 → 触发执行 → 验证备份结果 |

### 8.4 GUI 测试策略

| 测试类型 | 说明 |
|---------|------|
| 编译检查 | cargo build --features gui 编译成功 |
| 页面导航 | GUI 启动后所有 7 个页面可正常切换 |
| Clone 页面 | 打开 Clone 页面显示 "Coming Soon" 且无可用操作 |
| Phase 1 core 不受影响 | cargo build --no-default-features 不带 gui 和 history 时仅构建 core |
| 回归测试 | cargo test 确保 22 个 Phase 1 测试仍然通过 |

---

## 9. Risk Assessment

| 风险 | 概率 | 影响 | 缓解措施 |
|------|:----:|------|---------|
| egui 增加构建时间 | MEDIUM | egui 编译较慢，影响开发迭代 | 只在 vendor 或 CI 时完整编译，本地 target 缓存 |
| SQLite bundled 体积 | LOW | rusqlite bundled 包含 SQLite C 代码 | history feature 默认启用，CLI 和 GUI 都需要 |
| GUI 引入新 bug 影响 Phase 1 core | MEDIUM | gui 模块可能意外引入修改 backup.rs | 严格 CI 检查 Phase 1 测试 + code review |
| schtasks 在不同 Windows 版本行为差异 | LOW | 中文/英文 Windows 的 schtasks XML 输出不同 | 使用 /FO CSV 格式解析 + 提前测试多语言环境 |
| SMB 路径 canonicalize 问题 | MEDIUM | UNC 路径在 Phase 1 canonicalize 函数上有兼容风险 | Phase 1 已有 canonicalize_partial 机制应扩展到 UNC |

---

## 10. Phase 1 Core Freeze Guarantee

Phase 2 以下文件**严格不修改**：

`
src/backup.rs      src/restore.rs     src/verify.rs      src/list.rs
src/manifest.rs    src/checksum.rs    src/storage.rs     src/errors.rs
src/diskspace.rs   src/main.rs (可能新增 mode 切换但不修改现有 CLI 行为)
src/lib.rs         (可能新增 pub mod 但不修改现有模块代码)
tests/backup_restore_tests.rs  (仅新增测试，不修改现有)
`

任何 Phase 1 core 修改必须：
1. 符合 AGENTS.md §29 要求
2. 用户明确批准
3. 有 restore validation 计划
4. 向后兼容
5. **不影响现有测试通过率**

## 11. Entry Point Design

### CLI 入口 (main.rs)

`
ust
fn main() {
    let args: Vec<String> = std::env::args().collect();

    #[cfg(feature = "gui")]
    if args.len() == 1 {
        // 无参数时启动 GUI
        gui::run();
        return;
    }

    // CLI 模式 — Phase 1 所有命令不受影响
    cli::run();
}
`

### GUI 入口 (gui/mod.rs)

`
ust
#[cfg(feature = "gui")]
pub fn run() {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1200.0, 800.0)),
        min_window_size: Some(egui::vec2(900.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "Nüwa Backup",
        options,
        Box::new(|_cc| Box::new(NuwaApp::new())),
    );
}
`

---

## 12. Task Breakdown

| Task ID | 功能 | 依赖 | 估计工时 |
|:-------:|------|:----:|:--------:|
| T2-01 | 配置文件系统 (config.rs, nuwa init, 多job) | Phase 1 core | 2h |
| T2-02 | 备份历史 (history.rs, SQLite schema) | T2-01 | 2h |
| T2-03 | CLI 输出增强 (cli_output.rs, JSON, 进度) | 无 | 1h |
| T2-04 | 保留策略 (prune.rs) | T2-02 | 1.5h |
| T2-05 | Windows Task Scheduler (scheduler.rs) | T2-01 | 2h |
| T2-06 | SMB/UNC 路径支持 | Phase 1 core | 1h |
| T2-07 | GUI — 依赖引入 + 脚手架 | T2-01 ~ T2-06 | 1h |
| T2-08 | GUI — 7 个页面实现 | T2-07 | 4h |
| T2-09 | GUI — Clone placeholder 页面 | T2-07 | 0.5h |

### Task Dependency Graph

`
T2-01 (Config)
  ├── T2-02 (History)
  │     └── T2-04 (Prune)
  ├── T2-05 (Scheduler)
  └── T2-07 (GUI Scaffold)
        ├── T2-08 (GUI Pages)
        ├── T2-09 (Clone Placeholder)
        └── 依赖 T2-01 ~ T2-06 (GUI 调用 lib.rs API)

T2-03 (CLI Output) — 独立，可与任意任务并行
T2-06 (SMB) — 相对独立，可与 T2-01~T2-05 并行
`

---

## Revision History

| Version | Date | Reason for change |
|---------|------|-------------------|
| v1.0 | 2026-07-05 | Initial Technical Design for Phase 2 — module structure, 7 features design, egui analysis, worker thread model, task breakdown, testing strategy. Includes all 5 corrections from Phase 2 revision. |

---

## 12. Product Runtime Language Policy

The current product version is **English-only at runtime**.

- CLI output, error messages, success messages, and generated config templates must be English.
- GUI text (page names, labels, buttons, status messages) must be English.
- Code comments, identifiers, and test strings must be English.
- All generated output (JSON, config templates) must use ASCII-safe English.
- Documentation can remain in Chinese.
- See AGENTS.md §"Product Runtime Language Policy" for full rules and compliance check.
