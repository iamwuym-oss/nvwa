# Nüwa Backup Single-Node Product Support Matrix v1.0

**状态：** RELEASE BASELINE  
**日期：** 2026-07-12  
**适用范围：** 单机版首个正式版本

---

## 1. 支持等级

| 等级 | 对外含义 |
|---|---|
| `CERTIFIED` | 通过备份、恢复、故障和适用的真实启动验证；可以正式承诺 |
| `LIMITED` | 可以执行明确列出的操作，但存在记录的限制 |
| `DATA_ONLY` | 可恢复文件或逻辑数据，不承诺原拓扑和BMR |
| `RAW_IMAGE` | 只能按原始扇区保存和恢复，不提供文件语义 |
| `DETECTED_UNSUPPORTED` | 可以识别，但必须阻止危险或误导性操作 |
| `FUTURE` | 已预留格式/Provider接口，首版不实现 |

任何场景只有在测试记录为PASS并完成发布签署后才能从计划等级升级为`CERTIFIED`。

## 2. 产品范围

首版包含：Full、Differential、单文件/分卷NWB、本地磁盘、USB、SMB/NAS、文件/卷/磁盘备份恢复、Recovery Media、BMR、认证范围内的异机恢复、Verify和Salvage。

首版不包含：Incremental、Repository、集中管理、云对象存储、磁带、数据库日志/PITR、VMware/Hyper-V无代理备份、P2V/V2P、跨Archive全局去重。

## 3. CPU架构

| 平台 | 架构 | 首版等级 |
|---|---|---|
| Windows | x86-64 | `CERTIFIED`目标 |
| Windows 7 Legacy | x86 BIOS | `LIMITED`目标，文件/卷/磁盘与同架构BMR |
| Linux | x86-64 | `CERTIFIED`目标 |
| Windows/Linux | ARM64 | `FUTURE` |
| POWER/RISC-V | 各架构 | `FUTURE` |

NWB格式与CPU无关；新增架构不修改Format 1.0。

## 4. Windows版本

| 系统 | 文件 | 卷/磁盘 | BMR | 异机恢复 |
|---|---|---|---|---|
| Windows 7 SP1 | Certified目标 | Certified目标 | Certified目标 | Certified目标 |
| Windows 8/8.1 | Certified目标 | Certified目标 | Certified目标 | Certified目标 |
| Windows 10 | Certified目标 | Certified目标 | Certified目标 | Certified目标 |
| Windows 11 | Certified目标 | Certified目标 | Certified目标 | Certified目标 |
| Server 2008 R2 | Certified目标 | Certified目标 | Certified目标 | Limited目标 |
| Server 2012/2012 R2 | Certified目标 | Certified目标 | Certified目标 | Limited目标 |
| Server 2016/2019/2022/2025 | Certified目标 | Certified目标 | Certified目标 | Limited目标 |
| 未来Windows | Provider升级后认证 | Provider升级后认证 | 验证后开放 | 验证后开放 |

服务器首版不承诺AD、SQL Server、Exchange、SharePoint、Oracle专用应用一致性与时间点恢复。VSS基础一致性结果必须如实记录。

## 5. Windows文件系统

| 文件系统 | 文件恢复 | 卷恢复 | 浏览 | 缩容 | 备注 |
|---|---|---|---|---|---|
| NTFS | `CERTIFIED` | `CERTIFIED` | `CERTIFIED` | 条件支持 | ACL、ADS、Hard Link、Sparse、Reparse、EFS状态 |
| FAT/FAT32 | `CERTIFIED` | `CERTIFIED` | `CERTIFIED` | 条件支持 | 包括ESP |
| exFAT | `CERTIFIED`目标 | `CERTIFIED`目标 | `CERTIFIED`目标 | 条件支持 | 常见移动介质 |
| ReFS | `CERTIFIED`文件；原卷恢复 | `LIMITED` | Provider能力决定 | 不支持 | 不承诺缩容和跨版本转换 |
| 未知文件系统 | 无 | `RAW_IMAGE` | 无 | 无 | 原始扇区模式 |

## 6. Windows存储与启动

| 技术 | 首版等级 | 行为 |
|---|---|---|
| MBR基本磁盘 | `CERTIFIED` | BIOS BMR |
| GPT基本磁盘 | `CERTIFIED` | UEFI BMR |
| 512n/512e | `CERTIFIED` | 同类型及认证转换 |
| 4Kn | `LIMITED`至测试通过 | 必须验证分区对齐与启动 |
| BitLocker解锁卷 | `CERTIFIED`目标 | 文件/已分配块备份，可浏览 |
| BitLocker锁定卷 | `RAW_IMAGE`/`LIMITED` | 需要原恢复密钥 |
| EFS | `LIMITED` | 保存状态；可用性依赖证书私钥 |
| 动态磁盘LDM | `DATA_ONLY` | 保存识别信息，不承诺复杂拓扑BMR |
| Storage Spaces | `DATA_ONLY` | 备份可见文件/逻辑卷，不重建Pool |
| Windows软件镜像 | `LIMITED` | 按测试开放拓扑恢复 |
| 硬件RAID | `LIMITED` | 备份OS可见逻辑盘；异机依赖驱动/控制器 |
| iSCSI/SAN | `DATA_ONLY` | 启动依赖场景不承诺首版BMR |
| USB磁盘 | `CERTIFIED` | 源和目标均支持 |

## 7. Linux发行版

首版x86-64认证目标：

- Ubuntu 20.04/22.04/24.04/26.04 LTS；
- Debian 11/12/13；
- RHEL、Rocky Linux、AlmaLinux 8/9/10；
- CentOS 7 Legacy；
- SUSE Linux Enterprise 15/16；
- 对应主流openSUSE Leap。

Fedora、Arch、Gentoo等滚动或快速版本提供兼容支持和Raw恢复，不作为每个小版本的发布阻塞矩阵。实际支持以Provider探测和发布测试记录为准。

## 8. Linux文件系统

| 文件系统 | 首版等级 | 备注 |
|---|---|---|
| ext2/ext3/ext4 | `CERTIFIED` | 文件、卷、BMR |
| XFS | `CERTIFIED`目标 | 不支持缩小 |
| Btrfs单设备/简单子卷 | `CERTIFIED`目标 | 子卷和快照语义需测试 |
| Btrfs复杂多设备 | `DATA_ONLY` | 不承诺原拓扑重建 |
| FAT/FAT32 | `CERTIFIED` | 包括ESP |
| exFAT | `CERTIFIED`目标 | 文件和卷 |
| NTFS | `LIMITED` | 文件和原卷恢复 |
| Swap | 布局支持 | 默认不保存内容 |
| ZFS | `DATA_ONLY`/`RAW_IMAGE` | 不重建Pool |
| F2FS | `LIMITED`/`RAW_IMAGE` | 按Provider能力 |
| 未知文件系统 | `RAW_IMAGE` | 不浏览、不缩容 |

Linux文件语义包括UID/GID、Mode、POSIX ACL、xattr、SELinux标签、Hard/Symbolic Link、Sparse、Device Node、FIFO、Capabilities和纳秒时间。`/proc`、`/sys`、`/dev`运行态内容默认排除。

## 9. Linux存储与启动

| 技术 | 首版等级 |
|---|---|
| MBR/GPT | `CERTIFIED` |
| BIOS/UEFI | `CERTIFIED` |
| GRUB2 | `CERTIFIED` |
| systemd-boot | `CERTIFIED`目标 |
| LVM2 | `CERTIFIED` |
| LUKS解锁卷 | `CERTIFIED`目标 |
| LUKS锁定卷 | `RAW_IMAGE`/`LIMITED` |
| mdraid RAID1 | `CERTIFIED`目标 |
| mdraid RAID0/5/6/10 | `DATA_ONLY`至拓扑测试通过 |
| Device Mapper | `LIMITED` |
| Multipath/SAN | `DATA_ONLY` |
| ZFS Pool | `DETECTED_UNSUPPORTED`用于BMR |

## 10. 恢复能力

### 10.1 首版正式目标

- 文件/目录恢复到原位置或新位置；
- Full恢复和Full+Diff恢复；
- 卷和整盘恢复；
- 更大目标磁盘；
- 文件系统允许且已验证时恢复到较小磁盘；
- Windows基本磁盘和Linux普通分区/LVM的同机BMR；
- Windows基本磁盘和Linux认证拓扑的异机恢复；
- BIOS→BIOS、UEFI→UEFI；
- 从本地、USB、SMB/NAS读取NWB；
- 缺卷和损坏时选择性Salvage。

### 10.2 有限或未来

- BIOS与UEFI相互转换：`LIMITED`；
- 动态磁盘到基本磁盘转换：`LIMITED`；
- Storage Spaces、复杂RAID、ZFS Pool重建：首版不承诺；
- XFS和ReFS缩容：不支持；
- P2V/V2P、跨操作系统BMR、跨CPU BMR：`FUTURE`。

## 11. 备份目标

| 目标 | 首版等级 |
|---|---|
| 本地内部磁盘 | `CERTIFIED` |
| USB/移动磁盘 | `CERTIFIED` |
| NTFS/exFAT/ReFS目标 | `CERTIFIED`目标 |
| ext4/XFS目标 | `CERTIFIED`目标 |
| SMB/CIFS | `CERTIFIED`目标，含断线测试 |
| NFS | Linux `CERTIFIED`目标 |
| FAT32 | `CERTIFIED`，强制自动分卷 |
| S3/云盘/磁带 | `FUTURE` |

目标位于源目录或源卷时必须自动排除目标NWB及临时文件，无法安全排除时阻止任务。

## 12. BMR资格预检

只有以下全部满足时显示“BMR Ready”：

1. 所有启动必需磁盘和卷已捕获；
2. Snapshot Set时间点一致；
3. 分区表、ESP/启动区和系统卷完整；
4. BCD/GRUB、WinRE/initramfs和启动依赖已捕获；
5. 加密卷存在可用解锁路径；
6. 所需Provider和Recovery Media Feature存在；
7. 所有必要分卷存在；
8. Verify无关键错误；
9. 目标容量、Sector和拓扑兼容；
10. 场景处于本矩阵认证范围。

否则只能显示“Data Restore Ready”或明确的不支持原因。

## 13. 升级规则

支持等级变更必须有对应测试结果ID、证据和发布批准。发现严重恢复缺陷时可以把能力从`CERTIFIED`降级，并在产品启动、任务创建和恢复预检中同步体现，不能只修改市场文档。

