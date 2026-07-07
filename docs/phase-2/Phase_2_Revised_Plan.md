# Phase 2 — Revised Plan v2

**Product:** Nüwa Backup (女娲备份)
**Date:** 2026-07-05
**Status:** v2 — User confirmed: Phase 2 includes local desktop GUI coding (approval scope: PRD + Technical Design first)

> **HISTORICAL NOTE: The GUI technology decision in this document (egui + eframe) was SUPERSEDED by Phase 2.5.**
> **Current GUI technology: Tauri 2.0 + React + TypeScript + Vite. See docs/phase-2.5/Phase_2_5_Tauri_Migration_Decision.md.**
> **The non-GUI Phase 2 deliverables (config, history, scheduler, SMB, CLI enhancements) are unaffected by this change.**


---

## 1. Phase 2 Core Principle

**Phase 2的核心目标是"可日常使用的文件级备份工具"，不是镜像备份、卷级备份或系统恢复。**

Phase 1 完成了文件级备份的"能不能"（能备份、能恢复、能验证）。
Phase 2 解决的是"好不好用"（自动运行、管理历史、配置持久化、本地图形管理界面）。

---

## 2. Confirmed Scope (Approved for Phase 2 Execution)

以下 7 项是 Phase 2 确认实施的功能：

| # | 功能 | 说明 |
|:-:|------|------|
| 1 | **配置文件 / backup job** | TOML config，`nuwa init`，多 job 定义（每个 job 独立 source/dest/策略），`nuwa backup --job <name>` |
| 2 | **备份历史** | 操作日志记录。SQLite 候选方案，但 manifest 是备份点权威凭证，SQLite 可通过扫描 manifest 重建 |
| 3 | **保留策略** | `nuwa prune --keep-count N / --keep-days N`，dry-run 模式，不删除最后一个可用备份，删除前检查 manifest |
| 4 | **Windows Task Scheduler 计划任务** | 集成 schtasks.exe，支持一次性/每日/每周/每月/开机延迟。不做 daemon/service |
| 5 | **SMB / UNC 目标** | 支持 `\\server\share\path`，使用当前 Windows 登录凭据，不保存 SMB 用户名密码，不做凭据管理器 |
| 6 | **CLI 输出增强** | 更清晰的摘要输出、`--json` 输出模式、进度显示 |
| 7 | **本地桌面 GUI 管理界面** | Acronis True Image-like 交互风格，egui + eframe 候选技术，非 Web GUI，非 FastAPI，非 Vanilla JS。**Phase 2 须有可运行 GUI** |

### 执行顺序

```
Phase 2 上半段（CLI 基础设施）
  ├── 配置文件 / backup job        ← 基础，GUI 和 scheduler 都依赖它
  ├── 备份历史                     ← 数据层，记录所有操作
  └── CLI 输出增强                 ← 用户体验改进

Phase 2 中段（自动化能力）
  ├── 保留策略                     ← 依赖备份历史中的时间信息
  └── Windows Task Scheduler       ← 依赖 config job 定义

Phase 2 下半段（扩展 + GUI）
  ├── SMB / UNC 目标               ← 相对独立，可并行
  └── 本地桌面 GUI 管理界面         ← Phase 2 后期，必须出现可运行管理界面
```

---

## 3. GUI — Phase 2 编码已批准

### 设计确认

| 项目 | 值 | 来源 |
|------|-----|------|
| 形态 | **Acronis True Image-like 桌面 GUI** | ADL-023 + User Decision 2026-07-05 |
| 候选技术 | **egui + eframe**（纯 Rust，零运行时依赖） | ADL-034 |
| 不是 Web GUI | ✅ 确认排除 | Task 2.0A/B |
| 不是 FastAPI / Python / Vanilla JS / Tailwind / React / Vue | ✅ 确认排除 | AGENTS.md Tech Stack Clarification |
| 风格 | 深色科技风，蓝/青主色调 | ADL-023 |
| 运行方式 | 本地桌面应用，单机运行，非浏览器管理后台 | User Decision 2026-07-05 |

### Phase 2 GUI 页面结构

| 页面 | 说明 | Phase 2 状态 |
|------|------|:------------:|
| **Dashboard** | 最近备份状态、备份目标状态、最近任务结果、风险提示 | ✅ 实现 |
| **Backup** | 选择备份源/目标、选择/创建 backup job、执行文件级备份、显示进度和结果 | ✅ 实现 |
| **Restore** | 浏览备份点、浏览 manifest 文件和目录、选择恢复目标、执行文件级恢复、显示校验结果 | ✅ 实现 |
| **History** | 备份/恢复/验证历史、成功/失败状态、文件数/容量/耗时/退出码 | ✅ 实现 |
| **Schedule** | 已配置计划任务、创建/删除/查看计划任务（通过 Windows Task Scheduler） | ✅ 实现 |
| **Clone** | **必须出现在 UI 中，但作为 Phase 5 placeholder。功能禁用。** | ❌ 禁用（Coming Soon） |
| **Settings** | 默认 source/dest、retention 配置、job 配置、UI 选项 | ✅ 实现 |

### Clone 页面严格规则

Clone 页面是 **Phase 2 唯一一个允许存在但功能禁用的页面**。必须遵守：

| 规则 | 说明 |
|------|------|
| ✅ 页面可以出现在导航栏和菜单中 | 作为 Future placeholder |
| ✅ 页面必须显示明确的禁用提示 | **"Disk Clone is planned for Phase 5 and is not available in Phase 2."** |
| ❌ 不得实现任何真实克隆功能 | 无 clone engine、无 clone CLI 命令 |
| ❌ 不得访问磁盘、卷、分区或 PhysicalDrive | 禁止读取分区表、GPT/MBR、NTFS bitmap |
| ❌ 不得创建 clone 相关的 Rust 模块或 structs | 禁止 Phase 5 代码提前引入 |
| ❌ 不得让用户误以为 Phase 2 可以克隆磁盘 | UI 上必须明确标注 "Coming Soon" 或类似状态 |

---

## 4. Explicitly Removed from Phase 2

| 功能 | 原出处 | 处理方式 |
|------|--------|---------|
| `.nwb v0.1` 实验格式 | Checklist P2-16 | **移出 Phase 2 → 放到 Phase 3 Planning** |
| SMTP 邮件通知 | Checklist P2-12 | **后置**，不作为 Phase 2 第一批任务 |
| 备份速度限制（Token Bucket） | Checklist P2-14 | **后置**，不作为 Phase 2 第一批任务 |
| 损坏告警 | Checklist P2-06 | **后置**，Phase 2 中后期再评估 |
| 搜索备份内容 | Checklist P2-07 | **后置**，Phase 2 中后期再评估 |
| 本机同盘备份 | Checklist P2-10 | **后置**，Phase 2 中后期再评估 |
| Windows Toast / 声音通知 | Checklist P2-11, P2-13 | **后置**，Phase 2 中后期再评估 |
| 备份窗口设置 | Checklist P2-15 | **后置**，Phase 2 中后期再评估 |

---

## 5. Phase 2 Strict Boundaries

### 技术栈确认

| 技术 | 状态 |
|------|:----:|
| Rust | ✅ **CONFIRMED** — 唯一后端语言 |
| egui + eframe | 🔵 **候选（技术设计后正式确认）** — Phase 2 可引入 |
| CLI (expandable) | ✅ **CONFIRMED** |
| serde_json | ✅ **CONFIRMED** |
| sha2 | ✅ **CONFIRMED** |
| zstd | ✅ **CONFIRMED** |
| SQLite | 🔵 **候选** — 备份历史存储方案，Phase 2 技术设计中评估 |
| toml | 🔵 **候选** — 配置文件格式 |
| FastAPI / SQLAlchemy / Python | ❌ **PERMANENTLY EXCLUDED** |
| Vanilla JS / Tailwind CSS / React / Vue / jQuery / Web | ❌ **PERMANENTLY EXCLUDED** |

### 禁止清单（共 21 项）

| # | 功能 | 禁止原因 | 所属阶段 |
|:-:|------|---------|:--------:|
| 1 | `.nwb` v0.1 实验格式 | 已移出 Phase 2 | Phase 3 Planning |
| 2 | `.nwb` 正式格式 | 不在 Phase 2 范围 | Phase 3 |
| 3 | VSS 快照集成 | 不在 Phase 2 范围 | Phase 3 |
| 4 | 卷级备份/恢复 | 不在 Phase 2 范围 | Phase 3 |
| 5 | 磁盘级备份 | 不在 Phase 2 范围 | Phase 5 |
| 6 | 系统卷备份 | 不在 Phase 2 范围 | Phase 4 |
| 7 | 系统恢复 | 不在 Phase 2 范围 | Phase 4 |
| 8 | WinPE 恢复介质 | 不在 Phase 2 范围 | Phase 4 |
| 9 | BCD 修复 | 不在 Phase 2 范围 | Phase 4 |
| 10 | **磁盘克隆（真实功能）** | Phase 2 UI 中仅作为 placeholder | Phase 5 |
| 11 | PhysicalDrive 访问 | 不在 Phase 2 范围 | Phase 5+ |
| 12 | 分区表读取 | 不在 Phase 2 范围 | Phase 3 |
| 13 | NTFS bitmap | 不在 Phase 2 范围 | Phase 3 |
| 14 | 异机还原 / Universal Restore | 不在 Phase 2 范围 | Phase 6+ |
| 15 | 驱动注入 | 不在 Phase 2 范围 | Phase 6+ |
| 16 | 差异备份 | 不在 Phase 2 范围 | Phase 6+ |
| 17 | 增量备份 | 永久排除 | 永久排除 |
| 18 | AES 加密 | 不在 Phase 2 范围 | Phase 6+ |
| 19 | daemon / 系统服务 | 不做 daemon/service | Phase 3+ |
| 20 | Web GUI / 浏览器管理后台 | 永久排除 | 永久排除 |
| 21 | FastAPI / Python backend / Vanilla JS / React / Vue / jQuery | 永久排除 | 永久排除 |

### 跨阶段禁止

| 功能 | 永久排除？ |
|------|:----------:|
| 云备份 / 云同步 / 云存储 | ✅ |
| 移动设备备份 | ✅ |
| Microsoft 365 备份 | ✅ |
| 防病毒 / 防恶意软件 / 勒索软件防护 | ✅ |
| AI 威胁检测 | ✅ |
| 企业集中管理 | ✅ |
| 多设备云仪表盘 | ✅ |
| SaaS 账号系统 | ✅ |
| 远程管理 | ✅ |
| 安全套件功能 | ✅ |

---

## 6. SMB 安全原则

SMB 只做 UNC 路径和当前 Windows 凭据：

```
允许：  nuwa backup --source C:\Data --dest \\server\share\Backup
不允许：nuwa backup --source C:\Data --dest \\server\share\Backup --username user --password pass
```

- 不保存 SMB 账号密码到磁盘
- 不实现 SMB 凭据管理界面
- 使用调用进程的 Windows 安全上下文访问共享路径
- 网络共享不可达时：返回 I/O 错误，由用户检查网络连接

---

## 7. SQLite / Manifest 关系原则

### 权威层级

```
manifest.json  ← 每个备份点的唯一权威凭证（不可替代）
     ↑
SQLite 历史库  ← 历史索引，可通过扫描 manifest 重建（可删除、可重建）
```

### 设计约束

- manifest 是备份点完整性的**唯一来源**
- SQLite 只存摘要字段（backup_id, created_at, source_root, file_count, total_bytes, status）
- 如果 SQLite 损坏或丢失，可以扫描 backup dest 目录重建历史
- 如果 manifest 损坏，SQLite 中的历史记录仍然指向一个不可用的备份点（verify 会失败）

---

## 8. GUI 技术引入前提

正式引入 egui / eframe 前，必须在 Phase 2 Technical Design 中说明以下事项：

| # | 问题 |
|:-:|------|
| 1 | 为什么选择 egui + eframe |
| 2 | 对二进制体积的影响 |
| 3 | 对离线构建的影响 |
| 4 | crate 是否已缓存或需要 vendor |
| 5 | GUI 与现有 CLI core 如何解耦 |
| 6 | GUI 是否直接调用 Rust library API（推荐方案） |
| 7 | GUI 是否禁止调用未来阶段功能 |
| 8 | 如何测试 GUI 不破坏 Phase 1 core |

---

## 9. Next Steps

按照你的要求，下一步**不写代码**，生成两份正式设计文档：

| 步骤 | 交付物 | 说明 |
|:----:|--------|------|
| **Step 1** | `docs/phase-2/Phase_2_PRD.md` | Phase 2 产品需求文档（含 GUI 需求、页面定义、Clone placeholder 规则） |
| **Step 2** | `docs/phase-2/Phase_2_Technical_Design.md` | Phase 2 技术设计文档（含 GUI 架构、依赖引入策略、测试策略、任务拆分） |

两份文档完成后，你审阅通过，然后给出**第一个 Phase 2 coding task 的明确批准**，我再开始输出代码。

---

## 10. Phase 2 Coding Status

| 状态 | 值 |
|------|-----|
| Phase 2 规划 | ✅ v2 已确认 |
| CLI / 配置 / 历史 / 保留 / 调度 / SMB 编码 | ⏳ 等待 PRD + Technical Design 审阅后批准 |
| GUI 编码 | ✅ **已批准**（作为 Phase 2 的一部分，但需确认 egui 依赖引入策略后启动） |
| Clone 占位页面 | ✅ **允许**（仅禁用 placeholder，无真实功能） |
| **Phase 2 产品编码** | **⏳ 尚未启动 — 需先审阅 PRD + 技术设计后再批准第一个 coding task** |

---

## Revision History

| Version | Date | Reason for change |
|---------|------|-------------------|
| v1.0 | 2026-07-05 | Initial revision based on Phase 2 Planning Source Audit (Task 2.0A/B) and user convergence decisions |
| v2.0 | 2026-07-05 | User confirmed: Phase 2 includes local desktop GUI coding. Clone page allowed as disabled placeholder only. 7 confirmed items. 21-item forbidden list. |

## 11. Phase 2 GUI Coding Status — T2-08 Dashboard Done

**Updated:** 2026-07-06

| Task | Status | Files |
|------|--------|-------|
| T2-08 Dashboard | ✅ COMPLETE | `src/gui/pages/dashboard.rs` (new), `app.rs`, `theme.rs` |
| Backup page | ⏳ Not started | `src/gui/pages/backup.rs` (stub) |
| Restore page | ⏳ Not started | `src/gui/pages/restore.rs` (stub) |
| History page | ⏳ Not started | `src/gui/pages/history.rs` (stub) |
| Schedule page | ⏳ Not started | `src/gui/pages/schedule.rs` (stub) |
| Settings page | ⏳ Not started | `src/gui/pages/settings.rs` (stub) |
| Clone page | ✅ Done (Phase 5 placeholder) | `src/gui/pages/clone.rs` |

### Dashboard Design Decisions Log

See `PROJECT_ENGINEERING_MEMORY.md` §12 for full details.

---

## Revision History

| Version | Date | Reason for change |
|---------|------|-------------------|
| v1.0 | 2026-07-05 | Initial revision |
| v2.0 | 2026-07-05 | User confirmed Phase 2 GUI coding. Clone placeholder. |
| v2.1 | 2026-07-06 | Added T2-08 Dashboard completion record |

## 12. Phase 3 Reference

Phase 2 is closed. Phase 3 scope has been reset.

For Phase 3 details, see:

- `docs/phase-3/Phase_3_Plan.md` — Phase 3 scope, task chain, safety boundary
- `docs/phase-3/Phase_3_Plan.md` § Phase Boundaries — Phase 3.5/4 status
- `docs/phase-0/08_Design_Decision_Log.md` § ADR-P3-001~003 — Design decisions
- `docs/project/PROJECT_ENGINEERING_MEMORY.md` §14 — Engineering memory entry

---

## Revision History

| Version | Date | Reason for change |
|---------|------|-------------------|
| v1.0 | 2026-07-05 | Initial revision |
| v2.0 | 2026-07-05 | User confirmed Phase 2 GUI coding |
| v2.1 | 2026-07-06 | Added T2-08 Dashboard completion record |
| v2.2 | 2026-07-06 | Added Phase 3 reference |
