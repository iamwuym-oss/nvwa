# HANDOFF-001 — Nüwa Backup 项目交接文档

> **创建日期:** 2026-07-12
> **提交:** `f795785` (codex/p-08-full-regression-and-cleanup)
> **写给:** 完全没有上下文的后续会话
> **阅读此文档前请勿做任何代码修改。**

---

## 1. 项目是什么

Nüwa Backup（女娲备份）是一个**本地优先、单机部署**的备份与灾难恢复产品，面向 Windows 工作站、服务器、个人用户和小型办公场景。

核心原则：**数据可恢复性高于开发速度、界面效果和代码数量。**

**不是**云备份、企业集中管理、杀毒软件、磁盘克隆工具（后两项虽在远期规划中但当前禁止实现）。

---

## 2. 技术栈（已冻结，不可更改）

| 层 | 技术 | 说明 |
|----|------|------|
| 语言 | Rust | 核心引擎 |
| CLI | 手动解析 | 无 clap |
| JSON | serde_json | — |
| SHA-256 | sha2 crate | — |
| 压缩 | zstd crate | feature-gated (`compress`) |
| 存储 | Repository Engine | 唯一数据底座 |
| 桌面 GUI | Tauri 2.0 | WebView2 |
| 前端 | React 19 + TypeScript + Vite 6 | `ui/` 目录 |
| 数据库 | SQLite (rusqlite) | feature-gated (`repository`) |
| IPC | Tauri invoke() | — |

**严禁引入:** FastAPI, Python 后端, Vue, jQuery, Vanilla JS 替代 React

---

## 3. 已完成阶段总览

| 阶段 | 内容 | 状态 |
|:----:|------|:----:|
| **Phase 0** | 项目护栏、MVP定界、设计决策日志 | CLOSED |
| **Phase 1** | 文件级备份/恢复 CLI（flat-file 格式） | CLOSED（冻结基线） |
| **Phase 2** | CLI 可用性：配置、历史、调度、SMB/UNC、egui GUI | CLOSED |
| **Phase 2.5** | Tauri 2.0 桌面 GUI + React 前端 + Application Layer | CLOSED |
| **Phase S** | Repository 存储引擎（13个子任务 S-01~S-13） | CLOSED |
| **Phase P** | Flat File → Repository 迁移（9个子任务 P-00C~P-08） | **ALL COMPLETE** |

---

## 4. 当前状态（开始工作前必须了解）

### 4.1 架构图

```
React UI (ui/src/)
    |
Tauri invoke() IPC (src-tauri/)
    |
Tauri Command Layer (thin wrapper, src-tauri/src/commands/)
    |
Application Service Layer (src/app/services/) — 数据编排
    |
Repository Engine (src/repository/) — 唯一存储后端
```

### 4.2 CLI 命令

| 命令 | 说明 |
|------|------|
| `nuwa init [--config <path>]` | 初始化配置文件 |
| `nuwa backup --source <path> --repo <path> [--compress] [--json]` | Repository 备份 |
| `nuwa restore --backup <repo_path> --dest <path> [--overwrite] [--json]` | Repository 恢复 |
| `nuwa history --dest <repo_path> [--limit N] [--json]` | 操作历史 |
| `nuwa schedule (create|list|delete)` | Windows 任务计划程序 |

### 4.3 源码结构（关键目录）

```
src/
  main.rs                     # 程序入口（Repository-only）
  cli.rs                      # CLI 参数解析
  lib.rs                      # 库入口（无 flat-file 模块）
  checksum.rs                 # SHA-256 工具
  config.rs                   # 配置系统
  errors.rs                   # 错误码
  history.rs                  # SQLite 操作历史
  scheduler.rs                # Windows 任务计划
  path_support.rs             # 路径验证
  cli_output.rs               # CLI 输出格式化
  diskspace.rs                # 磁盘空间检测

  app/                        # Application Layer
    services/                 #   业务服务（9个服务）
    models/                   #   API 模型（8个模型）
    error.rs                  #   App 层错误

  repository/                 # Repository Engine（核心）
    repo_manager.rs           #   Repository 打开/创建/句柄
    backup_writer.rs          #   备份写入器
    file_restore_reader.rs    #   文件恢复读取器（370行）
    path_security.rs          #   共享路径安全模块
    block_store/              #   块存储
    metadata/                 #   元数据链
    chunk_engine/             #   分块引擎
    catalog/                  #   目录引擎
    block_map/                #   块映射
    transaction/              #   事务日志 + 崩溃一致性
    verify/                   #   三级校验
    retention/                #   保留策略
    recovery/                 #   修复重建
    cli/                      #   Repository CLI 子命令

ui/src/                       # React 前端
  pages/                      #   页面组件
  components/                 #   通用组件
  api/                        #   API 调用层

tests/                        # 集成测试
  repository_integration_tests.rs   # 31 个 Repository 集成测试
  backup_service_tests.rs           # 9 个备份服务测试
  restore_service_tests.rs          # 8 个恢复服务测试
  config_service_tests.rs           # 19 个配置服务测试
  dashboard_service_tests.rs        # 4 个仪表盘测试
  file_browser_service_tests.rs     # 9 个文件浏览器测试
  restart_persistence_tests.rs      # 3 个持久化测试
```


### 4.4 质量门结果（当前基线）

| 门禁 | 结果 |
|------|------|
| `cargo fmt --check` | ✅ PASS |
| `cargo clippy --all-targets --features repository -- -D warnings` | ✅ PASS |
| `cargo build --features repository` | ✅ PASS |
| `cargo test --features repository` | ✅ **365 passed, 0 failed** |

### 4.5 构建方式

```bash
# 标准构建（启用 Repository 功能）
cargo build --features repository

# 运行测试
cargo test --features repository

# 运行代码质量检查
cargo fmt --check
cargo clippy --all-targets --features repository -- -D warnings
```

### 4.6 Git 远端

```
origin  https://github.com/iamwuym-oss/nvwa.git
```

当前分支: `codex/p-08-full-regression-and-cleanup`
最近提交: `f795785` — P-07/P-08 完成

> ⚠️ **重要:** 当前分支未合并到 master。如果需要开始新任务，请在 master 或基于 master 的新分支上工作。

---

## 5. Phase P 已完成任务详情

| 任务 | 名称 | 关键输出 | 状态 |
|:----:|------|---------|:----:|
| **P-00** | 数据契约冻结 | 恢复点状态机、Catalog路径安全合约、崩溃恢复规则 | ✅ |
| **P-00C** | 崩溃恢复安全修复 | CrashConsistencyManager 重写、RepoHandle CRUD API | ✅ |
| **P-01** | Repository 备份写入器 | RepositoryBackupWriter (397行) + CLI `--repo` 参数 | ✅ |
| **P-02** | Repository 恢复读取器 | FileRestoreReader + 16步预检 + 流式恢复 + 27测试 | ✅ |
| **P-03** | 目录与边界语义 | 多级目录、空文件、Unicode、源文件变化检测 | ✅ |
| **P-04** | 故障注入 | 10个故障场景测试（崩溃/磁盘满/取消/损坏检测） | ✅ |
| **P-05** | FS 工具提取 | path_security.rs 共享路径安全模块 | ✅ |
| **P-06** | 全链路集成 | CLI + App Services + Tauri + React UI 端到端打通 | ✅ |
| **P-07** | 删除 Flat File 模块 | 删除 11 个源文件、2 个测试文件、45 个临时脚本 | ✅ |
| **P-08** | 全量回归+残留扫描 | 6层扫描零残留、370测试全通过 | ✅ |

### 关键验收指标

- **数据可恢复性:** 备份后关闭句柄、重新打开 Repository、读取真实磁盘数据、恢复后 SHA-256 比对一致
- **崩溃安全:** 任何阶段崩溃不会产生"显示成功实际不可恢复"的恢复点
- **无双格式:** 所有新功能以 Repository Engine 为唯一数据底座

---

## 6. 踩过的坑（⚠️ 新会话必读）

### ⚠️ 坑 1: 文件编码 — UTF-8 without BOM 会导致中文/特殊字符损坏

**现象:** 项目的 .md 文件中使用中文引号、em-dash、箭头等字符时，如果通过 PowerShell 重定向或某些 Rust 文件操作写入，UTF-8 编码可能被破坏，显示为乱码。

**解决方法:**
- 始终使用 `fs.writeFileSync(path, content, "utf8")` 明确指定编码
- 不要使用 PowerShell 的 `>` 重定向操作 .md 文件

### ⚠️ 坑 2: Rust format!() + 原始字符串中的花括号

**现象:** 在 `format!()` 中用 `"{{"json":"value"}}"` 生成 JSON 输出时，普通字符串中的 `"{{"` 会被编译器解释为字符串结束，导致编译错误。

**解决方法:**
```rust
// 正确：使用 r#"..."# 原始字符串 + {{ 作为字面花括号
format!(r#"{{"key":"{}","count":{}}}"#, value, count)
```

### ⚠️ 坑 3: SQL SELECT 列索引必须与 struct 字段完全一致

**现象:** SELECT 列顺序错乱 + 重复列导致 `InvalidColumnType` — 13 个测试集体失败。

**教训:** SELECT 顺序必须精确匹配 Rust struct 字段顺序；列不得重复；`row.get(n)` 的 n 是 SELECT 中列的 0-based 索引。

### ⚠️ 坑 4: clippy needless_borrow 对 String 和 &str 行为不同

**解决方法:** 使用 `msg.as_str()` 将 `String` 转换为 `&str`，而非 `&msg`。

### ⚠️ 坑 5: PowerShell 不支持 `&&` 命令链

**解决方法:** 使用 `;` 代替 `&&`。

### ⚠️ 坑 6: Tauri 前端 VITE_MOCK_DATA=true 会切断真实 API

**解决方法:** 端到端测试时必须 `VITE_MOCK_DATA=false`。

### ⚠️ 坑 7: Phase 1 flat-file 模块已全部删除

不要寻找 `backup.rs`、`restore.rs`、`manifest.rs` 等文件 — 它们在 P-07 中已删除。当前只有 Repository Engine。

### ⚠️ 坑 8: flat-file 回归测试已删除

`backup_restore_tests.rs` 和 `gui_integration_tests.rs` 已删除。当前只有 7 个集成测试文件和 lib 单元测试。

## 7. 文档修复日志（2026-07-12）

本次 handoff 创建后又进行了一轮文档 gap 修复，找出了文档与项目实际代码之间的脱节并修复。

### 7.1 修复清单

| 类型 | 文件 | 修复内容 |
|:----:|------|---------|
| 过时架构 | \`docs/project/03_Nuwa_Architecture_Design.md\` | §1.1/§1.2/§8/§9/§11.2/§12.2/§13.2/§15/Appendix A — 共 11 处 egui+daemon+ZeroMQ 引用更新为 Tauri 2.0 + React 现状 |
| 过时引用 | \`docs/project/04_Functional_Specification.md\` | Linux 桌面版 "egui UI" → "Tauri 2.0 + React" |
| 编码损坏 | \`docs/project/02_Development_Plan.md\` | GB2312 → UTF-8 with BOM（解决中文乱码） |

### 7.2 修复原则

- **Phase 1 冻结基线文档**（Technical_Baseline、Code_Map、Final_Acceptance）保持不变——它们是 Phase 1 时期的历史记录，flat-file 引用在当时的上下文中是正确的
- **Phase 0 边界文档**（00_Codex_Working_Guardrails.md、09_MVP_Boundary_and_Risk_Correction.md）保留 egui 引用作为历史记录——这些是 MVP 定界时期的产物，egui 在当时是正确的技术名称
- **AGENTS.md** 已在之前更新至 v5.4，正确反映 Phase P Complete 状态

### 7.3 验证

| 门禁 | 结果 |
|------|------|
| \`cargo build --features repository\` | ✅ PASS |
| \`cargo test --features repository\` | ✅ **365 passed, 0 failed** |
| \`cargo fmt --check\` | ✅ PASS |
| \`cargo clippy --all-targets --features repository -- -D warnings\` | ✅ PASS |

---

## 8. 下一步计划

### 短期（用户批准后可立即开始）

| 任务 | 优先级 | 说明 |
|:----:|:------:|------|
| **合并 PR** | 🔴 最高 | 当前分支 → merge 到 master |
| **Phase 3 规划** | 🟡 中 | NTFS 非系统卷映像备份（T3-00 已完成文档） |
| **Phase 3 编码** | 🟡 中 | .nwb 格式 v0.2、VSS 快照、卷级备份/恢复 CLI |

### 中期（已规划但未授权）

| 阶段 | 内容 | 状态 |
|:----:|------|:----:|
| Phase 3.5 | GUI 卷管理页面 | 未授权 |
| Phase 4 | 系统恢复 / WinPE / BMR | 未授权 |
| Phase 5 | 磁盘克隆 | 未授权 |
| Phase 6+ | 差异备份、加密、跨平台 | 未授权 |

### 远期排除项（永不实现）

云备份、云同步、手机备份、M365 备份、杀毒、勒索软件防护、AI 检测、企业集中管理、SaaS 账户系统。

---

## 9. 新会话启动清单

当新会话开始时，必须按顺序执行：

1. **阅读本文档** — 建立项目认知
2. **确认当前分支和工作树** — `git status`、`git branch`
3. **确认远端** — `git remote -v` 确认是 `iamwuym-oss/nvwa`
4. **阅读 AGENTS.md** — 项目规则、安全红线、汇报格式
5. **阅读工程记忆** — `docs/project/PROJECT_ENGINEERING_MEMORY.md`
6. **阅读文档索引** — `docs/project/DOCUMENT_INDEX.md`
7. **确认可编译** — `cargo build --features repository`
8. **确认测试通过** — `cargo test --features repository`
9. **询问用户任务** — 确认当前要做什么

### 安全红线（精简版）

- ❌ 不得对生产数据使用 `unwrap()` / `expect()`
- ❌ 不得吞掉关键错误（`let _ =`、空 catch、默认值 fallback）
- ❌ 不得删除/弱化/`#[ignore]` 测试以获得绿色结果
- ❌ 不得在 Repository 失败时回退到旧 Flat File
- ✅ 恢复路径必须防止 `..`、symlink、junction、reparse point 逃逸
- ✅ 只有完整写入 + 落盘 + 验证后才可标记 COMMITTED

---

## 10. 关键文档索引

| 文档 | 路径 | 作用 |
|------|------|------|
| AGENTS.md | `/AGENTS.md` | 项目规则与安全红线 |
| 工程记忆 | `docs/project/PROJECT_ENGINEERING_MEMORY.md` | 阶段状态与模块清单 |
| 文档索引 | `docs/project/DOCUMENT_INDEX.md` | 所有文档导航 |
| 数据契约 | `docs/phase-s/P-00_File_Backup_Repository_Data_Contract.md` | 恢复点状态机、崩溃恢复规则 |
| 恢复计划 | `docs/phase-s/P-02_Repository_Restore_Plan.md` | FileRestoreReader 设计 |
| Phase S 架构 | `docs/phase-s/Nuwa_Repository_Engine_Architecture_v1.0.md` | Repository 引擎架构 |
| Phase S v1.1 | `docs/phase-s/Nuwa_Repository_Engine_Architecture_v1.1.md` | 企业级就绪修订 |
| 设计决策 | `docs/phase-0/08_Design_Decision_Log.md` | 所有已确认设计决策 |
