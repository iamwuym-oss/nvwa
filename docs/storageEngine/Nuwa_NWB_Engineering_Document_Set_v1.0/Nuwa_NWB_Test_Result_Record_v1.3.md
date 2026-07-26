# Nüwa NWB Test Result and Acceptance Record v1.3

**记录状态：** CURRENT
**更新日期：** 2026-07-25
**项目状态：** GATE-0 IN_PROGRESS；IMP-000～IMP-003 CLOSED / PASS / ACCEPTANCE MET

---

## 1. 被测版本

| 字段 | 实际值 |
|---|---|
| 产品版本 | `0.1.0-dev` |
| NWB Format | `0.x DRAFT` |
| 分支 | `codex/nwb-storage-engine` |
| IMP-000 提交 | `e1f1adb14b315f803c62d19f695a2b1b2775cd7e` |
| IMP-001 最终被测提交 | `3bceb34b4697a7552bccf9863b821a5b9d63e4b4` |
| IMP-002 最终被测提交 | `8b2a68b0528d37587636774eb76bc196b94dd59a` |
| IMP-003 初始实现提交 | `539b796110782629cefa6d5680da2dab2c61d7ff` |
| IMP-003 审查修复提交 | `2e262394d2410387e5878b3d98e4cb9687654f39` |
| IMP-003 最终被测 PR head | `d08a92baf289a9ae4eb1cb6cb668fed474ccdc5c` |
| IMP-003 CI checkout | PR 合并测试提交 `2b18c0a4588abd1760f17d8851a1a99e88b0c52f` |
| 测试日期 | 2026-07-25 |
| Pull Request | [PR #1](https://github.com/iamwuym-oss/nvwa/pull/1) — Draft、未合并 |

本文件取代 v1.2 成为当前唯一 Test Result Record。v1.2 与 v1.1 现分类为 `HISTORICAL / SUPERSEDED`；v1.0 继续分类为 `HISTORICAL / SUPERSEDED / MALFORMED_SOURCE_RETAINED_FOR_TRACEABILITY`。

## 2. IMP-003 测试环境

| Env ID | 环境 | 架构 | 工具链 | 证据 |
|---|---|---|---|---|
| ENV-CI-WIN-IMP003 | GitHub Actions `windows-latest` | x86_64 MSVC | rustc/cargo 1.97.1 | run `30184529945`，job `89746726158` |
| ENV-CI-UBU-IMP003 | GitHub Actions `ubuntu-latest` | x86_64 GNU/Linux | rustc/cargo 1.97.1 | run `30184529945`，job `89746726215` |
| ENV-CODEX-LOCAL | 当前执行容器 | x86_64 Linux | 当前会话无可用 Cargo 命令 | 最终验收不引用本地 Cargo 结果 |

## 3. Gate 与工作包状态

| Gate / IMP | 状态 | 证据 |
|---|---|---|
| GATE-0 | IN_PROGRESS | IMP-004–005 仍未完成 |
| IMP-000 | CLOSED / PASS / ACCEPTANCE MET | IMP-000 EVD；run `29426443433` |
| IMP-001 | CLOSED / PASS / ACCEPTANCE MET | IMP-001 EVD v1.2；run `29684903853` |
| IMP-002 | CLOSED / PASS / ACCEPTANCE MET | IMP-002 EVD v1.0；run `29691731514` |
| IMP-003 | CLOSED / PASS / ACCEPTANCE MET | IMP-003 EVD v1.0；run `30184529945` |
| IMP-004–005 | NOT_RUN | 按依赖与授权另行执行 |
| GATE-1–GATE-9 | NOT_RUN | 尚无 Gate 验收 |

## 4. IMP-003 质量门结果

| 命令 | Windows | Ubuntu | 实际结果 |
|---|---|---|---|
| `cargo fmt --all -- --check` | PASS | PASS | Rust 1.97.1，退出码 0 |
| `cargo run --locked -p format-registry-generator -- check` | PASS | PASS | 4 个生成文件均为最新 |
| `cargo run --locked -p traceability-checker -- check` | PASS | PASS | 37 requirements / 145 tests / 148 mappings / 34 P0 |
| `cargo test --locked -p nwb-diagnostics secret_canary` | PASS | PASS | 2 passed / 0 failed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS | PASS | 退出码 0 |
| `cargo test --workspace --locked` | PASS | PASS | 195 passed / 0 failed / 0 ignored |
| `cargo build --workspace --locked` | PASS | PASS | Debug，退出码 0 |
| `cargo build --workspace --release --locked` | PASS | PASS | Release，退出码 0 |

**CI Run：** [30184529945](https://github.com/iamwuym-oss/nvwa/actions/runs/30184529945)
**PR head：** `d08a92baf289a9ae4eb1cb6cb668fed474ccdc5c`
**Conclusion：** `SUCCESS`

## 5. IMP-003 验收映射

| AC ID | 验收目标 | 测试/证据 | 结果 |
|---|---|---|---|
| AC-IMP003-001 | 诊断使用唯一 Error Registry 身份 | `TST-ERR-001` | PASS |
| AC-IMP003-002 | 保留的零 ErrorId 不可记录 | `TST-ERR-002` | PASS |
| AC-IMP003-003 | 日志为确定性单行 JSON | `TST-ERR-003` | PASS |
| AC-IMP003-004 | Secret 普通格式化固定脱敏 | `TST-ERR-004` | PASS |
| AC-IMP003-005 | 底层 I/O 错误文本不泄漏 | `TST-ERR-005` | PASS |
| AC-IMP003-006 | Wire Schema 没有自由文本、路径或载荷字段 | `TST-ERR-006` | PASS |
| AC-IMP003-007 | 诊断事件严重度与阶段单一来源 | `TST-ERR-007` | PASS |
| AC-IMP003-008 | Windows/Linux CI 强制 Secret Canary | jobs `89746726158`、`89746726215` | PASS |
| AC-IMP003-009 | 日志脱敏与最终代码独立审查 | 独立审查与最终格式增量复核 | APPROVED |

## 6. 审查、失败与修复记录

| 阶段 | 结论 | 说明 |
|---|---|---|
| 初始独立代码审查 | CHANGES_REQUIRED | 冲突严重度/阶段、I/O 错误缺结构化上下文、ErrorId 名称第二映射 |
| 修复 | COMPLETED | `2e26239` 关闭三项问题并新增 `TST-ERR-007` |
| 独立复审 | APPROVED | 三项阻塞和文档计数问题均关闭 |
| run `29710969096` | FAILURE | Rust 1.97.1 Format check；后续门未执行，不用于验收 |
| 格式修复 | COMPLETED | `d08a92b` 仅调整两个文件的三处 rustfmt 布局 |
| 最终增量独立复核 | APPROVED | 无行为、Schema、断言或追溯变化 |
| run `30184529945` | SUCCESS | Windows/Ubuntu 全质量门通过 |

## 7. 追溯库存

| 指标 | 数量 / 状态 |
|---|---:|
| 正式需求 | 37 |
| P0 / P1 | 34 / 3 |
| 正式测试 | 145 |
| IMPLEMENTED / PLANNED | 30 / 115 |
| 唯一映射 | 148 |
| `REQ-019` 的 `TST-ERR` 测试 | 7 / 7 IMPLEMENTED |

`PLANNED` 只表示测试已登记，不表示已经执行。IMP-003 的通过结论只覆盖实际执行的 7 项 `TST-ERR` 和适用质量门。

## 8. 不适用项与限制

- IMP-003 不读写 NWB 归档，不执行备份或恢复；真实 Backup/Restore、Writer/Reader、故障注入、Verify/Salvage、卷/磁盘恢复和 BMR 均为 `N/A / NOT_RUN`。
- `Secret` 证据不覆盖崩溃转储、交换文件、进程内存扫描或未来真实密钥层级；`TST-CRY-007` 仍为 `PLANNED / NOT_RUN`。
- 日志轮转、保留、访问权限、导出和遥测不在 IMP-003。
- PR #1 仍为 Draft、未合并。
- 支持矩阵认证、Format Freeze、GATE-1–GATE-9 和产品发布仍为 `NOT_RUN / NOT_APPROVED`。

## 9. 签署

| 角色 | 结论 | 日期 |
|---|---|---|
| Independent Code Reviewer | APPROVED after remediation | 2026-07-25 |
| Final Format Delta Reviewer | APPROVED | 2026-07-25 |
| Independent Validation | FINAL-COMMIT DUAL-PLATFORM CI VERIFIED | 2026-07-25 |
| Recovery Integrity Review | N/A — no recovery execution or persistent format change | 2026-07-25 |
| Evidence Documenter | EVIDENCE ARCHIVED | 2026-07-25 |
| Project Manager | CLOSED / PASS / ACCEPTANCE MET | 2026-07-25 |

## 10. 最终结论

IMP-003 的结构化错误、固定 Schema、Secret 脱敏、I/O 错误安全上下文、单一严重度/阶段来源、7 项契约测试、独立审查和双平台 CI 均满足适用验收条件。

**IMP-003：CLOSED / PASS / ACCEPTANCE MET。**

**GATE-0：IN_PROGRESS。产品发布：NOT_APPROVED。**
