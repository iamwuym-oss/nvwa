# Nüwa NWB Verification, Acceptance and Test Plan v1.0

**状态：** APPROVED TEST BASELINE  
**日期：** 2026-07-12  
**初始执行状态：** 所有实际用例均为`NOT_RUN`

---

## 1. 测试原则

1. 预期结果不是实际结果；
2. 备份测试必须对应恢复测试；
3. BMR必须以真实启动作为结果；
4. 损坏测试必须证明不会伪成功；
5. 每个PASS必须有可重放证据；
6. 失败证据不得被重试覆盖；
7. 支持矩阵每个Certified格必须有用例映射；
8. 测试环境、提交、配置和产物必须可追溯。

## 2. 测试层级

| 层级 | 目标 |
|---|---|
| Unit | 字段、算法、状态机和错误分支 |
| Component | Segment、Catalog、Crypto、Provider独立行为 |
| Integration | Full/Diff、分卷、Cache、Verify/Salvage组合 |
| System | 文件、卷、磁盘与Recovery Media |
| BMR | 恢复后真实启动与系统检查 |
| Fault Injection | 断电、截断、ENOSPC、断网、坏块和进程崩溃 |
| Security | Fuzz、路径、密钥、恶意档案、权限 |
| Compatibility | OS/FS/Reader/Provider版本矩阵 |
| Performance | 吞吐、内存、Catalog规模和恢复延迟 |
| Soak | 多轮备份、恢复、验证和Cache重建 |

## 3. 标准测试数据集

### `DATASET-FILE-01`

包含：空文件、1B、4KiB、256KiB边界、超大文件、随机文件、全零文件、稀疏文件、重复内容、10层以上目录、长路径、中文、Emoji、Windows原始UTF-16边界名、Linux非UTF8名、Hard Link、Symbolic Link、Junction/Reparse、ADS、ACL、POSIX ACL、xattr、SELinux标签和设备节点。

### `DATASET-DIFF-01`

从FILE-01产生：原地修改、头/中/尾修改、追加、截断、删除、新增、重命名、跨目录移动、只改权限、只改时间、Hard Link成员变化和稀疏范围变化。

### `DATASET-BLOCK-01`

包含已分配、未分配、全零、随机、跨Chunk边界和可注入不可读范围的虚拟块设备。

### `DATASET-BMR-WIN-01`

Windows UEFI/GPT：ESP、MSR、NTFS系统卷、WinRE、测试用户、启动服务和已知哈希文件。

### `DATASET-BMR-LINUX-01`

Linux UEFI/GPT：ESP、LVM PV/VG/LV、ext4根、GRUB2、fstab、initramfs、启动服务和已知哈希文件。

所有Dataset都有Manifest，记录生成器版本、随机Seed、对象清单、内容哈希和预期语义。

## 4. 容器与格式测试

| ID | 用例 | 预期结果 | Gate |
|---|---|---|---|
| `TST-FMT-001` | Header编码解码 | 固定4096字节，字段与校验一致 | G1 |
| `TST-FMT-002` | 坏Magic/CRC/SHA | Reader明确拒绝 | G1 |
| `TST-FMT-003` | 未知Required Feature | 拒绝并报告最低Reader能力 | G1 |
| `TST-FMT-004` | 未知Optional Feature | 安全跳过并报告 | G1 |
| `TST-FMT-005` | Record长度溢出/越界 | 无panic、无越界分配、拒绝 | G1/G8 |
| `TST-FMT-006` | Segment正常密封 | Header/Footer/哈希一致 | G1 |
| `TST-FMT-007` | Segment各边界截断 | 未密封，不进入正常Catalog | G1/G4 |
| `TST-FMT-008` | Commit A损坏 | 使用B并报告冗余降级 | G1 |
| `TST-FMT-009` | Commit A/B均损坏 | 不正常打开，只允许Salvage | G1/G4 |
| `TST-FMT-010` | 两Commit合法但不一致 | 拒绝并进入诊断 | G1 |
| `TST-FMT-011` | Manifest错误引用 | 拒绝打开受影响Archive | G1 |
| `TST-FMT-012` | 独立Checker互验 | 字段和全部根哈希一致 | G8 |

## 5. Full与Differential测试

| ID | 用例 | 预期结果 | Gate |
|---|---|---|---|
| `TST-FILE-001` | FILE-01 Full | 全对象和数据进入完整Catalog | G2 |
| `TST-FILE-002` | 删除外部Cache后浏览 | 仍可列出和恢复 | G2 |
| `TST-FILE-003` | Full全量恢复 | 内容哈希和支持元数据一致 | G2 |
| `TST-DIFF-001` | DIFF-01创建 | 仅新/变数据入Diff，Catalog为当前完整视图 | G2 |
| `TST-DIFF-002` | Full+Diff恢复 | 与当前源快照逐对象一致 | G2 |
| `TST-DIFF-003` | 删除中间Diff | 其他Diff仍正常恢复 | G2 |
| `TST-DIFF-004` | 错Full同名替换 | ID/根哈希不匹配，拒绝 | G2 |
| `TST-DIFF-005` | 变化日志回绕/丢失 | 全扫描或转Full，不漏数据 | G2/G5 |
| `TST-DIFF-006` | 只改元数据 | 内容可引用Full，元数据为当前值 | G2 |

## 6. 分卷测试

| ID | 用例 | 预期结果 | Gate |
|---|---|---|---|
| `TST-VOL-001` | 64MiB测试阈值多卷 | 正确创建和恢复 | G3 |
| `TST-VOL-002` | Segment接近卷边界 | 提前密封，不跨卷 | G3 |
| `TST-VOL-003` | Catalog Page接近边界 | 完整落在单卷 | G3 |
| `TST-VOL-004` | 分卷重命名和乱序 | 依靠ID自动识别 | G3 |
| `TST-VOL-005` | 缺中间卷 | 列出缺卷及受影响对象 | G3/G4 |
| `TST-VOL-006` | 缺Final Volume | 不视为已提交 | G3 |
| `TST-VOL-007` | 混入其他Set卷 | 哈希链/Set ID拒绝 | G3 |
| `TST-VOL-008` | FAT32真实目标 | AUTO不超过安全上限 | G3 |
| `TST-VOL-009` | 空间不足 | 无伪Commit，临时数据可管理 | G3/G4 |

## 7. 压缩与加密测试

| ID | 用例 | 预期结果 | Gate |
|---|---|---|---|
| `TST-CRY-001` | Zstd和None选择 | 不可压缩数据不膨胀异常 | G3 |
| `TST-CRY-002` | 正确密码 | Catalog与数据正常恢复 | G3 |
| `TST-CRY-003` | 错误密码 | 无明文输出，错误确定 | G3 |
| `TST-CRY-004` | Ciphertext/Tag/AAD位翻转 | 认证失败，数据不释放 | G3/G8 |
| `TST-CRY-005` | Key Slot损坏 | 对应Slot失败，其他合法Slot可用 | G3 |
| `TST-CRY-006` | Nonce唯一性 | 同一DEK下无重复 | G3 |
| `TST-CRY-007` | 日志/崩溃/转储Secret扫描 | 无密码、DEK、KEK和敏感明文 | G3/G9 |
| `TST-CRY-008` | 密码变更 | 输出新归档，原档案字节不变 | G3 |

## 8. Verify与Salvage测试

| ID | 注入 | 预期结果 | Gate |
|---|---|---|---|
| `TST-VER-001` | 正常Archive Quick/Full Verify | PASS，范围和用时记录 | G4 |
| `TST-VER-002` | 单Chunk位翻转 | 定位Volume/Segment/Chunk/对象 | G4 |
| `TST-VER-003` | 多处损坏 | 报告全部可扫描损坏，不早退掩盖 | G4 |
| `TST-SAL-001` | 尾部截断 | 从sealed Segment和Checkpoint列出可救对象 | G4 |
| `TST-SAL-002` | Catalog页损坏 | 正常打开失败；Salvage有明确置信度 | G4 |
| `TST-SAL-003` | 缺卷选择性恢复 | 不引用缺卷的数据可恢复 | G4 |
| `TST-SAL-004` | 输出抢救NWB | 新Archive含Provenance和缺失记录 | G4 |
| `TST-SAL-005` | Salvage空间不足/取消 | 原档案不变，无伪成功 | G4 |

## 9. 故障注入测试

对以下写入点逐一终止进程或返回I/O错误：Header后、Record Header中、Chunk中、Segment Footer前后、Checkpoint中、Final Catalog中、Manifest中、Commit A前后、Commit B中、最终重命名前后、Cache更新中。

| ID | 故障 | 预期结果 |
|---|---|---|
| `TST-FAULT-001` | 进程强杀 | 只接受完整Commit；前置Segment可诊断 |
| `TST-FAULT-002` | ENOSPC | 状态INCOMPLETE，无伪Commit |
| `TST-FAULT-003` | USB拔出 | 可重试或失败，已发布卷不被改写 |
| `TST-FAULT-004` | SMB断开重连 | 从已密封边界安全继续或明确重启 |
| `TST-FAULT-005` | 用户取消 | 释放快照，按策略保留/删除临时数据 |
| `TST-FAULT-006` | 系统重启 | 快照失效时不得跨新快照续写 |
| `TST-FAULT-007` | Cache写失败 | Archive保持COMMITTED，返回专用状态 |
| `TST-FAULT-008` | 陈旧锁/孤儿临时卷 | 安全识别，不误删有效Archive |

## 10. 块、磁盘与坏扇区测试

| ID | 用例 | 预期结果 | Gate |
|---|---|---|---|
| `TST-BLK-001` | BLOCK-01 Full/restore | 逻辑范围哈希一致 | G5 |
| `TST-BLK-002` | 块Diff | 变化范围正确，未变化引用Full | G5 |
| `TST-BLK-003` | GPT主表损坏 | 使用备份/语义记录诊断和重建 | G5 |
| `TST-BLK-004` | GPT主备冲突 | 不静默选择，按策略报告 | G5 |
| `TST-BLK-005` | 目标盘过小 | 写入前阻止 | G5 |
| `TST-BLK-006` | 目标盘更大 | 按布局策略恢复并验证 | G5 |
| `TST-BLK-007` | 512e↔4Kn | 认证组合启动/挂载或明确阻止 | G5/G6 |
| `TST-BLK-008` | 源坏扇区 | UNREADABLE范围和重试记录入档 | G5 |
| `TST-BLK-009` | 恢复目标I/O错误 | 停止或重试，结果不标成功 | G5 |

## 11. Windows测试

### 11.1 文件语义

验证NTFS ACL、Owner、ADS、Hard Link、Sparse、Compression、Reparse、Junction、长路径、原始UTF-16名、EFS状态、时间精度和大小写敏感目录。

### 11.2 VSS

验证单卷/多卷Snapshot Set、Writer成功、Writer失败、超时、降级为Crash Consistent、快照被删除和取消清理。归档声明必须与实际结果一致。

### 11.3 BMR

| ID | 场景 | 通过标准 |
|---|---|---|
| `TST-BMR-W-001` | UEFI/GPT/NTFS Full同机 | Recovery Media恢复后启动登录，验证清单通过 |
| `TST-BMR-W-002` | UEFI/GPT/NTFS Diff同机 | Full+Diff恢复当前状态并启动 |
| `TST-BMR-W-003` | 恢复到更大盘 | 布局符合策略，系统启动 |
| `TST-BMR-W-004` | SATA→NVMe虚拟异机 | 驱动注入后启动 |
| `TST-BMR-W-005` | ESP/BCD重建 | 启动项和Fallback路径有效 |
| `TST-BMR-W-006` | 缺启动必需卷 | BMR预检阻止 |
| `TST-BMR-W-007` | BitLocker认证场景 | 密钥预检、恢复和启动符合矩阵 |
| `TST-BMR-W-008` | Windows 7 SP1 | 备份、Recovery Media恢复和启动 |

启动证据至少包括：虚拟机/硬件标识、恢复日志、磁盘布局、启动画面或串口日志、登录后脚本结果、关键文件哈希和系统版本。

## 12. Linux测试

### 12.1 文件语义

验证原始文件名字节、UID/GID、Mode、ACL、xattr、SELinux、Capabilities、Hard/Symbolic Link、Sparse、设备节点、FIFO和纳秒时间。Socket只验证类型处理，不备份运行内容。

### 12.2 Snapshot

验证LVM Snapshot、Btrfs Snapshot、fsfreeze、快照空间不足、冻结超时、无快照降级和取消释放。

### 12.3 BMR

| ID | 场景 | 通过标准 |
|---|---|---|
| `TST-BMR-L-001` | UEFI/GPT/LVM/ext4 Full | 恢复后启动登录、挂载和服务检查通过 |
| `TST-BMR-L-002` | 同场景Diff | Full+Diff恢复当前状态并启动 |
| `TST-BMR-L-003` | 恢复到更大盘 | PV/VG/LV与FS符合策略 |
| `TST-BMR-L-004` | 控制器变化异机 | initramfs更新后启动 |
| `TST-BMR-L-005` | UUID变化 | fstab/GRUB/initramfs一致 |
| `TST-BMR-L-006` | XFS恢复 | 原卷/更大目标挂载；缩容被阻止 |
| `TST-BMR-L-007` | Btrfs简单子卷 | 子卷语义和启动验证 |
| `TST-BMR-L-008` | 复杂RAID/ZFS | 预检按矩阵阻止误导BMR |

## 13. 安全测试

| ID | 用例 | 通过标准 |
|---|---|---|
| `TST-SEC-001` | `../`、绝对路径、设备路径 | 不能逃出恢复根 |
| `TST-SEC-002` | Symbolic/Reparse链接逃逸 | 链接最后创建，后续写入不逃逸 |
| `TST-SEC-003` | Catalog循环和深度炸弹 | 有界拒绝，无无限循环 |
| `TST-SEC-004` | 超大长度/数量 | checked arithmetic和资源上限生效 |
| `TST-SEC-005` | 压缩炸弹 | 最大明文和比例限制生效 |
| `TST-SEC-006` | Provider恶意/崩溃 | Core不提交、密钥不泄漏 |
| `TST-SEC-007` | Fuzz Record/Page/Manifest | 无Crash/OOM/越界；发现问题有回归样例 |
| `TST-SEC-008` | 目标磁盘误选 | 稳定ID、容量和二次确认阻止 |

## 14. 性能与资源验收

性能结果必须记录硬件，以下是首版基线而非跨硬件绝对承诺：

- 默认流水线内存目标不超过512MiB；低内存模式有界运行；
- 大档案正常打开只读取Header、Commit、Manifest和必要Catalog页，不扫描全部数据；
- 本地顺序不可压缩数据的Core管线吞吐达到“源读取与目标写入较慢者”的70%以上，无法达到需ADR；
- 相同硬件和Dataset相对已批准基线回退超过15%阻塞发布；
- 百万文件Catalog构建、打开、目录枚举和单文件定位有独立基准；
- 10TiB稀疏模拟Archive和多卷索引不得发生32位溢出；
- SMB断线、慢盘和限速下内存不随等待时间无限增长。

## 15. Soak与长期兼容

- 连续执行不少于100轮小型Full/Diff/Verify/Restore循环；
- 执行大型档案长时间备份、暂停、恢复和校验；
- 重复删除和重建Cache；
- 所有NWB 1.0 Golden Files在每次发布CI回归；
- 新Reader读取旧档案；旧Reader对新Required Feature明确拒绝；
- 测试完成后无快照、挂载点、锁和临时文件泄漏。

## 16. Gate验收规则

Gate通过需要：

1. 该Gate全部Mandatory测试PASS；
2. P0/P1缺陷为零；
3. 测试证据可访问且哈希匹配；
4. 需求追溯完整；
5. 实施文档与实际行为一致；
6. 测试和架构负责人签署；
7. 失败和豁免有正式记录。

不允许用“后续再测试”通过Gate。P2豁免必须说明用户影响、绕行、修复计划和为什么不影响数据可恢复性。

