# IMP-000 EVD（工程验证文档）构建证据 v1.0

**工作包：** IMP-000 — 建立Workspace、模块和依赖方向  
**所属阶段：** GATE-0（工程与契约基线）  
**归档日期：** 2026-07-13  
**归档人：** nwb_evidence_documenter

---

## 1. 范围与目标

### 1.1 正式需求

根据 Nuwa_NWB_Implementation_Plan_v1.0.md 第5.1节：

| 维度 | 需求 |
|---|---|
| 工作包ID | IMP-000 |
| 名称 | 建立workspace、模块和依赖方向 |
| 标准 | Core不得依赖平台Provider实现 |
| 验收 | 干净环境一条命令构建 |
| 必须测试 | Debug/Release、Windows/Linux CI |
| 文档/证据 | 构建说明、依赖图、EVD构建日志 |

### 1.2 本证据文档覆盖的内容

| 项 | 说明 |
|---|---|
| 基线提交 | ad6695b（原始证据绑定） |
| 当前分支 | codex/nwb-storage-engine |
| IMP-000实际变更 | Cargo.toml / src-tauri/Cargo.toml / Cargo.lock / crates/nwb-format/ |
| 证据边界 | 原始证据绑定 ad6695b 及当时未提交修改，不能自动视为当前提交 518f9fe 的证据 |

### 1.3 与本工作包无关的内容

AGENTS.md的变更属于独立治理任务 BASELINE-GOVERNANCE-001，不归入IMP-000。

---

## 2. IMP-000实际变更清单

### 2.1 根 Cargo.toml（IMP-000）

变更前：只有 [package] 声明
变更后：在 [package] 前添加 [workspace]，resolver = 2，members = [crates/*, .]，exclude = [src-tauri]

目的：建立Cargo Workspace，使nwb-format成为成员，保持src-tauri独立构建。

### 2.2 src-tauri/Cargo.toml（IMP-000）

变更：在文件末尾添加空 [workspace]

目的：保持Tauri Desktop GUI的独立Cargo构建能力。

### 2.3 Cargo.lock（IMP-000，自动更新）

因新增workspace成员而产生的Cargo自动依赖锁更新。
Cargo.lock SHA-256：12423e97bd8cabadbc2d4ec3bec411e9e92923f053fb1aabb4cdf2c7e552bf74

### 2.4 crates/nwb-format/ — 新工程骨架（IMP-000）

| 文件 | 大小（字节） | SHA-256 | 说明 |
|---|---|---|---|
| crates/nwb-format/Cargo.toml | 175 | 3e5e73a8fc7ae57ce6127d3bf8c67195dfb9028a4ef8d42b941bd7d977af0ab8 | 工程声明，仅依赖sha2 |
| crates/nwb-format/src/lib.rs | 565 | 8aea2052a4d830452a47f4695570d3bb8bbe5d77e3d05a213ba35da340b18eae | 空库+版本常量+单例测试 |

nwb-format crate为工程骨架，不包含NWB格式的Bootstrap Header、Record、Segment、Manifest、Commit或其他格式逻辑。

### 2.5 依赖图

根 Workspace（Cargo.toml，resolver=2）
  nuwa-backup（根package）：chrono、hex、rusqlite（optional）、serde、serde_json、sha2、thiserror、toml、uuid、（dev: tempfile）
  nwb-format（crates/nwb-format/）：sha2

Excluded：src-tauri（独立workspace）：tauri、serde、serde_json

nwb-format不依赖根package，根package不依赖nwb-format，Core无平台Provider依赖。

---

## 3. 构建证据

### 3.1 运行环境

| 字段 | 实际值 |
|---|---|
| 操作系统 | Windows |
| Rust版本 | rustc 1.96.1（31fca3adb 2026-06-26） |
| Cargo版本 | cargo 1.96.1（356927216 2026-06-26） |
| 构建目标 | x86_64-pc-windows-msvc |
| 工作目录 | C:\Users\Tony\LCB |

### 3.2 质量门执行结果

| 命令 | 结果 | 备注 |
|---|---|---|
| cargo fmt --check | PASS（exit 0） | Windows-only |
| cargo check（根Workspace） | PASS（exit 0） | Windows-only |
| cargo test --workspace | PASS（123/123，exit 0） | Windows-only |
| cargo build（根Workspace） | PASS（exit 0） | Windows-only，Debug only |
| cd src-tauri; cargo check | PASS（exit 0） | Windows-only |

> ⚠ **证据边界：** 以上结果仅限 Windows x86-64 Debug 构建。无 Release 构建证据、无 Linux 构建/测试证据、无 CI 流水线（仓库无 .github/workflows）。原始执行绑定 ad6695b + 未提交修改，不能自动视为 518f9fe 提交的证据。

测试明细：
- nuwa_backup src/lib.rs：78 passed，0 failed
- nuwa_backup src/main.rs：0 tests
- backup_service_tests：9 passed，0 failed
- config_service_tests：19 passed，0 failed
- dashboard_service_tests：4 passed，0 failed
- file_browser_service_tests：9 passed，0 failed
- restart_persistence_tests：3 passed，0 failed
- nwb_format src/lib.rs：1 passed，0 failed

总计：123 PASS，0 FAIL，0 IGNORED

---

## 4. IMP-000 与 BASELINE-GOVERNANCE-001 的区分

### 4.1 IMP-000（Workspace与工程骨架）
Cargo.toml：修改；src-tauri/Cargo.toml：修改；Cargo.lock：自动更新；crates/nwb-format/：新建

### 4.2 BASELINE-GOVERNANCE-001（独立治理任务）
AGENTS.md：修改，替换为全局工程治理规则

AGENTS.md不属于IMP-000工作包范围。

---

## 5. 不变量验证

- nwb-format不依赖nuwa-backup根package
- 根package不依赖nwb-format
- Core（nwb-format）无平台Provider依赖
- nwb-format为工程骨架，不包含格式功能实现
- src-tauri可独立构建（exit code 0）
- 旧Phase S Repository代码未被误恢复
- 无新NWB格式逻辑提前引入
- 所有测试使用临时目录，不接触用户数据

---

## 6. 已解决的限制

以下限制已通过 IMP-000-REMEDIATION-1, 2A, 2B, 2C 消除:

- Release 构建: Windows Debug + Release 均通过; CI 包含 debug + release
- Linux CI: Ubuntu CI 通过全部 5 个质量门
- GitHub Actions workflow 已建立并通过
- 证据绑定: e1f1adb14b315f803c62d19f695a2b1b2775cd7e 提交, CI run 29426443433 (https://github.com/iamwuym-oss/nvwa/actions/runs/29426443433)
- CI toolchain: rustc/cargo 1.97.0 (CI); Local Windows: rustc/cargo 1.96.1
- Windows job ID: 87389958591; Ubuntu job ID: 87389958600
- clippy -D warnings: Windows + Ubuntu 全部通过
- test --workspace: Windows 130 passed, Ubuntu 全部通过
- 跨平台: file_browser_service, path_support, schedule_service 测试均已正确隔离

### 持续有效的限制

- nwb-format 的 sha2 依赖在 IMP-001 中未使用 (LOW-DEP-001)
- Format Registry 尚未建立 Generator (属于 IMP-001)
- pnpm / UI 构建已知失败, 不属于 IMP-000 范围
- src-tauri 跨平台检查待独立工作包处理

## 7. 结论

**Status: CLOSED / PASS / ACCEPTANCE MET**

### 验收证据

| 验收维度 | 状态 | 证据 |
|---|---|---|
| Debug 构建通过 | PASS | CI Windows + Ubuntu, 本地 Windows |
| Release 构建通过 | PASS | CI Windows + Ubuntu, 本地 Windows |
| Windows CI | PASS | windows-latest: fmt / clippy / test / build / release |
| Linux CI | PASS | ubuntu-latest: fmt / clippy / test / build / release |
| CI 流水线 | ESTABLISHED | .github/workflows/nwb-workspace-ci.yml |
| 命令/退出码/日志 | RECORDED | 本节 + CI run 29426443433 (Windows job 87389958591, Ubuntu job 87389958600) |
| 独立 Code Review | APPROVED | nwb_code_reviewer |
| 独立 Validation | PASS | nwb_validation_engineer |

### 已实现

- Cargo Workspace 建立 (resolver=2, members = [crates/*, .], exclude = [src-tauri])
- crates/nwb-format 工程骨架
- .github/workflows/nwb-workspace-ci.yml (Windows + Ubuntu, 5 质量门)
- Linux 跨平台修复 (cfg gates + POSIX 路径测试)
- 测试隔离 (schedule 测试不依赖文件系统)
- 130 项 Rust 测试全部通过 (Windows)
- 10 项 CI 步骤全部通过 (Windows + Ubuntu)

### 验收人员签署

| 角色 | 结论 | 日期 |
|---|---|---|
| nwb_code_reviewer | APPROVED | 2026-07-15 |
| nwb_validation_engineer | PASS | 2026-07-15 |
| nwb_evidence_documenter | EVD 归档 | 2026-07-15 |
| nwb_storage_project_manager | CLOSED | 2026-07-16 |

### GATE-0 关联说明

IMP-000 关闭不代表 GATE-0 关闭. GATE-0 仍为 IN_PROGRESS, 其他 IMP (001-005) 未完成.

---
