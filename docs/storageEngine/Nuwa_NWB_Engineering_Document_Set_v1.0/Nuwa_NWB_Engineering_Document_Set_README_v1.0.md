# Nüwa NWB Backup Storage Engine Engineering Document Set v1.0

**文档状态：** EXECUTION BASELINE  
**发布日期：** 2026-07-12  
**适用项目：** Nüwa Backup 单机版  
**目标读者：** 产品负责人、架构师、Codex、研发、测试、安全与发布负责人

---

## 1. 文档目的

本套文档把 Nüwa NWB 存储引擎从架构讨论转化为可以逐项实施、逐项验收和逐项留证的工程计划。任何功能只有同时具备设计、实现、自动测试、恢复验证和证据记录，才允许标记为完成。

本套文档不把“成功生成备份文件”视为完成。NWB 的最终完成标准是：

1. 能够创建合法的 Full 和 Differential 归档；
2. 能够在没有外部数据库的条件下识别、浏览、校验和恢复；
3. 能够完成 Windows 与 Linux 文件、卷、磁盘恢复；
4. 能够在认证场景下完成 BMR 并真实启动；
5. 能够准确发现截断、缺卷、错链、篡改和局部损坏；
6. 能够在不伪造完整性的前提下抢救未损坏数据；
7. 新版 Reader 能够永久读取已发布的 NWB 1.0 Golden Corpus。

## 2. 文档清单与权威顺序

| 优先级 | 文档 | 用途 |
|---:|---|---|
| 1 | `Nuwa_NWB_Storage_Engine_Architecture_v2.0.md` | 总体架构、不可变原则、Catalog、分卷、BMR与安全边界 |
| 2 | `Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md` | NWB磁盘格式、字段、提交与兼容协议 |
| 3 | `Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md` | Windows/Linux及未来数据类型Provider契约 |
| 4 | `Nuwa_NWB_Product_Support_Matrix_v1.0.md` | 首版正式支持、有限支持与不支持范围 |
| 5 | `Nuwa_NWB_Implementation_Plan_v1.0.md` | 工作包、依赖、里程碑、退出条件和交付物 |
| 6 | `Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md` | 测试设计、质量门、验收标准与故障注入 |
| 7 | `Nuwa_NWB_Test_Result_Record_v1.0.md` | 实际执行结果、证据、缺陷和签署记录 |

发生冲突时，后面的实施文档不得改变前面的格式或架构原则。确需改变时，必须先新增 ADR，再修改所有受影响文档和测试。

## 3. 已冻结的产品决策

以下决策不得由实现人员自行改变：

1. 产品首先是单机版本，不依赖 Repository、管理服务器或外部恢复数据库；
2. 每次备份产生一个逻辑 NWB；通常是一个 `.nwb` 文件，必要时为一个受控 Volume Set；
3. 只实现 Full 和 Differential，不实现 Incremental；
4. Differential 只依赖指定 Full，不依赖其他 Differential；
5. Full 与 Differential 都具有自身完整逻辑 Catalog；
6. Internal Catalog 是恢复事实源，本地 Catalog Cache 只负责速度，Search Index 可选；
7. 已提交 NWB 不允许原地修改；转换、修复、改密必须输出新归档；
8. 文件、卷、磁盘、BMR共用同一容器协议；
9. Windows 7 SP1及之后的Windows产品线、主流x86-64 Linux为目标平台；
10. BIOS/MBR、UEFI/GPT、BMR和认证范围内的异机恢复属于首版核心能力；
11. Provider负责理解来源和执行恢复，NWB Core独占归档物理写入、加密、校验和提交；
12. 未经过真实恢复和启动验证的场景不得标记为正式支持。

## 4. 工程状态模型

每个需求、工作包和测试只能使用以下状态：

| 状态 | 含义 |
|---|---|
| `PLANNED` | 已定义但尚未开始 |
| `IN_PROGRESS` | 正在实现或测试 |
| `BLOCKED` | 存在明确阻塞项，并已记录责任与解除条件 |
| `IMPLEMENTED` | 代码已完成，但尚未通过全部验收 |
| `VERIFIED` | 自动测试和静态验收通过 |
| `ACCEPTED` | 恢复验证、证据和文档齐全，质量门签署通过 |
| `REJECTED` | 验收失败，需要返工或撤销 |
| `NOT_APPLICABLE` | 经架构批准不适用，并记录理由 |

禁止使用“基本完成”“应该可以”“看起来正常”等不可审计描述。

## 5. 每一步的强制闭环

每个实施工作包都必须产生以下记录：

```text
需求ID
设计或ADR
实现提交
单元测试
集成测试
故障测试
验收用例
实际测试结果
日志与产物哈希
已知限制
文档更新
验收人和日期
```

缺少任意一项时，状态最高只能是 `IMPLEMENTED`，不能进入 `ACCEPTED`。

## 6. Codex执行规则

Codex执行本项目时必须遵守：

1. 先读取本套全部文档和仓库中的 `AGENTS.md`；
2. 先盘点现有代码，不假设模块已经存在；
3. 每次只实施一个有明确ID的工作包；
4. 开始工作包前建立或更新实施计划；
5. 不得通过修改测试来掩盖实现错误；
6. 不得在NWB格式中直接序列化Rust/C/C++内存对象；
7. 不得让Provider直接写NWB文件或取得明文归档密钥；
8. 不得把外部数据库作为唯一恢复信息来源；
9. 不得把未提交、被截断或缺卷归档标记为成功；
10. 每完成一个质量门，必须回填测试结果记录；
11. 出现需要改变已冻结决策的问题时停止实施，提交ADR，不得自行绕过；
12. 测试失败必须保留原始证据和复现步骤。

## 7. 需求与证据编号

| 前缀 | 类型 |
|---|---|
| `REQ-` | 产品或系统需求 |
| `FMT-` | 二进制格式需求 |
| `PRV-` | Provider接口需求 |
| `SEC-` | 安全需求 |
| `REL-` | 可靠性需求 |
| `BMR-` | 裸机恢复需求 |
| `IMP-` | 实施工作包 |
| `TST-` | 测试用例 |
| `GATE-` | 质量门 |
| `ADR-` | 架构决策 |
| `DEF-` | 缺陷记录 |
| `EVD-` | 测试证据 |

每个测试必须追溯至少一个需求，每个P0需求必须至少有一个正向用例和一个失败/故障用例。

## 8. 总体质量门

| Gate | 名称 | 通过结果 |
|---|---|---|
| `GATE-0` | 文档与仓库基线 | 模块边界、构建、CI、追溯表建立 |
| `GATE-1` | NWB容器 | 单文件追加写入、解析、提交和截断识别通过 |
| `GATE-2` | 文件Full/Diff | Windows/Linux测试数据逐字节和语义恢复一致 |
| `GATE-3` | 分卷与密码学 | 分卷、压缩、加密、错密、缺卷和篡改测试通过 |
| `GATE-4` | Verify/Salvage | 损坏定位准确，未损坏对象可控抢救 |
| `GATE-5` | 卷与磁盘 | GPT/MBR、分区、块范围恢复验证通过 |
| `GATE-6` | Windows BMR | UEFI/NTFS标准场景恢复后真实启动 |
| `GATE-7` | Linux BMR | UEFI/LVM/ext4标准场景恢复后真实启动 |
| `GATE-8` | 格式冻结 | 跨平台Reader、Golden Corpus、安全解析和兼容测试通过 |
| `GATE-9` | 产品发布 | 支持矩阵内全部强制用例通过，文档与证据齐全 |

后一个Gate不得替代前一个Gate。`GATE-8`通过前，格式版本只能使用 `0.x`。

## 9. 测试结果真实性

在研发开始前，所有实际测试状态都是 `NOT_RUN`。文档中的预期结果是验收条件，不是测试结果。

只有满足以下条件才可将用例改为 `PASS`：

- 记录被测版本和提交哈希；
- 记录操作系统、文件系统、磁盘拓扑和硬件/虚拟机配置；
- 保存命令、日志、退出码和关键产物；
- 保存输入与恢复结果哈希；
- BMR用例保存启动证据；
- 失败重试不得覆盖第一次失败证据；
- 测试执行人与复核人完成记录。

## 10. 完成定义

存储引擎只有在 `GATE-8` 通过后才可以称为“NWB Format 1.0”；单机备份产品只有在 `GATE-9` 通过后才可以称为“首版完成”。

本README是执行入口，不替代任何详细规范。
## 11. 当前真实性声明

本README是执行入口，不替代任何详细规范。以下为截至2026-07-15的实际工程状态：

| 项 | 状态 |
|---|---|
| GATE-0 | IN_PROGRESS |
| IMP-000 (Workspace) | IMPLEMENTED — 验收证据不足 |
| IMP-001 (Format Registry) | IN_PROGRESS — 正式的Generator要求尚未满足 |
| IMP-002 | NOT_RUN |

- IMP-000和IMP-001均不得描述为VERIFIED、ACCEPTED或CLOSED
- Test Result Record和EVD已纠正为反映当前未满足验收的状态；Engineering Document Set Manifest仍待重新生成。
- 任何工作包只有同时具备设计、实现、自动测试、恢复验证和证据记录才允许标记为ACCEPTED
- IMP-002 remains NOT_RUN until explicitly authorized
- IMP-003–005 execute within GATE-0 according to dependencies and authorization
- IMP-006–009 are undefined and must not start until defined
- Only IMP-100 and later must wait for GATE-0 closure