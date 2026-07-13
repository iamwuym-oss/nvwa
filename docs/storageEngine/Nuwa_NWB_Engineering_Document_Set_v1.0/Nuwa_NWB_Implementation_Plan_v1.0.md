# Nüwa NWB Storage Engine Implementation Plan v1.0

**状态：** APPROVED PLAN  
**日期：** 2026-07-12  
**执行方式：** 质量门驱动，不以日历时间替代完成标准

---

## 1. 实施目标

交付一个跨Windows/Linux、无外部恢复数据库依赖、支持Full/Differential、单文件/分卷、文件/卷/磁盘、Verify/Salvage及认证BMR场景的NWB存储引擎。

实施顺序必须先证明格式可靠，再扩展平台范围。不得先制作完整UI再补救底层格式。

## 2. 建议仓库结构

```text
docs/
  architecture/
  format/
  provider/
  test/
crates/
  nwb-format/            规范结构、编码、解析和Registry
  nwb-core/              Writer、Reader、Archive状态机
  nwb-catalog/           Catalog Page、树和查询
  nwb-chunk/             Chunk、压缩、Extent
  nwb-crypto/            KDF、Key Slot、AEAD、密钥清理
  nwb-provider-api/      公共对象与Provider协议
  nwb-verify/            完整性验证
  nwb-salvage/           只读扫描与抢救计划
  nwb-cli/               inspect/create/restore/verify/salvage
providers/
  common-raw/
  windows-vss/
  windows-ntfs/
  windows-boot/
  linux-files/
  linux-lvm/
  linux-boot/
recovery/
  image-build/
  driver-pack/
tests/
  fixtures/
  golden/
  integration/
  fault-injection/
  fuzz/
  bmr/
tools/
  format-registry/
  corpus-manifest/
```

可以根据现有仓库调整目录，但模块边界、依赖方向和测试责任不得丢失。

## 3. 工程通用标准

### 3.1 代码标准

- 禁止`unsafe`解析不可信NWB字节；确需`unsafe`必须单独审查和测试；
- 所有Offset/Length使用checked arithmetic；
- 所有I/O返回结构化错误和上下文；
- 解析器不能`panic`处理损坏档案；
- Writer不得Seek回改已密封内容；
- 日志不得记录密码、密钥、敏感文件内容；
- 平台代码只能通过Provider进入Core；
- 每个公共格式结构必须有编码、解码、往返和坏数据测试；
- 所有随机ID和Nonce使用CSPRNG；
- 构建必须可重复记录依赖锁文件和工具链版本。

### 3.2 每个工作包完成定义

1. 设计与需求ID明确；
2. 代码和错误处理完成；
3. 单元、集成和负向测试完成；
4. 测试结果回填；
5. 用户可见行为和限制有文档；
6. 无未关闭P0/P1缺陷；
7. 产物和日志SHA-256已记录；
8. 通过对应Gate。

## 4. 阶段总览

| 阶段 | 工作包 | Gate | 核心输出 |
|---|---|---|---|
| 0 | `IMP-000–009` | GATE-0 | 仓库、CI、Registry、追溯 |
| 1 | `IMP-100–119` | GATE-1 | 单文件容器、Record、Segment、Commit |
| 2 | `IMP-200–229` | GATE-2 | Catalog、文件Full、文件Diff |
| 3 | `IMP-300–329` | GATE-3 | 分卷、压缩、加密、密钥 |
| 4 | `IMP-400–429` | GATE-4 | Verify、Salvage、故障恢复 |
| 5 | `IMP-500–529` | GATE-5 | 卷、磁盘、GPT/MBR、块恢复 |
| 6 | `IMP-600–639` | GATE-6 | Windows VSS、NTFS、UEFI BMR、异机 |
| 7 | `IMP-700–739` | GATE-7 | Linux文件、LVM、UEFI BMR、异机 |
| 8 | `IMP-800–829` | GATE-8 | 跨平台、Golden、Fuzz、格式冻结 |
| 9 | `IMP-900–939` | GATE-9 | 支持矩阵认证、发布与长期兼容 |

## 5. 阶段0：工程与契约基线

### 5.1 工作包

| ID | 计划与实现 | 标准 | 验收 | 测试 | 文档/证据 |
|---|---|---|---|---|---|
| `IMP-000` | 建立workspace、模块和依赖方向 | Core不得依赖平台Provider实现 | 干净环境一条命令构建 | Debug/Release、Windows/Linux CI | 构建说明、依赖图、EVD构建日志 |
| `IMP-001` | 实现Format Registry生成器 | Record/Feature/Error ID唯一 | 重复ID导致构建失败 | Registry单元和快照测试 | Registry文档、生成产物哈希 |
| `IMP-002` | 建立需求-测试追溯表 | P0需求100%映射 | 检查脚本无孤儿P0 | CI追溯检查 | Traceability Matrix |
| `IMP-003` | 建立结构化错误和日志 | 无密钥/密码泄露 | 日志脱敏审查通过 | Secret canary测试 | Error Registry、日志规范 |
| `IMP-004` | 建立测试Fixture生成器 | Fixture可重复、Manifest固定 | 两次生成哈希一致或解释随机字段 | 数据集自校验 | Fixture说明、Manifest |
| `IMP-005` | 建立格式0.x版本策略 | Draft不伪装1.0 | CLI明确显示Draft | 兼容拒绝测试 | 版本策略ADR |

### 5.2 GATE-0退出条件

- Windows与Linux CI可构建和运行测试；
- Registry、错误码、需求ID和测试ID有唯一来源；
- 格式文档与生成代码字段能够互相检查；
- 所有测试初始状态为`NOT_RUN`或真实执行状态；
- 无代码实现可以跳过格式层直接写档案。

## 6. 阶段1：NWB容器最小闭环

### 6.1 工作包

| ID | 计划与实现 | 标准 | 验收 | 必须测试 | 文档/证据 |
|---|---|---|---|---|---|
| `IMP-100` | Bootstrap Header codec | 严格4096字节、Little Endian、校验 | 独立检查器解析一致 | 往返、坏Magic、坏CRC、未知Feature | 字段表、十六进制样例 |
| `IMP-101` | Record Envelope codec | 64字节、checked length | 所有非法长度被拒绝 | overflow、truncate、unknown optional/required | Parser行为表 |
| `IMP-102` | Segment Writer/Reader | 追加写、4KiB对齐、独立密封 | 只读取sealed Segment | 各字节边界截断、Footer损坏 | Segment状态图、日志 |
| `IMP-103` | Manifest | 规范排序、引用全部必需对象 | 重建哈希完全一致 | 错Offset、重复ID、漏Segment | Manifest Schema |
| `IMP-104` | 双Commit | 两副本独立校验 | 一坏可开、两坏只Salvage | A/B损坏、不一致、截断 | Commit决策表 |
| `IMP-105` | Archive Writer状态机 | NEW→WRITING→FINALIZING→COMMITTED | 未Commit不返回成功 | 进程终止、I/O失败、取消 | 状态机与错误映射 |
| `IMP-106` | Archive Reader/Inspect CLI | 不扫描数据即可打开正常档案 | 显示身份、Feature、Segment摘要 | 正常/未知/损坏样例 | CLI使用说明 |

### 6.2 GATE-1退出条件

- 能创建、提交、重新打开最小单文件NWB；
- Header、Record、Segment、Manifest、Commit均有独立编码/解码测试；
- 任意关键边界截断不会被识别为有效归档；
- 两份Commit规则与规范一致；
- Reader对不可信长度不崩溃、不越界；
- 生成最小Draft Golden样例，但不得标为1.0。

## 7. 阶段2：Catalog与文件Full/Diff

### 7.1 工作包

| ID | 计划与实现 | 标准 | 验收 | 必须测试 | 文档/证据 |
|---|---|---|---|---|---|
| `IMP-200` | Chunk/Extent流水线 | 256KiB基线，特殊范围不混淆 | 内容和范围往返一致 | ZERO/HOLE/UNREADABLE/尾Chunk | Chunk布局与基准结果 |
| `IMP-201` | 64KiB Catalog Page | 不可变、有序、独立校验 | 百万级Entry构建查询一致 | 坏页、循环、重复Key、深度上限 | Catalog Page Spec |
| `IMP-202` | Catalog树和Root | 八类必需树完整 | 无外部DB可枚举对象 | 空档案、深目录、大量对象 | 查询和重建说明 |
| `IMP-203` | Checkpoint Catalog | 只引用sealed内容 | 中断后可诊断/Salvage | 各Checkpoint边界终止 | Checkpoint语义 |
| `IMP-204` | Generic File Provider | 保存公共文件语义 | Fixture恢复逐字节一致 | 空/小/大/稀疏/链接/长路径 | Provider限制 |
| `IMP-205` | 文件Full | 自包含完整Catalog | 删除Cache后仍恢复 | 数据与元数据比较 | Full流程记录 |
| `IMP-206` | 文件Differential | 只依赖指定Full | Full+Diff恢复当前状态 | 新增/修改/删除/重命名/错Full | Diff算法与证据 |
| `IMP-207` | Local Catalog Cache | 可删除可重建 | Cache损坏不影响恢复 | 删除、篡改、版本不匹配 | Cache Schema/重建说明 |

### 7.2 GATE-2退出条件

- Full恢复全部Fixture内容和支持语义；
- Diff含完整逻辑Catalog，只引用一个Full；
- 错误或被替换Full被根哈希拒绝；
- Cache完全删除后正常浏览和恢复；
- Windows/Linux名称原始表示不会丢失；
- 10万及以上对象压力样例无结构性错误，百万级Catalog基准有记录。

## 8. 阶段3：分卷、压缩与加密

### 8.1 工作包

| ID | 计划与实现 | 标准 | 验收 | 必须测试 | 文档/证据 |
|---|---|---|---|---|---|
| `IMP-300` | Zstd/None逐Chunk压缩 | 不可压缩自动None | 恢复哈希一致 | 零、重复、随机、高压缩比 | Codec Registry、性能记录 |
| `IMP-301` | AES-256-GCM | 每Chunk独立认证，同一数据子密钥Nonce不重复 | 位翻转必失败 | tag/nonce/ciphertext/AAD篡改 | Crypto设计审查 |
| `IMP-302` | Argon2id与Key Slot | 参数入档、恢复Slot可选 | 正/错密码行为确定 | 空密码策略、错密、损坏Slot | Key生命周期文档 |
| `IMP-303` | 密钥内存处理 | 不写日志/转储，使用后清理 | Secret扫描无泄漏 | canary secret、异常路径 | 安全测试证据 |
| `IMP-304` | Volume Writer | Segment/Page不跨卷 | 小阈值多卷恢复 | 1/2/多卷、边界、空间不足 | Volume协议记录 |
| `IMP-305` | Volume链和Set Manifest | ID/序号/哈希绑定 | 重命名仍识别、错卷拒绝 | 缺中卷、错序、替换、缺Final | 缺卷诊断证据 |
| `IMP-306` | AUTO/NO_SPLIT/FIXED_SIZE | 目标能力预检 | FAT32自动安全分卷 | 模拟限制、真实FAT32、SMB | 策略说明 |

### 8.2 GATE-3退出条件

- 明文与加密、压缩与不压缩、单文件与分卷组合测试通过；
- 错密码与档案损坏错误可区分且不泄漏敏感信息；
- Nonce唯一性检查和崩溃路径通过；
- 缺卷准确列出缺失成员和受影响对象；
- Final Volume未提交时不得报告成功；
- 加密Catalog不向Cache泄漏文件名，除非用户明确允许受保护缓存。

## 9. 阶段4：Verify、Salvage与故障恢复

### 9.1 工作包

| ID | 计划与实现 | 标准 | 验收 | 必须测试 | 文档/证据 |
|---|---|---|---|---|---|
| `IMP-400` | Quick Verify | Header/Commit/Manifest/Catalog Root | 快速判断身份和提交状态 | 所有顶部结构损坏 | Verify级别定义 |
| `IMP-401` | Full Verify | 读取并验证所有Segment/Chunk | 精确报告坏对象和范围 | 单/多点位翻转、坏卷 | 报告Schema |
| `IMP-402` | Salvage Scanner | 只读、边界严格、不改原档案 | 输出可恢复对象和新档案计划 | 截断、坏Catalog、缺卷 | Salvage原则 |
| `IMP-403` | Salvage Writer | 只输出新NWB | 新档案明确标记来源和缺失 | 部分恢复、冲突、空间不足 | Provenance记录 |
| `IMP-404` | 故障注入框架 | 每个写步骤可终止/报错 | 结果状态与规范一致 | I/O错误、ENOSPC、拔盘、断网 | 故障矩阵 |
| `IMP-405` | 暂停/取消/会话内续写 | 只续写同一有效快照 | 快照失效后拒绝危险续写 | pause/cancel/crash/reboot | 续写状态机 |

### 9.2 GATE-4退出条件

- Verify无漏报已注入损坏；
- 报告能定位Volume、Segment、Record、Chunk和对象影响；
- Salvage不覆盖原档案；
- 未校验数据不伪装为正常数据；
- 任意写入阶段故障不会产生伪Commit；
- 残留临时文件和陈旧锁可安全识别清理。

## 10. 阶段5：卷与磁盘

### 10.1 工作包

| ID | 计划与实现 | 标准 | 验收 | 必须测试 | 文档/证据 |
|---|---|---|---|---|---|
| `IMP-500` | Block Source Provider | 保存Sector、容量、范围和错误 | 原始块哈希一致 | 非对齐读、尾范围、坏扇区 | Block API |
| `IMP-501` | 分配位图与Raw fallback | 未知FS仍可Raw | 已分配/原始模式结果明确 | NTFS/ext4/未知FS | 模式选择说明 |
| `IMP-502` | GPT/MBR双重捕获 | 原始字节+解析语义 | 重建后布局一致 | 主/备GPT损坏、MBR扩展分区 | Partition Spec |
| `IMP-503` | 卷/磁盘Full与Diff | Hint不决定正确性 | 变化块恢复一致 | Hint失效、全量回退 | 差异证据 |
| `IMP-504` | Block Restore Planner | 稳定ID、容量、Sector预检 | 错盘/小盘被阻止 | 512e/4Kn、大小变化 | Destructive Safety |
| `IMP-505` | Block Restore与验证 | 写后验证关键区域 | 恢复盘逐块或抽样一致 | I/O错误、重试、填零策略 | Restore报告 |

### 10.2 GATE-5退出条件

- GPT/MBR、卷和原始磁盘可创建和恢复；
- 目标盘选择不能只依赖序号；
- 比源使用空间小的目标被阻止；
- 512e/4Kn策略有实际测试记录；
- UNREADABLE范围在归档和恢复报告中保留；
- 块Diff在变化跟踪失效时安全回退。

## 11. 阶段6：Windows BMR与异机恢复

### 11.1 工作包

| ID | 计划与实现 | 标准 | 验收 | 必须测试 | 文档/证据 |
|---|---|---|---|---|---|
| `IMP-600` | Windows Discovery/VSS | 多卷Snapshot Set和Writer结果入档 | 一致性等级准确 | Writer成功/失败/超时、快照失效 | VSS行为记录 |
| `IMP-601` | NTFS语义 | ACL/ADS/HardLink/Sparse/Reparse等 | Fixture语义对比通过 | 特殊名称、EFS状态、长路径 | NTFS支持表 |
| `IMP-602` | Windows Boot Capture | GPT/ESP/BCD/WinRE/NVRAM状态 | BMR预检完整 | ESP缺失、BCD异常、Secure Boot | Boot Profile |
| `IMP-603` | Recovery Media | UEFI x64、磁盘/USB/SMB、NWB Reader | 无原系统可恢复 | Secure Boot、网络、驱动加载 | Media构建与版本 |
| `IMP-604` | Windows同机BMR | 恢复后真实启动 | 登录并验证数据/服务 | Full、Diff、更大盘、认证小盘 | 启动证据 |
| `IMP-605` | Windows异机恢复 | 存储驱动注入和BCD修复 | 不同虚拟/物理控制器启动 | SATA↔NVMe、基础RAID驱动 | Driver/Boot报告 |
| `IMP-606` | Windows 7兼容构建 | 不依赖新OS专属运行时 | Win7 SP1安装、备份、恢复 | x64及批准的x86 Legacy | 兼容构建说明 |

### 11.2 GATE-6退出条件

- Windows UEFI/GPT/NTFS标准机Full与Diff BMR均启动；
- 备份源VSS状态与归档一致性声明一致；
- Recovery Media可以在无本机OS/Cache条件下完成恢复；
- 异机恢复至少覆盖两类存储控制器变化；
- BitLocker场景按支持矩阵通过或明确阻止；
- Windows 7 SP1至当前认证版本的结果分别记录，不能用一次测试代替。

## 12. 阶段7：Linux BMR与异机恢复

### 12.1 工作包

| ID | 计划与实现 | 标准 | 验收 | 必须测试 | 文档/证据 |
|---|---|---|---|---|---|
| `IMP-700` | Linux Discovery/Snapshot | LVM/Btrfs/fsfreeze能力明确 | 一致性结果入档 | 快照成功/空间不足/降级 | Snapshot策略 |
| `IMP-701` | Linux文件语义 | 原始名、UID/GID、ACL/xattr/SELinux等 | Fixture语义一致 | 非UTF8名、设备节点、链接 | Linux语义表 |
| `IMP-702` | ext4/XFS/Btrfs Provider | 按支持等级实现 | 文件和卷恢复 | 各FS Full/Diff/损坏 | FS限制文档 |
| `IMP-703` | LVM2 | PV/VG/LV和UUID保存 | 重建认证拓扑 | 多PV、缺PV、大小变化 | LVM Profile |
| `IMP-704` | Linux Boot Provider | GRUB2/systemd-boot、fstab、initramfs | 恢复后启动 | UUID变化、驱动变化、UEFI | Boot报告 |
| `IMP-705` | Linux BMR/异机 | 标准LVM/ext4启动 | 登录、挂载、服务验证 | Full/Diff、更大盘、控制器变化 | 启动证据 |

### 12.2 GATE-7退出条件

- Ubuntu UEFI/LVM/ext4标准场景Full和Diff BMR启动；
- ext4、XFS和Btrfs的声明能力分别有证据；
- 非UTF8文件名和Linux安全元数据不被静默丢失；
- GRUB、fstab、UUID和initramfs在目标变化后正确；
- 复杂RAID/ZFS未认证场景被识别并阻止BMR误导。

## 13. 阶段8：Format 1.0冻结

### 13.1 工作包

| ID | 计划与实现 | 标准 | 验收 | 必须测试 | 文档/证据 |
|---|---|---|---|---|---|
| `IMP-800` | 独立格式检查器 | 与Core无共享解析代码 | 所有Golden字段/哈希一致 | 正常与故意损坏档案 | Checker报告 |
| `IMP-801` | 跨平台互读 | Windows/Linux字节解释一致 | 双向读取和恢复 | 明文/加密/分卷/Full/Diff | Compatibility报告 |
| `IMP-802` | Golden Corpus | 固定档案永不再生成 | SHA-256清单封存 | 未来Reader回归 | Corpus Manifest |
| `IMP-803` | Parser Fuzz | 无崩溃、越界、无限分配 | 规定时长无P0/P1 | Record/Page/Manifest/压缩输入 | Fuzz报告与Corpus |
| `IMP-804` | 格式性能和资源基线 | 流式有界内存 | 达到基线或有ADR | 大文件、千万对象、深目录 | Benchmark报告 |
| `IMP-805` | 格式审查与签署 | 字段、Registry、规范同步 | 无需破坏性改盘缺陷 | 全Gate回归 | Format Freeze Record |

### 13.2 GATE-8退出条件

满足Binary Format Specification第21节全部条件。通过后：

- `format_major=1`；
- Golden Corpus成为永久兼容资产；
- 任何破坏性修改必须提出NWB 2.0 ADR；
- 1.0 Reader进入长期维护组件。

## 14. 阶段9：首版发布

### 14.1 工作包

| ID | 计划与实现 | 标准 | 验收 | 必须测试 | 文档/证据 |
|---|---|---|---|---|---|
| `IMP-900` | 支持矩阵认证 | 每格有测试证据 | 无“推测支持” | OS/FS/拓扑矩阵 | Certification Ledger |
| `IMP-901` | 长时间稳定性 | 多轮备份恢复无累积错误 | 连续循环通过 | Full/Diff/删除/重建Cache | Soak报告 |
| `IMP-902` | 性能与限流 | 不压垮交互系统 | CPU/RAM/I/O策略生效 | 本地、USB、SMB、慢盘 | Performance报告 |
| `IMP-903` | 安全审查 | 密钥、解析、恢复路径安全 | P0/P1关闭 | Fuzz、Secret、Traversal | Security报告 |
| `IMP-904` | 安装/升级/Recovery Media | 版本一致、可回滚 | 支持系统安装和救援 | 升级旧Reader、Media更新 | 运维指南 |
| `IMP-905` | 发布签署 | 代码、测试、文档一致 | 所有Gate Accepted | 全回归 | Release Record |

## 15. 缺陷等级与Gate规则

| 等级 | 定义 | Gate规则 |
|---|---|---|
| P0 | 数据丢失、错误恢复、密钥泄露、伪成功、格式不可读 | 任意Gate阻塞 |
| P1 | 主要能力不可用、严重兼容/性能/安全问题 | 当前及后续Gate阻塞 |
| P2 | 有可接受绕行的功能缺陷 | 必须有计划和发布决定 |
| P3 | 轻微体验或文档问题 | 可延期但需记录 |

## 16. 变更控制

影响Header、Record Envelope、Chunk、Segment、Catalog、Manifest、Commit、密钥或Diff依赖语义的变更必须有ADR。ADR至少包含：问题、选项、选择、兼容影响、迁移、测试、回滚和文档更新。

## 17. 项目最终交付物

- NWB Format 1.0规范与Registry；
- Reference Writer/Reader；
- Verify/Salvage工具；
- Provider SDK与官方Provider；
- Windows/Linux Recovery Media；
- Golden Corpus；
- 支持矩阵认证账本；
- 自动化测试与Fuzz Corpus；
- 所有测试结果和发布证据；
- 开发、运维、恢复、安全和长期兼容文档。
