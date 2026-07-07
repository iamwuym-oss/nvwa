> **⚠️ 架构演进声明（v2.2 修订）**
>
> **重要：本文档主体描述的是 Nüwa Backup 的早期架构设计（egui + ZeroMQ + daemon）。该架构已被取代。**
>
> ## 当前架构（2026-07）
>
> | 层次 | 技术 |
> |------|------|
> | Desktop Runtime | **Tauri 2.0**（替换 egui） |
> | Frontend | **React 19 + TypeScript + Vite 6** |
> | IPC | **Tauri invoke()**（替换 ZeroMQ） |
> | Command Layer | Rust thin wrapper（src-tauri/src/commands/） |
> | Application Layer | **src/app/**（models + services + error） |
> | Core Engine | Rust（src/）— backup, restore, verify, storage |
>
> ## 当前架构图详见「附录 D：Current Desktop GUI Architecture (Tauri 2.0)」
>
> ---
>
> ## 主体文档状态：HISTORICAL / SUPERSEDED
>
> 本文档第 1~8 章及附录 B/C 描述的是**已被替代的早期架构设计**（egui + eframe UI、ZeroMQ IPC、nuwa-daemon/nuwa-agent 守护进程架构）。
>
> 保留这些内容仅用于技术决策追溯，**不得作为当前开发依据**。
>
> 当前开发请参考：
> - docs/phase-2.5/Phase_2_5_Closing_Report.md — Phase 2.5 完成报告
> - docs/phase-2.5/Phase_2_5_Tauri_Migration_Decision.md — Tauri 迁移决策
> - AGENTS.md — 当前开发执行规则
> - 本文档「附录 D」— 当前桌面 GUI 架构
>
# Nüwa Backup 完整架构设计

**Nüwa Backup — Local Container Backup System**

| Field | Value |
|---|---|
| **文档标题** | Nüwa Backup 完整架构设计 |
| **版本** | v2.0 |
| **状态** | Draft |
| **产品名称** | Nüwa Backup (Local Container Backup) |
| **一级目标** | Windows Workstation (7 SP1~11) + Server (2008 R2~2025) — 文件级、卷级、磁盘级备份与恢复 |
| **二级目标** | Linux 及国产操作系统（麒麟 V10、统信 UOS）扩展 |
| **日期** | 2026-07-04 |
| **作者** | Codex |

---

## 0. 设计哲学

### 0.1 核心原则（不妥协的底线）

| 原则 | 说明 |
|------|------|
| **恢复优先于备份** | 每个备份必须有一个被验证过的恢复路径。无法恢复的备份=没有备份 |
| **数据完整性高于速度** | 校验和、写后验证、格式容错，优先于吞吐量 |
| **最小内核依赖** | 不写内核驱动。Windows 依赖 VSS/volsnap.sys，Linux 依赖 fsfreeze/LVM |
| **跨平台核心** | 核心引擎（镜像格式、压缩、加密、校验）零平台依赖，OS 交互通过适配层隔离 |
| **破坏性操作安全** | 分区恢复、磁盘克隆前必须多重确认，必须有回滚能力 |
| **向前兼容** | 镜像格式 v1.0 生成的备份，v3.0 必须能恢复 |

### 0.2 关键决策记录

| 决策 ID | 决策 | 理由 |
|---------|------|------|
| DEC-001 | 核心引擎用 **Rust** | 跨平台编译（x86_64/ARM64/LoongArch）、无运行时依赖、内存安全、C ABI 导出 |
| DEC-002 | 不写内核驱动 | VSS+volsnap.sys 已提供一致性快照，无需重复造轮子。零蓝屏风险 |
| DEC-003 | 镜像格式自研 (`.nwb`) | 需要完全控制格式演进、容错设计、增量链完整性 |
| ~~DEC-004~~ | ~~UI 用 egui + eframe（纯 Rust）~~ | **🔄 SUPERSEDED** — Replaced by Tauri 2.0 + React (see Appendix D). Phase 2.5 (2026-07) |
| DEC-005 | 服务层用 Rust + Tokio | 高性能异步 I/O，统一的后端技术栈 |
| DEC-006 | 元数据用 SQLite | 零配置、跨平台、嵌入式中性能足够、所有平台原生支持 |

---

## 1. 全景架构

### 1.1 四层架构

```
┌──────────────────────────────────────────────────────────────────┐
│              UI LAYER (egui — HISTORICAL, see Appendix D)           │
│  Desktop App · System Tray · Recovery Wizard · CLI               │
│  ┌────────────────────┐  ┌──────────────────┐                    │
│  │  Windows Desktop   │  │  Linux/Kylin/UOS │                    │
│  │  (egui)            │  │  (egui)            │                    │
│  └────────┬───────────┘  └────────┬─────────┘                    │
│           │                        │                              │
│           └─────────┬──────────────┘                              │
│                     │ (ZeroMQ IPC)                                │
├─────────────────────┴────────────────────────────────────────────┤
│                      SERVICE LAYER                                │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  Core Daemon (nuwa-daemon)                                  │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────────────────┐  │  │
│  │  │ Task Mgr   │ │ Scheduler  │ │ Job Executor           │  │  │
│  │  └────────────┘ └────────────┘ └────────────────────────┘  │  │
│  │  ┌────────────┐ ┌────────────┐ ┌────────────────────────┐  │  │
│  │  │ Image Mgr  │ │ Catalog Mgr│ │ Verification Mgr       │  │  │
│  │  └────────────┘ └────────────┘ └────────────────────────┘  │  │
│  └────────────────────────────────────────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  Agent (nuwa-agent) — 系统托盘 + 状态监控                   │  │
│  └────────────────────────────────────────────────────────────┘  │
├─────────────────────┬────────────────────────────────────────────┤
│                 CORE ENGINE LAYER (Rust)                         │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  libnuwa_core.so / nuwa_core.dll                             │  │
│  │  ┌───────────────┐ ┌──────────────┐ ┌──────────────────┐  │  │
│  │  │ Image Format  │ │ Compression  │ │ Encryption       │  │  │
│  │  │ (.nwb r/w)    │ │ (zstd)       │ │ (AES-256-GCM)    │  │  │
│  │  └───────────────┘ └──────────────┘ └──────────────────┘  │  │
│  │  ┌───────────────┐ ┌──────────────┐ ┌──────────────────┐  │  │
│  │  │ Block Map Mgr │ │ Checksum     │ │ Catalog Engine   │  │  │
│  │  │               │ │ (SHA-256)    │ │                  │  │  │
│  │  └───────────────┘ └──────────────┘ └──────────────────┘  │  │
│  └────────────────────────────────────────────────────────────┘  │
├─────────────────────┬────────────────────────────────────────────┤
│              OS ABSTRACTION LAYER (Rust Traits)                  │
│  ┌────────────────────────┐  ┌────────────────────────────────┐  │
│  │  Windows Adapter       │  │  Linux Adapter                 │  │
│  │  ────────────────      │  │  ──────────────                │  │
│  │  VolumeSnapshot (VSS)  │  │  VolumeSnapshot (fsfreeze/LVM)  │  │
│  │  BlockDevice (\\.\X:)  │  │  BlockDevice (/dev/sdX)         │  │
│  │  FsParser (NTFS)       │  │  FsParser (ext4/XFS)          │  │
│  │  PartitionTable(GPT/MBR)│  │  PartitionTable(GPT/MBR)      │  │
│  │  BootConfig (BCD API)  │  │  BootConfig (GRUB)            │  │
│  │  DiskDiscovery (IOCTL) │  │  DiskDiscovery (/sys/block)   │  │
│  └────────────────────────┘  └────────────────────────────────┘  │
├─────────────────────┴────────────────────────────────────────────┤
│                    STORAGE LAYER                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌────────────────────────┐  │
│  │  Local Disk  │  │  USB/E-SATA  │  │  NAS (SMB/NFS)        │  │
│  └──────────────┘  └──────────────┘  └────────────────────────┘  │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  SQLite (元数据: 任务、日志、目录、配置)                       │  │
│  └────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────┘
```

### 1.2 进程模型

```
┌────────────────────────────────────────────────────────────┐
│  Windows 进程模型                                           │
│                                                            │
│  nuwa-daemon.exe    ← Windows Service (SYSTEM)              │
│  │  核心守护进程：执行备份/恢复/克隆/调度                     │
│  │  以 SYSTEM 权限运行（需要 Volume Shadow Copy 访问权限）     │
│  │                                                            │
│  nuwa-agent.exe     ← 用户态进程（用户登录时启动）               │
│  │  系统托盘图标 + 状态监控 + 通知                             │
│  │  通过 IPC (ZeroMQ) 与 daemon 通信                          │
│  │                                                            │
│  nuwa-ui.exe        ← 用户态进程（用户双击启动）                 │
│  │  主界面（egui）                                         │
│  │  通过 IPC (ZeroMQ) 与 daemon 通信                          │
│  │                                                            │
│  nuwa-cli.exe       ← 命令行工具                               │
│  │  通过 IPC (ZeroMQ) 与 daemon 通信                          │
│                                                                 │
├────────────────────────────────────────────────────────────┤
│  Linux/国产 OS 进程模型                                     │
│                                                            │
│  nuwa-daemon        ← systemd 服务 (root)                   │
│  nuwa-agent         ← 用户 daemon 或 systemd --user         │
│  nuwa-ui            ← 桌面应用（egui）                     │
│  nuwa-cli           ← 命令行工具                              │
└────────────────────────────────────────────────────────────┘
```

---

## 2. 核心引擎设计 (libnuwa_core)

### 2.1 核心接口（Rust Traits）

```rust
// ============================================================
// 镜像格式引擎 — 核心中的核心
// ============================================================
pub trait ImageEngine: Send + Sync {
    /// 创建新镜像
    fn create_writer(&self, path: &Path, opts: ImageOptions)
        -> Result<Box<dyn ImageWriter>>;

    /// 打开现有镜像
    fn open_reader(&self, path: &Path)
        -> Result<Box<dyn ImageReader>>;

    /// 验证镜像完整性
    fn verify(&self, path: &Path) -> Result<VerificationReport>;
}

// ============================================================
// 镜像写入器
// ============================================================
pub trait ImageWriter: Send {
    /// 写入数据块
    /// offset: 在原始卷中的字节偏移
    /// data: 块数据
    fn write_block(&mut self, offset: u64, data: &[u8])
        -> Result<BlockWriteResult>;

    /// 写入空洞（跳过未分配区域）
    fn write_hole(&mut self, offset: u64, length: u64) -> Result<()>;

    /// 完成镜像写入
    fn finalize(&mut self) -> Result<ImageSummary>;
}

// ============================================================
// 镜像读取器
// ============================================================
pub trait ImageReader: Send {
    /// 读取指定偏移处的数据块
    fn read_block(&self, offset: u64) -> Result<Option<Vec<u8>>>;

    /// 获取块索引迭代器（用于恢复）
    fn block_iter(&self) -> Result<Box<dyn BlockIterator>>;

    /// 读取镜像元数据
    fn metadata(&self) -> Result<ImageMetadata>;
}

// ============================================================
// 文件系统解析器（按平台实现）
// ============================================================
pub trait FsParser: Send + Sync {
    /// 探测文件系统
    fn detect(device: &BlockDevice) -> Result<FsType>;

    /// 枚举已分配块
    fn enumerate_allocated_blocks(&self, device: &BlockDevice)
        -> Result<Vec<BlockRange>>;

    /// 将块范围转换为文件列表（用于文件级恢复）
    fn block_to_file(&self, device: &BlockDevice, block: u64)
        -> Result<Option<String>>;
}

// ============================================================
// 卷快照接口（按平台实现）
// ============================================================
pub trait VolumeSnapshot: Send + Sync {
    /// 创建卷快照
    fn create(volume: &VolumeInfo) -> Result<Self>
        where Self: Sized;

    /// 获取快照设备路径
    fn snapshot_path(&self) -> &Path;

    /// 释放快照
    fn release(self) -> Result<()>;
}
```

### 2.2 技术栈

| 组件 | 选型 | 理由 |
|------|------|------|
| 语言 | **Rust 2024 edition** | 跨平台编译、零成本抽象、内存安全、C ABI |
| 压缩 | **zstd (zstd-rs)** | 高压缩比+高吞吐量，工业标准 |
| 加密 | **AES-256-GCM (RustCrypto)** | 认证加密，同时提供保密性+完整性 |
| 哈希 | **SHA-256 / xxHash3** | SHA-256 用于校验，xxHash3 用于快速去重 |
| 异步运行时 | **Tokio** | 高性能异步 I/O，稳定生态 |
| 序列化 | **Serde + MessagePack** | 紧凑二进制序列化，适应元数据交换 |
| 零MQ | **zmq (ZeroMQ)** | 进程间通信，轻量可靠 |
| 数据库 | **SQLite (rusqlite)** | 零配置、跨平台、嵌入式中性能足够 |
| 镜像挂载 | **FUSE (Windows: WinFsp)** | 将 .nwb 镜像挂载为虚拟驱动器 |

---

## 3. Nüwa Backup 镜像格式设计

### 3.1 文件扩展名

`.nwb` — Local Container Backup

### 3.2 格式布局

```
┌──────────────────────────────────────────────────────────────┐
│  Nüwa Backup Image File Layout                                       │
│                                                              │
│  [Magic Header]           · Magic: "Nüwa Backup1" (4 bytes)          │
│  [File Header]            · Version, Timestamp, Image Type   │
│  [Volume Metadata]        · 卷信息(大小、类型、文件系统、GUID)   │
│  [Partition Table Copy]   · 原始分区表快照                      │
│  [Block Index]            · 已使用块索引表（有序 B-Tree）        │
│  [Block Data Stream]      · 块数据（zstd 压缩流）               │
│  [Block Map Footer]       · 块索引的位置和校验                  │
│  [File Trailer]           · 文件签名、全部校验和                 │
└──────────────────────────────────────────────────────────────┘
```

### 3.3 数据类型定义

```rust
/// 镜像文件头部（固定 64 字节）
#[repr(C, packed)]
struct NuwaHeader {
    magic:          [u8; 4],        // b"Nüwa Backup1"
    version_major:  u16,            // 主版本号
    version_minor:  u16,            // 次版本号
    image_type:     ImageType,      // Full / Incremental / Differential
    compression:    CompressionType, // None / Zstd(level)
    encryption:     EncryptionInfo,  // None / AES-256-GCM
    timestamp:      i64,            // Unix 时间戳
    source_guid:    [u8; 16],       // 源卷/磁盘 UUID
    source_size:    u64,            // 源卷总大小（字节）
    block_size:     u32,            // 块大小（默认 64KB）
    index_offset:   u64,            // 块索引表的文件内偏移
    header_crc32:   u32,            // Header 自身的 CRC32 校验
}

/// 块索引条目（每个已使用块一个，24 字节）
#[repr(C, packed)]
struct BlockIndexEntry {
    original_offset: u64,            // 原卷中的字节偏移
    compressed_offset: u64,          // 在 .nwb 文件中的偏移
    compressed_size:  u32,           // 压缩后的大小（0 = 空洞）
    original_size:    u32,           // 解压后的大小
    flags:            u16,           // 标志位（是否空洞、是否加密等）
    checksum:         [u8; 8],       // xxHash3-64（快速完整性校验）
}
```

### 3.4 增量链设计

```
全量备份 → 增量备份1 → 增量备份2 → 增量备份3
(FULL)      (INCR #1)   (INCR #2)   (INCR #3)
   │            │           │           │
   └────────────┴───────────┴───────────┘
             恢复时按链合并

增量依赖链完整性：
  · 每个增量文件记录其父镜像的 SHA-256
  · 恢复时必须从全量开始沿链叠加
  · 任何一环损坏 → 仅能恢复到上一个完好节点
  · 支持定期"合并"（rebase）将增量链压平为新的全量
```

### 3.5 格式关键特性

| 特性 | 实现方式 |
|------|---------|
| 空洞检测 | BlockIndexEntry.flags 标记空洞，不占存储 |
| 数据完整性 | 每块 xxHash3-64 + 尾部文件级 SHA-256 |
| 容错 | 索引表和块数据分离，索引损坏可部分恢复 |
| 加密 | AES-256-GCM 认证加密，每块独立 Nonce |
| 压缩 | zstd 逐块压缩（级别可配置 1-22） |
| 增量 | 基于 Change Tracking 记录差异块 |

---

## 4. Windows 平台层

### 4.1 卷快照（VSS 集成）

```
Windows VSS 备份流程（核心不可妥协）：

1. 调用 ::CoInitializeSecurity(COM)
2. 创建 IVssBackupComponents 接口
3. 初始化 VSS 备份上下文
4. 添加需要备份的卷 (AddComponent)
5. 调用 StartSnapshotSet() 启动快照集
6. 调用 PrepareForBackup() 准备备份
   ├── VSS 将通知所有 Writer（Exchange/SQL/NTFS）刷日志
   └── 等待所有 Writer 确认冻结状态
7. 调用 DoSnapshotSet() 执行快照
   ├── volsnap.sys 创建写时复制快照
   └── 返回快照设备路径
8. 引擎通过快照路径读取块数据
9. 调用 BackupComplete() 通知 VSS 完成
10. 释放 IVssBackupComponents 释放快照

注意：
  · VSS 必须在 SYSTEM 账户下运行（nuwa-daemon 以 SYSTEM 启动）
  · VSS 快照默认 10 秒后超时，DoSnapshotSet() 必须在 10 秒内返回
  · 一旦快照创建成功，读取操作无时间限制（快照持续存在直到释放）
```

### 4.2 NTFS 块解析

```
NTFS 卷布局：
┌──────┬────────┬─────────┬────────────────────────┬─────────┐
│ VBR  │ MFT 镜像│  $MFT   │      数据区域           │  MFT 尾 │
│(引导) │  (4KB) │ (文件表)│  ($Bitmap 标记已用块)    │ 部副本  │
└──────┴────────┴─────────┴────────────────────────┴─────────┘

$MFT 中关键系统文件：
  · $MFT (0): MFT 自身
  · $MFTMirr (1): MFT 镜像
  · $LogFile (2): 日志文件
  · $Volume (3): 卷信息
  · $AttrDef (4): 属性定义
  · $Root (5): 根目录索引
  · $Bitmap (6): 簇位图 ← ★ 核心！标记哪些簇已分配
  · $Boot (7): 引导扇区
  · $BadClus (8): 坏簇列表
  · $Secure (9): 安全描述符
  · $UpCase (10): 大写映射表
  · $Extend (11): 扩展文件

NTFS 块级备份流程：
  1. 通过 VSS 获取快照设备路径
  2. 从 VBR 读取 BPB (BIOS Parameter Block)
  3. 计算簇大小、MFT 起始位置、$Bitmap 位置
  4. 直接读取 $Bitmap 数据流（未压缩原始数据）
  5. 遍历 $Bitmap 的每个 bit：
     ├ bit=1 → 已分配，需要备份
     └ bit=0 → 空闲，标记为空洞（不消耗镜像空间）
  6. 对所有已分配簇执行块级读取（簇对齐）
  7. 通过 zstd 压缩后写入 .nwb 文件
```

### 4.3 引导修复（BCD）

```
Windows 系统恢复完成后引导修复：

1. 使用 BCD WMI API (ROOT\WMI:BCD*)
   └ 或直接调用 bcdboot.exe：
      bcdboot C:\Windows /s S: /f UEFI
      参数说明：
        C:\Windows → 刚恢复的 Windows 目录
        S:          → 恢复后挂载的 ESP 分区
        /f UEFI    → 生成 UEFI 引导配置

2. 验证 ESP 分区完整性：
   └ ESP 应包含: \EFI\Microsoft\Boot\bootmgfw.efi
                 \EFI\Microsoft\Boot\BCD

3. 如果引导修复失败：
   └ 提示用户插入 Windows 安装盘 → bootrec /fixmbr
                                                 /fixboot
                                                 /rebuildbcd
```

### 4.4 磁盘和分区发现

```
通过 Windows IOCTL 获取完整的磁盘拓扑：

1. CreateFile("\\\\.\\PhysicalDriveX", ...)
    → 获取物理磁盘句柄

2. IOCTL_DISK_GET_DRIVE_LAYOUT_EX
    → 读取 GPT/MBR 分区表

3. IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS
    → 获取卷 → 磁盘/分区的映射关系

4. IOCTL_STORAGE_GET_DEVICE_NUMBER
    → 获取存储设备信息

5. 最终输出：
   ├── Physical Drive #0
   │   ├── Partition 1: ESP (100MB, FAT32)
   │   ├── Partition 2: MSR (16MB, 保留)
   │   ├── Partition 3: C:\ (200GB, NTFS)
   │   └── Partition 4: Recovery (800MB, NTFS)
   ├── Physical Drive #1 (USB)
   │   └── Partition 1: D:\ (1TB, NTFS)
   └── ...
```

---

## 5. 备份工作流

### 5.1 文件级备份

```
文件级备份流程：
1. 用户选择源文件/文件夹 + 目标路径
2. 扫描源文件夹，建立文件清单（路径、大小、修改时间、属性）
3. 读取目标目录中的元数据（SQLite），检测上次备份状态
4. 对比文件变更：
   ├── 新文件 → 全量复制
   ├── 修改的文件 → 读取内容，写入 .nwb
   ├── 未变更的文件 → 创建引用（硬链接或跳过）
   └── 已删除的文件 → 在目录标记为已删除
5. 计算每个文件的 SHA-256
6. 写入 .nwb 镜像（压缩后）
7. 写入目录索引（catalog.json）
8. 校验：随机抽取 5% 文件验证 SHA-256
9. 更新 SQLite 目录数据库

注意：
  · 文件级备份 ≠ 块级备份
  · 文件级不依赖 VSS（但推荐关闭文件独占锁）
  · 文件级备份的恢复是"按文件还原"而非"按块还原"
```

### 5.2 卷级（分区镜像）备份

```
卷级备份流程（对应 Acronis "My Disks" → "C:" 备份）：

1. 用户选择源卷（如 C:）和目标路径
2. nuwa-daemon 调用 VolumeSnapshot::create(C:)
   ├── CoCreateInstance(CLSID_VSSBackupComponents)
   ├── AddToSnapshotSet(C:)
   └── DoSnapshotSet() → 返回 \\?\GLOBALROOT\Device\...\ShadowCopyX
3. 引擎打开快照设备路径
4. FsParser::detect(n Snap) → 识别 NTFS
5. FsParser::enumerate_allocated_blocks(n Snap) 
   ├── 读取 NTFS $Bitmap
   └── 返回 [BlockRange(offset, length), ...]
6. ImageEngine::create_writer(.nwb) →
   写入这些数据到 .nwb：
   ├── Header + Volume Metadata + $Bitmap 副本
   └── 逐已分配块：read → compress → write(offset, data)
7. 释放快照：VolumeSnapshot::release()
8. 写入文件尾部校验和
9. 更新 SQLite 目录

关键技术点：
  块大小选择：默认 64KB
    · 太小 → 索引膨胀（1TB 盘 + 4KB 块 = 2.68 亿条目）
    · 太大 → 压缩效率下降
    64KB 在压缩率 / 索引大小 / 随机恢复性能间平衡
```

### 5.3 磁盘级（全盘克隆）备份

```
磁盘级备份 = 卷级备份 × N + 分区表

1. 用户选择源磁盘（如 Disk #0）和目标路径
2. 读取磁盘分区表（GPT/MBR Header + Entry Array）
3. 对每个非空闲分区：
   ├── 如果是系统分区/数据分区 → 执行卷级快照备份
   └── 如果是 ESP/MSR/Recovery → 直接 raw block read（无 VSS）
4. 将分区表和所有分区镜像打包为一个 .nwb 文件
5. 写入统一目录索引
```

### 5.4 文件系统感知的智能备份策略

```
智能备份策略（从 Acronis 设计的逆向工程）：

卷级备份中需要理解文件系统的理由：

1. 跳过空白空间（最重要）
   ├── 1TB 盘 · 使用 200GB → 7GB .nwb（zstd 压缩后）
   └── 不跳过 → 1TB 备份 ≈ 500GB+（压缩也有限）

2. 排除页面文件和休眠文件
   ├── pagefile.sys (16GB) → 完全无用，标记为空洞
   ├── hiberfil.sys (32GB) → 完全无用，标记为空洞
   └── $LogFile / $TxfLog / System Volume Information → 跳过

3. 跳过已知可恢复的系统缓存
   ├── Prefetch, Font Cache, Icon Cache
   ├── Windows 更新临时文件
   └── 各种应用程序缓存

实现方式：
  · 解析 $MFT → 获取文件路径 → 判断是否需要排除
  · 对于需要排除的文件，将其簇范围标记为"空洞"
  · 这是纯用户态操作（读取 $MFT 的 Already-Allocated 属性）
```

---

## 6. 恢复工作流

### 6.1 文件级恢复

```
文件级恢复流程：

1. 用户浏览备份目录（SQLite）
2. 选择要恢复的文件/文件夹 + 版本
3. 引擎定位 .nwb 文件中该文件的数据块
4. 按块读取：
   ├── 定位块在 .nwb 中的偏移 (BlockIndexEntry.compressed_offset)
   ├── 读取压缩数据
   ├── zstd 解压
   ├── SHA-256 校验验证
   └── 写到目标路径
5. 恢复完成后：
   └── 对恢复的文件做一次完整校验（与目录中的 SHA-256 对比）
```

### 6.2 卷级恢复

```
卷级恢复流程（最关键的恢复路径）：

--- 前置安全检查 ---
1. 验证 .nwb 文件完整性：
   ├── 读 File Trailer → 验证文件级 SHA-256
   └── 读 Block Index → 遍历索引条目的校验和
2. 展示恢复预览：
   ├── 源卷信息：[C:\] [NTFS] [200GB]
   ├── 目标磁盘拓扑（只读扫描）
   └── 警告：目标分区所有数据将被永久覆盖

--- 恢复执行 ---
3. 获取目标卷的独占写锁：
   ├── Windows: CreateFile(\\\\.\\X:, GENERIC_WRITE, 0, ...)
   └── 如果无法独占 → 拒绝继续（可能有其他进程写入）
4. 逐块写入：
   ├── 从 .nwb 读取压缩块
   ├── zstd 解压
   ├── 校验 xxHash3-64
   ├── 写入目标卷的原始偏移处
   └── 每写 1000 块 → fsync 防止缓冲区溢出
5. 全部写入完成后 → fsync 整个卷

--- 后置验证 ---
6. 读回验证：
   ├── 随机选取 100 个块重新读取 + SHA-256 对比
   └── 如果验证失败 → 标记为"恢复可能不完整"

--- 引导修复 ---
7. 恢复的分区包含 Windows：
   ├── 挂载 ESP 分区（如果存在）
   ├── 执行 bcdboot C:\Windows /s S: /f UEFI
   └── 如果 ESP 也被恢复 → 无需额外操作

--- 结果报告 ---
8. 写入恢复日志到 SQLite
9. 展示恢复结果
```

### 6.3 磁盘级恢复

```
磁盘级恢复流程：

1. 验证磁盘级 .nwb 文件完整性
2. 读取分区表副本
3. 扫描目标磁盘：
   ├── 目标磁盘大小 >= 源磁盘大小？
   └── 若小于 → 检测是否有缩减空间的可行性
4. 展示恢复预览：
   ├── 源：Disk #0 (512GB) GPT
   │   ├── ESP (500MB)
   │   ├── C:\ (200GB, NTFS)
   │   └── D:\ (311.5GB, NTFS)
   └── 目标：Disk #2 (1TB) — 将被完全覆寫

5. 写入分区表到目标磁盘
6. 逐分区恢复（同卷级恢复流程）
7. 全部完成后 fsync
8. 读回验证
9. 引导修复（对所有系统分区）
10. 弹出目标磁盘
```

---

## 7. 磁盘克隆

### 7.1 克隆流程

```
磁盘克隆 = 磁盘级备份 + 磁盘级恢复（但在写入时直接流式传输）

流程：
1. 用户选择源磁盘 + 目标磁盘
2. 安全检查：
   ├── 目标磁盘 != 源磁盘（禁止自我克隆）
   ├── 目标磁盘没有系统卷（禁止克隆到含运行中系统的盘）
   ├── 目标磁盘大小 >= 源磁盘已使用大小（注意：不是总大小）
   └── 目标磁盘上没有重要数据（用户需确认）
3. 如果目标磁盘 > 源磁盘，提供分区扩展选项：
   └── 将最后一个分区的末尾扩展到磁盘末端
4. 执行克隆（流式 → 无需中间文件）：
   ├── 读取源磁盘分区表 → 写入目标磁盘
   ├── 逐分区：
   │   ├── 创建 VSS 快照（仅系统卷）
   │   ├── 读取原始块 → zstd 压缩 → 解压 → 写入目标
   │   └── 释放快照
   └── (注意：这里压缩再解压看似多余，但我们在传输过程中保持校验)
5. 修复目标磁盘引导：
   ├── 移除源盘后，目标盘启动需要更新 BCD
   └── bcdboot X:\Windows /s S:
6. 弹出目标磁盘，提示用户可安全移除
```

### 7.2 克隆到更小的磁盘

```
特殊场景：512GB SSD 克隆到 256GB SSD

条件：源盘已使用空间 < 目标盘总大小

策略：
  · 逐分区计算所需空间
  · 自动缩容最后一个分区
  · 如果缩容不够 → 拒绝克隆，提示"已用空间超过目标容量"

技术实现：
  1. 读取源盘已使用块统计
  2. 为每个分区计算其在目标盘上的新位置
  3. 重建分区表（依次从目标盘起始处摆放分区）
  4. 逐分区恢复 + 调整分区末尾边界
```

---

## 8. 服务架构

### 8.1 nuwa-daemon（核心守护进程 — HISTORICAL / SUPERSEDED）

```
nuwa-daemon 是 Nüwa Backup 的核心，以系统服务运行：

Windows: 作为 Windows Service (SYSTEM 账户)
Linux:   作为 systemd 服务 (root)

职责：
  · 任务调度和执行
  · 备份/恢复/克隆操作的实际执行
  · VSS 快照管理（Windows）
  · 块设备访问（所有平台）
  · 映像文件 I/O
  · SQLite 元数据管理
  · 通过 ZeroMQ IPC 暴露 API

进程结构（内部线程模型 — Tokio 异步）：
┌─────────────────────────────────────────┐
│  nuwa-daemon (Tokio Runtime)             │
│                                         │
│  ├── IPC Listener (ZeroMQ REP)          │
│  │   ← 接收 UI/CLI 的连接请求            │
│  │                                         │
│  ├── Task Scheduler （可选 Timer）        │
│  │   ← 按计划触发的备份任务               │
│  │                                         │
│  ├── Job Executor Pool （4-8 workers）   │
│  │   ← 并行执行备份/恢复/克隆            │
│  │   ← 每个 Job 在独立 OS 线程中运行      │
│  │                                         │
│  ├── Image Format Handler                │
│  │   ← .nwb 文件的读写访问               │
│  │                                         │
│  ├── Catalog Manager                     │
│  │   ← SQLite 读写                      │
│  │                                         │
│  └── Verification Engine                 │
│      ← 校验和验证                        │
└─────────────────────────────────────────┘

IPC 协议（基于 ZeroMQ）：
  请求-响应模式（REQ-REP）
  消息格式：MessagePack 序列化
  认证：Named Pipe 路径认证（仅允许本地同一用户连接）
```

### 8.2 nuwa-agent（代理进程）

```
nuwa-agent 是用户态的轻量级代理：

Windows: 用户登录时启动（Startup folder / Run registry key）
Linux:   ~/.config/autostart 或 systemd --user

职责：
  · 系统托盘图标
  · 显示进度通知
  · 监控 daemon 健康状态
  · 备份/恢复完成后的 toast 通知
  · 低磁盘空间告警

注意：agent 自己不执行任何备份操作，只是 daemon 的客户端
```

### 8.3 调度器

```
调度器设计（内置在 daemon 中）：

支持的调度类型：
  · 一次性（指定时间执行）
  · 每日（每天 HH:MM）
  · 每周（每周 X 天的 HH:MM）
  · 每月（每月的第 X 天的 HH:MM）
  · 间隔（每 N 小时执行一次）

实现：
  · Rust: tokio-cron-scheduler
  · 任务触发 → 创建 BackupJob → 加入 Job Executor 队列
  · 错过窗口的任务（如关机期间）：启动后补执行

调度持久化：
  · 存储在 SQLite 的 schedules 表中
  · 进程重启后自动加载所有活跃调度
```

---

## 9. UI 架构

### 9.1 技术选型：egui + eframe（纯 Rust）

| 组件 | 选型 | 理由 |
|------|------|------|
| UI 框架 | egui + eframe（纯 Rust）| 零运行时依赖、Win7~Win11 全覆盖 |
| 声明式 UI | egui 的即时模式(immediate mode) | 纯 Rust 实现，无需 JS/C++ 桥接 |原生渲染 |
| 后端 | Rust (nuwa-daemon IPC 客户端) | 与核心引擎同语言，双向调用 |
| QML 集成 | qmetaobject-rs 或 qml.rs | Rust ↔ QML 双向绑定 |

### 9.2 页面结构

```
Main Window
├── Sidebar Navigation
│   ├── 📊 Dashboard（仪表盘）
│   ├── 💾 Backup（备份）
│   │   ├── File Backup（文件备份）
│   │   └── Volume Backup（卷备份）
│   ├── 🔄 Restore（恢复）
│   │   ├── File Restore（文件恢复）
│   │   └── Volume Restore（卷恢复）
│   ├── 🛠 Tools（工具）
│   │   ├── Disk Clone（磁盘克隆）
│   │   └── Bootable Media（启动介质）
│   └── ⚙ Settings（设置）
│
├── Dashboard Page
│   ├── 概览卡片
│   │   ├── 上次备份时间
│   │   ├── 总备份大小
│   │   ├── 保护的数据量
│   │   └── 健康状态（绿色/黄色/红色）
│   ├── 最近备份列表
│   └── 快速操作按钮
│
├── File Backup Page
│   ├── 源文件选择（文件夹树 + 常用文件夹快捷选择）
│   ├── 目标路径选择（磁盘列表 + SMB）
│   ├── 调度设置（频率/时间）
│   └── 立即备份按钮
│
├── Volume Backup Page
│   ├── 磁盘拓扑可视化（图形化磁盘分区展示）
│   ├── 选择要备份的卷
│   ├── 目标路径选择
│   └── 立即备份按钮
│
├── File Restore Page
│   ├── 备份选择（日历 / 列表时间线）
│   ├── 文件浏览器（树形结构浏览备份内容）
│   ├── 搜索框（快速定位文件）
│   └── 恢复设置（原位置 / 新位置）
│
├── Volume Restore Page
│   ├── 备份选择
│   ├── 目标磁盘选择（带兼容性检查提示）
│   ├── 分区调整选项（如适用）
│   └── 恢复执行（带进度和 ETA）
│
└── Settings Page
    ├── 通用设置
    ├── 性能（CPU/IO 限制）
    ├── 通知
    └── 关于
```

### 9.3 WinPE/Linux 恢复环境 UI

```
恢复环境中的 UI 与桌面 UI 不同：

设计原则：
  · 更简单、更少步骤
  · 大字体、高对比度
  · 适用于键盘/鼠标/触控
  · 专注于"恢复"这一个目标

恢复环境 UI 流程：
  1. Welcome（欢迎页）
     ├── 显示系统信息
     └── 按钮：开始恢复
  2. Disk Detection（磁盘检测）
     ├── 扫描所有本地磁盘
     ├── 扫描 USB 磁盘中的 .nwb 文件
     ├── 支持手动输入 SMB 路径浏览
     └── 按钮：继续
  3. Backup Selection（选择备份）
     ├── 列表显示所有找到的 .nwb 备份
     └── 详细信息（创建时间、大小、源卷）
  4. Target Selection（选择目标）
     ├── 显示磁盘拓扑
     └── 目标磁盘确认（二次确认对话框）
  5. Restore Execution（执行恢复）
     ├── 进度条 + 已写入量 + ETA
     └── 完成后提示移除介质并重启
```

---

## 10. 恢复环境（Bootable Recovery Environment）

### 10.1 Windows 恢复环境 — WinPE

```
WinPE 恢复环境方案：

构建工具：
  · Windows ADK (Assessment and Deployment Kit)
  · 使用 copype.cmd 创建 WinPE 基础镜像
  · 注入 Nüwa Backup 恢复程序 + 驱动包

WinPE 镜像结构：
  winpe.wim
  ├── Windows (WinPE 系统文件)
  ├── Program Files
  │   └── Nüwa Backup
  │       ├── nuwa_restore.exe (Rust + Win32 API)
  │       ├── nuwa_core.dll (核心引擎)
  │       ├── libcrypto-3-x64.dll
  │       ├── zstd.dll
  │       └── winpe_resources.dll (恢复 UI + 资源)
  ├── WindowsSystem32drivers
  │   └── (额外的存储/NIC 驱动)
  └── boot (BCD 配置)

WinPE 启动 → 运行 Nüwa Backup 恢复向导的过程：
  1. 加载 WinPE 环境
  2. 自动启动 nuwa_restore.exe
  3. 调用 Windows Disk APIs → 枚举所有本地磁盘
  4. 调用 Setup API → 枚举 USB 设备
  5. 支持 SMB 网络路径浏览（WinPE 内置 SMB 客户端）
  6. 用户选择 .nwb 备份 → 开始恢复
```

### 10.2 跨平台恢复环境（未来扩展）

```
Linux 恢复环境方案：

架构选择：
  · 基于 Alpine Linux（极致轻量，~100MB）
  · 打包 Nüwa Backup 核心引擎 + 恢复 UI
  · 支持所有目标架构交叉编译

构建策略（因为我们核心引擎是 Rust）：
  · Rust 编译为对应架构的静态二进制
  · 在 Alpine Linux 的 initramfs 中打包
  · 使用 GRUB 作为引导管理器

ISO 内包含多架构内核：
  ─ boot/
   ├── vmlinuz-x86_64
   ├── vmlinuz-aarch64
   ├── vmlinuz-loongarch64
   ├── initrd-x86_64.img
   ├── initrd-aarch64.img
   ├── initrd-loongarch64.img
   └── grub.cfg

恢复 UI：
  · 在恢复环境中直接使用 egui 应用（Alpine 提供基本的显示服务支持）
  · 或在 initramfs 中使用终端 TUI（更可靠但 UX 更差）
  · 推荐方案：内核加载完成后启动 egui 恢复程序
```

---

## 11. 跨平台策略总结

### 11.1 平台差异对照

```
┌─────────────────────┬─────────────────────┬─────────────────────────┐
│     功能需求         │  Windows 实现       │ Linux/国产OS 实现       │
├─────────────────────┼─────────────────────┼─────────────────────────┤
│ 卷一致性快照         │ VSS (volsnap.sys)   │ fsfreeze / LVM snap    │
│ 块设备访问           │ \\.\PhysicalDriveX  │ /dev/sda                │
│                                                     │
│ 文件系统解析         │                       │                         │
│  ├ NTFS            │ $Bitmap + $MFT      │ ntfs-3g / libntfs      │
│  ├ ext4             │ N/A                 │ ext4 block bitmap      │
│  └ XFS              │ N/A                 │ XFS B+tree (xfs_db)    │
│                                                     │
│ 分区表访问           │ IOCTL_DISK_GET_     │ libblkid / fdisk       │
│                      │ DRIVE_LAYOUT_EX     │                         │
│                                                     │
│ 引导修复             │ bcdboot.exe         │ grub-install           │
│                      │                     │ grub-mkconfig          │
│                                                     │
│ 系统服务             │ Windows Service      │ systemd unit           │
│                      │ (SYSTEM 账户)        │ (root)                 │
│                                                     │
│ 桌面 UI              │ egui             │ egui                │
│ 系统托盘              │ Win32 Tray API      │ egui SystemTrayIcon│
│                                                     │
│ 恢复环境             │ WinPE + ADK         │ Alpine Linux ISO       │
│ 文件共享             │ SMB (Windows API)   │ SMB (libsmbclient)     │
│                     │                     │ NFS (mount -t nfs)     │
├─────────────────────┼─────────────────────┼─────────────────────────┤
│ 共享代码（Rust 核心）│ >80% 代码共享        │ >80% 代码共享           │
│ OS 适配层            │ <20% 平台特有代码     │ <20% 平台特有代码        │
└─────────────────────┴─────────────────────┴─────────────────────────┘
```

### 11.2 文件结构

```
Nüwa Backup/
├── Cargo.toml                    # Rust workspace
├── crates/
│   ├── nuwa-core/                 ★ 核心引擎（镜像格式/压缩/加密）
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── image/            · 镜像格式读写
│   │   │   │   ├── header.rs
│   │   │   │   ├── block_index.rs
│   │   │   │   ├── block_stream.rs
│   │   │   │   └── trailer.rs
│   │   │   ├── compression/      · zstd 压缩
│   │   │   ├── encryption/       · AES-256-GCM
│   │   │   ├── checksum/         · SHA-256 / xxHash3
│   │   │   └── catalog/          · 备份目录索引
│   │   └── Cargo.toml
│   │
│   ├── nuwa-os-api/               ★ OS 抽象层（Traits 定义）
│   │   ├── src/
│   │   │   ├── lib.rs            · 所有 trait 定义
│   │   │   ├── snapshot.rs       · VolumeSnapshot trait
│   │   │   ├── block_device.rs   · BlockDevice trait
│   │   │   ├── fs_parser.rs      · FsParser trait
│   │   │   ├── partition.rs      · PartitionTable trait
│   │   │   ├── boot_config.rs    · BootConfig trait
│   │   │   └── disk_discovery.rs · DiskDiscovery trait
│   │   └── Cargo.toml
│   │
│   ├── nuwa-os-win/               ★ Windows 适配实现
│   │   ├── src/
│   │   │   ├── vss.rs            · VSS COM API 封装
│   │   │   ├── ntfs.rs           · NTFS $Bitmap 解析
│   │   │   ├── bcd.rs            · BCD WMI API
│   │   │   └── disk_ioctl.rs     · Win32 IOCTL 调用
│   │   ├── build.rs              · windows-rs 绑定生成
│   │   └── Cargo.toml
│   │
│   ├── nuwa-os-linux/             ★ Linux 适配实现
│   │   ├── src/
│   │   │   ├── snapshot.rs       · fsfreeze + LVM snapshot
│   │   │   ├── ext4.rs           · ext4 块位图解析
│   │   │   ├── xfs.rs            · XFS B+tree 解析
│   │   │   ├── grub.rs           · GRUB 安装/修复
│   │   │   └── disk_udev.rs      · udev 设备枚举
│   │   └── Cargo.toml
│   │
│   ├── nuwa-daemon/               ★ 核心守护进程
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── ipc.rs            · ZeroMQ 服务端
│   │   │   ├── scheduler.rs      · 任务调度器
│   │   │   ├── job_executor.rs   · 备份/恢复作业执行
│   │   │   └── catalog_mgr.rs    · SQLite 目录管理
│   │   └── Cargo.toml
│   │
│   ├── nuwa-agent/                ★ 代理进程（系统托盘）
│   │   └── src/
│   │       ├── main.rs
│   │       └── notifier.rs
│   │
│   ├── nuwa-cli/                  ★ 命令行工具
│   │   └── src/
│   │       └── main.rs
│   │
│   └── lcb-ipc/                  ★ 共享 IPC 协议和消息类型
│       └── src/
│           └── messages.rs
│
├── ui/                           ★ egui UI
│   ├── src/
│   │   ├── main.rs               · egui 入口，加载 Rust 核心引擎
│   │   ├── ipc_client.cpp        · ZeroMQ IPC 客户端
│   │   └── models/               · QML 数据模型（C++ 侧）
│   ├── qml/
│   │   ├── main.qml
│   │   ├── pages/
│   │   │   ├── Dashboard.qml
│   │   │   ├── FileBackup.qml
│   │   │   ├── VolumeBackup.qml
│   │   │   ├── FileRestore.qml
│   │   │   ├── VolumeRestore.qml
│   │   │   ├── DiskClone.qml
│   │   │   └── Settings.qml
│   │   ├── components/           · 可复用组件
│   │   │   ├── DiskMap.qml       · 磁盘拓扑可视化
│   │   │   ├── BackupCalendar.qml
│   │   │   └── ProgressCard.qml
│   │   └── theme/                · 深色科技风主题
│   ├── CMakeLists.txt
│   └── resources/
│
├── recovery/                     ★ 恢复环境构建脚本
│   ├── winpe/
│   │   ├── build_winpe.ps1       · WinPE 构建脚本
│   │   └── winpe_config.xml      · WinPE 配置
│   ├── linux/
│   │   ├── build_iso.sh          · Linux ISO 构建脚本
│   │   └── initramfs_overlay/    · initramfs 叠加层
│   └── common/
│       └── lcb_restore/          · 恢复环境中的恢复程序
│
├── tests/                        ★ 集成测试
│   ├── integration/
│   │   ├── backup_restore_loop.rs· 备份恢复闭环测试
│   │   └── image_format_test.rs  · 镜像格式兼容性测试
│   └── fixtures/                 · 测试用数据
│       └── small_disk_image.bin
│
├── docs/                         ★ 文档
│   ├── 01_PRD.md
│   ├── 02_MVP_Plan.md
│   ├── 03_Architecture.md        ← 本文档
│   └── 04_Image_Format_Spec.md
│
└── scripts/                      ★ 构建和部署脚本
    ├── build_all.ps1             · Windows 全量构建
    ├── build_all.sh              · Linux 全量构建
    └── sign.ps1                  · 代码签名脚本
```

---

## 12. 开发路线图

### 12.1 Phase 1 — Windows 核心引擎 + CLI

``"
目标：验证 Rust 核心引擎在 Windows 上的完整备份/恢复循环

Milestone 1.1: nuwa-core 镜像引擎
  · .nwb 格式读写（全量）
  · zstd 压缩/解压
  · SHA-256 校验和
  · 单元测试: 创建 → 写入 → 读取 → 验证

Milestone 1.2: Windows OS 适配层
  · VSS 快照创建/释放
  · NTFS $Bitmap 读取
  · GPT/MBR 分区表读取
  · 磁盘拓扑枚举 (IOCTL)
  · 单元测试: 在真实磁盘上验证（测试环境需要物理磁盘）

Milestone 1.3: nuwa-daemon (Windows)
  · Windows Service 框架
  · ZeroMQ IPC 服务端
  · 串行备份执行
  · 串行恢复执行
  · 集成测试: 创建备份 → 恢复 → 验证文件内容一致

Milestone 1.4: nuwa-cli
  · 备份命令: lcb backup --volume C: --dest D:\backup
  · 恢复命令: lcb restore --backup D:\backup\C_20260701.nwb --target E:
  · 列表命令: lcb list --path D:\backup
  · 验证命令: lcb verify --backup D:\backup\C_20260701.nwb

Phase 1 交付物：
  · 全量卷级备份 + 恢复 命令行工具
  · 支持 VSS 一致性快照
  · 支持 NTFS $Bitmap 智能读取
  · 支持 GPT/MBR 分区表
  · 支持本地磁盘 / USB 目标
  · 输出的 .nwb 文件可验证
``"

### 12.2 Phase 2 — Windows 桌面 UI

``"
Milestone 2.1: egui 基础框架
  · 主窗口 + 侧边导航
  · 仪表盘页面
  · 深色科技风主题

Milestone 2.2: 备份 UI
  · 文件备份页面（文件夹选择 + 目标选择 + 调度）
  · 卷备份页面（磁盘拓扑可视化 + 卷选择）
  · 进度显示

Milestone 2.3: 恢复 UI
  · 备份浏览器（时间线/列表/日历）
  · 文件级浏览
  · 卷级恢复（目标选择 + 确认 + 进度）

Milestone 2.4: 设置 + 系统集成
  · 设置页面
  · 系统托盘集成（nuwa-agent）
  · 通知
  · 计划任务（调度器）

Phase 2 交付物：
  · 完整的桌面应用
  · 文件级备份/恢复
  · 卷级备份/恢复
  · 计划任务
  · 系统托盘 + 通知
``"

### 12.3 Phase 3 — Windows 高级功能

``"
Milestone 3.1: 磁盘克隆
  · 源/目标磁盘选择
  · 逐步式安全确认
  · 流式磁盘克隆
  · 引导修复

Milestone 3.2: 恢复环境 (WinPE)
  · WinPE 构建脚本
  · Nüwa Backup 恢复程序
  · USB 介质创建工具
  · 在恢复环境中检测磁盘和备份

Milestone 3.3: 增量备份
  · USN Journal 或块级变化追踪
  · 增量 .nwb 文件生成
  · 增量链恢复
  · 链完整性验证

Milestone 3.4: 高级选项
  · 压缩级别选择
  · 加密（AES-256-GCM）
  · 备份验证
  · 邮件/网络通知

Phase 3 交付物：
  · 对标 Acronis True Image 核心功能的 Windows 产品
  · 磁盘克隆 + 引导修复
  · WinPE 恢复环境
  · 增量备份
``"

### 12.4 Phase 4 — Linux 和国产 OS 扩展

``"
Milestone 4.1: Linux OS 适配层
  · fsfreeze 快照
  · ext4 块位图解析
  · GPT/MBR 读取 (/sys/block + libblkid)
  · GRUB 引导修复
  · systemd 服务集成

Milestone 4.2: nuwa-daemon + CLI (Linux)
  · 移植 daemon 到 Linux
  · CLI 工具 Linux 构建
  · 集成测试在 ext4 上验证备份/恢复循环

Milestone 4.3: Linux 桌面 UI
  · egui 在 Linux 上构建
  · 系统托盘（egui 原生托盘）
  · 文件备份/恢复（Linux 文件系统）

Milestone 4.4: 国产 OS 适配
  · 麒麟 V10 (x86_64 + ARM64) 适配测试
  · 统信 UOS (x86_64 + ARM64) 适配测试
  · LoongArch 交叉编译支持
  · 认证适配

Milestone 4.5: Linux 恢复环境 (Alpine ISO)
  · 基于 Alpine Linux 的恢复 ISO
  · 多架构 ISO 构建
  · 恢复环境中 Nüwa Backup 应用

Phase 4 交付物：
  · Linux 桌面版（支持 ext4/部分 XFS）
  · 麒麟 V10 + 统信 UOS 适配
  · Alpine Linux 恢复 ISO
``"

---

## 13. 关键风险和技术决策

### 13.1 风险登记

| 风险 ID | 风险描述 | 可能性 | 影响 | 缓解措施 |
|---------|---------|--------|------|---------|
| R-01 | NTFS $Bitmap 读取在 Windows 新版本中被限制 | 低 | 高 | 使用 VSS 快照路径读取，Rust 实现 NTFS 解析器不依赖 API |
| R-02 | VSS 快照在 Windows Server Core 上不可用 | 中 | 中 | Server Core 场景回退到 fsfreeze-like 机制（临时暂停写入） |
| R-03 | 绕开 Windows 文件锁时可能导致文件不一致 | 低 | 高 | VSS 确保 application-consistent，非 VSS 场景使用 crash-consistent |
| R-04 | LVM 快照需要预先配置 LVM thin pool | 中 | 中 | 降级到 fsfreeze（短暂冻结但保证一致性） |
| R-05 | WinPE 不包含特定硬件驱动（RAID/NVMe） | 中 | 高 | WinPE 构建时包含目标平台驱动包 |
| R-06 | LoongArch GRUB 行为与 x86 不同 | 低 | 高 | 在目标硬件上验证，LoongArch 列为 Beta 支持 |

### 13.2 技术决策理由

| 决策 | 选项 | 选择 | 理由 |
|------|------|------|------|
| 核心语言 | Rust / C++ / Go | **Rust** | 跨平台编译、零运行时、内存安全、C ABI |
| UI 框架 | egui / Qt6 / Web+Electron / WPF | **egui + eframe（纯 Rust）** | 原生性能、真跨平台、Electron 太重 |
| IPC | ZeroMQ / gRPC / Named Pipes | **ZeroMQ** | 轻量、跨平台、异步模式支持 |
| 加密 | OpenSSL / RustCrypto / bcrypt | **AES-256-GCM (RustCrypto)** | 认证加密、无外部依赖 |
| 压缩 | zstd / lz4 / deflate | **zstd** | 压缩比和速度的平衡，Acronis 也用它 |
| 镜像挂载 | WinFsp+FUSE / 自研 | **WinFsp + FUSE** | 工业级 Windows FUSE 实现 |
| 数据库 | SQLite / LevelDB / own format | **SQLite** | 零配置、成熟稳定 |
| 构建系统 | Cargo / CMake / MSBuild | **Cargo (Rust)** | 纯 Rust 构建，无外部构建依赖 |

---

## 14. 与 Acronis True Image 的功能对标

### 14.1 功能覆盖

```
功能                          Acronis TI   Nüwa Backup v1.0   Nüwa Backup v2.0   Nüwa Backup v3.0
─────────────────────────────────────────────────────────────────────
文件/文件夹备份                  ✅           ✅          ✅          ✅
文件/文件夹恢复                  ✅           ✅          ✅          ✅
卷镜像备份                       ✅           ❌          ✅          ✅
卷镜像恢复                       ✅           ❌          ✅          ✅
磁盘克隆                         ✅           ❌          ❌          ✅
WinPE 恢复环境                   ✅           ❌          ❌          ✅
增量备份                         ✅           ❌          ❌          ✅
加密备份 (AES-256)              ✅           ❌          ❌          ✅
计划任务                         ✅           ❌          ✅          ✅
备份验证                         ✅           ✅          ✅          ✅
SMB/NAS 目标                     ✅           ❌          ❌          ✅
Linux 支持                       ❌           ❌          ❌          ✅
国产 OS 支持                     ❌           ❌          ❌          ✅
安全功能（防勒索等）              ✅           ❌          ❌          ❌
云备份                           ✅           ❌          ❌          ❌
```

### 14.2 差异化优势

``"
Nüwa Backup vs Acronis True Image — 我方优势：

1. 跨平台架构：从第一天起就是跨平台设计，Acronis 仅限 Windows
2. 开源核心引擎：镜像格式非专有，可审计、可扩展
3. 零内核代码：依赖 Windows VSS，无蓝屏风险
4. 国产化支持：原生适配麒麟/UOS，Acronis 不支持

Nüwa Backup vs Acronis True Image — 需追赶：

1. 功能的成熟度：Acronis 有 20 年迭代
2. 增量链的稳定性：Acronis 经过千百万用户验证
3. 恢复环境的完善度：Acronis 的 Linux 恢复环境非常成熟
4. 硬件兼容性：Acronis 的驱动库覆盖几乎所有 RAID 卡
```

---

## 15. 总结

Nüwa Backup 的架构设计遵循以下核心思想：

1. **核心引擎跨平台**：Rust 编写的 .nwb 镜像格式、压缩、加密、校验——所有平台 100% 共享
2. **OS 交互隔离**：通过 Rust traits 将 VSS、NTFS、BCD（Windows 侧）与 fsfreeze、ext4、GRUB（Linux 侧）隔离开来
3. **服务化架构**：核心功能以 daemon 形式运行，UI/CLI 通过 IPC 调用——解耦、安全、可测试
4. **恢复优先**：每个功能模块在开发时，恢复路径的测试优先级高于备份路径
5. **渐进式交付**：先核心引擎 + CLI → 再桌面 UI → 再高级功能 → 再跨平台

下一份文档将是 **Nüwa Backup 镜像格式详细规范**，覆盖 .nwb 文件的完整二进制布局、增量链设计、容错机制和版本兼容性策略。

---

## 附录 A：Phase 1 MVP 架构

### A.1 架构范围

Phase 1 MVP 仅实现文件级备份/恢复的最小闭环，架构极其简单：

`
用户
  │
  ├── CLI 命令 (nuwa backup/restore/verify/list)
  │       │
  │       └── nuwa-core（单一 crate，静态编译）
  │               ├── 文件系统遍历器
  │               ├── 平文件存储引擎（目录 + JSON manifest）
  │               ├── zstd 压缩（可选，单线程）
  │               ├── SHA-256 校验
  │               └── 恢复引擎
  │
  └── 输出：目标路径下的备份目录
       D:\Backup\
       └── 20260705_MyDocs/
           ├── manifest.json（备份元数据 + 文件清单 + SHA-256）
           ├── file1.txt（或 file1.txt.zst）
           ├── file2.docx.zst
           └── ...
`

### A.2 排除项

| 架构组件 | 状态 | 引入阶段 |
|---------|------|---------|
| .nwb 镜像格式 | ❌ 不实现 | Phase 2 实验 / Phase 3 正式 |
| VSS 快照 | ❌ 不实现 | Phase 3 |
| lcb-daemon / IPC | ❌ 不实现 | Phase 4 |
| 桌面 GUI (egui) | ❌ 不实现 | Phase 6+ |
| 差异备份 | ❌ 不实现 | Future |
| XOR Parity | ❌ 不实现 | Future |
| AES 加密 | ❌ 不实现 | Future |

### A.3 Phase 1 唯一合法性

Phase 1 开发以本附录 +  2_Development_Plan.md +  9_MVP_Boundary_and_Risk_Correction.md 为准。
本文档正文中涉及的复杂架构内容仅为终局设计，不作为 Phase 1 开发依据。

---

## Appendix D: Current Desktop GUI Architecture (Tauri 2.0)

> **Note:** This appendix documents the **current desktop GUI architecture** which replaced the egui/eframe design described elsewhere in this document.
> Phase 2.5 (2026-07) migrated from egui + eframe to Tauri 2.0 + React + TypeScript + Vite.
> See docs/phase-2.5/Phase_2_5_Tauri_Migration_Decision.md for the migration decision.

### Technology Stack

| Layer | Technology | Purpose |
|-------|-----------|---------|
| Desktop Runtime | Tauri 2.0 (WebView2) | Native desktop window, IPC bridge |
| Frontend | React 19 + TypeScript + Vite 6 | UI rendering, state management |
| IPC | Tauri invoke() | Frontend-backend communication |
| Command Layer | Rust (src-tauri/src/commands/) | Thin wrapper: params, invoke, error convert |
| Application Layer | Rust (src/app/) | Models, services, error types |
| Core Engine | Rust (src/) | Backup, restore, verify, storage |

### Architecture Diagram

`
                React UI (TypeScript)
                      |
                      | invoke() IPC
                      v
          Tauri Command Layer
     (thin wrapper, no business logic)
                      |
                      v
        Application Service Layer
     (src/app/ -- models, services, errors)
                      |
                      v
            Core Engine (src/)
      backup restore verify storage

  CLI (src/main.rs) also calls Core Engine directly
`

### Layer Responsibilities

| Layer | Responsibility | Forbidden |
|-------|---------------|-----------|
| React UI | Display, interaction, state rendering | Direct core access, SQLite queries |
| Tauri Command | Parameter validation, invoke handling, error conversion | Business logic |
| Application Service | Data aggregation, orchestration, model mapping | Core module modification |
| Core Engine | Backup/restore, verification, storage operations | UI coupling |

### Key Differences from egui Design

| Aspect | Original (egui) | Current (Tauri) |
|--------|-----------------|------------------|
| UI Framework | egui + eframe (Rust) | React + TypeScript + Vite |
| IPC | ZeroMQ (planned) | Tauri invoke() |
| Daemon | lcb-daemon (planned) | None (direct calls) |
| UI Location | src/gui/ | ui/ |
| Dev Toolchain | Rust only | Rust + Node.js |
| Application Layer | Not planned | src/app/ (models + services) |


