# Phase 2 — UI Direction Decision (HISTORICAL)

> **This document is HISTORICAL. It records the Phase 2 UI direction that was superseded by Phase 2.5.**
> **Current UI technology: Tauri 2.0 + React + TypeScript + Vite — see Phase_2_5_Tauri_Migration_Decision.md.**
**Status:** HISTORICAL — Superseded by Phase 2.5 GUI Technology Migration Decision (Phase_2_5_Tauri_Migration_Decision.md). The egui/eframe direction was replaced by Tauri 2.0 + React + TypeScript + Vite. This document is kept for traceability only.

---

## 1. Decision Record

### Decision Made By

User (explicit approval during Phase 2 planning convergence, 2026-07-05).

### Decision Summary

| # | Decision | Value |
|:-:|----------|-------|
| 1 | **Phase 2 是否进行 UI 管理界面编码？** | **是** — Phase 2 要进行 UI 管理界面编码 |
| 2 | **Phase 2 是否必须出现可运行本地管理界面？** | **是** — Phase 2 必须出现可运行的本地管理界面 |
| 3 | **UI 方式参考什么产品？** | **Acronis True Image** |
| 4 | **UI 方向是 Web GUI 还是本地桌面 GUI？** | **本地桌面 GUI**，不是 Web GUI |
| 5 | **Clone 是否必须出现在 UI 中？** | **是** — Clone 必须出现在 UI 界面中 |
| 6 | **Clone 功能是否在 Phase 2 实现？** | **否** — Clone 只允许作为 UI 页面 / 菜单 / Future placeholder 出现 |
| 7 | **Phase 2 允许实现真实磁盘克隆功能吗？** | **不允许** — 不允许访问磁盘块、分区表、PhysicalDrive、VSS、.nwb、WinPE、系统恢复、异机还原等 Phase 3/4/5/6+ 能力 |

---

## 2. UI Technology Confirmation

| 项目 | 值 |
|------|-----|
| UI 形态 | 本地桌面 GUI（Acronis True Image-like） |
| 候选技术 | egui + eframe（纯 Rust，零运行时依赖） |
| 不是 | ❌ Web GUI / 浏览器管理后台 |
| 不是 | ❌ FastAPI / Python backend |
| 不是 | ❌ Vanilla JS / Tailwind CSS / React / Vue / jQuery |
| 运行方式 | 单机本地运行，与现有 Rust CLI core 复用逻辑 |

---

## 3. GUI Page Structure

| 页面 | 描述 | Phase 2 状态 |
|------|------|:------------:|
| **Dashboard** | 最近备份状态、备份目标状态、最近任务结果、风险提示 | ✅ 实现 |
| **Backup** | 选择备份源/目标、选择/创建 backup job、执行文件级备份、显示进度和结果 | ✅ 实现 |
| **Restore** | 浏览备份点、浏览 manifest 文件和目录、选择恢复目标、执行文件级恢复、显示校验结果 | ✅ 实现 |
| **History** | 显示备份/恢复/验证历史、成功/失败状态、文件数/容量/耗时/退出码 | ✅ 实现 |
| **Schedule** | 显示已配置计划任务、创建/删除/查看计划任务（通过 Windows Task Scheduler） | ✅ 实现 |
| **Clone** | **必须出现在 UI 中。功能禁用，仅作 Phase 5 placeholder。** | ❌ Coming Soon |
| **Settings** | 配置默认 source/dest、retention 配置、job 配置、UI 选项 | ✅ 实现 |

---

## 4. Clone Page Placeholder Rules

| 规则 | 说明 |
|------|------|
| ✅ Clone 页面可以出现在导航栏和菜单中 | 作为 Future placeholder / Coming Soon |
| ✅ 页面必须显示明确的禁用提示 | **"Disk Clone is planned for Phase 5 and is not available in Phase 2."** |
| ❌ 不得实现任何真实克隆功能 | 无 clone engine、无 clone CLI 命令、无 clone Rust modules |
| ❌ 不得访问磁盘块设备、卷、分区或 PhysicalDrive | 禁止读取分区表、GPT/MBR、NTFS bitmap |
| ❌ 不得创建 clone 相关的 Rust 模块、structs、traits 或 CLI 命令 | 禁止 Phase 5 代码提前引入 |
| ❌ 不得让用户误以为 Phase 2 可以克隆磁盘 | UI 必须明确显示 "Coming Soon / Phase 5" 禁用状态 |

---

## 5. Related Documents

| Document | Location | Role |
|----------|----------|------|
| Phase 2 Revised Plan v2 | `docs/phase-2/Phase_2_Revised_Plan.md` | Confirmed scope, execution order, forbidden list |
| Phase 2 Planning Source Baseline | `docs/phase-2/Phase_2_Planning_Source_Baseline.md` | Original audit trail, contamination cleanup |
| ADL-023 (UI Principles) | `docs/phase-0/08_Design_Decision_Log.md` | Detailed UI component design reference |
| ADL-034 (GUI Framework) | `docs/phase-0/08_Design_Decision_Log.md` | Qt6 → egui framework decision |

---

## Revision History

| Version | Date | Reason for change |
|---------|------|-------------------|
| v1.0 | 2026-07-05 | User confirmed: Phase 2 includes local desktop GUI coding. UI direction: Acronis True Image-like. Clone page as disabled placeholder. |

