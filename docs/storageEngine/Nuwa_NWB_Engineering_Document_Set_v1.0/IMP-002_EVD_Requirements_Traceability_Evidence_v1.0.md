# IMP-002 EVD：需求—测试追溯表最终验证证据 v1.0

**工作包：** IMP-002 — 建立需求—测试追溯表
**阶段：** GATE-0
**证据状态：** CURRENT / CLOSED / PASS / ACCEPTANCE MET
**归档日期：** 2026-07-19
**归档角色：** `nwb_evidence_documenter`

---

## 1. 权威要求与验收边界

权威来源为 `Nuwa_NWB_Implementation_Plan_v1.0.md` 的 IMP-002：

| 维度 | 要求 |
|---|---|
| 名称 | 建立需求—测试追溯表 |
| 标准 | P0 需求 100% 映射 |
| 验收 | 检查脚本无孤儿 P0 |
| 必须测试 | CI 追溯检查 |
| 交付物 | Traceability Matrix |

本工作包建立追溯 Registry、由其确定性生成的 Matrix、独立 checker 与双平台 CI 质量门。Registry 只对需求 ID、优先级、测试登记、映射和来源范围治理具有 `TRACEABILITY AUTHORITY`；它不得改变工程文档集权威技术契约 #1–6 的产品、架构、格式、Provider、支持范围、实施或测试语义。

## 2. 版本、提交与变更范围

| 项 | 值 |
|---|---|
| 分支 | `codex/nwb-storage-engine` |
| IMP-002 实现提交 | `c61b17e4a6ade9499c36e77598c689240792109f` |
| CI 集成提交 | `8b2a68b0528d37587636774eb76bc196b94dd59a` |
| 最终被测提交 | `8b2a68b0528d37587636774eb76bc196b94dd59a` |
| 实施前基线 | `261469026cb13a591db27aa9f09e64680aebdc32` |
| Pull Request | [PR #1](https://github.com/iamwuym-oss/nvwa/pull/1) — Draft、未合并 |

实现范围包括：

- `Nuwa_NWB_Traceability_Registry_v1.0.toml`：机器可读的需求、优先级、正式测试、映射与 source-scoped 测试登记；
- `Nuwa_NWB_Requirements_Test_Traceability_Matrix_v1.0.md`：由 Registry 确定性生成的只读参考矩阵；
- `tools/traceability-checker/`：严格解析、来源校验、映射校验、P0 双角色覆盖校验和 Matrix 漂移检查；
- Cargo workspace 接入与 GitHub Actions 双平台追溯质量门。

本工作包没有修改或实现 NWB Writer、Reader、Archive、Catalog、Chunk、Crypto、Backup/Restore、Verify/Salvage、Provider 或 BMR 行为，也没有修改冻结 Registry 数值或磁盘格式。

## 3. 追溯基线结果

最终 checker 在两个 CI 平台输出：

```text
Traceability check passed: 36 requirements, 138 tests, 141 mappings (33 P0).
```

| 指标 | 结果 |
|---|---:|
| 正式需求 | 36 |
| P0 需求 | 33 |
| P1 需求 | 3 |
| 正式测试 | 138 |
| 唯一映射 | 141 |
| 已实现测试登记 | 23 |
| 计划测试登记 | 115 |
| 已映射测试 | 119 |
| Source-scoped 测试 | 19 |
| P0 正向与负向/故障/安全双角色覆盖 | 33 / 33 |
| P1 至少一项测试覆盖 | 3 / 3 |

`SOURCE_SCOPED` 表示测试已绑定权威来源与理由，但当前 36 项正式需求没有为该主题分配独立 Requirement ID，因此该测试不计入需求覆盖边。它不是未登记测试，也不得被记录为已执行。

## 4. 独立审查与验证链

| 角色 | 结论 | 说明 |
|---|---|---|
| Independent Code Reviewer | APPROVED | 三轮独立审查；最终增量通过 |
| Independent Validation | CI EVIDENCE VERIFIED / LOCAL BLOCKED_BY_ENVIRONMENT | 本地 Rust 工具链不可用；最终执行证据由双平台 GitHub CI 提供 |
| Recovery Integrity Reviewer | APPROVED | 追溯元数据和 checker 不改变恢复或持久化语义 |
| Evidence Documenter | EVIDENCE ARCHIVED | 当前 EVD、TRR、索引和 Manifest 已回填 |

本地 Cargo 验证为 `NOT_RUN / BLOCKED_BY_ENVIRONMENT`，原因是当前执行容器没有 Rust 工具链。最终验收仅引用被测提交 `8b2a68b` 的 GitHub Actions 双平台执行结果，不复制旧提交结果。

## 5. GitHub Actions 执行证据

**Workflow：** NWB Workspace CI
**Run：** [29691731514](https://github.com/iamwuym-oss/nvwa/actions/runs/29691731514)
**Run head：** `8b2a68b0528d37587636774eb76bc196b94dd59a`
**结论：** `SUCCESS`

| 质量门 | Ubuntu `88205559857` | Windows `88205559861` |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | PASS |
| `cargo run --locked -p format-registry-generator -- check` | PASS | PASS |
| `cargo run --locked -p traceability-checker -- check` | PASS | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS | PASS |
| `cargo test --workspace --locked` | PASS | PASS |
| `cargo build --workspace --locked` | PASS | PASS |
| `cargo build --workspace --release --locked` | PASS | PASS |

两个平台均使用 rustc/cargo 1.97.1，测试结果均为：

```text
188 passed, 0 failed, 0 ignored
```

## 6. 关键自动化证明

Checker 与其测试覆盖以下拒绝条件和不变量：

- 重复 Requirement、Test 或 Mapping；
- 非法 ID；
- 悬空 Requirement/Test 引用；
- P0 缺少正向用例；
- P0 缺少负向、故障或安全用例；
- Registry 中没有 P0；
- `MAPPED` 与 `SOURCE_SCOPED` 处置不一致；
- 权威来源文件、Anchor 或 Excerpt 不一致；
- 生成 Matrix 缺失或漂移；
- 代码中的正式 TST ID 漏登记；
- CLI 成功、I/O、语法和语义错误退出码。

Matrix 由 Registry 确定性生成；直接修改 Matrix 会被 checker 识别为漂移并阻断 CI。

## 7. 产物哈希

以下 SHA-256 覆盖最终被测提交中的精确文件字节：

| 文件 | SHA-256 |
|---|---|
| `Nuwa_NWB_Traceability_Registry_v1.0.toml` | `58A578D283BEFB10D988D365F9C6264E6E353F9C9125C8281D8E5506717E8C8E` |
| `Nuwa_NWB_Requirements_Test_Traceability_Matrix_v1.0.md` | `19CE717A522388193ABBB8C05DD088E0E48C9FE515B6DFA120F5F4B38199448C` |
| `tools/traceability-checker/Cargo.toml` | `81B170D7F8B4F2FA503D6966E667F1E0020554D4C5B9B54DF0B29E0D6AAB7162` |
| `tools/traceability-checker/src/lib.rs` | `1EF97E18BC9E48C505EFF933153E3B322F0F08582F2E075212B58A8EBAF0D79A` |
| `tools/traceability-checker/src/main.rs` | `21818D06302885AA026A3624E3EADB7ACEE3CD630E7534FF1C968FA3104EA795` |
| `tools/traceability-checker/tests/traceability_tests.rs` | `1F9EAFD7A8B52A566BC32336F84E49D92934302054ACB88D668937B46B993E54` |
| `.github/workflows/nwb-workspace-ci.yml` | `791EA5DD3F7917BC67B6175463AC5C5CCD7012A52DF609A3F3C30E3FFBA5C0CD` |

## 8. 恢复完整性边界

Recovery Integrity Review 结论为 `APPROVED`，原因是本工作包仅新增追溯元数据、生成参考和只读校验逻辑，没有改变持久化数据、格式语义或恢复路径。

以下项目对 IMP-002 为 `N/A / NOT_RUN`：

- 真实 Backup/Restore 往返；
- Writer/Reader 互操作；
- Archive 损坏、截断或故障注入；
- Verify/Salvage；
- 真实卷、磁盘和 BMR 恢复启动。

不得用本 EVD 声称 115 个 `PLANNED` 测试已执行，也不得声称完整 NWB 存储引擎、恢复能力、支持矩阵认证或产品发布已经通过。

## 9. 已知限制与后续状态

1. 115 个计划测试仍为 `PLANNED / NOT_RUN`；它们只证明测试设计和追溯登记存在。
2. 19 个 `SOURCE_SCOPED` 测试已绑定权威来源，但没有 Requirement 映射边；这是当前 Registry 显式治理状态，不是执行结果。
3. 当前执行容器缺少 Rust 工具链，本地 Cargo 命令未运行。
4. PR #1 仍为 Draft 且未合并。
5. GATE-0 仍为 `IN_PROGRESS`；IMP-003–005 仍为 `NOT_RUN`。
6. GATE-1–GATE-9、Format Freeze、支持认证和产品发布仍为 `NOT_RUN / NOT_APPROVED`。

## 10. 验收结论

| 验收项 | 结果 |
|---|---|
| P0 需求 100% 映射 | PASS — 33 / 33 |
| P0 正向与失败/故障/安全双角色覆盖 | PASS — 33 / 33 |
| P1 测试覆盖 | PASS — 3 / 3 |
| Checker 无孤儿 P0 | PASS |
| Matrix 与 Registry 确定性一致 | PASS |
| Windows / Ubuntu CI 追溯检查 | PASS |
| 独立代码审查 | APPROVED |
| 独立验证 | CI EVIDENCE VERIFIED；本地 NOT_RUN / BLOCKED_BY_ENVIRONMENT |
| Recovery Integrity Review | APPROVED |

**IMP-002 最终状态：`CLOSED / PASS / ACCEPTANCE MET`。**

GATE-0 继续为 `IN_PROGRESS`。IMP-003–005 仍为 `NOT_RUN`；完整产品发布为 `NOT_APPROVED`。
