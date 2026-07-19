# Nüwa NWB Test Result and Acceptance Record v1.1

**记录状态：** CURRENT
**更新日期：** 2026-07-19
**项目状态：** GATE-0 IN_PROGRESS；IMP-000、IMP-001 CLOSED / PASS / ACCEPTANCE MET

---

## 1. 被测版本

| 字段 | 实际值 |
|---|---|
| 产品版本 | `0.1.0-dev` |
| NWB Format | `0.x DRAFT` |
| 分支 | `codex/nwb-storage-engine` |
| IMP-000 提交 | `e1f1adb14b315f803c62d19f695a2b1b2775cd7e` |
| IMP-001 最终被测提交 | `3bceb34b4697a7552bccf9863b821a5b9d63e4b4` |
| Cargo.lock SHA-256 | `05999C3759DCFE29E11B0AE1F17E286CA0951C9592C22B61542601FA149C9E28` |
| 测试日期 | 2026-07-19 |
| 验证角色 | `nwb_validation_engineer` |
| 复核角色 | `nwb_code_reviewer`、`nwb_recovery_integrity_reviewer` |

历史 v1.0 记录含控制字符并绑定旧基线，现分类为 `HISTORICAL / SUPERSEDED / MALFORMED_SOURCE_RETAINED_FOR_TRACEABILITY`。本文件是当前唯一 Test Result Record。

## 2. 测试环境

| Env ID | 环境 | 架构 | 工具链 | 证据 |
|---|---|---|---|---|
| ENV-CI-WIN-IMP001 | GitHub Actions `windows-latest` | x86_64 MSVC | rustc/cargo 1.97.1 | run `29684903853`, job `88187397699` |
| ENV-CI-UBU-IMP001 | GitHub Actions `ubuntu-latest` | x86_64 GNU/Linux | rustc/cargo 1.97.1 | run `29684903853`, job `88187397701` |
| ENV-CODEX-LOCAL | 当前执行容器 | x86_64 Linux | Rust toolchain unavailable | Cargo tests `NOT_RUN / BLOCKED_BY_ENVIRONMENT` |

## 3. Gate 状态

| Gate / IMP | 状态 | 证据 |
|---|---|---|
| GATE-0 | IN_PROGRESS | IMP-002 及后续 GATE-0 工作仍未完成 |
| IMP-000 | CLOSED / PASS / ACCEPTANCE MET | IMP-000 EVD；run `29426443433` |
| IMP-001 | CLOSED / PASS / ACCEPTANCE MET | IMP-001 EVD v1.2；run `29684903853` |
| IMP-002 | NOT_RUN / NOT_STARTED | 未自动授权 |
| IMP-003–005 | NOT_RUN | 按依赖另行授权 |
| GATE-1–GATE-9 | NOT_RUN | 不属于本记录的已验收范围 |

## 4. IMP-001 质量门结果

| 命令 | Windows | Ubuntu | 实际结果 |
|---|---|---|---|
| `cargo fmt --all -- --check` | PASS | PASS | 退出码 0 |
| `cargo run --locked -p format-registry-generator -- check` | PASS | PASS | 生成文件最新，退出码 0 |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS | PASS | 退出码 0 |
| `cargo test --workspace --locked` | PASS | PASS | 172 passed / 0 failed / 0 ignored |
| `cargo build --workspace --locked` | PASS | PASS | Debug，退出码 0 |
| `cargo build --workspace --release --locked` | PASS | PASS | Release，退出码 0 |

**CI Run：** [29684903853](https://github.com/iamwuym-oss/nvwa/actions/runs/29684903853)
**Run head：** `3bceb34b4697a7552bccf9863b821a5b9d63e4b4`
**Conclusion：** SUCCESS

## 5. IMP-001 验收映射

| ID | 验收目标 | 测试/证据 | 结果 |
|---|---|---|---|
| AC-001 | RecordType 名称、值、Display 与数量锁定 | `test_registry_snapshot`、TOML 快照 | PASS |
| AC-002 | ErrorId 名称、值、Display 与数量锁定 | `test_registry_snapshot`、TOML 快照、trybuild | PASS |
| AC-003 | Feature Bit 名称、bit 与类型锁定 | Rust/TOML 快照、常量类型断言 | PASS |
| AC-004 | BackupKind/PlatformHint 名称、值、Display、repr 锁定 | Rust/TOML 快照、生成源断言 | PASS |
| AC-005 | TOML 与生成 Rust 一致 | `test_generated_files_match_toml_sources`、Generator check | PASS |
| AC-006 | 重复 discriminant/bit 与 bit=64 被拒绝 | CLI semantic error 测试，`check`/`generate` 退出码 4 | PASS |
| AC-007 | 跨平台格式、Clippy、测试、双模式构建 | Windows/Ubuntu CI | PASS |
| AC-008 | 格式/恢复完整性不回退 | 独立 Recovery Integrity Review | APPROVED |

## 6. 关键测试结果

以下测试在两个 CI job 的原始日志中均为 `ok`：

- `test_registry_snapshot`
- `test_authoritative_toml_registry_snapshot`
- `test_generated_registry_repr_and_constant_types`
- `test_generated_files_match_toml_sources`
- `test_error_id_duplicate_discriminant`
- `test_duplicate_discriminant_values`
- `cli_exit_4_semantic_errors_match_for_check_and_generate`

## 7. 失败与纠正记录

| Run | 提交 | 结果 | 处置 |
|---|---|---|---|
| `29684808801` | `1033f9f` | FAIL：两平台 `cargo fmt --check` | 保留失败日志；仅应用 Rustfmt 布局并独立复审 |
| `29684903853` | `3bceb34` | SUCCESS | 最终验收绑定此 run |

## 8. 审查和签署

| 角色 | 结论 | 日期 |
|---|---|---|
| Format Architect | APPROVED_MINIMAL_REMEDIATION | 2026-07-19 |
| Independent Code Reviewer | APPROVED | 2026-07-19 |
| Independent Validation | VALIDATION_PASS | 2026-07-19 |
| Recovery Integrity Reviewer | RECOVERY_INTEGRITY_APPROVED | 2026-07-19 |
| Evidence Documenter | EVIDENCE_ARCHIVED | 2026-07-19 |
| Project Manager | CLOSED | 2026-07-19 |

## 9. 限制与未运行项目

- 本记录只关闭 IMP-001，不表示完整 NWB Storage Engine 已实现；
- Writer、Reader、真实 Backup/Restore、Verify/Salvage、故障注入和 BMR 仍为 `NOT_RUN`；
- 支持矩阵认证仍为 `NOT_TESTED`；
- Format Freeze 与 Release 签署仍为 `NOT_APPROVED / NOT_RUN`；
- RIR-001（BackupKind Invalid=0）和 RIR-003（Generator 非原子 generate）继续延期；
- 裸 `cargo build` 不检查 TOML；工程质量门必须先执行 Generator `check`。

## 10. 最终结论

IMP-001 的适用需求、质量门、代码审查、独立验证、恢复完整性审查与证据归档均已完成。

**IMP-001：CLOSED / PASS / ACCEPTANCE MET。**

**整体产品发布：NOT_APPROVED / NOT_RUN。**
