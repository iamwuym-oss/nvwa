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
| 基线提交 | ad6695b |
| 当前分支 | codex/nwb-storage-engine |
| IMP-000实际变更 | Cargo.toml / src-tauri/Cargo.toml / Cargo.lock / crates/nwb-format/ |

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

| 命令 | 结果 |
|---|---|
| cargo fmt --check | PASS（exit 0） |
| cargo check（根Workspace） | PASS（exit 0） |
| cargo test --workspace | PASS（123/123，exit 0） |
| cargo build（根Workspace） | PASS（exit 0） |
| cd src-tauri; cargo check | PASS（exit 0） |

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

## 6. 已知限制

- 当前仅验证Windows x64构建；Linux CI尚未在本次范围内验证
- nwb-format的sha2依赖未在IMP-000工作单中明确授权，但已在独立验证环节确认属于允许的最低依赖
- Format Registry尚未建立（属于IMP-001）

---

## 7. 结论

| 验收项 | 状态 |
|---|---|
| 干净环境一条命令构建 | PASS |
| Core不依赖平台Provider | PASS |
| Debug构建 | PASS |
| src-tauri保持独立 | PASS |
| GATE-0 IMP-000完成 | PASS |

---
