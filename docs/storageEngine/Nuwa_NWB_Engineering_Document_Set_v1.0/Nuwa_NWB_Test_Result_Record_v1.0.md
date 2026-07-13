# Nüwa NWB Test Result and Acceptance Record v1.0

**记录状态：** INITIAL / NOT_RUN  
**说明：** 本文件是实际测试结果的唯一汇总模板。当前没有执行产品测试，因此不得把预期结果写为PASS。

---

## 1. 被测版本

| 字段 | 实际值 |
|---|---|
| 产品版本 | `0.1.0-dev` |
| NWB Format版本 | `0.x DRAFT`（IMP-000 工程基线就绪） |
| Git提交 | `ad6695b` + IMP-000未提交修改 |
| 分支 | `codex/nwb-storage-engine` |
| 构建ID | `IMP-000-EVD-001` |
| Rust版本 | `rustc 1.96.1 (31fca3adb 2026-06-26)` |
| 依赖锁文件SHA-256 | `12423e97bd8cabadbc2d4ec3bec411e9e92923f053fb1aabb4cdf2c7e552bf74` |
| Recovery Media版本 | `NOT_SET` |
| 测试开始/结束时间 | `2026-07-13` |
| 测试负责人 | `nwb_validation_engineer` |
| 复核人 | `nwb_code_reviewer` |

## 2. 测试环境记录

每个环境复制一行：

| Env ID | 物理/虚拟 | OS/Build | CPU架构 | Firmware | 磁盘/控制器 | Sector | 文件系统/拓扑 | RAM | 备注 |
|---|---|---|---|---|---|---|---|---:|---|
| `ENV-TBD` | TBD | TBD | TBD | TBD | TBD | TBD | TBD | TBD | NOT_RUN |

## 3. Gate状态总览

| Gate | 名称 | 状态 | 通过日期 | 证据集合 | 阻塞缺陷 | 签署 |
|---|---|---|---|---|---|---|
| GATE-0 | 文档与仓库基线 | IN_PROGRESS | — | IMP-000_EVD | — | — |
| GATE-0.IMP-000 | 建立Workspace | PASS | 2026-07-13 | IMP-000_EVD_Build_Evidence_v1.0.md | — | nwb_evidence_documenter |
| GATE-1 | NWB容器 | NOT_RUN | — | — | — | — |
| GATE-2 | 文件Full/Diff | NOT_RUN | — | — | — | — |
| GATE-3 | 分卷与密码学 | NOT_RUN | — | — | — | — |
| GATE-4 | Verify/Salvage | NOT_RUN | — | — | — | — |
| GATE-5 | 卷与磁盘 | NOT_RUN | — | — | — | — |
| GATE-6 | Windows BMR | NOT_RUN | — | — | — | — |
| GATE-7 | Linux BMR | NOT_RUN | — | — | — | — |
| GATE-8 | Format 1.0冻结 | NOT_RUN | — | — | — | — |
| GATE-9 | 产品发布 | NOT_RUN | — | — | — | — |

## 4. 单项测试结果模板

为每个测试复制以下区块：

### `[TEST-ID] 测试名称`

| 字段 | 实际记录 |
|---|---|
| 关联需求/工作包 | TBD |
| 测试环境 | TBD |
| 前置条件 | TBD |
| 输入Dataset及Manifest哈希 | TBD |
| 执行命令/自动化Job | TBD |
| 开始/结束时间 | TBD |
| 实际退出码 | TBD |
| 预期结果 | 引用测试计划，不在此改写标准 |
| 实际结果 | NOT_RUN |
| 状态 | NOT_RUN / PASS / FAIL / BLOCKED / SKIPPED |
| 日志证据 | TBD |
| 输出NWB SHA-256 | TBD |
| 恢复结果Manifest SHA-256 | TBD |
| 性能数据 | TBD |
| 关联缺陷 | NONE/TBD |
| 执行人 | TBD |
| 复核人 | TBD |

复现步骤：

```text
NOT_RUN
```

观察与限制：

```text
NOT_RUN
```

## 5. BMR结果模板

| 字段 | 实际记录 |
|---|---|
| Test ID | TBD |
| 源机器/VM ID | TBD |
| 目标机器/VM ID | TBD |
| 源/目标Firmware | TBD |
| 源/目标磁盘控制器 | TBD |
| Full/Diff Archive ID | TBD |
| Archive Verify结果 | TBD |
| 恢复计划哈希 | TBD |
| 恢复开始/结束 | TBD |
| 首次启动结果 | NOT_RUN |
| 启动修复次数 | TBD |
| 登录后验证脚本结果 | TBD |
| 分区/卷布局证据 | TBD |
| BCD/GRUB/initramfs证据 | TBD |
| 文件Manifest比较 | TBD |
| 截图/串口/视频证据ID | TBD |
| 最终状态 | NOT_RUN |

只有恢复后真实启动、登录并完成验证脚本，BMR用例才允许PASS。仅看到启动菜单不算通过。

## 6. 故障注入结果模板

| Fault ID | 注入点 | 注入方式 | 预期状态 | 实际状态 | 是否伪Commit | 可恢复sealed范围 | 证据 | 结果 |
|---|---|---|---|---|---|---|---|---|
| TBD | TBD | TBD | TBD | NOT_RUN | TBD | TBD | TBD | NOT_RUN |

故障测试必须验证清理后状态，包括快照、锁、临时卷、临时文件和Cache。

## 7. Golden Corpus登记

| 文件 | Format | 场景 | Archive ID | 大小 | SHA-256 | 生成提交 | 状态 |
|---|---|---|---|---:|---|---|---|
| `nwb-1.0-empty.nwb` | 1.0 | Empty | TBD | TBD | TBD | TBD | NOT_CREATED |
| `nwb-1.0-small-files.nwb` | 1.0 | File Full | TBD | TBD | TBD | TBD | NOT_CREATED |
| `nwb-1.0-encrypted.nwb` | 1.0 | Encrypted | TBD | TBD | TBD | TBD | NOT_CREATED |
| `nwb-1.0-full.nwb` | 1.0 | Full | TBD | TBD | TBD | TBD | NOT_CREATED |
| `nwb-1.0-diff.nwb` | 1.0 | Diff | TBD | TBD | TBD | TBD | NOT_CREATED |
| `nwb-1.0-split.*` | 1.0 | Volume Set | TBD | TBD | TBD | TBD | NOT_CREATED |
| `nwb-1.0-windows-disk.nwb` | 1.0 | Windows BMR | TBD | TBD | TBD | TBD | NOT_CREATED |
| `nwb-1.0-linux-disk.nwb` | 1.0 | Linux BMR | TBD | TBD | TBD | TBD | NOT_CREATED |
| `nwb-1.0-damaged-chunk.nwb` | 1.0 | Negative | TBD | TBD | TBD | TBD | NOT_CREATED |
| `nwb-1.0-truncated.nwb` | 1.0 | Negative | TBD | TBD | TBD | TBD | NOT_CREATED |

Golden文件只有在GATE-8签署时创建一次。之后不得用新Writer覆盖；新增兼容样例必须使用新文件名。

## 8. 缺陷登记

| Defect ID | 严重度 | 发现用例 | 描述 | 数据/恢复影响 | 复现率 | Owner | 状态 | 修复提交 | 回归用例 |
|---|---|---|---|---|---:|---|---|---|---|
| — | — | — | 当前无执行结果 | — | — | — | NOT_RUN | — | — |

## 9. 豁免登记

| Waiver ID | 关联测试/缺陷 | 理由 | 用户影响 | 绕行 | 到期条件 | 批准人 |
|---|---|---|---|---|---|---|
| — | — | 当前无豁免 | — | — | — | — |

P0不得豁免。P1原则上不得豁免；确需发布必须由产品、架构、安全和测试共同批准且不涉及数据丢失、错误恢复、密钥泄漏或伪成功。

## 10. 支持矩阵认证账本

| 平台/场景 | 计划等级 | 实际认证等级 | 测试ID | 证据 | 限制 | 批准日期 |
|---|---|---|---|---|---|---|
| Windows UEFI/GPT/NTFS x64 | CERTIFIED目标 | NOT_TESTED | TBD | TBD | TBD | — |
| Linux UEFI/GPT/LVM/ext4 x64 | CERTIFIED目标 | NOT_TESTED | TBD | TBD | TBD | — |
| 其他支持矩阵项目 | 见矩阵 | NOT_TESTED | TBD | TBD | TBD | — |

产品运行时显示的能力等级必须来自已经批准的认证数据，不得超过本账本。

## 11. Format Freeze签署

| 角色 | 姓名 | 结论 | 日期 | 签名/审批记录 |
|---|---|---|---|---|
| 产品负责人 | TBD | NOT_REVIEWED | — | — |
| 存储架构负责人 | TBD | NOT_REVIEWED | — | — |
| Windows Provider负责人 | TBD | NOT_REVIEWED | — | — |
| Linux Provider负责人 | TBD | NOT_REVIEWED | — | — |
| 测试负责人 | TBD | NOT_REVIEWED | — | — |
| 安全负责人 | TBD | NOT_REVIEWED | — | — |

结论选项：`APPROVED`、`REJECTED`、`CONDITIONAL`。只要任一P0/P1未关闭或Golden Corpus未封存，不得选择`APPROVED`。

## 12. Release签署

| 检查项 | 状态 | 证据 |
|---|---|---|
| GATE-0至GATE-9全部通过 | NOT_RUN | — |
| 支持矩阵认证完成 | NOT_RUN | — |
| P0/P1为零 | NOT_RUN | — |
| 安全审查通过 | NOT_RUN | — |
| Recovery Media验证通过 | NOT_RUN | — |
| 用户/管理员/恢复文档完成 | NOT_RUN | — |
| 发布包和SBOM封存 | NOT_RUN | — |

**最终发布结论：NOT_APPROVED / NOT_RUN**


