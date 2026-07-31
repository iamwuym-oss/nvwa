# IMP-001 EVD：Format Registry Generator 最终验证证据 v1.2

**工作包：** IMP-001 — 实现 Format Registry 生成器
**阶段：** GATE-0
**证据状态：** CURRENT / CLOSED / PASS / ACCEPTANCE MET
**归档日期：** 2026-07-19
**归档角色：** `nwb_evidence_documenter`

---

## 1. 权威需求与验收边界

权威来源为 `Nuwa_NWB_Implementation_Plan_v1.0.md` 的 IMP-001：

| 维度 | 要求 |
|---|---|
| 名称 | 实现 Format Registry 生成器 |
| 标准 | Record、Feature、Error ID 唯一 |
| 验收 | 重复 ID 导致工程构建质量门失败 |
| 必须测试 | Registry 单元测试和快照测试 |
| 证据 | Registry 文档和生成产物哈希 |

本项目的工程构建质量门按顺序执行 Generator `check`、Rust 格式、Clippy、测试以及 Debug/Release 构建。裸 `cargo build` 不读取 Registry TOML，本证据不声称它能独立发现 TOML 语义错误；重复值和越界位会在前置 Generator `check` 中以退出码 4 阻断流水线。

## 2. 版本与提交绑定

| 项 | 值 |
|---|---|
| 分支 | `codex/nwb-storage-engine` |
| 最终被测提交 | `3bceb34b4697a7552bccf9863b821a5b9d63e4b4` |
| Generator remediation | `face1eaa3c11f01bbe639cef25550b02347420b1` |
| LF Registry 契约修复 | `d1707c3bd626f09d7a9f16ce582bcbdf797e8566` |
| Registry 契约测试与错误分类强化 | `1033f9ff728db6cc97f26afbd9f7857eba2e0d81` |
| Rustfmt 收尾 | `3bceb34b4697a7552bccf9863b821a5b9d63e4b4` |
| 历史关闭基线 | `d5509071a5300e6c13c26c527b8015ff4bc322c4`，仅代表无 Generator 的旧基线 |

旧 IMP-001 EVD v1.0/v1.1 已被本文件取代。旧文件因格式损坏及状态过期而仅保留作追溯，不得作为当前状态来源。

## 3. 完成范围

最终实现与验证覆盖：

- 四类权威 TOML Registry：RecordType、FeatureBit、HeaderEnum、ErrorId；
- TOML 到 Rust 生成器及 `check` / `generate` 命令；
- 生成文件逐字节一致性检查与 LF 契约；
- RecordType、ErrorId、FeatureBit、BackupKind、PlatformHint 的完整精确快照；
- 权威 TOML 的成员数量、名称、值、bit 与 `repr` 快照；
- 生成源文件的 `repr` 和常量类型断言；
- 重复 discriminant、重复 bit、bit=64 越界在 `check` 与 `generate` 两条路径均返回语义错误退出码 4。

本轮 `d1707c3..3bceb34` 未修改四个 Registry TOML 或四个生成 Rust 文件，未改变冻结数值、Feature 位、Display 文本或磁盘格式语义。

## 4. 独立审查链

| 角色 | 结论 | 日期 | 说明 |
|---|---|---|---|
| `nwb_format_architect` | `APPROVED_MINIMAL_REMEDIATION` | 2026-07-19 | 批准仅强化快照、`repr`/类型断言和语义错误分类；禁止改变冻结 Registry |
| `nwb_code_reviewer` | `APPROVED` | 2026-07-19 | 最终实现增量与 Rustfmt 收尾独立复审通过 |
| `nwb_validation_engineer` | `VALIDATION_PASS` | 2026-07-19 | 最终提交、双平台 CI 与原始日志独立核对 |
| `nwb_recovery_integrity_reviewer` | `RECOVERY_INTEGRITY_APPROVED` | 2026-07-19 | 无冻结格式或恢复语义变化 |
| `nwb_evidence_documenter` | `EVIDENCE_ARCHIVED` | 2026-07-19 | 当前证据与历史记录分离归档 |
| `nwb_storage_project_manager` | `CLOSED` | 2026-07-19 | Definition of Done 已满足 |

## 5. GitHub Actions 验证

**Workflow：** NWB Workspace CI
**Run：** [29684903853](https://github.com/iamwuym-oss/nvwa/actions/runs/29684903853)
**Run head：** `3bceb34b4697a7552bccf9863b821a5b9d63e4b4`
**结论：** `SUCCESS`

| 质量门 | Windows `88187397699` | Ubuntu `88187397701` |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | PASS |
| `cargo run --locked -p format-registry-generator -- check` | PASS | PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS | PASS |
| `cargo test --workspace --locked` | PASS | PASS |
| `cargo build --workspace --locked` | PASS | PASS |
| `cargo build --workspace --release --locked` | PASS | PASS |

两个平台的 Generator check 均输出 `All generated files are up to date.`。

### 工具链

| 平台 | Rust host | rustc | cargo |
|---|---|---|---|
| Windows | `x86_64-pc-windows-msvc` | 1.97.1 (`8bab26f4f`) | 1.97.1 (`c980f4866`) |
| Ubuntu | `x86_64-unknown-linux-gnu` | 1.97.1 (`8bab26f4f`) | 1.97.1 (`c980f4866`) |

### 测试计数

两平台测试分组一致：

```text
0 + 0 + 6 + 31 + 78 + 0 + 9 + 19 + 4 + 9 + 3 + 1 + 1 + 11 + 0 + 0 + 0
= 172 passed, 0 failed, 0 ignored
```

关键测试在 Windows 与 Ubuntu 原始日志中均为 `ok`：

- `test_registry_snapshot`
- `test_authoritative_toml_registry_snapshot`
- `test_generated_registry_repr_and_constant_types`
- `test_generated_files_match_toml_sources`
- `test_error_id_duplicate_discriminant`
- `test_duplicate_discriminant_values`
- `cli_exit_4_semantic_errors_match_for_check_and_generate`

当前执行容器没有 Rust 工具链，因此本地 Cargo 验证为 `NOT_RUN / BLOCKED_BY_ENVIRONMENT`。最终验收只引用上述被测提交和双平台 CI 原始日志，不复制旧基线结果。

## 6. 失败保留与修复记录

首次推送 `1033f9f` 对应 run [29684808801](https://github.com/iamwuym-oss/nvwa/actions/runs/29684808801)，Windows 与 Ubuntu 均在 `cargo fmt --check` 失败，其余步骤被跳过。失败日志已保留。提交 `3bceb34` 仅应用日志给出的 Rustfmt 布局，独立复审确认无语义变化；随后 run `29684903853` 全部通过。

## 7. Registry 产物 SHA-256

哈希定义为仓库提交树中的原始文件字节 SHA-256；不混用 Git blob SHA-1 或平台换行转换后的工作树哈希。

| 文件 | SHA-256 |
|---|---|
| `crates/nwb-format/registry/record_types.toml` | `770A464ECDB7C697F6F9D936AD0ED70ED5C237F6CCDF82E678A6C8E6FC9D5129` |
| `crates/nwb-format/registry/feature_bits.toml` | `D84AE317BA27FFACE34BA1B946627430CA3F40232D498ABF184C104034172D3D` |
| `crates/nwb-format/registry/header_enums.toml` | `21906F548F7CA8531CC53A996A93932D310A490D414548CC1F1BE4D45FB7A5B4` |
| `crates/nwb-format/registry/error_ids.toml` | `E4B48DFF48A6C98D151693030348358DB8DA721CC85DDC63746766470A627585` |
| `crates/nwb-format/src/registry/record_type.rs` | `DA5D9E02648AC95BBD9D7DD2FE4CA00226D93753E8F32743C7333A26C0BF90B` |
| `crates/nwb-format/src/registry/feature_bit.rs` | `C457E0D3E6C60A850BB94083B8591920CED7F8FEA9CBE81AE8026753E1602659` |
| `crates/nwb-format/src/registry/header_enums.rs` | `A926F41CE27BF70B3DB4D604BAF42CA57A9239059FCC9EC5CB338EA57FA33851` |
| `crates/nwb-format/src/registry/error_id.rs` | `171E141C35BF07CB900580B7661469CA0571F783BCEB26DFFC718226D1236F5D` |

## 8. 恢复完整性边界

本工作包没有修改或实现 NWB Writer、Reader、Archive 状态机、Backup/Restore、Verify/Salvage、Header/Segment/Commit codec 或 BMR。真实备份恢复往返、故障注入和 BMR 测试为 `N/A / NOT_RUN`，不得据本 EVD 宣称完整 NWB 存储引擎或恢复能力已经完成。

恢复完整性审查确认：Registry 值、位、`repr`、成员增删或重命名会被至少一层精确测试发现；Parser 修改只更正诊断类别和退出码，不改变有效输入、生成内容或格式语义。

## 9. 已知限制与非阻塞风险

| ID | 状态 | 说明 |
|---|---|---|
| RIR-001 | DEFERRED | `BackupKind Invalid=0` 由架构裁决延期，本轮不得改变冻结 Registry |
| RIR-003 | DEFERRED | `generate` 写入不是事务式原子更新；`check` 为只读，本轮未扩大风险 |
| CI-NOTE-001 | NON_BLOCKING | `actions/checkout@v4` 出现 Node 20 弃用提示；GitHub 当前强制以 Node 24 执行，不影响本次结论 |
| UI-BASELINE | OUT_OF_SCOPE | 既有 UI 构建缺陷不属于 IMP-001 |

## 10. 验收结论

| 验收项 | 结果 |
|---|---|
| Generator 与四类 Registry | PASS |
| 重复值/越界位阻断工程质量门 | PASS |
| Registry 单元、精确快照和负向测试 | PASS |
| 生成内容与 TOML 一致 | PASS |
| Windows / Ubuntu CI | PASS |
| 独立代码审查 | APPROVED |
| 独立验证 | PASS |
| 恢复完整性审查 | APPROVED |
| 证据可追溯性 | PASS |

**最终状态：`CLOSED / PASS / ACCEPTANCE MET`。**

GATE-0 仍为 `IN_PROGRESS`。IMP-002 仍为 `NOT_RUN / NOT_STARTED`；关闭 IMP-001 不构成对 IMP-002 的自动授权。
