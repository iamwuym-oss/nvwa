# Nüwa Backup 文档整改报告

**报告版本：** v1.0
**报告日期：** 2026-07-05
**审查依据：** 灾备产品专家审查意见
**执行状态：** ✅ 全部完成 — 所有 17 项验收标准均已满足

---

## 1. 任务目标

本次文档整改的核心目标：

1. **收敛范围** — 将 MVP（Phase 1）明确冻结为文件级备份/恢复最小闭环
2. **降级高级功能** — 差异备份、XOR Parity、加密、异机还原、磁盘克隆等全部后置
3. **恢复验证优先** — 所有完成标准从"备份成功"改为"恢复验证成功"
4. **格式简化** — .nwb 从 v1.0 降级为 v0.1 实验阶段，剥离高级特性
5. **阶段边界一致** — 所有文档中的阶段划分和范围描述互相一致
6. **增加工作规范** — 新增 Codex 工作边界文档和 MVP 边界文档

---

## 2. 修改文件清单

| 文件 | 操作 | 版本变化 |
|------|------|---------|
| 00_Codex_Working_Guardrails.md | **新增** | v1.0 |
| 01_Product_Requirements_Document.md | **修改** | v1.1 → v2.1 |
| 02_Development_Plan.md | **重写** | v1.0 → v2.1 |
| 03_Nuwa_Architecture_Design.md | **修改** | v1.0 → v2.1 |
| 04_Functional_Specification.md | **修改** | v1.0 → v2.1 |
| 05_Pipeline_Optimization_Design.md | **降级为 Future 附录** | v1.0 → v2.1 |
| 06_Nuwa_Image_Format_Spec.md | **完全重写** | v1.0 → v0.1 |
| 07_Checklist.md | **完全重写（按阶段排列）** | v1.0 → v2.1 |
| 08_Design_Decision_Log.md | **修改（新增 10 ADL + SUPERSEDED）** | v1.0 → v2.1 |
| 09_MVP_Boundary_and_Risk_Correction.md | **新增** | v1.0 |
| 10_Document_Correction_Report.md | **新增（本文档）** | v1.0 |

---

## 3. 每个文件的主要修改内容

### 00_Codex_Working_Guardrails.md（新增）
- 工作边界总则（阶段确认前置、阶段锁定、范围蔓延禁止）
- MVP 阶段禁止实现清单（21 项）
- 恢复验证优先原则（3 级验证层级）
- 文档变更规则（版本号、Revision History、一致性检查）
- Codex 汇报规则

### 01_Product_Requirements_Document.md（修改）
- 版本号更新为 v2.1
- 新增范围声明和 Phase 1/1A/1B 划分
- 新增工程护栏章节（12 条护栏）
- 新增 Phase 退出标准（5 个阶段共 33 条标准）
- 新增修订历史：v2.1 灾备专家审查修订说明

### 02_Development_Plan.md（重写）
- 从 4 阶段扩展为 6 阶段（Phase 6+ 为 Future）
- Phase 1 范围收敛：纯文件级备份/恢复 CLI，移除 .nwb/VSS/daemon/GUI
- 给出 14 项具体任务和每个任务的说明
- 给出明确的验收命令行用例

### 03_Nuwa_Architecture_Design.md（修改）
- 新增附录 A：Phase 1 MVP 架构（极简平文件存储）
- 全文添加范围声明："正文涉及的复杂架构内容仅为终局设计"
- 排除项清单：.nwb/VSS/daemon/GUI/差异/XOR/加密
- 阶段绑定所有架构组件到具体 Phase

### 04_Functional_Specification.md（修改）
- 新增范围声明头
- 每项功能标注实现阶段（Phase 1 / Phase 2 / Phase 3 / Phase 4 / Future）
- 差异备份从"支持"改为"Future (Phase 5+)"
- NFS 从"支持"改为"Future (Phase 5+)"
- 所有高级功能标记对应的 Future 阶段

### 05_Pipeline_Optimization_Design.md（降级）
- 全文移入 Future 附录
- 新增重要声明："本文档所有设计不属于 Phase 1 或 Phase 2"
- 保留原始内容仅供 Future 参考

### 06_Nuwa_Image_Format_Spec.md（完全重写）
- 从 v1.0 降级为 v0.1（实验阶段）
- 移除 Differential、XOR Parity、AES 加密、Block Group 复杂索引
- 简化 Block Index Entry 为平坦结构
- 新增附录 A：Phase 1 平文件存储格式参考
- 新增附录 B：Future Format Expansion（列出被移除的特性）

### 07_Checklist.md（完全重写）
- 按 Phase 0/1/2/3/4/5/Future 排列
- 修复编码问题（UTF-8 BOM）
- 每项包含：Phase、Allowed Now、Acceptance Criteria、Restore Validation、Risk Level
- 新增 Phase 1 禁止实现清单（21 项）

### 08_Design_Decision_Log.md（修改）
- 新增 10 条决策（ADL-New-001 ~ ADL-New-010）
- 标记 9 项冲突旧 ADL 为 SUPERSEDED
- 修正 Phase 划分附录
- 性能基准测试标准标记为暂定/Future

### 09_MVP_Boundary_and_Risk_Correction.md（新增）
- MVP 核心定义和验收标准
- 12 项风险清单（5 高 + 4 中 + 3 低）
- 21 项明确禁止实现清单
- 10 项降级为 Future 的功能清单
- MVP 合规要求

---

## 4. 已落实的专家建议清单

| # | 专家建议 | 落实方式 | 涉及文档 |
|---|---------|---------|---------|
| 1 | 产品终局能力与 MVP 混在一起 | 全文添加阶段标注、范围声明 | 01/02/03/04 |
| 2 | PRD 相对克制但其他文档过早复杂化 | 全面阶段化 | 04/05/06/07/08 |
| 3 | .nwb 格式过早复杂 | 降级为 v0.1 实验阶段，剥离高级特性 | 06 |
| 4 | 差异备份过于乐观 | 降级为 Future (Phase 5+) | 02/04/06/07/08 |
| 5 | 非 Windows 分区智能识别的承诺过大 | 标记为 Future (Phase 5+) | 04/07/08 |
| 6 | 异机还原/克隆/WinPE 不应进入 MVP | 全部后置 | 02/04/07/09 |
| 7 | 缺少 Codex 工作边界 | 新增 00_Codex_Working_Guardrails.md | 00 |
| 8 | 验收标准应为恢复验证 | 全文贯彻"恢复验证优先" | 00/01/02/07/09 |
| 9 | 数据完整性高于速度 | 写入设计原则 | 00/09 |
| 10 | 先闭环再扩展 | 阶段化设计 | 02/07/09 |

---

## 5. 被降级为 Future 的功能清单

| 功能 | 原设定阶段 | 现设定阶段 | 预计引入版本 |
|------|-----------|-----------|-------------|
| 差异备份 (Differential) | Phase 1 | Phase 5+ | v0.3+ |
| .nwb v1.0 正式格式 | Phase 1 | Phase 3+ | v0.2+ |
| XOR Parity / Erasure Coding | Phase 1 | Phase 5+ | v1.0+ |
| AES-256-GCM 加密 | Phase 1 | Phase 5+ | v1.0+ |
| VSS 快照 | Phase 1 | Phase 3 | — |
| NTFS $Bitmap 智能块识别 | Phase 1 | Phase 3 | — |
| GPT/MBR 分区表解析 | Phase 1 | Phase 3 | — |
| 启动分区识别 | Phase 1 | Phase 3 | — |
| WinPE / 启动介质 | Phase 1 | Phase 4 | — |
| 裸机恢复 | Phase 1 | Phase 4 | — |
| 异机还原 (Universal Restore) | Phase 1 | Phase 5+ | — |
| 驱动注入 | Phase 1 | Phase 5+ | — |
| 磁盘克隆 | Phase 2 | Phase 5 | — |
| lcb-daemon 系统服务 | Phase 1 | Phase 3 | — |
| 桌面 GUI（egui） | Phase 1 | Phase 2 后期 | — |
| NFS 网络目标 | Phase 1 | Phase 5+ | — |
| SMTP 邮件通知 | Phase 1 | Phase 2 | — |
| Linux / 国产 OS / LoongArch | Phase 3 | Phase 5+ | — |
| BitLocker / ReFS 支持 | Phase 1 | Phase 5+ | — |
| ext4 / XFS / btrfs 智能识别 | Phase 1 | Phase 5+ | — |
| 性能管线优化（500MB/s 目标）| Phase 1 | Phase 4+ | — |

---

## 6. 当前阶段（Phase 1）禁止实现清单

参见 `09_MVP_Boundary_and_Risk_Correction.md` §3 和 `07_Checklist.md` 的"当前 Phase 1 禁止实现清单"。

**摘要（共 21 项）：**
差异备份、.nwb 镜像格式、XOR Parity、AES 加密、VSS 快照、卷级备份、GPT/MBR 分区、启动分区识别、WinPE、裸机恢复、异机还原、驱动注入、磁盘克隆、lcb-daemon、GUI、NFS、SMTP、Linux/国产 OS、BitLocker/ReFS、ext4/XFS/btrfs 智能识别、性能优化目标。

---

## 7. 文档一致性验证

所有 11 份文档已通过以下一致性检查：

| 检查项 | 结果 | 说明 |
|--------|:----:|------|
| Phase 1 范围定义一致 | ✅ 通过 | 所有文档中 Phase 1 = 文件级备份/恢复 CLI |
| MVP 禁止项一致 | ✅ 通过 | 所有文档的禁止清单一致 |
| 恢复验证优先原则一致 | ✅ 通过 | 所有文档强调恢复验证 |
| .nwb 阶段定位一致 | ✅ 通过 | v0.1 实验阶段，Phase 2 引入 |
| 差异备份阶段一致 | ✅ 通过 | 全部标记为 Phase 5+/Future |
| XOR Parity 阶段一致 | ✅ 通过 | 全部标记为 Phase 5+/Future |
| 加密阶段一致 | ✅ 通过 | 全部标记为 Phase 5+/Future |
| 磁盘克隆阶段一致 | ✅ 通过 | Phase 5（非 Phase 3）|
| GUI 阶段一致 | ✅ 通过 | Phase 2 后期（egui）|
| Win7 支持策略一致 | ✅ 通过 | Phase 2+ |
| 性能目标策略一致 | ✅ 通过 | 非 Phase 1 验收标准 |

**最终结论：所有文档之间无冲突。**

---

## 8. 后续 Codex 开发的唯一合法起点

所有 Phase 1 开发必须以以下文档为准，**严禁参考正文中涉及 Future 功能的内容**：

| 优先级 | 文档 | 用途 |
|--------|------|------|
| 🥇 | `00_Codex_Working_Guardrails.md` | 最高约束，Codex 行为规则 |
| 🥇 | `09_MVP_Boundary_and_Risk_Correction.md` | MVP 边界和风险依据 |
| 🥇 | `02_Development_Plan.md` §二 | Phase 1 具体任务清单 |
| 🥈 | `03_Nuwa_Architecture_Design.md` 附录 A | Phase 1 MVP 架构参考 |
| 🥈 | `01_Product_Requirements_Document.md` | 仅参考 Phase 1 相关章节 |
| 🥉 | `04_Functional_Specification.md` | 仅参考标记为 Phase 1 的条目 |
| 🥉 | `07_Checklist.md` | 仅参考 Phase 1 检查项 |

---

## 9. Phase 1 开发前必须满足的条件

| # | 条件 | 状态 |
|---|------|:----:|
| 1 | 所有文档阶段边界一致 | ✅ 满足 |
| 2 | MVP 范围明确冻结 | ✅ 满足 |
| 3 | 高级功能全部后置 | ✅ 满足 |
| 4 | .nwb v1.0 降级为 v0.1 实验 | ✅ 满足 |
| 5 | 差异备份后置 | ✅ 满足 |
| 6 | XOR parity 后置 | ✅ 满足 |
| 7 | 异机还原后置 | ✅ 满足 |
| 8 | 磁盘克隆后置 | ✅ 满足 |
| 9 | Linux/国产 OS 后置 | ✅ 满足 |
| 10 | NFS 后置 | ✅ 满足 |
| 11 | 非 Windows 智能块识别后置 | ✅ 满足 |
| 12 | 性能优化目标后置 | ✅ 满足 |
| 13 | 完成标准以恢复验证为核心 | ✅ 满足 |
| 14 | Codex 工作规范文档已创建 | ✅ 满足 |
| 15 | MVP 边界与风险修正文档已创建 | ✅ 满足 |
| 16 | 文档整改报告已创建 | ✅ 满足 |
| 17 | 不产生任何生产代码变更 | ✅ 满足 |

---

## 10. 最终结论

| 项目 | 判定 |
|------|:----:|
| 是否所有 17 项验收标准均已满足？ | ✅ **全部满足** |
| 文档之间是否存在冲突？ | ❌ **无冲突** |
| 是否可以进入 Phase 1 开发？ | ✅ **可以进入 Phase 1** |
| Phase 1 的开发参考起点？ | 参见第 8 节"唯一合法起点" |

---

## Final Status

```
Final Status:  ✅ PASS
Modified Files:  01_PRDocument.md, 02_Development_Plan.md, 03_Nuwa_Architecture_Design.md,
                 04_Functional_Specification.md, 05_Pipeline_Optimization_Design.md,
                 06_Nuwa_Image_Format_Spec.md, 07_Checklist.md, 08_Design_Decision_Log.md
Created Files:   00_Codex_Working_Guardrails.md, 09_MVP_Boundary_and_Risk_Correction.md,
                 10_Document_Correction_Report.md

Key Corrections:
  - Phase 1 从"核心引擎+CLI"收敛为"文件级备份/恢复 CLI"
  - .nwb 从 v1.0 降级为 v0.1 实验
  - 差异备份从 Phase 1 降级为 Phase 5+ (Future)
  - 所有高级功能标注阶段并后置

Deferred Features: 21 项
Current MVP Scope: 文件级完整备份 + 文件级恢复 + CLI
Current Forbidden Scope: 21 项（详见 §6）

Remaining Risks:
  无文档层面的剩余风险。Phase 1 开发过程中需注意：
  - Win7 工控机场景后置，但 UI 框架已选 egui（支持 Win7）
  - 性能优化后置，验收时不对速度做要求
  - daemon 后置，CLI 直接运行

Recommendation: ✅ 可以进入 Phase 1
```

---

## Revision History

| 版本 | 日期 | 变更原因 |
|------|------|---------|
| v1.0 | 2026-07-05 | 初始版本，基于灾备产品专家审查整改完成 |
