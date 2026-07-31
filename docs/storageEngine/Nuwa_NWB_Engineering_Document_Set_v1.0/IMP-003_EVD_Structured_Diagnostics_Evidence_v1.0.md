# IMP-003 EVD：结构化错误与安全日志最终验证证据 v1.0

**工作包：** IMP-003 — 建立结构化错误和日志
**阶段：** GATE-0
**证据状态：** CURRENT / CLOSED / PASS / ACCEPTANCE MET
**归档日期：** 2026-07-25
**归档角色：** `nwb_evidence_documenter`

---

## 1. 权威要求与验收边界

权威来源为 `Nuwa_NWB_Implementation_Plan_v1.0.md` 的 IMP-003：

| 维度 | 要求 |
|---|---|
| 名称 | 建立结构化错误和日志 |
| 标准 | 无密钥/密码泄露 |
| 验收 | 日志脱敏审查通过 |
| 必须测试 | Secret Canary 测试 |
| 交付物 | Error Registry、日志规范 |

派生实施标准为 `Nuwa_NWB_Structured_Error_and_Logging_Specification_v1.0.md`。它服从工程文档集权威契约 #1–6，不改变 ErrorId 数值、NWB 二进制格式、恢复语义或产品支持范围。

## 2. 版本、提交与变更范围

| 项 | 值 |
|---|---|
| 分支 | `codex/nwb-storage-engine` |
| 实施前基线 | `2752102eac47f0744958637a5b631ae91c3e3686` |
| 初始实现提交 | `539b796110782629cefa6d5680da2dab2c61d7ff` |
| 独立审查修复提交 | `2e262394d2410387e5878b3d98e4cb9687654f39` |
| 文档计数修复提交 | `3aa0e13444bbaeeab2a02dbb16f3e3886b90286f` |
| Rust 1.97.1 格式修复提交 | `d08a92baf289a9ae4eb1cb6cb668fed474ccdc5c` |
| 最终 PR head | `d08a92baf289a9ae4eb1cb6cb668fed474ccdc5c` |
| CI 实际 checkout | PR 合并测试提交 `2b18c0a4588abd1760f17d8851a1a99e88b0c52f`，包含 head `d08a92b` 与 base `83905dc` |
| Pull Request | [PR #1](https://github.com/iamwuym-oss/nvwa/pull/1) — Draft、未合并 |

最终实现范围包括：

- 新增 `nwb-diagnostics` crate；
- 使用已生成的 `ErrorId`，不建立第二套 ID 或名称表；
- `Diagnostic` 的严重度、阶段、可重试性和恢复影响结构化语义；
- 绑定诊断的 `LogEvent` 从 `Diagnostic` 单一来源派生严重度和阶段；
- 固定枚举、数值和不透明 ID 构成的 JSON Lines Schema；
- `Secret` 的固定脱敏格式与析构前缓冲区覆盖；
- 丢弃底层 I/O 错误文本、返回固定结构化 `IoError` 上下文；
- 双平台专用 Secret Canary CI 门；
- `REQ-019` 与 `TST-ERR-001`～`TST-ERR-007` 的追溯登记。

本工作包没有实现 Writer、Reader、Archive、Catalog、Chunk、Crypto、Backup/Restore、Verify/Salvage、Provider 或 BMR，也没有修改冻结的 ErrorId 数值或任何磁盘格式。

## 3. 独立审查与修复链

独立代码审查首先给出 `CHANGES_REQUIRED`，发现三项阻塞问题：

| 发现 | 风险 | 处理 |
|---|---|---|
| 顶层 `level/stage` 与嵌套诊断可以分别设置 | 同一机器日志可能产生冲突语义 | 增加 `LogEvent::from_diagnostic`，从 `Diagnostic` 单一来源派生 |
| 日志 I/O 错误缺少结构化上下文 | 脱敏后上层无法判断错误类别和恢复影响 | 返回固定 `IoError`、Writer 阶段、严重度、可重试性与恢复影响 |
| ErrorId 名称存在手工第二映射 | Registry 与日志身份可能漂移 | 删除手工映射，直接使用生成的 `ErrorId` |

修复提交 `2e26239` 增加 `TST-ERR-007` 并关闭三项问题。随后修正 Document Index 的受治理文件计数。独立复审结论为 `APPROVED`。

最终 `d08a92b` 只包含 Rust 1.97.1 `rustfmt` 对两个已批准文件的三处布局调整。独立最终增量复核确认没有改变结构化错误、Secret 脱敏、日志 Schema、测试断言或追溯语义，因此此前语义审查结论可延伸到最终被测 head。

## 4. GitHub Actions 最终执行证据

**Workflow：** NWB Workspace CI
**Run：** [30184529945](https://github.com/iamwuym-oss/nvwa/actions/runs/30184529945)
**PR head：** `d08a92baf289a9ae4eb1cb6cb668fed474ccdc5c`
**结论：** `SUCCESS`

| 质量门 | Windows `89746726158` | Ubuntu `89746726215` |
|---|---|---|
| Rust toolchain | 1.97.1 | 1.97.1 |
| `cargo fmt --all -- --check` | PASS | PASS |
| `cargo run --locked -p format-registry-generator -- check` | PASS | PASS |
| `cargo run --locked -p traceability-checker -- check` | PASS | PASS |
| `cargo test --locked -p nwb-diagnostics secret_canary` | 2 PASS | 2 PASS |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS | PASS |
| `cargo test --workspace --locked` | 195 PASS | 195 PASS |
| `cargo build --workspace --locked` | PASS | PASS |
| `cargo build --workspace --release --locked` | PASS | PASS |

两个平台的完整测试结果均为：

```text
195 passed, 0 failed, 0 ignored
```

专用 Canary 步骤在完整测试之外再次执行两个 Secret 测试，均为：

```text
2 passed, 0 failed
```

追溯检查在两个平台均输出：

```text
Traceability check passed: 37 requirements, 145 tests, 148 mappings (34 P0).
```

## 5. IMP-003 契约测试

| Test ID | 证明目标 | Windows | Ubuntu |
|---|---|---|---|
| `TST-ERR-001` | 使用生成的 Error Registry 身份 | PASS | PASS |
| `TST-ERR-002` | 拒绝保留的 `ErrorId::Invalid` | PASS | PASS |
| `TST-ERR-003` | JSON Line 确定、合法且只有一个 LF | PASS | PASS |
| `TST-ERR-004` | Secret Canary 在 `Debug` / `Display` 中脱敏 | PASS | PASS |
| `TST-ERR-005` | 含 Canary 的底层 I/O 错误文本不泄漏 | PASS | PASS |
| `TST-ERR-006` | Wire Schema 无自由文本、路径或载荷字段 | PASS | PASS |
| `TST-ERR-007` | 诊断事件严重度和阶段只有一个来源 | PASS | PASS |

`REQ-019` 为 P0，已由 2 项正向、2 项负向和 3 项安全测试覆盖，满足正向与负向/安全双角色要求。

## 6. 失败证据与最终绑定

历史运行 [29710969096](https://github.com/iamwuym-oss/nvwa/actions/runs/29710969096) 在 Windows 与 Ubuntu 的第一道 Format check 失败。根因是 stable 工具链从 Rust 1.97.0 更新到 1.97.1 后产生三处纯格式差异；该 run 的后续测试没有执行，不能作为通过证据。

提交 `d08a92b` 由 Rust 1.97.1 `rustfmt` 生成精确修复。最终验收只绑定新 run `30184529945`，不复用失败 run 或更早候选提交的测试结论。

## 7. 最终被测产物哈希

以下 SHA-256 对应最终 PR head `d08a92b` 的精确文件字节：

| 文件 | SHA-256 |
|---|---|
| `.github/workflows/nwb-workspace-ci.yml` | `8C0F211620B26FE3F9A1CB5ECD2C91DF6BD74289F7FB9ACC55400CEFC3B4FD92` |
| `Cargo.lock` | `8285FDC6210336655A926C8D6979CCABDEB3C097BE75A935D3C55BDE0C1FCE08` |
| `crates/nwb-diagnostics/Cargo.toml` | `1A33958DA34B4C0880768418CFFAE8EDB1D0C642767F589FD1BA4201DFAB63E0` |
| `crates/nwb-diagnostics/src/lib.rs` | `5BC00B0F4ADBC1B4B34DD77B4BFBBAC0882B78878A689803FF964EFF4D8AF722` |
| `crates/nwb-diagnostics/tests/diagnostics_contracts.rs` | `754104E9353D3DB9F3F945FA4F3215FD694160DC5413C9F1311485A9E9239FB8` |
| `Nuwa_NWB_Structured_Error_and_Logging_Specification_v1.0.md` | `985BF47028EFB7F18987F1A974077A5044B11A59AE98AA38D5CA25C301F99C10` |
| `Nuwa_NWB_Traceability_Registry_v1.0.toml` | `AC5A53F987762AF2122990BF1CDC37D57653BEDF130A0EE790D302EE77DEADD2` |
| `Nuwa_NWB_Requirements_Test_Traceability_Matrix_v1.0.md` | `8B3F2C8FC31176FFD6BBDCD6EC83F601DAD763A0D77FD888157A19368E0FE543` |
| `tools/traceability-checker/src/lib.rs` | `E54D17B836E54A9457F6BF6225BC8146985A962684E11A9B9F3E5B30820F6A84` |
| `tools/traceability-checker/tests/traceability_tests.rs` | `BB1E840C2DB8580DF4975420626DD83F01923FB7031CB2DD013226888918A342` |

## 8. 恢复完整性适用性

IMP-003 不读写 NWB 归档，不执行备份或恢复，也不改变任何持久化与恢复路径，因此真实 Backup/Restore、Writer/Reader 互操作、故障注入、Verify/Salvage、卷/磁盘恢复和 BMR 启动均为 `N/A / NOT_RUN`。

恢复完整性角色对本工作包标记为 `N/A`，理由是没有恢复执行面；其安全相关风险由独立代码审查、固定 Schema、Secret Canary、I/O 错误脱敏和双平台 CI 覆盖。该 `N/A` 不表示完整恢复能力已获批准。

## 9. 已知边界与剩余风险

1. `Secret` 的本工作包证明范围仅为日志 API、普通格式化和所有权缓冲区析构前覆盖；不证明崩溃转储、交换文件、进程内存或真实密钥层级安全。
2. `TST-CRY-007` 仍为 `PLANNED / NOT_RUN`，由未来密码学工作包验收。
3. 日志轮转、保留、权限、导出和遥测上传不在 IMP-003；未获批准前不得上传日志。
4. 当前 145 项正式测试中仍有 115 项为 `PLANNED / NOT_RUN`；不得因追溯检查通过而宣称它们已执行。
5. PR #1 仍为 Draft 且未合并。
6. GATE-0 仍为 `IN_PROGRESS`；IMP-004–005 仍为 `NOT_RUN`。
7. GATE-1–GATE-9、Format Freeze、支持认证和产品发布仍为 `NOT_RUN / NOT_APPROVED`。

## 10. 验收结论

| 验收项 | 结果 |
|---|---|
| 无密钥/密码通过结构化日志 API 泄漏 | PASS |
| 日志脱敏独立审查 | APPROVED |
| Secret Canary 专用双平台测试 | PASS |
| 7 项 `TST-ERR` 契约测试 | PASS |
| 结构化 I/O 错误上下文 | PASS |
| 诊断严重度与阶段单一来源 | PASS |
| Error Registry 单一来源 | PASS |
| Windows / Ubuntu 全质量门 | PASS |
| 最终格式增量独立复核 | APPROVED |

**IMP-003 最终状态：`CLOSED / PASS / ACCEPTANCE MET`。**

GATE-0 继续为 `IN_PROGRESS`。IMP-004–005 仍为 `NOT_RUN`；完整产品发布为 `NOT_APPROVED`。
