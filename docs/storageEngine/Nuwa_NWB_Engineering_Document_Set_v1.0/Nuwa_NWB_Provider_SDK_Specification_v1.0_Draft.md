# Nüwa NWB Provider SDK Specification v1.0 Draft

**状态：** DRAFT  
**日期：** 2026-07-12  
**规范标识：** `NWB-PROVIDER-1`

---

## 1. 目的与边界

Provider负责理解操作系统、文件系统、磁盘拓扑、启动环境及未来应用数据；NWB Core负责可靠存储。该边界确保新增数据类型不改变NWB物理格式。

核心规则：

| ID | 规则 |
|---|---|
| `PRV-001` | Provider不得直接创建、定位或修改NWB物理记录。 |
| `PRV-002` | Provider只向Core提交对象、元数据、数据流、范围、依赖和状态。 |
| `PRV-003` | Core独占Chunk、Segment、压缩、加密、Catalog、Commit和分卷。 |
| `PRV-004` | Provider不得获取Archive DEK、用户密码或恢复密钥。 |
| `PRV-005` | Provider崩溃不得使未提交归档成为成功状态。 |
| `PRV-006` | Provider私有Metadata必须有公共Envelope和Schema版本。 |
| `PRV-007` | 首版只加载Nüwa签名Provider；第三方SDK以后开放。 |
| `PRV-008` | 归档不得嵌入或执行Provider代码。 |

## 2. Provider能力模型

一个Provider可以实现一个或多个能力：

- `DISCOVERY`：发现机器、磁盘、卷、文件系统和拓扑；
- `SNAPSHOT`：建立一致性时间点；
- `FILE_CAPTURE`：枚举和读取文件对象；
- `BLOCK_CAPTURE`：读取卷或磁盘逻辑范围；
- `METADATA_CAPTURE`：捕获平台专用语义；
- `FILE_RESTORE`：恢复文件对象及语义；
- `BLOCK_RESTORE`：恢复卷/磁盘范围；
- `BOOT_CAPTURE`：捕获启动环境；
- `BOOT_RESTORE`：重建启动环境；
- `VERIFY`：验证Provider语义；
- `CHANGE_HINT`：提供非权威变化提示；
- `APPLICATION_CONSISTENCY`：未来数据库/应用一致性能力。

能力是运行时协商结果，不得只根据Provider名称推断。

## 3. Provider描述符

每个Provider暴露不可变描述符：

```text
provider_id             稳定反向域名式标识
provider_name           显示名称
provider_version        实现版本
schema_major/minor      私有元数据Schema
protocol_major/minor    Provider协议版本
supported_os            操作系统范围
supported_arch          CPU架构
capabilities            能力集合
minimum_core_version    最低Core版本
minimum_reader_version  最低恢复Reader版本
publisher               发布者
binary_digest           Provider二进制SHA-256
signature               发布签名
```

示例Provider ID：

- `com.nuwa.windows.discovery`
- `com.nuwa.windows.vss`
- `com.nuwa.filesystem.ntfs`
- `com.nuwa.boot.windows-uefi`
- `com.nuwa.linux.discovery`
- `com.nuwa.filesystem.ext`
- `com.nuwa.linux.lvm`
- `com.nuwa.boot.linux-grub2`
- `com.nuwa.raw-block`

Provider ID发布后不得改名或复用。

## 4. 公共对象模型

Core定义以下公共对象类型：

```text
Machine
Firmware
PhysicalDisk
DiskGroup
PartitionTable
Partition
Volume
FileSystem
Directory
File
DataStream
DataExtent
SnapshotSet
BootEnvironment
BootComponent
DriverPackage
ApplicationObject
ProviderMetadata
UnreadableRange
ConsistencyResult
```

每个对象包含：

- 128位`object_id`；
- 可选`parent_id`；
- `object_type`；
- `provider_id`；
- `schema_version`；
- 稳定来源标识；
- 属性集合；
- 数据流和Extent；
- 依赖关系；
- 错误和警告；
- 捕获状态。

Provider生成对象；Core验证ID唯一性、父子无环、长度合法和引用存在后才进入Catalog。

## 5. Provider Metadata Envelope

所有私有元数据使用：

```text
provider_id
schema_major
schema_minor
metadata_type
flags: REQUIRED_FOR_RESTORE | OPTIONAL | SENSITIVE
payload_length
payload_sha256
payload
```

规则：

- 未知`OPTIONAL`元数据可以跳过；
- 未知`REQUIRED_FOR_RESTORE`元数据必须阻止相关恢复操作；
- `SENSITIVE`元数据必须进入加密Catalog；
- Payload不得包含明文密码、临时Token、Archive Key或无生命周期凭据；
- Schema Major改变表示不兼容；Minor只能增加可选字段；
- Provider必须继续读取自身已发布的旧Major，或提供迁移Reader。

## 6. 通信与实现形态

公共SDK不得暴露Rust Trait、C++对象布局或编译器ABI作为长期协议。建议：

- Core内部可以使用Rust Trait；
- Core与隔离Provider Host之间使用版本化消息协议；
- 大数据通过只读句柄、共享内存或受控流传递；
- 控制消息使用长度前缀和明确Schema；
- 每个调用携带`job_id`、`operation_id`、deadline和cancellation token；
- Provider Host崩溃由任务引擎转换为结构化错误。

首版官方Provider允许与受信任服务共同发布，但接口测试必须按进程可隔离方式设计，为未来第三方Provider留出边界。

## 7. 生命周期

### 7.1 捕获生命周期

```text
Probe
→ Prepare
→ CreateSnapshot
→ Enumerate
→ CaptureMetadata/Data
→ FinalizeCapture
→ ReleaseSnapshot
```

无论成功、失败、取消或超时，都必须执行`ReleaseSnapshot`和资源清理。

### 7.2 恢复生命周期

```text
ProbeTarget
→ PlanRestore
→ ValidateDependencies
→ ConfirmDestructiveActions
→ PrepareTarget
→ RestoreData
→ RestoreMetadata
→ RestoreBoot
→ VerifyTarget
→ FinalizeRestore
```

擦除磁盘、创建分区和覆盖系统卷前，Core必须完成依赖预检并取得明确授权。Provider不得绕过此阶段。

## 8. Discovery接口

必要操作：

- `probe_host()`；
- `enumerate_disks()`；
- `enumerate_partitions()`；
- `enumerate_volumes()`；
- `detect_filesystems()`；
- `detect_firmware()`；
- `detect_boot_environment()`；
- `detect_storage_topology()`；
- `evaluate_support_level()`。

返回结果必须区分：

- `SUPPORTED`；
- `SUPPORTED_WITH_LIMITATIONS`；
- `DATA_ONLY`；
- `DETECTED_UNSUPPORTED`；
- `UNKNOWN`。

`UNKNOWN`不得自动升级为支持。

## 9. Snapshot与一致性接口

必要操作：

- `prepare_snapshot_set(sources)`；
- `create_snapshot_set()`；
- `query_consistency()`；
- `open_snapshot_source()`；
- `release_snapshot_set()`。

统一一致性级别：

- `APPLICATION_CONSISTENT`；
- `FILESYSTEM_CONSISTENT`；
- `CRASH_CONSISTENT`；
- `INCONSISTENT`。

Windows VSS Provider保存Snapshot Set ID、卷、Writer状态、错误码、Freeze/Thaw结果和降级原因。Linux保存LVM/Btrfs/fsfreeze策略、涉及挂载点、冻结窗口和降级原因。

多个启动必需卷必须属于同一Machine Snapshot Set，不能用独立时间点拼成BMR并静默标记为一致。

## 10. 数据读取接口

### 10.1 文件捕获

Provider以稳定顺序枚举目录和对象，提交：名称原始表示、类型、稳定ID、父ID、数据流、元数据和错误。

Windows名称保留UTF-16LE；Linux名称保留原始字节。Provider不得在枚举阶段自行拼接不受控目标路径。

### 10.2 块捕获

Provider提交：

- 逻辑设备大小和Sector尺寸；
- 已分配/未分配范围；
- 数据范围流；
- ZERO、HOLE、UNALLOCATED和UNREADABLE；
- 文件系统与分区元数据；
- Source Read错误和重试结果。

Core决定Chunk边界和物理布局。

### 10.3 Backpressure

Core通过有界队列控制内存。Provider必须支持暂停读取、取消和deadline，不得无限缓存整个文件或磁盘。

## 11. Differential变化提示

Provider可以实现：

- `validate_change_source(base, current)`；
- `get_change_hints()`；
- `enumerate_current_objects()`；
- `read_changed_ranges()`。

USN Journal、时间戳、LVM/Btrfs信息和未来CBT只能是Hint。出现Journal回绕、ID变化、记录缺失、卷重建或来源不可信时，必须返回`CHANGE_TRACKING_UNRELIABLE`。Core随后全量扫描比对或转Full。

任何Provider不得声称“未变化”却无法提供可审计依据。

## 12. 恢复接口与安全

文件恢复顺序：

1. 创建受控目录；
2. 写临时文件数据；
3. 验证内容；
4. 原子替换或按冲突策略处理；
5. 重建Hard Link；
6. 恢复ACL、所有者、扩展属性和时间；
7. 最后创建Symbolic Link/Reparse Point。

Core先规范化并验证恢复根；Provider仍必须拒绝绝对路径、`..`越界、链接逃逸和设备路径注入。

块恢复接口必须暴露目标稳定ID、容量、Logical/Physical Sector、可擦除状态和系统盘风险。目标选择不得只使用“Disk 0”序号。

## 13. Boot Provider

### 13.1 Windows

必须能够捕获和恢复：GPT/MBR、ESP、BCD、WinRE、Boot Manager、Secure Boot状态、BitLocker/TPM依赖、Boot-critical Driver和目标硬件存储驱动。

### 13.2 Linux

必须能够捕获和恢复：ESP、GRUB2或systemd-boot、`/etc/fstab`、文件系统UUID、LVM配置、initramfs、内核、存储驱动和必要网络配置。

Boot Provider必须提供`plan_boot_repair()`、`restore_boot()`和`verify_bootability()`。验证结果进入Catalog和测试证据。

## 14. 错误模型

统一类别：

- `RETRYABLE`
- `DEGRADED`
- `SOURCE_UNREADABLE`
- `UNSUPPORTED`
- `CANCELLED`
- `TIMEOUT`
- `FATAL`
- `PROVIDER_BUG`

错误对象包含Provider、阶段、对象、标准错误码、OS原始错误、是否影响可恢复性、已重试次数和建议动作。

Provider不得通过日志文本传递唯一错误语义。

## 15. 首版官方Provider

### 公共

- Raw Block
- Generic File
- GPT/MBR
- Verify

### Windows

- Discovery
- VSS
- NTFS
- ReFS
- BitLocker/EFS State
- Windows UEFI/BCD/WinRE
- Driver Inventory/Injection
- Windows BMR

### Linux

- Discovery
- ext2/3/4
- XFS
- Btrfs
- LVM2
- Linux UEFI/GRUB2/systemd-boot
- initramfs
- Linux BMR

## 16. Conformance Suite

每个Provider必须通过：

1. 描述符与签名验证；
2. Capability声明和实际行为一致；
3. 生命周期任意阶段取消均释放资源；
4. 非法对象、重复ID、循环父子引用被Core拒绝；
5. Backpressure下不死锁、不无限内存；
6. 超时和崩溃转换为确定错误；
7. 私有Metadata新旧Schema兼容；
8. 必需Provider缺失时恢复预检准确阻止；
9. Provider无法接触Archive Key；
10. Provider无法绕过Core写NWB；
11. 恶意路径和链接被拒绝；
12. 变化提示失效时安全回退。

Provider只有在Conformance Suite、平台功能测试和恢复测试全部通过后才能进入支持矩阵的“正式支持”。

## 17. SDK交付物

- 协议Schema与生成代码；
- Provider Descriptor Schema；
- 公共对象Schema；
- 错误码注册表；
- Reference Provider；
- Mock Core和Mock Provider Host；
- Conformance Test Runner；
- 示例Metadata升级；
- 签名与打包工具；
- Provider开发、调试、发布和兼容指南。

## 18. 冻结条件

Provider SDK 1.0只有在Windows NTFS/VSS和Linux ext4/LVM两条完整捕获恢复链、异常取消、Schema兼容和隔离故障测试通过后冻结。外部第三方Provider开放不阻塞单机首版，但协议不得妨碍未来进程隔离和签名验证。

