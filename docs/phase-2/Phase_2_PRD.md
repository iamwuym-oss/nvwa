# Phase 2 — Product Requirements Document (PRD)

**Product:** Nüwa Backup (女娲备份)
**Phase:** 2 — 可用文件备份
**Version:** v1.0
**Date:** 2026-07-05
**Status:** DRAFT — 待用户审阅后批准编码

---

## 1. Product Overview

### 1.1 Phase 2 Goal

Phase 2 的目标是让 Nüwa Backup **从一个 CLI 原型变成一个可日常使用的文件级备份工具**。

| 维度 | Phase 1 | Phase 2 |
|------|---------|---------|
| 操作方式 | 纯 CLI 手动命令 | CLI + 本地桌面 GUI 双模式 |
| 配置 | 每次敲参数 | TOML 配置文件 + backup job |
| 自动化 | 无 | Windows Task Scheduler 定时备份 |
| 历史 | 无 | 本地历史记录可查 |
| 清理 | 手动删除目录 | 保留策略自动清理 |
| 网络目标 | 无 | SMB / UNC 共享路径 |
| 输出 | 纯文本 | 进度条 + JSON 输出 |

### 1.2 Core Principle

**Phase 2 的核心目标是"可日常使用的文件级备份工具"，不是镜像备份、卷级备份或系统恢复。**

所有功能必须围绕文件级备份展开。任何涉及磁盘块、分区、VSS、系统卷、WinPE、克隆真实功能的代码均被严格禁止。

---

## 2. Target Users

| 用户类型 | 场景 |
|---------|------|
| 个人 Windows 用户 | 保护重要文件，定时自动备份 |
| 创作者/开发者 | 项目文件、素材定期备份 |
| 小型办公室 | 共享文件定期备份到 NAS/SMB |
| PC 维修店 | 系统迁移前的文件备份 |
| Windows Server 独立节点 | 边缘服务器文件保护 |

---

## 3. Feature Requirements

### 3.1 配置文件 / Backup Job

**ID:** P2-REQ-01
**Priority:** P0 — 基础依赖，其他功能依赖此模块

| 需求 | 说明 |
|------|------|
| 配置文件格式 | TOML，路径：Windows 上 `%APPDATA%/nuwa/config.toml` |
| 初始化命令 | `nuwa init` — 生成默认配置文件 |
| 多 job 定义 | 每个 job 包含：name, source, dest, compress (bool), retention (可选) |
| 按 job 执行 | `nuwa backup --job <name>` |
| 默认 job | 未指定 job 时使用 `[job.default]` |
| 配置编辑 | 手动编辑 TOML，或通过 GUI Settings 页面管理 |
| CLI 覆盖 | `--source` / `--dest` 可临时覆盖 job 中的配置 |

**验收标准：**
1. `nuwa init` 生成合法 TOML 文件 ✅
2. 定义 3 个不同 job 后，`nuwa backup --job <name>` 各自正确执行 ✅
3. 配置文件丢失时给出明确错误提示（非 panic）✅
4. 非法 TOML 给出解析错误及行号 ✅

---

### 3.2 备份历史

**ID:** P2-REQ-02
**Priority:** P0 — 数据层核心

| 需求 | 说明 |
|------|------|
| 记录范围 | 每次 backup / restore / verify 操作的结果 |
| 存储方案 | SQLite 候选，位置：`<备份目标目录>/.nuwa_history.db` |
| 记录字段 | backup_id, operation(backup/restore/verify), timestamp, source, dest, file_count, total_bytes, duration_ms, exit_code, status(success/failure/partial) |
| 查询命令 | `nuwa history [--limit N] [--operation backup]` |
| GUI 显示 | History 页面表格展示 |
| JSON 输出 | `nuwa history --json` |
| 可重建性 | 通过扫描所有备份目标目录下的 manifest.json 可重建历史库 |

**验收标准：**
1. 10 次 backup 后 `nuwa history` 显示 10 条记录 ✅
2. 重建命令（`nuwa history --rebuild`）从 manifest 重建历史库 ✅
3. SQLite 损坏后，删除 `.nuwa_history.db` 并重建，历史不丢失 ✅
4. 历史记录不影响备份/恢复正确性 ✅

---

### 3.3 保留策略

**ID:** P2-REQ-03
**Priority:** P1 — 自动化能力

| 需求 | 说明 |
|------|------|
| 按数量保留 | `nuwa prune --keep-count N` — 保留最近 N 个备份点 |
| 按天数保留 | `nuwa prune --keep-days N` — 保留最近 N 天的备份点 |
| 组合模式 | 同时指定 `--keep-count` 和 `--keep-days`，取更严格者 |
| dry-run 模式 | `nuwa prune --dry-run` — 只显示会被删除的备份点，不实际删除 |
| 安全删除 | 删除前验证该备份点的 manifest.json 存在且可读 |
| 保留最后一个 | 即使超过保留数量，也不删除最后一个可用的备份点 |
| 自动触发 | 每次 backup 成功后自动执行 prune（使用 job 中配置的 retention 策略） |
| 手动触发 | 用户可随时执行 `nuwa prune` |

**验收标准：**
1. 创建 10 个备份后 `nuwa prune --keep-count 5` 只保留最近 5 个 ✅
2. `--dry-run` 只显示不删除 ✅
3. manifest 损坏的备份点仍被计入数量，但给出损坏警告 ✅
4. 保留策略执行后 `nuwa list` 显示正确的剩余备份点数 ✅

---

### 3.4 Windows Task Scheduler 计划任务

**ID:** P2-REQ-04
**Priority:** P1 — 自动化能力

| 需求 | 说明 |
|------|------|
| 实现方式 | 集成 Windows Task Scheduler（schtasks.exe），不做 daemon/service |
| 一次性 | `nuwa schedule create --job <name> --once --at "2026-08-01 10:00"` |
| 每日 | `nuwa schedule create --job <name> --daily --at "22:00"` |
| 每周 | `nuwa schedule create --job <name> --weekly --days Mon,Fri --at "03:00"` |
| 每月 | `nuwa schedule create --job <name> --monthly --day 1 --at "02:00"` |
| 开机延迟 | `nuwa schedule create --job <name> --on-startup --delay 300`（秒） |
| 查看任务 | `nuwa schedule list` |
| 删除任务 | `nuwa schedule delete --task-id <id>` |
| GUI 管理 | Schedule 页面查看/创建/删除计划任务 |

**验收标准：**
1. 创建每日任务后，Windows Task Scheduler 中可见 ✅
2. 删除任务后 Windows Task Scheduler 中不再显示 ✅
3. 使用不存在的 job 名创建任务时给出错误提示 ✅
4. GUI Schedule 页面与 CLI 显示一致 ✅

---

### 3.5 SMB / UNC 目标

**ID:** P2-REQ-05
**Priority:** P1 — 扩展能力

| 需求 | 说明 |
|------|------|
| 路径格式 | 支持 `\\server\share\path` UNC 路径 |
| 凭据 | 使用当前 Windows 登录凭据，不保存 SMB 账号密码 |
| 备份到 SMB | `nuwa backup --source C:\Data --dest \\nas\backup\mydata` |
| 从 SMB 恢复 | `nuwa restore --backup \\nas\backup\mydata\20260705_Data --dest C:\Restored` |
| 错误处理 | SMB 不可达时返回 I/O 错误，提示用户检查网络连接 |
| 凭据管理 | 不做凭据管理器，不做 `--username` / `--password` 参数 |

**验收标准：**
1. 可备份到本地映射的 SMB 共享路径 ✅
2. 可从 SMB 共享路径恢复 ✅
3. SMB 服务不可达时给出清晰错误提示 ✅
4. UNC 路径与本地路径的 path safety 检查（src≠dst, dest-in-src, src-in-dest）正常工作 ✅

---

### 3.6 CLI 输出增强

**ID:** P2-REQ-06
**Priority:** P1 — 用户体验

| 需求 | 说明 |
|------|------|
| 进度显示 | backup 和 restore 过程中显示文件级进度条（当前文件/总文件数、已传输大小） |
| JSON 输出 | `--json` 参数，所有命令支持结构化 JSON 输出 |
| 摘要表格 | backup/restore/verify 完成后显示摘要表格（文件数、总大小、耗时、状态） |
| 颜色输出 | 成功（绿）、失败（红）、警告（黄） |

**验收标准：**
1. `nuwa backup --source <dir> --dest <dir> --json` 输出合法 JSON ✅
2. GUI 通过 lib.rs ProgressCallback 直接获取进度 ✅
3. 进度条不在非交互终端（如管道）中产生乱码 ✅

---

### 3.7 本地桌面 GUI 管理界面

**ID:** P2-REQ-07
**Priority:** P0 — Phase 2 核心交付物

#### 3.7.1 技术方向

| 项目 | 值 |
|------|-----|
| 形态 | 本地桌面 GUI，参考 Acronis True Image 交互风格 |
| 候选技术 | egui + eframe（纯 Rust，零运行时依赖） |
| 不是 | ❌ Web GUI / 浏览器管理后台 |
| 不是 | ❌ FastAPI / Python backend |
| 不是 | ❌ Vanilla JS / Tailwind / React / Vue / jQuery |
| 运行方式 | 双击运行，单机本地桌面应用 |

#### 3.7.2 页面结构

| 顺序 | 页面 | 功能 | Phase 2 状态 |
|:----:|------|------|:------------:|
| 1 | **Dashboard** | 最近备份状态、备份目标状态、最近任务结果、风险提示 | ✅ 实现 |
| 2 | **Backup** | 选择备份源/目标、选择/创建 backup job、执行文件级备份、进度和结果展示 | ✅ 实现 |
| 3 | **Restore** | 浏览备份点、浏览 manifest 文件和目录、选择恢复目标、执行文件级恢复、校验结果展示 | ✅ 实现 |
| 4 | **History** | 备份/恢复/验证历史表格、成功/失败状态、文件数/容量/耗时/退出码 | ✅ 实现 |
| 5 | **Schedule** | 已配置计划任务列表、创建/删除/查看计划任务 | ✅ 实现 |
| 6 | **Clone** | **Coming Soon / Phase 5 placeholder**。仅显示禁用提示页面 | ❌ 禁用 |
| 7 | **Settings** | 默认 source/dest 配置、retention 配置、job 配置、UI 选项 | ✅ 实现 |

#### 3.7.3 Clone 页面规则

| 规则 | 说明 |
|------|------|
| 页面可以出现在导航栏和菜单中 | 作为 Future placeholder |
| 页面必须显示明确的禁用提示 | **"Disk Clone is planned for Phase 5 and is not available in Phase 2."** |
| 不得实现任何真实克隆功能 | 无 clone engine、无 clone CLI 命令 |
| 不得访问磁盘、卷、分区或 PhysicalDrive | 禁止分区表/GPT/MBR/NTFS bitmap 读取 |
| 不得创建 clone 相关的 Rust 模块或 structs | 禁止 Phase 5 代码提前引入 |
| 不得让用户误以为 Phase 2 可以克隆磁盘 | 必须明确标注 "Coming Soon" |

#### 3.7.4 GUI 与 CLI 关系

```
┌─────────────────────┐      ┌──────────────────────┐
│   nuwa-backup.exe   │      │   nuwa-gui.exe       │
│    (CLI mode)       │      │   (GUI mode)         │
│                     │      │                      │
│  cli.rs  →  lib.rs  │      │  gui.rs  →  lib.rs   │
│                     │      │                      │
│  共享同一 lib.rs    │      │  共享同一 lib.rs      │
│  所有核心逻辑       │      │  所有核心逻辑         │
└─────────────────────┘      └──────────────────────┘
```

- CLI 和 GUI 共享同一份 lib.rs 核心库
- GUI 不调用任何未来阶段（Phase 3/4/5/6+）功能
- GUI 的 Clone 页面只能显示禁用提示，不能调用任何 clone 相关代码

---

## 4. Non-Functional Requirements

| 类别 | 要求 |
|------|------|
| 二进制体积 | GUI 版本可接受体积增加，但不应超过 50MB（含 egui 依赖） |
| 启动时间 | GUI 从双击到主界面显示 ≤ 3 秒 |
| 构建 | 必须支持 `--offline` 构建（所有依赖已 vendor 或缓存） |
| 兼容性 | Windows 10 / Windows 11 / Windows Server 2019/2022 |
| 性能 | GUI 操作不阻塞备份/恢复执行 |
| 安全性 | GUI 不保存任何凭据到磁盘 |

---

## 5. Phase Boundaries

### 5.1 Phase 2 Confirmed Scope（7 项）

| # | 功能 | 交付形态 |
|:-:|------|---------|
| 1 | 配置文件 / backup job | CLI + GUI Settings |
| 2 | 备份历史 | CLI + GUI History |
| 3 | 保留策略 | CLI + GUI Settings |
| 4 | Windows Task Scheduler | CLI + GUI Schedule |
| 5 | SMB / UNC 目标 | CLI + GUI Backup |
| 6 | CLI 输出增强 | CLI 改进 |
| 7 | 本地桌面 GUI | 独立可执行程序 + 6 个可用页面 + 1 个 placeholder |

### 5.2 Phase 2 Forbidden（21 项）

详见 `Phase_2_Revised_Plan.md` §5 — 含 .nwb、VSS、卷级备份、磁盘级备份、系统卷、系统恢复、WinPE、BCD、克隆（真实功能）、PhysicalDrive、分区表、NTFS bitmap、异机还原、驱动注入、差异备份、增量备份、AES 加密、daemon、Web GUI、FastAPI、Vanilla JS。

---

## 6. Acceptance Criteria Summary

| ID | 验收标准 |
|:--:|---------|
| AC-01 | 配置文件系统：`nuwa init` + 多 job + `nuwa backup --job <name>` 完整闭环 |
| AC-02 | 备份历史：10 次 backup 后 `nuwa history` 显示 10 条记录，可重建 |
| AC-03 | 保留策略：`nuwa prune --keep-count 5` 正确保留最近 5 个备份 |
| AC-04 | 计划任务：通过 CLI 创建每日任务后 Windows Task Scheduler 可见 |
| AC-05 | SMB 目标：可备份/恢复到 UNC 路径 |
| AC-06 | CLI 增强：`--json` 输出合法 JSON |
| AC-07 | GUI 管理界面：6 个可用页面 + Clone placeholder，操作流程与 Acronis True Image 一致 |
| AC-08 | Clone placeholder：页面显示 "Coming Soon" + 无法执行任何克隆操作 |
| AC-09 | 全部 22 项 Phase 1 测试仍通过 |
| AC-10 | cargo fmt / clippy / build / test 全部通过 |

---

## Revision History

| Version | Date | Reason for change |
|---------|------|-------------------|
| v1.0 | 2026-07-05 | Initial PRD for Phase 2 — 7 features, GUI requirements, clone placeholder rules |


---

## 7. Product Runtime Language Policy

The current product version is **English-only at runtime**.

- CLI output, error messages, success messages, and generated config templates must be English.
- GUI text (page names, labels, buttons, status messages) must be English.
- Code comments, identifiers, and test strings must be English.
- Documentation can remain in Chinese.
- See AGENTS.md §"Product Runtime Language Policy" for full rules.
