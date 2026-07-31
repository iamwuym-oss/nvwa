# ADR-IMP-005 — NWB Format 0.x Versioning and Compatibility Policy v1.0

**状态：** APPROVED FOR IMP-005 IMPLEMENTATION
**日期：** 2026-07-29
**范围：** GATE-0 / IMP-005

## 1. 决策

IMP-005 在 nwb-format crate 中建立统一的版本策略模块 `version.rs`，定义当前 Format 版本 (0.1)、生命周期（DRAFT）和兼容性判断逻辑。不改变 Header 布局（offset 8/10, u16 Little Endian），不实现真实的 Writer/Reader/Feature 协商。

## 2. 背景与关键区分

- **格式规范封面上的 "DRAFT 0.9"** 是文档修订成熟度标记，不是 Header 中存储的版本值。
- **仓库原有 `NWB_FORMAT_VERSION = "0.1.0-dev"`** 是当前实现的身份锚点，本 IMP 将其结构化。
- **Header offset 8/10、u16 Little Endian** 布局不变。
- **当前精确匹配矩阵**：只接受 0.1，拒绝 0.0、0.2+、1.0+。
- **Writer/Reader/CLI/诊断契约**：版本检查作为独立模块，不嵌入 IO 逻辑。
- **未知 Required Feature 仍拒绝**（已有 TST-FMT-003 规划）。
- **未知 Optional Feature 仍跳过并报告**（已有 TST-FMT-004 规划）。
- **IMP-005 不实现真实 Header、Writer、Reader 或 Feature 协商**。
- **Format 1.0 只能在 GATE-8 冻结条件全部满足后另行批准**。

## 3. 版本兼容矩阵

| 输入版本 | 接受？ | 说明 |
|---|---|---|
| 0.1 | ✓ | 当前版本，完全兼容 |
| 0.0 | ✗ | 无效预发布版本 |
| 0.2+ | ✗ | Draft 期间只支持精确 0.1 |
| 1.0+ | ✗ | 需 GATE-8 条件满足后另行批准 |

## 4. 生命周期文本

`DRAFT — internal testing only; not for release`

## 5. 诊断要求

拒绝时必须同时包含：
- 发现版本（found version）
- 支持版本（supported version）
- 不得静默降级或接受不兼容输入

## 6. 验收

以 TST-VSN-001～005 证明兼容矩阵、诊断和 Draft 身份显示。
