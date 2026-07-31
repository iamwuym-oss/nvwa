# Nüwa Backup NWB Storage Engine Architecture v2.0

**状态：** Architecture Baseline — 核心方向冻结，配套工程规范已建立，二进制数据契约待原型验证  
**日期：** 2026-07-12  
**适用产品：** Nüwa Backup 单机版  
**归档扩展名：** `.nwb`  
**目标平台：** Windows 7 SP1及后续Windows工作站与服务器；主流x86-64 Linux；未来扩展ARM64和其他平台

---


## 0. v2.0 文档控制与变更摘要

### 0.1 文档目的

本文档重新定义 Nüwa Backup 单机版的长期存储格式。物理实现不继承此前 Repository 方案中的外部恢复数据库、独立块文件和 Backup Object 目录；新的产品基线是每次完整备份或差异备份生成一个不可变、自描述、可验证、可离线恢复的逻辑NWB归档。归档默认是单文件；当目标文件系统、介质能力或用户策略要求时，同一逻辑归档可由多个NWB物理分卷组成。

### 0.2 v2.0 相对 v1.0 的核心变化

| 主题 | v1.0 | v2.0 |
|---|---|---|
| Catalog | 以内嵌 Catalog 为主 | 正式冻结三层 Catalog |
| 本地数据库 | 允许作为 UI 缓存 | 定义 Cache Schema、失效、重建与安全边界 |
| 跨备份搜索 | 预留 | 定义可选 Search Index 和渐进式索引 |
| Diff Catalog | Full + Overlay | Diff 内保存该恢复点完整逻辑 Catalog |
| 数据粒度 | 默认 1 MiB Chunk | 4KB–64KB 变化检测、256KB Chunk、128MB Segment |
| 导入备份 | 扫描归档 | Header-only Import + Lazy Catalog |
| Acronis 对照 | 原则性参考 | 区分公开事实、合理推断和 Nüwa 自主设计 |
| 可靠性 | Header/Manifest/Footer 冗余 | 增加 Catalog Page、Index 和 Cache 的独立恢复边界 |
| 大型归档 | 默认单文件 | 一个逻辑归档支持单文件或多物理分卷 |

### 0.3 决策状态

| 状态 | 含义 |
|---|---|
| FROZEN | 产品和架构原则，后续实现不得自行改变 |
| BASELINE | v2.0 默认参数，在二进制数据契约冻结前由基准测试最终确认 |
| FUTURE | 已预留兼容能力，当前版本不实现 |

### 0.4 v2.0 已冻结的产品决策

1. 只实现完整备份和差异备份，不实现增量备份；
2. 每次成功备份运行产生一个逻辑NWB归档；在目标允许时为一个文件，受限时为同一Volume Set中的多个NWB分卷；
3. Full 自包含，Diff 只依赖指定 Full，不依赖其他 Diff；
4. NWB 内部 Catalog 是恢复真相；
5. 本地 Catalog Cache 是可删除、可重建的性能缓存；
6. Search Index 是可选跨备份搜索索引；
7. Full 和 Diff 都保存各自恢复点的完整逻辑 Catalog；
8. 已提交归档不可原位修改；
9. 文件、卷、磁盘和 BMR 共用一个容器协议；
10. 外部数据库永远不能保存唯一一份恢复关键数据；
11. BMR、UEFI/GPT恢复和异机还原属于第一版标准能力，不得降级为未来增强；
12. 第一版恢复介质必须支持UEFI x64启动，并能在无原系统环境下完成整机恢复；
13. UEFI启动环境必须由模块化Boot Environment Provider采集、恢复和验证；
14. 分卷是NWB物理容器的原生能力，不得通过外部压缩包、普通文件拼接或Sidecar清单模拟；
15. 所有分卷共同代表一个恢复点，必须作为一个逻辑Archive执行验证、移动、删除和Retention。
16. 跨平台NWB Core不得依赖Windows或Linux专有对象布局；平台语义通过Provider提供；
17. 首版同时面向Windows与主流Linux的文件、卷、磁盘和认证BMR场景；实施顺序先闭环Windows，再复用同一Core闭环Linux；
18. Provider不得直接写NWB物理结构，也不得取得Archive明文密钥；
19. NWB 1.0采用Little Endian、64位偏移/长度、4KiB关键结构对齐和显式版本化Record；
20. 256KiB Chunk、约128MiB Segment、64KiB Catalog Page作为Format 1.0原型基线；
21. Format 1.0必须经Windows/Linux跨平台互读、Golden Corpus、故障注入、安全解析和BMR原型验证后冻结；
22. 尚未执行的测试一律记录为NOT_RUN；没有证据的功能不得标记为通过或正式支持。

---

## 1. 执行摘要

Nüwa Backup 采用面向单机备份恢复产品的 **Single Logical Archive Per Backup Run** 架构：每执行一次完整备份或差异备份，生成一个逻辑NWB归档。目标支持大型文件时该归档保存为一个 `.nwb` 文件；受文件系统、介质或用户策略限制时，保存为多个属于同一Volume Set的 `.nwb` 分卷。归档内嵌恢复所需的数据、目录、块映射、磁盘布局、启动信息、完整性信息、加密信息和格式描述，不依赖任何外部数据库完成识别、浏览、验证和恢复。

备份模型限定为：

- **完整备份（Full）**：一个完整逻辑归档自包含；归档可以是一个文件或一个完整分卷集合。
- **差异备份（Differential）**：一个差异逻辑归档完全自描述，只依赖其指定的完整逻辑归档，不依赖其他差异备份。
- **不实现增量备份**。
- 完整备份提交后永不原位修改；差异备份也永不合并回完整备份。

核心设计目标不是追求 Repository 级全局去重，而是：

1. 单文件或完整Volume Set可移动、可识别、可验证；
2. 顺序写入、高吞吐、有限内存；
3. 断电和进程崩溃不产生伪有效恢复点；
4. 局部损坏不扩大为整个归档不可读；
5. 关键元数据具备冗余、校验和离线重建能力；
6. 支持 Windows 文件、卷、磁盘、BMR 和异机还原；
7. 格式具备长期兼容与独立恢复能力。

---

## 2. 产品原则与非目标

### 2.1 冻结原则

| ID | 原则 |
|---|---|
| P-01 | 每次成功备份产生一个逻辑NWB归档；物理上可以是一个文件或一个Volume Set。 |
| P-02 | 完整逻辑归档独立可恢复；若采用分卷，则恢复需要全部被所选数据引用的必要分卷。 |
| P-03 | 差异备份仅依赖指定完整备份，不依赖其他差异备份。 |
| P-04 | `.nwb` 是恢复真相；UI 历史库、任务配置和缓存均不是恢复依赖。 |
| P-05 | 提交后的归档不可原位修改，修复或转换必须写入新文件。 |
| P-06 | 文件、卷、磁盘统一使用同一个容器格式和数据记录层。 |
| P-07 | 所有大小、偏移和计数使用 64 位字段。 |
| P-08 | 所有已提交内容必须可验证；加密归档必须同时提供机密性与真实性。 |
| P-09 | 格式读取器必须拒绝未知的强制特性、非法偏移、整数溢出和重叠区域。 |
| P-10 | BMR启动介质可以只凭完整NWB逻辑归档（单文件或完整Volume Set）和用户密钥完成恢复。 |
| P-11 | BMR和异机还原是第一版发布标准能力，不得作为后续可选增强。 |
| P-12 | 第一版必须支持Windows UEFI/GPT系统的原机原盘、原机新盘和异机恢复。 |
| P-13 | UEFI启动信息、GPT、ESP、BCD、WinRE和Boot-critical Driver必须纳入归档。 |
| P-14 | 单文件模式是Volume Set只有一个成员的特例，单文件与分卷必须共用同一Record/Segment/Catalog协议。 |
| P-15 | Chunk Record和Segment不得跨越物理分卷边界。 |
| P-16 | 多分卷归档只有在Final Volume中的Set Manifest提交后才是有效恢复点。 |

### 2.2 当前非目标

- Repository 级跨归档全局去重；
- 永久增量、反向增量和增量合并；
- 数据库原生日志截断与时间点恢复；
- VMware/Hyper-V/KVM 虚拟机原生备份；
- 分布式存储、对象存储原生对象布局；
- 多节点全局索引；
- 对已提交 `.nwb` 进行原位压缩整理。

---

## 3. 完整与差异备份模型

### 3.1 文件集合

```text
Workstation-A_20260712_120000_FULL.nwb
Workstation-A_20260713_120000_DIFF.nwb
Workstation-A_20260714_120000_DIFF.nwb
Workstation-A_20260715_120000_DIFF.nwb
```

所有差异文件都直接指向同一个完整备份：

```text
FULL-01
├── DIFF-01
├── DIFF-02
└── DIFF-03
```

恢复 `DIFF-03` 只需要：

```text
FULL-01 + DIFF-03
```

不需要 `DIFF-01` 或 `DIFF-02`。因此任意一个差异文件损坏或被删除，不会破坏其他差异恢复点。

### 3.2 “自恢复”的精确定义

- 完整归档是 **自包含（self-contained）**：数据与元数据全部位于自身。
- 差异归档是 **自描述（self-describing）**：它包含完整的格式、链身份、变更映射、删除/重命名记录、恢复配置和校验信息，但未变化的数据仍来自基准完整归档。
- 差异归档必须通过 `base_archive_uuid` 和 `base_root_hash` 精确绑定基准，绝不只依赖文件名或路径。
- 文件名可被用户修改；恢复器应扫描目录中的 `.nwb` Header，通过 UUID 匹配基准。
- 产品提供“导出为独立完整归档”功能，将 `FULL + DIFF` 合成为新的 Full 文件，但不修改原文件。

### 3.3 差异判定基准

差异始终相对完整备份产生。不得以最近差异为基准。

文件备份可使用：

- VSS 快照；
- USN Journal 作为性能加速提示；
- 文件身份、大小、时间戳和内容哈希复核；
- 基准 Full 内嵌的文件清单和内容摘要。

卷/磁盘备份可使用：

- 文件系统分配位图；
- 操作系统变化跟踪作为加速提示；
- 基准 Full 内嵌的区块摘要；
- 必要时回退到逐块读取和哈希比较。

外部变化日志只能加速备份，不能成为恢复依赖，也不能在丢失或截断时影响正确性。

---

## 4. 总体架构

```text
Snapshot Provider
      │
      ▼
Source Collector ──► Raw Extent Stream
      │
      ▼
Change Detector / Differential Planner
      │
      ▼
Chunker ─► Hash ─► In-Archive Dedupe ─► Compression ─► Encryption
      │
      ▼
Segment Writer ─► Catalog/Extent Map ─► Manifest ─► Commit Footer
      │
      ▼
single-file.nwb.tmp ──atomic commit──► single-file.nwb
```

### 4.1 模块边界

| 模块 | 职责 |
|---|---|
| Snapshot Provider | 创建一致性快照，隔离备份期间的源端变化。 |
| Collector | 读取文件、卷或磁盘，生成带逻辑地址的 Raw Extent Stream。 |
| Differential Planner | 将当前快照与基准 Full 比较，产生 ADD/REPLACE/DELETE/ZERO/HOLE 操作。 |
| Chunk Engine | 将数据划分为逻辑块，计算内容摘要。 |
| Archive Writer | 聚合记录、压缩、加密并顺序写入 Segment。 |
| Metadata Builder | 生成文件目录、卷布局、磁盘布局、BMR 和异机还原元数据。 |
| Integrity Builder | 生成记录哈希、Segment 哈希和归档 Merkle Root。 |
| Archive Reader | 安全解析、随机读取、验证、浏览和恢复。 |
| Salvage/Repair | 扫描完整 Segment，重建索引；修复时只生成新归档。 |

---

## 5. NWB物理容器、单文件与多分卷格式

### 5.1 单物理文件总体布局

```text
┌──────────────────────────────────────────┐
│ Bootstrap Header A (4 KiB)               │
├──────────────────────────────────────────┤
│ Bootstrap Header B (4 KiB)               │
├──────────────────────────────────────────┤
│ Archive Metadata                         │
├──────────────────────────────────────────┤
│ Segment 0                                │
│  Header + Records + Directory + Trailer  │
├──────────────────────────────────────────┤
│ Segment 1                                │
├──────────────────────────────────────────┤
│ ...                                      │
├──────────────────────────────────────────┤
│ Catalog / Disk Map / BMR Metadata         │
├──────────────────────────────────────────┤
│ Chunk Index                              │
├──────────────────────────────────────────┤
│ Recovery Manifest Copy A                 │
├──────────────────────────────────────────┤
│ Recovery Manifest Copy B                 │
├──────────────────────────────────────────┤
│ Commit Footer A (4 KiB)                  │
├──────────────────────────────────────────┤
│ Commit Footer B (4 KiB, final block)     │
└──────────────────────────────────────────┘
```

Header、Manifest 和 Footer 均保留两份，副本相距足够远，降低局部介质损坏同时破坏两个副本的概率。

### 5.2 Bootstrap Header

Header 采用固定 4096 字节、Little Endian，至少包含：

| 字段 | 说明 |
|---|---|
| magic | `NWBARCH\0` |
| format_major/minor | 格式主次版本 |
| header_size | Header 长度 |
| archive_uuid | 当前归档 UUID |
| chain_uuid | 备份链 UUID |
| backup_type | FULL 或 DIFFERENTIAL |
| source_type | FILE_SET、VOLUME、DISK、SYSTEM |
| base_archive_uuid | Full 为零；Diff 指定基准 Full |
| created_at_utc | UTC 时间 |
| logical_sector_size | 逻辑扇区大小 |
| physical_sector_size | 物理扇区大小 |
| feature_required | 读取器必须理解的特性位 |
| feature_optional | 可安全忽略的特性位 |
| crypto_suite | 加密套件 |
| key_slots | 密钥槽描述 |
| header_sequence | 副本序号 |
| header_hash | Header 自身 SHA-256 |

Header 不信任任何外部输入。Reader 在分配内存或 seek 前必须验证长度、范围、对齐、重复项和溢出。

### 5.3 Segment

默认 Segment 目标大小为 **128 MiB**，允许在 64–256 MiB 范围内配置。Segment 是损坏隔离、并行处理和可恢复扫描的基本单位。

每个 Segment 包含：

```text
Segment Header
Record 0
Record 1
...
Record N
Local Record Directory
Segment Trailer
Optional Recovery Parity
```

Segment Header/Trailer 保存：

- Segment ID 和顺序号；
- 起始与结束偏移；
- 记录数量；
- 未压缩/压缩大小；
- Segment 内容哈希；
- 前一 Segment 哈希，形成顺序哈希链；
- AEAD 状态；
- 可选纠删码参数；
- Trailer 校验值。

Segment 完全封闭后才进入索引。未封闭的尾部 Segment 不得被已提交 Manifest 引用。

### 5.4 Record

所有内容都使用统一的 framed record：

```text
Record Prefix
Payload
AEAD Tag / Payload Hash
Record Suffix
```

Record 类型包括：

- `DATA_CHUNK`
- `ZERO_RUN`
- `HOLE_RUN`
- `FILE_ENTRY`
- `FILE_EXTENT`
- `DISK_EXTENT`
- `PARTITION_LAYOUT`
- `BMR_ARTIFACT`
- `CATALOG_PAGE`
- `CHUNK_INDEX_PAGE`
- `DIFF_OPERATION`
- `TOMBSTONE`
- `MANIFEST`

每条记录都有独立长度、序号、类型、逻辑身份和校验。损坏一条记录时，恢复器能够继续定位下一条记录，而不是失去整个归档。

---


### 5.5 逻辑Archive与物理Volume

正式抽象：

~~~text
Backup Run
→ Logical Archive
→ Volume Set
→ One or More Physical NWB Volumes
~~~

单文件模式：

~~~text
Logical Archive
└── Volume 0
    └── Backup_FULL.nwb
~~~

分卷模式：

~~~text
Logical Archive
├── Volume 0 → Backup_FULL_ab12_p000001.nwb
├── Volume 1 → Backup_FULL_ab12_p000002.nwb
├── Volume 2 → Backup_FULL_ab12_p000003.nwb
└── Final Volume → Set Manifest + SET_COMMITTED
~~~

在UI、Retention、历史记录和恢复点模型中，一个Volume Set始终只显示为一个Archive。

### 5.6 分卷触发模式

| 模式 | 行为 |
|---|---|
| Automatic | 根据文件系统、介质和目标能力自动决定 |
| No Split | 强制单文件；目标不满足时在写入前失败 |
| Fixed Size | 按用户指定上限分卷，并对齐Segment边界 |
| Media Aware | FUTURE：按光盘、磁带导出介质或对象后端能力规划 |

第一版默认Automatic。

Automatic策略：

~~~text
NTFS/ReFS/exFAT/现代本地文件系统
→ 默认单文件

FAT32
→ 自动安全分卷

已知受限NAS/SMB目标
→ 按探测或管理员策略分卷

无法可靠识别目标限制
→ 使用保守策略或要求用户确认
~~~

### 5.7 目标能力探测

Target Capability Probe至少返回：

~~~text
TargetCapabilities
├── filesystem_type
├── max_file_size
├── available_space
├── atomic_rename_support
├── durable_flush_support
├── sparse_file_support
├── random_read_support
├── random_write_support
├── recommended_part_size
├── network_target
└── capability_confidence
~~~

探测原则：

- 优先使用操作系统文件系统类型和目标能力；
- 对SMB/NAS不能只相信本地客户端文件系统名称；
- 不通过实际写满超大文件来测试上限；
- 允许产品维护已验证目标能力表；
- 不确定时允许用户选择Fixed Size；
- 预计Archive超过限制时必须在备份开始前提示；
- 不能运行到文件系统上限后才临时失败。

### 5.8 FAT32自动分卷基线

FAT32单文件最大值小于4GiB。NWB默认Segment为128MiB，因此v2.0建议FAT32自动分卷数据上限：

~~~text
30 × 128MiB
= 3840MiB
= 3.75GiB
~~~

选择3.75GiB的原因：

- 低于FAT32单文件上限；
- 正好容纳30个128MiB Segment；
- 不切断Segment；
- 为Volume Header/Footer和实现差异保留空间；
- 便于估算和恢复介质处理。

该值属于BASELINE，Binary Format不应把3.75GiB写成唯一合法大小。

### 5.9 分卷命名

建议：

~~~text
Workstation_20260713_FULL_ab12cd_p000001.nwb
Workstation_20260713_FULL_ab12cd_p000002.nwb
Workstation_20260713_FULL_ab12cd_p000003.nwb
~~~

原则：

- 所有分卷保持 .nwb 扩展名；
- Volume Index使用固定宽度十进制；
- 文件名包含Archive短标识仅用于人工区分；
- 文件名不是身份真相；
- 用户重命名分卷后仍通过Header识别；
- 双击任意分卷应启动Nüwa并发现同一Set；
- 不使用普通ZIP/RAR多卷格式。

不建议：

~~~text
backup.nwb
backup.nwb.001
backup.nwb.002
~~~

因为双扩展名会降低文件关联和用户识别一致性。

### 5.10 Volume Header

每个分卷都有独立Bootstrap/Volume Header：

~~~text
NWB Volume Header
├── Magic
├── NWB Format Major/Minor
├── Archive UUID
├── Chain UUID
├── Backup Type
├── Base Archive UUID
├── Volume Set UUID
├── Volume Index
├── First Segment ID
├── Previous Volume Root Hash
├── Required/Optional Features
├── Crypto Suite
├── Key Slot Descriptors
├── Header Sequence
└── Header Hash
~~~

单文件模式也必须填写Volume Set UUID和Volume Index=0，使Reader无需两套解析逻辑。

### 5.11 Volume Footer

每个完成写入的分卷必须独立Seal：

~~~text
NWB Volume Footer
├── Archive UUID
├── Volume Set UUID
├── Volume Index
├── First/Last Segment ID
├── Segment Count
├── Stored Bytes
├── Volume Root Hash
├── Previous Volume Root Hash
├── Sealed Flag
├── Footer Sequence
└── Footer Hash
~~~

已Seal的Volume不得继续追加。下一条完整Segment写入下一个Volume。

### 5.12 Chunk和Segment边界

冻结原则：

~~~text
Chunk Record不能跨Segment
Segment不能跨Physical Volume
Catalog Page不能跨Physical Volume
Index Page不能跨Physical Volume
~~~

当当前Volume剩余空间不足以容纳下一个完整Segment或Page时：

1. 完成当前结构；
2. 写Local Directory；
3. 写Volume Footer；
4. Flush并Seal；
5. 创建下一个Volume；
6. 从新Volume写完整结构。

禁止把一个Segment前半段写在Part N、后半段写在Part N+1。

### 5.13 Volume Set哈希链

~~~mermaid
flowchart LR
    A["Volume 0<br/>Root A"] --> B["Volume 1<br/>Prev=A"]
    B --> C["Volume 2<br/>Prev=B"]
    C --> D["Final Set Manifest<br/>Archive Root"]
~~~

每个Volume Header/Footer保存前一Volume Root Hash。Final Set Manifest保存全部Volume摘要和Set Merkle Root。

这可以检测：

- 分卷缺失；
- 顺序错误；
- 混入其他Archive分卷；
- 单个分卷被替换；
- 重复Volume Index；
- Set截断。

### 5.14 Final Volume Set Manifest

最终分卷保存：

~~~text
VOLUME_SET_MANIFEST
├── Archive UUID
├── Chain UUID
├── Backup Type
├── Base Archive UUID/Root
├── Volume Set UUID
├── Total Volume Count
├── Volume Entries[]
│   ├── Volume Index
│   ├── Expected Size
│   ├── Volume Root Hash
│   ├── First/Last Segment ID
│   ├── Catalog/Index Range
│   └── Optional File Name Hint
├── Catalog Root
├── Extent Root
├── Chunk Index Root
├── Data Root
├── Metadata Root
├── Volume Set Merkle Root
├── Archive Root Hash
└── SET_COMMITTED
~~~

File Name Hint不参与身份和正确性判断。

### 5.15 多文件提交协议

文件系统通常不支持“多个文件一次原子Rename”。NWB采用Final Volume作为提交权威：

~~~text
写Volume 0.tmp
→ Seal + Flush

写Volume 1.tmp
→ Seal + Flush

...

写Final Volume.tmp
→ Set Manifest
→ SET_COMMITTED
→ Flush

发布普通Volumes
→ 最后发布Final Volume
~~~

Archive有效条件：

~~~text
Final Volume存在
AND SET_COMMITTED有效
AND Volume Count匹配
AND 所有Required Volume存在
AND Volume Roots匹配
AND Archive Root匹配
~~~

前面Volume已经发布但Final Volume不存在时，状态为Incomplete/Uncommitted，不显示为成功恢复点。

### 5.16 提交崩溃恢复

| 崩溃位置 | 结果 |
|---|---|
| 当前Volume未Seal | 当前tmp不可用；已Seal Volume可供Salvage |
| 普通Volume尚未发布 | Archive未提交 |
| 普通Volume部分发布 | Incomplete，Final Volume尚未发布 |
| Final Volume写完但未Rename | 未提交，可由事务恢复器验证后发布 |
| Final Volume已发布 | 必须完整验证Set Manifest |
| Cache尚未更新 | Archive有效；Cache可重建 |

事务日志只能帮助恢复发布流程，Set Commit本身必须位于NWB内部。

### 5.17 Catalog和Index跨分卷布局

Catalog、Extent和Index仍使用Page结构，可以位于一个或多个末尾Volume：

~~~text
Volumes 0..N
→ Data Segments

Volumes N+1..M
→ Catalog Pages
→ Extent Pages
→ Chunk Index Pages

Final Volume
→ Roots
→ Volume Set Manifest
→ SET_COMMITTED
~~~

Page不得跨Volume。Set Manifest记录Page Range到Volume/Offset的映射。

重新导入时：

1. 读取任意Volume Header；
2. 取得Volume Set UUID；
3. 扫描同目录候选NWB；
4. 找到Final Volume；
5. 读取Set Manifest；
6. 校验Volume集合；
7. 定位Catalog Root；
8. 写Catalog Cache。

不需要读取全部Data Segment。

### 5.18 Catalog Cache中的Volume Set

Catalog Cache建议增加：

~~~sql
archive_volumes(
  archive_uuid,
  volume_set_uuid,
  volume_index,
  expected_size,
  volume_root_hash,
  path,
  availability,
  first_segment_id,
  last_segment_id,
  last_checked_at
)
~~~

Cache可以记录路径和可用性，但Volume Set Manifest仍是Archive成员关系的恢复真相。

### 5.19 缺失分卷状态

示例：

~~~text
Volume 0–10  Present
Volume 11    Missing
Volume 12–53 Present
~~~

报告：

~~~text
Archive Status: Incomplete
Missing Volume Index: 11
Missing Segment Range: 330–359
Catalog Availability: Available/Partial/Unavailable
Selected Object Recoverability: Computed Per Object
Full/BMR Recoverability: Blocked
~~~

### 5.20 选择性文件恢复

如果Archive不完整，但所选文件的Catalog、Extent和所有Chunk都位于可用Volumes中，可以提供选择性恢复：

~~~text
Archive Health: Incomplete
Selected File: Fully Recoverable
Full Restore: Unavailable
~~~

必须满足：

- Internal Catalog足以解析对象；
- 所有Extent可定位；
- 所需Volume Root通过；
- 所有Chunk Hash通过；
- UI明确显示不完整状态；
- Restore Report记录缺失Volumes。

不能使用Search Index替代损坏或缺失的Internal Catalog真相。

### 5.21 BMR和整卷恢复预检

BMR、整盘和整卷恢复开始任何破坏性写入前，必须：

1. 解析完整Restore Plan；
2. 计算所有Required Volumes；
3. 验证Volume Set Manifest；
4. 检查每个Required Volume存在；
5. 验证Volume Header/Footer和Root；
6. 验证Full/Diff Base关系；
7. 检查解密密钥；
8. 检查目标容量；
9. 生成Preflight Report；
10. 确认不存在Missing Required Volume。

缺少Required Volume时必须阻止BMR开始，不能恢复到一半才提示缺卷。

### 5.22 加密与Nonce

一个逻辑Archive使用同一Archive Master Key。每个Volume复制或引用足以解锁Archive的Key Slot Descriptor。

Nonce域必须包含：

~~~text
Archive UUID
+ Volume Index
+ Segment ID
+ Record Sequence
~~~

这保证相同Master Key下跨Volume不重复nonce。

每个Volume Header中的Key Slot信息使Reader可以：

- 识别该Volume属于加密归档；
- 请求正确凭据；
- 执行局部Salvage；
- 校验Volume身份。

### 5.23 Full与Diff分卷关系

Full和Diff分别是独立Volume Set：

~~~text
Full Logical Archive
├── Full Volume 0
├── Full Volume 1
└── Full Final Volume

Diff Logical Archive
├── Diff Volume 0
└── Diff Final Volume
~~~

Diff引用的是Base Full Archive UUID和Root Hash，不引用固定的Full Volume编号。Chunk Index Resolver负责把Base Chunk定位到具体Full Volume。

### 5.24 Retention、移动、复制和删除

所有操作以Logical Archive为单位：

| 操作 | 规则 |
|---|---|
| Validate | 验证Set Manifest和Required Volumes |
| Move | 移动全部Volumes并在目标重新验证 |
| Copy | 全部复制完成后才发布目标Final Volume |
| Delete | 删除Archive全部Volumes |
| Retention | 按Archive/Chain处理，不按Part处理 |
| Export | 输出完整新Archive或新的Volume Set |
| Rename | 可以重命名，但身份依赖Header |
| Discover | 通过Volume Set UUID重新分组 |

用户在文件管理器手工删除一个Volume后，Cache必须将整个Archive标为Incomplete。

### 5.25 分卷配置模型

~~~text
ArchiveSplitPolicy
├── mode = Automatic / NoSplit / FixedSize
├── fixed_size_bytes
├── align_to_segment = true
├── target_capability_policy
├── minimum_part_size
├── maximum_part_count
└── incomplete_set_cleanup_policy
~~~

Fixed Size必须：

- 大于最小Segment/Page开销；
- 对齐Segment边界；
- 小于目标最大文件限制；
- 使用64位大小；
- 在开始备份前计算预计Part Count；
- Part Count过大时警告用户。

### 5.26 第一版默认策略

| 项目 | 第一版基线 |
|---|---|
| 默认模式 | Automatic |
| NTFS/ReFS/exFAT | 默认不拆分 |
| FAT32 | 自动约3.75GiB分卷 |
| 用户Fixed Size | 支持 |
| Segment跨Volume | 禁止 |
| Page跨Volume | 禁止 |
| 每Volume独立Hash | 必须 |
| Set Manifest | 必须 |
| Final Volume最后发布 | 必须 |
| 双击任意Volume识别Set | 必须 |
| BMR前完整Volume预检 | 必须 |
| 不完整Archive选择性文件恢复 | 可支持，必须警告 |
| 外部Sidecar Manifest | 禁止成为恢复依赖 |

### 5.27 Acronis参考边界

Acronis Cyber Backup公开文档提供Automatic和Fixed Size两种拆分方式；Automatic会在备份超过目标文件系统最大文件大小时拆分。[Acronis Cyber Backup 12.5 User Guide](https://dl.acronis.com/u/pdf/AcronisCyberBackup_12.5_userguide_en-US.pdf)

Nüwa借鉴其产品行为，但Volume Header、Set Manifest、提交协议、Hash链和命名是Nüwa自主设计。

---

## 6. 数据粒度、Chunk、Segment 与归档内去重

“一个块多大”不能用单一数值回答。NWB 将源介质粒度、变化检测粒度、逻辑 Chunk 和物理 Segment 严格分开。

### 6.1 四级粒度模型

| 层级 | v2.0 默认 | 职责 | 状态 |
|---|---:|---|---|
| 物理扇区/文件系统簇 | 记录源真实值，常见 4KB | 磁盘语义、对齐、BMR | FROZEN |
| 变化检测 | 4KB–64KB | 判断哪些区间发生变化 | BASELINE |
| 数据 Chunk | 256KB | 哈希、压缩、加密、随机恢复 | BASELINE |
| Segment | 128MB | 顺序写、批量 I/O、损坏隔离 | BASELINE |
| Catalog Page | 64KB | 目录分页读取与局部损坏隔离 | BASELINE |

变化检测粒度小，并不代表为每个 4KB 建立一个永久 Chunk。推荐路径：

~~~text
4KB/64KB 变化位图
→ 合并连续变化范围
→ 按 256KB Chunk 处理
→ 聚合进 128MB Segment
→ 写入一个 NWB 文件
~~~

### 6.2 为什么不把 4KB 当作统一 Chunk

1TB 数据按 4KB 切分会产生 268,435,456 个 Chunk。仅保存每块 32 字节 SHA-256 就约 8GB，还没有计算 Offset、Length、Flags、数据库页和内存 Hash Table。

Acronis 旧版企业托管去重仓库公开过磁盘级 4KB 去重粒度，是因为背后有 Storage Node、DDB、SSD 和内存索引。Nüwa 是单机单归档产品，不能只复制 4KB 数值而忽略其索引基础设施。

### 6.3 为什么 v2.0 选择 256KB

- 相比 4KB，索引数量下降约 64 倍；
- 相比 1MB，文件和卷的差异精度更高；
- 对文件、卷和磁盘可以共用主要 Chunk 管线；
- 对单机工作站数据集较平衡；
- 读取仍可使用 8–32MB 聚合窗口；
- Segment 保持大块顺序写；
- 格式字段允许未来增加 512KB、1MB 或内容定义切块。

1TB 数据的理论 Chunk 数：

| Chunk | 数量 |
|---:|---:|
| 4KB | 268,435,456 |
| 256KB | 4,194,304 |
| 1MB | 1,048,576 |

### 6.4 文件切块规则

- 每个文件从偏移 0 独立对齐；
- 小于等于 256KB 的文件默认作为一个 Chunk；
- 大文件按 256KB 切分；
- 最后一个 Chunk 保存真实长度；
- 不把两个不同文件拼成一个逻辑 Chunk；
- ADS 作为独立命名 Stream 切块；
- ZERO 和 HOLE 使用 Run Record；
- 相同内容通过归档内去重引用同一物理 Chunk Record。

示例：

~~~text
Report.docx 700KB
├── A1 256KB
├── A2 256KB
└── A3 188KB

Logo.png 100KB
└── B1 100KB
~~~

### 6.5 卷和磁盘切块规则

- 逻辑地址使用 64 位 Byte Offset/LBA；
- 变化检测可以比 Chunk 更细；
- 连续变化区间合并后再进入 Chunk Pipeline；
- 未分配区、零区、稀疏区和不可读区必须显式区分；
- MBR、GPT、引导区等关键小范围允许使用专用 Record；
- Used-Block 模式只保存已分配范围和必要文件系统元数据；
- Sector-by-Sector 模式保存全部选定扇区。

### 6.6 Chunk 处理流水线

~~~mermaid
flowchart TD
    A["源数据"] --> B["256KB Chunk"]
    B --> C["SHA-256"]
    C --> D["归档内去重"]
    D --> E["Zstd压缩"]
    E --> F["可选AEAD加密"]
    F --> G["Chunk Record"]
    G --> H["128MB Segment"]
    H --> I["单个NWB"]
~~~

### 6.7 Chunk Record

~~~text
Chunk Record
├── Record Prefix
├── Chunk ID
├── Raw Length
├── Stored Length
├── Compression
├── Crypto Suite
├── Content SHA-256
├── Compressed/Stored Payload
├── AEAD Tag 或 Payload Checksum
└── Record Suffix
~~~

每个 Chunk 独立压缩、认证、解密和恢复。压缩收益低于阈值时保存 Stored Payload。

### 6.8 Segment 聚合

~~~text
Segment 17
├── Segment Header
├── Chunk Record A1
├── Chunk Record A2
├── Chunk Record A3
├── Chunk Record B1
├── ...
├── Local Record Directory
└── Segment Trailer
~~~

128MB Segment 理论上可容纳约 512 个未压缩 256KB Chunk。Segment 目标大小不是强制精确值，Writer 应在完整 Record 边界封闭。

Segment Header/Trailer 至少保存：

- Segment ID 和顺序号；
- Previous Segment Hash；
- Record Count；
- Raw/Stored Bytes；
- Local Directory Offset；
- Segment Content Hash；
- Header/Trailer Hash；
- 可选纠错信息。

### 6.9 归档内去重

~~~text
content_hash = SHA-256(canonical_plaintext)
~~~

同一 NWB 内 Hash 和长度相同的 Chunk 可共享物理 Record。v2.0 不做跨归档物理块共享。

Diff 对 Full 的复用通过显式 Base Extent 完成，不需要全局 DDB，也不需要跨文件引用计数或 GC。

### 6.10 ZERO、HOLE、UNALLOCATED 与 UNREADABLE

| 类型 | 含义 | 恢复行为 |
|---|---|---|
| ZERO | 逻辑内容明确为零 | 写零或目标允许时优化 |
| HOLE | 稀疏文件未分配区 | 创建稀疏区 |
| UNALLOCATED | 文件系统未使用区 | Used-Block 恢复可跳过 |
| UNREADABLE | 源读取失败 | 按策略失败或明确标记 |

### 6.11 参数冻结条件

256KB Chunk、128MB Segment 和 64KB Catalog Page 是 v2.0 默认基线，但必须比较：

- Chunk 128KB/256KB/512KB/1MB；
- Segment 64MB/128MB/256MB；
- Catalog Page 32KB/64KB/128KB；
- SSD/HDD/USB/SMB；
- Office、系统盘、大文件、百万文件；
- Full/Diff、加密和压缩组合。

在 Binary Format Specification 冻结前允许调整默认值，但格式中必须显式记录算法与大小，Reader 不得依赖编译期常量。

---

## 7. 压缩

- 默认使用 Zstandard，默认等级建议 3。
- 每个 Chunk 独立压缩，支持随机恢复和局部修复。
- 压缩结果未节省至少约 1% 时，保存原始块并标记 `STORED`。
- 不对已压缩/加密格式做昂贵的重复压缩尝试，可使用快速采样判断。
- 压缩算法和参数记录在每个 Chunk Header 中，不能只记录在任务配置中。
- Reader 必须设置解压后大小上限，防止压缩炸弹和恶意归档。

---

## 8. 加密、认证与密钥恢复

### 8.1 加密模型

建议默认支持并优先使用：

```text
AES-256-GCM + HKDF-SHA-256
```

若平台无高性能 AES 指令，可在后续格式特性中增加 XChaCha20-Poly1305。不得使用仅加密不认证的模式。

### 8.2 密钥层次

```text
用户密码 / Recovery Key / Device Protector
                │
                ▼
           Key Encryption Key
                │ unwrap
                ▼
       Random Archive Master Key
          ├── Data Key
          ├── Metadata Key
          └── Nonce Derivation Key
```

- 每个归档生成独立的随机 Archive Master Key。
- 密码槽使用 Argon2id 派生 KEK，并保存 salt、内存、迭代和并行度参数。
- 可配置多个 Key Slot：用户密码、48 位恢复密钥、企业恢复证书。
- 归档中只保存被包装的 Master Key，不保存明文密钥。
- BMR 介质必须包含解锁实现；不得依赖 Windows 凭据库才能恢复。

### 8.3 Nonce 安全

- 每个加密记录使用唯一 nonce。
- nonce 由归档随机前缀与单调 Record Sequence 组合或安全派生。
- Record Sequence 永不复用；失败重试必须重新创建归档 UUID 和 Master Key。
- AEAD Additional Data 包含 Archive UUID、Segment ID、Record Type、Record Sequence 和逻辑身份，防止记录被跨位置替换。

### 8.4 签名

可选的归档签名覆盖 Commit Manifest 的 Merkle Root。签名用于证明归档未被未授权修改，不能替代 AEAD 和块校验。

---

## 9. 三层 Catalog 与恢复索引架构

### 9.1 冻结决策：NWB-ADR-001

~~~text
第一层：NWB Internal Catalog
    负责可靠恢复，是归档内的恢复真相

第二层：Local Catalog Cache
    负责应用启动、备份列表和日常浏览速度

第三层：Optional Search Index
    负责跨备份、跨恢复点全文搜索
~~~

这三层不是三份同等地位的数据库。Internal Catalog 是必须保存且受完整性保护的核心数据；另外两层都可以删除并重新生成。

### 9.2 职责矩阵

| 数据 | Internal Catalog | Catalog Cache | Search Index |
|---|---:|---:|---:|
| Archive/Chain/Base UUID | 必须 | 缓存 | 可选 |
| Archive Root Hash | 必须 | 缓存并用于失效 | 可选 |
| 完整目录树 | 必须 | 热页面可缓存 | 建立搜索项 |
| 文件元数据 | 必须 | 摘要可缓存 | 部分字段 |
| Extent Map | 必须 | 禁止唯一保存 | 否 |
| Chunk 位置 | 必须 | 禁止唯一保存 | 否 |
| BMR 信息 | 必须 | 摘要可缓存 | 否 |
| 恢复点时间线 | 必须可推导 | 主要展示来源 | 可选 |
| 验证结果 | Manifest/报告 | 缓存最近结果 | 否 |
| 跨备份搜索 | 可逐归档查询 | 不负责 | 负责 |
| 删除后是否可重建 | 不适用 | 是 | 是 |

### 9.3 三层不变量

1. 删除 catalog-cache.db 不影响任何恢复；
2. 删除 catalog-search.db 不影响归档浏览和恢复；
3. Boot Media 不需要桌面端数据库；
4. Cache 条目必须通过 Archive UUID + Archive Root Hash 绑定；
5. Cache 命中但 Root Hash 不匹配时必须失效；
6. Cache 不能在 Internal Catalog 损坏时伪造一个健康恢复点；
7. Extent Map、Chunk Index、BMR 和密钥槽不得只存于外部数据库；
8. 加密 NWB 的 Search Index 必须遵守文件名隐私策略。

### 9.4 NWB Internal Catalog 的职责

Internal Catalog 回答：

- 此恢复点包含哪些文件、目录、卷、分区和磁盘；
- 对象的路径、属性、权限、时间戳和 Windows 特殊语义；
- 文件或逻辑地址由哪些 Extent 组成；
- 每个 Extent 来自当前归档还是 Base Full；
- 每个 Chunk 在哪个 Segment/Offset；
- BMR 和异机还原需要哪些工件；
- 如何验证恢复点完整性。

### 9.5 Catalog、Extent Map、Chunk Index 的分工

~~~mermaid
flowchart TD
    A["File Catalog"] --> B["Extent Map"]
    B --> C["Chunk Index"]
    C --> D["Segment + Offset"]
~~~

| 层 | 映射 |
|---|---|
| Catalog | Path/Object ID → Metadata + Extent List |
| Extent Map | Object + Logical Offset → Local/Base/Zero/Hole |
| Chunk Index | Chunk ID/Hash → Segment + Offset + Length |

### 9.6 Catalog Entry

每个文件或目录条目至少保存：

- Archive-local Object ID；
- Source Object Key；
- Parent Object ID；
- 原始名称和规范化名称；
- Entry Type；
- Logical/Allocated Size；
- Created/Modified/Access/Change Time；
- Windows Attributes；
- Owner 与 Security Descriptor Reference；
- Hard Link Group；
- Stream Table Reference；
- Reparse Table Reference；
- Sparse Map Reference；
- Extent List Reference；
- File Content Root Hash；
- Feature Flags。

### 9.7 Source Object Key

用于 Full 与 Diff 之间识别对象：

| 平台 | 优先身份 |
|---|---|
| NTFS | Volume Identity + File Reference Number + Sequence |
| ReFS | Volume Identity + 平台稳定 File ID |
| Linux | Filesystem UUID + inode + generation |
| 无稳定 ID | 规范路径 + 类型 + 创建时间 + 辅助摘要 |

稳定 ID 不可用时，Rename 识别只是优化；正确性必须能够回退到 DELETE + ADD。

### 9.8 目录树表达

Catalog 使用 Parent Object ID，不为每条记录重复完整路径：

| Object ID | Parent ID | Name | Type |
|---:|---:|---|---|
| 1 | 0 | D: | Root |
| 10 | 1 | Documents | Directory |
| 11 | 10 | Report.docx | File |
| 12 | 10 | Images | Directory |
| 13 | 12 | Logo.png | File |

这支持父链重建、目录移动、硬链接和低重复字符串存储。

### 9.9 Catalog Page

v2.0 默认目标 Page 大小 64KB：

~~~text
Catalog Page
├── Page Header
│   ├── Page ID
│   ├── Schema Version
│   ├── Entry Count
│   ├── Min/Max Key
│   ├── Raw/Stored Size
│   ├── Compression/Crypto
│   └── Page Hash
├── Framed Entries
└── Page Trailer / AEAD Tag
~~~

每页独立压缩、加密和校验。Page 损坏不能导致全部 Catalog 不可定位。

### 9.10 不可变排序页

Catalog 采用类似只读 SSTable 的不可变页：

~~~text
Primary Key
= Parent Object ID
+ Normalized Name
+ Object ID
~~~

优势：

- 顺序生成；
- Page 级二分定位；
- 有界内存外排排序；
- Page Directory 可重建；
- 不携带 SQLite WAL 等运行时状态；
- Boot Media Reader 实现可控；
- 局部损坏边界明确。

### 9.11 Internal Catalog 索引

| 索引 | 映射 | 用途 |
|---|---|---|
| Directory Index | Parent ID + Name → Child ID | 目录浏览 |
| Object Index | Object ID → Entry Page/Offset | 元数据与父链 |
| Source Key Index | Source Object Key → Object ID | Diff 和 Rename |
| Extent Index | Object ID → Extent Page Range | 内容恢复 |
| Security Index | Security Hash → Descriptor | ACL 元数据去重 |
| Stream Index | Object ID → ADS | NTFS Stream |
| Chunk Index | Chunk ID → Segment/Offset | 数据读取 |
| Page Directory | Key Range → Page Offset | Page 定位 |

### 9.12 Catalog Root 与 Merkle

~~~text
Catalog Root
├── Entry Root Hash
├── Directory Index Root
├── Object Index Root
├── Source Key Index Root
├── Extent Root Hash
├── Security Root Hash
├── Stream Root Hash
├── Page Directory Root
└── Overall Catalog Merkle Root
~~~

Catalog Root 同时写入两份 Recovery Manifest。

### 9.13 可重建与不可重建边界

| 内容 | 能否重建 | 来源 |
|---|---|---|
| Page Directory | 是 | Page Header |
| Directory Index | 是 | Parent ID + Name |
| Object Index | 是 | Catalog Entry |
| Security Index | 是 | Security Records |
| Chunk Index | 是 | Segment Local Directory |
| 路径和文件属性 | 不能凭空重建 | Catalog Entry |
| Extent 逻辑顺序 | 不能凭空重建 | Extent Map |
| BMR 配置 | 不能凭空重建 | BMR Records |

因此 Entry、Extent 和 BMR Records 是核心备份数据，不是普通索引缓存。

### 9.14 Local Catalog Cache

建议位置：

~~~text
C:\ProgramData\Nüwa Backup\catalog-cache.db
~~~

可以使用 SQLite WAL。建议保存：

~~~sql
archives(
  archive_uuid,
  chain_uuid,
  archive_type,
  base_archive_uuid,
  archive_root_hash,
  path,
  file_size,
  created_at,
  source_type,
  format_major,
  format_minor,
  verify_status,
  last_seen_at
)

chains(
  chain_uuid,
  full_archive_uuid,
  display_name,
  source_machine,
  restore_point_count
)

restore_points(
  archive_uuid,
  chain_uuid,
  created_at,
  backup_type,
  logical_bytes,
  stored_bytes,
  catalog_object_count
)

catalog_hot_pages(
  archive_uuid,
  archive_root_hash,
  page_id,
  page_hash,
  payload,
  last_access_at
)
~~~

Schema 只属于应用层缓存，不属于 NWB 永久格式。

### 9.15 Cache 启动和失效

启动：

~~~text
打开 Nüwa
→ 查询 Catalog Cache
→ 立即显示备份链和恢复点
→ 后台检查常用归档路径
→ 不扫描大型数据区
~~~

以下条件触发重新验证或失效：

- 路径变化；
- 文件长度变化；
- Archive UUID 不匹配；
- Root Hash 不匹配；
- 文件身份变化；
- Format Major 变化；
- Cache Schema 升级；
- 用户清除缓存；
- 检测到归档损坏。

### 9.16 Optional Search Index

建议位置：

~~~text
C:\ProgramData\Nüwa Backup\catalog-search.db
~~~

建议使用 SQLite FTS5，字段包括：

~~~sql
file_search(
  archive_uuid,
  chain_uuid,
  restore_point_time,
  object_id,
  normalized_path,
  file_name,
  extension,
  logical_size,
  modified_at,
  content_root_hash
)
~~~

Search Index 支持：

- 跨 Full/Diff 搜索；
- 文件名、扩展名、大小和时间过滤；
- 查找文件在哪些恢复点出现；
- 定位最近版本；
- 未来文件类型统计。

### 9.17 渐进式索引

1. 添加归档时只读 Header/Footer/Manifest；
2. UI 立即显示恢复点；
3. 第一次浏览时按需读取 Catalog Page；
4. 开启内容索引后后台分页读取；
5. 写 Search Index；
6. 中断后从 Page ID 检查点继续；
7. 删除 Search Index 后随时重建。

### 9.18 加密归档的搜索隐私

若 NWB 加密，文件名和路径也应默认加密。本地 Search Index 会产生解密后的路径副本，产品必须明确：

- 是否默认启用；
- 是否使用当前用户或系统保护；
- 企业策略是否禁止明文路径索引；
- 用户锁定后如何处理解密密钥；
- 移除备份记录时是否删除搜索条目。

### 9.19 Header-only Import

重新添加备份目录：

~~~mermaid
flowchart TD
    A["扫描NWB文件"] --> B["读取Header/Footer"]
    B --> C["验证UUID与Root"]
    C --> D["按Chain UUID分组"]
    D --> E["连接Full与Diff"]
    E --> F["写入Catalog Cache"]
    F --> G["立即显示恢复点"]
~~~

初始导入不读取 Data Segment。每个归档通常只读取双 Header、双 Footer、小型 Manifest 和必要 Root Page。

### 9.20 文件名不是身份

用户重命名或移动 NWB 后仍通过以下字段识别：

- Archive UUID；
- Chain UUID；
- Base Archive UUID；
- Archive Root Hash。

禁止凭相似文件名自动绑定 Full 与 Diff。

### 9.21 缺失 Full 时的 Diff 行为

由于 Diff 内保存完整逻辑 Catalog：

- 可以展示恢复点摘要；
- 可以浏览目录和文件元数据；
- 数据恢复状态必须标为不可用；
- 显示所需 Full UUID 和 Root Hash；
- 允许用户选择其他目录重新定位；
- 找到正确 Full 后重新验证 Base Extent。

---

## 10. 文件、Full Catalog 与 Differential Catalog

### 10.1 文件恢复语义

文件 Catalog 至少保存：

- 规范路径与原始大小写；
- 文件、目录、符号链接和 Reparse Point 类型；
- Source Object Key；
- 逻辑大小与分配大小；
- Created/Modified/Access/Change Time；
- Owner、DACL、SACL；
- DOS Attributes 和 Extended Attributes；
- Alternate Data Streams；
- Sparse Extents；
- Hard Link Group；
- Reparse Tag 和原始数据；
- EFS 状态；
- Extent List；
- 文件内容 Root Hash。

### 10.2 Full Catalog 生成

~~~mermaid
flowchart TD
    A["VSS一致性快照"] --> B["遍历对象"]
    B --> C["采集元数据"]
    C --> D["生成Object与Extent"]
    D --> E["临时Spool"]
    E --> F["外排排序"]
    F --> G["Catalog Pages"]
    G --> H["Indexes + Roots"]
~~~

不能把百万对象全部放入内存。推荐：

~~~text
扫描批次
→ 内存达到上限
→ 写临时有序 Run
→ 多路归并
→ 生成不可变 Catalog Pages
→ 写入 NWB
→ 删除临时工作区
~~~

临时 SQLite 或 Spool 可以用于构建，但不是恢复依赖。

### 10.3 Differential 输入

- 当前一致性快照；
- 对应 Full 的 Internal Catalog；
- 可选 USN/CBT 变化提示；
- 当前源对象枚举；
- Full Archive UUID 和 Root Hash。

外部 USN/CBT 只能加速，丢失、截断或不可信时必须回退到 Full Catalog 与当前快照比较。

### 10.4 变化类型

| 类型 | v2.0 处理 |
|---|---|
| ADD | 新 Entry + Local Extent |
| DELETE | 不出现在最终逻辑 Catalog；Change Summary 记录 |
| RENAME/MOVE | 更新 Parent/Name，保留 Source Object Key |
| METADATA_ONLY | 写当前元数据，内容 Extent 使用 Base |
| CONTENT_CHANGED | 变化 Chunk 写 Diff，未变化 Chunk引用 Full |
| TYPE_CHANGED | 旧对象删除 + 新对象创建 |
| HARDLINK_CHANGED | 更新 Link Group |
| ACL_CHANGED | 写新 Security Reference |

### 10.5 Diff 保存完整逻辑 Catalog

正式冻结：

> 每个 Diff 保存该时间点已经物化完成的完整逻辑 Catalog，而不是要求恢复器在浏览时实时合并 Full Catalog 与操作日志。

生成过程：

~~~text
Full Catalog
+ Current Snapshot
+ Change Operations
→ Materialized Logical Catalog
→ 写入 DIFF.nwb
~~~

这样 Diff 可以独立展示目录。实际数据仍然分为：

~~~text
LOCAL Extent → DIFF.nwb
BASE Extent  → FULL.nwb
ZERO/HOLE    → 逻辑记录
~~~

### 10.6 Base Extent

每个 Base Extent 至少绑定：

- Base Archive UUID；
- Base Archive Root Hash；
- Source ID；
- Logical Offset；
- Raw Length；
- Chunk Content Hash；
- Base Chunk Identity。

恢复前校验 Full Root，读取后校验 Chunk Hash。不得从其他 Diff 静默寻找替代数据。

### 10.7 Rename 与目录移动

NTFS 优先使用 Volume Identity + File Reference + Sequence 识别同一对象。

稳定身份相同而 Parent/Name 改变时记录 Rename/Move。移动包含大量子项的目录时，不必为每个后代生成路径级删除和新增；完整逻辑 Catalog 通过父关系反映新位置。

若稳定身份不可靠，则回退到 DELETE + ADD，不能为了提高 Rename 命中率牺牲正确性。

### 10.8 文件恢复顺序

1. 创建目录骨架；
2. 创建普通文件和稀疏布局；
3. 写主数据流；
4. 写 ADS；
5. 建立 Hard Link；
6. 创建 Reparse Point；
7. 应用 Owner/ACL/SACL；
8. 最后恢复时间戳与 Attributes；
9. 执行文件级 Root Hash 验证。

目标文件系统不支持某种源语义时，不得静默丢弃，应提供 Fail、Best-effort 和导出元数据报告策略。

### 10.9 浏览与恢复路径

浏览 Diff：

~~~text
Diff Footer
→ Diff Manifest
→ Diff Catalog Root
→ Diff Directory Page
→ 直接显示目录
~~~

恢复文件：

~~~mermaid
flowchart TD
    A["选择文件"] --> B["Diff Catalog Entry"]
    B --> C["Extent Map"]
    C --> D{"Extent来源"}
    D -->|LOCAL| E["读取Diff"]
    D -->|BASE| F["读取Full"]
    E --> G["认证/解密/解压"]
    F --> G
    G --> H["重建文件"]
~~~

### 10.10 Catalog 空间权衡

Diff 保存完整逻辑 Catalog 会增加元数据，但对单机产品通常可接受。若压缩后平均每个对象约 100–300 字节，50 万对象约为 50–150MB。它换来：

- Diff 快速独立浏览；
- Cache 丢失后直接恢复；
- Boot Media 简化；
- 无需实时合并大型 Catalog；
- 明确损坏边界。

Nüwa 当前不面向海量小文件数据中心，因此优先选择可靠性和恢复体验。

---

## 11. 卷、磁盘、BMR、UEFI与异机还原

### 11.1 第一版产品能力定位

BMR和异机还原不是v2之后的未来增强，而是Nüwa Backup第一版的标准能力和发布门槛。

第一版必须覆盖：

| 能力 | 第一版要求 |
|---|---|
| 非系统卷恢复 | 必须 |
| 整块磁盘恢复 | 必须 |
| Windows系统盘BMR | 必须 |
| Legacy BIOS/MBR恢复 | 必须 |
| UEFI/GPT恢复 | 必须 |
| 原机原盘恢复 | 必须 |
| 原机更换磁盘 | 必须 |
| 不同容量磁盘恢复 | 必须，受已用空间和布局约束 |
| 异机还原 | 必须 |
| SATA/AHCI/NVMe/RAID驱动适配 | 必须 |
| UEFI Secure Boot检测 | 必须 |
| BitLocker状态与恢复策略 | 必须 |
| 物理机到虚拟机 | 第一版验收矩阵必须覆盖至少一种目标虚拟化平台 |
| BIOS与UEFI互转 | 可分级支持；不得在未验证时宣称支持 |

第一版的定义不是“能够把系统盘字节写回”，而是：

> 从NWB启动恢复介质，在原Windows完全不可用的情况下，恢复磁盘布局、系统卷、启动环境和必要驱动，并通过实际启动验证。

### 11.2 UEFI在架构中的位置

UEFI不是普通文件类型，也不是NWB Core应直接理解的业务逻辑。它属于System/BMR Provider中的一级Boot Environment能力。

~~~mermaid
flowchart TD
    A["UEFI Firmware"] --> B["NVRAM Boot Entries"]
    B --> C["GPT Disk Layout"]
    C --> D["EFI System Partition"]
    D --> E["BCD + Boot Manager"]
    E --> F["Windows System Volume"]
~~~

NWB Core负责可靠保存Record、Chunk、Catalog和Hash；UEFI Provider负责理解、采集、规划、重建和验证启动环境。

### 11.3 固件边界

必须区分磁盘上的UEFI启动数据和主板固件数据：

| 内容 | 位置 | Nüwa行为 |
|---|---|---|
| Protective MBR | 磁盘LBA 0 | 保存并解析 |
| Primary GPT | 磁盘头部 | 保存并解析 |
| Backup GPT | 磁盘尾部 | 保存并解析 |
| EFI System Partition | 磁盘FAT32分区 | Raw + Semantic双重保存 |
| Windows Boot Manager | ESP文件 | 保存、校验、必要时重建 |
| BCD Store | ESP文件 | 保存原始数据并语义解析 |
| MSR | GPT分区 | 保存布局，不当作普通文件系统 |
| Windows/Recovery | 磁盘分区 | 保存 |
| BootOrder/Boot#### | 主板NVRAM | 采集用于诊断，目标端重建 |
| Secure Boot PK/KEK/db/dbx | 固件变量 | 默认不跨硬件迁移 |
| TPM封装密钥 | TPM芯片 | 不能直接复制 |
| UEFI固件代码 | 主板Flash | 不属于磁盘BMR范围 |

Nüwa恢复的是UEFI启动环境，不负责备份或刷写主板固件代码。

### 11.4 模块化Provider结构

~~~text
providers/
└── system_bmr/
    ├── system_provider/
    ├── disk_layout_provider/
    │   ├── gpt_provider/
    │   └── mbr_provider/
    ├── boot_environment_provider/
    │   ├── windows_uefi_provider/
    │   └── windows_legacy_bios_provider/
    ├── security_boot_provider/
    │   ├── secure_boot_inspector/
    │   ├── bitlocker_inspector/
    │   └── tpm_inspector/
    ├── recovery_environment_provider/
    │   └── windows_re_provider/
    └── hardware_adaptation_provider/
        ├── storage_driver_injector/
        ├── bcd_rebuilder/
        └── first_boot_preparer/
~~~

依赖方向：

~~~text
BMR/UEFI Provider
→ Provider SDK
→ Coordinator
→ NWB Core
~~~

NWB Core不得直接调用BCDBoot、REAgentC、DISM或UEFI变量API。

### 11.5 Boot Environment Provider接口

~~~rust
trait BootEnvironmentProvider {
    fn provider_info(&self) -> ProviderInfo;

    fn detect(
        &self,
        source: &SystemSource,
    ) -> Result<BootEnvironmentProfile>;

    fn capture(
        &self,
        session: &SnapshotSession,
    ) -> Result<BootEnvironmentArtifacts>;

    fn build_restore_plan(
        &self,
        source: &BootEnvironmentProfile,
        target: &TargetHardwareProfile,
    ) -> Result<BootRestorePlan>;

    fn repair_boot(
        &self,
        plan: &BootRestorePlan,
    ) -> Result<BootRepairReport>;

    fn verify_bootability(
        &self,
        target: &RestoredSystem,
    ) -> Result<BootabilityReport>;
}
~~~

第一版内置：

~~~text
WindowsUefiBootProvider
WindowsLegacyBiosBootProvider
~~~

首版必须实现Linux UEFI、GRUB2、systemd-boot、fstab、LVM和initramfs Provider；这些Provider不得污染Windows Provider和NWB Core。未来ARM64和其他启动环境继续通过相同Provider边界扩展。

### 11.6 BMR领域Record

NWB格式必须允许以下Provider领域记录：

~~~text
BMR_PROFILE
BOOT_ENVIRONMENT_PROFILE
GPT_LAYOUT
MBR_LAYOUT
ESP_RAW_IMAGE
ESP_FILE_MANIFEST
BCD_STORE
UEFI_VARIABLE_SNAPSHOT
SECURE_BOOT_PROFILE
BITLOCKER_PROFILE
TPM_PROFILE
WINRE_PROFILE
BOOT_CRITICAL_DRIVER_MANIFEST
HARDWARE_PROFILE
RESTORE_COMPATIBILITY_PROFILE
~~~

每个Record都必须具备：

- Provider ID；
- Metadata Type；
- Schema Version；
- Required/Optional Features；
- Payload Length；
- Payload Hash；
- 加密与认证状态。

### 11.7 BOOT_ENVIRONMENT_PROFILE

建议字段：

~~~text
BootEnvironmentProfile
├── boot_mode = UEFI / LEGACY_BIOS
├── architecture = X64 / ARM64 / X86
├── os_version
├── os_edition
├── os_build
├── secure_boot_state
├── tpm_state
├── bitlocker_state
├── system_disk_identity
├── esp_partition_identity
├── windows_partition_identity
├── recovery_partition_identity
├── firmware_variable_support
├── boot_manager_path
└── provider_schema_version
~~~

Provider API版本、NWB格式版本和BMR Metadata Schema版本必须分别管理。

### 11.8 GPT双重保存

归档必须同时保存GPT原始数据和解析模型：

~~~text
GPT_LAYOUT
├── Protective MBR
├── Primary GPT Header
├── Primary Partition Entry Array
├── Backup Partition Entry Array
├── Backup GPT Header
├── Disk GUID
├── Partition GUIDs
├── Partition Type GUIDs
├── Starting/Ending LBA
├── Attributes
├── UTF-16 Partition Names
├── Header CRC
└── Entry Array CRC
~~~

原因：

- 原始数据支持原机精确恢复和取证；
- 解析模型支持不同容量磁盘重排；
- 主、备GPT提供介质结构冗余；
- Restore Planner必须验证范围、重叠、对齐和CRC；
- 不得只保存Windows当前分配的盘符视图。

微软要求UEFI系统盘至少包含ESP、MSR和承载Windows的基本数据分区；当前推荐布局通常是System、MSR、Windows、Recovery。[Microsoft Windows and GPT FAQ](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/windows-and-gpt-faq?view=windows-11), [UEFI/GPT partition layout](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/configure-uefigpt-based-hard-drive-partitions?view=windows-11)

### 11.9 EFI System Partition双重保护

ESP必须同时保存：

~~~text
EFI System Partition
├── ESP_RAW_IMAGE
└── ESP_FILE_MANIFEST
~~~

#### ESP_RAW_IMAGE

用途：

- 原机原盘精确恢复；
- 保留FAT32布局；
- 保留OEM和第三方启动文件；
- 保留未知但可能重要的EFI应用。

#### ESP_FILE_MANIFEST

至少包含：

~~~text
ESP_FILE_MANIFEST
├── \EFI\Microsoft\Boot\bootmgfw.efi
├── \EFI\Microsoft\Boot\BCD
├── \EFI\Boot\bootx64.efi
├── Fonts/Locale Resources
├── OEM EFI Files
├── File Size
├── SHA-256
├── Signature Information
└── File Attributes
~~~

用途：

- 异机创建新ESP；
- 不同大小ESP迁移；
- EFI文件级验证；
- Raw Image局部损坏时恢复关键文件；
- Secure Boot签名检查。

原机精确恢复可以优先使用Raw Image；异机恢复应优先采用语义重建。

### 11.10 ESP大小策略

不得把ESP永久写死为100MB。

~~~text
EspSizingPolicy
├── PreserveOriginal
├── MicrosoftRecommended
└── UserSpecified
~~~

新建ESP时必须考虑：

- 原ESP使用量；
- 逻辑扇区大小；
- 4Kn磁盘；
- OS版本；
- OEM文件；
- Secure Boot更新空间；
- 多启动场景。

微软当前部署示例通常使用200MB ESP，4Kn场景使用更大值；Nüwa应把这些作为版本化Restore Policy，而不是NWB格式常量。[Microsoft deployment sample scripts](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/oem-deployment-of-windows-desktop-editions-sample-scripts?view=windows-11), [Hard drives and partitions](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/hard-drives-and-partitions?view=windows-11)

### 11.11 BCD保存与重建

NWB保存：

~~~text
BCD_STORE
├── Original BCD Bytes
├── Original Hash
├── Windows Boot Manager Entry
├── Windows Boot Loader Entries
├── Windows Memory Tester
├── Device/OSDevice References
├── System Root
├── Recovery Sequence
├── WinRE Reference
├── Hypervisor/Debug Settings
└── Parsed Provider Metadata
~~~

策略：

| 场景 | BCD处理 |
|---|---|
| 原机原盘 | 可恢复原BCD，随后验证 |
| 原机新盘 | 优先重建 |
| 异机还原 | 必须优先重建 |
| Partition GUID变化 | 必须重建或修正 |
| BIOS到UEFI转换 | 创建新的UEFI BCD，不复制旧BIOS配置 |

BCD可能包含旧Disk/Partition GUID和设备路径，因此异机还原不得盲目复制。

### 11.12 UEFI NVRAM变量

采集：

~~~text
UEFI_VARIABLE_SNAPSHOT
├── BootCurrent
├── BootOrder
├── Windows Boot Manager Boot####
├── EFI Device Path
├── Description
├── Attributes
└── Variable Hash
~~~

这些信息用于：

- 诊断源机器；
- 识别原启动路径；
- 提示多启动环境；
- 构建目标恢复计划。

目标恢复原则：

- 原机原盘可以保留有效启动项；
- 原机新盘和异机不得照搬旧硬件Device Path；
- 目标端必须创建新的Windows Boot Manager启动项；
- 同时建立默认Fallback Boot Path；
- 失败时提供明确修复报告。

微软说明BCDBoot通常会创建NVRAM Windows Boot Manager项；使用/s指定系统分区时不创建该项，而是依赖默认EFI启动路径。[Microsoft BCDBoot options](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/bcdboot-command-line-options-techref-di?view=windows-11)

### 11.13 三种磁盘布局恢复策略

#### Exact Layout

适用于原机原盘或受控同容量新盘：

- 保留Disk GUID；
- 保留Partition GUID；
- 保留起止LBA；
- 保留原ESP/MSR/Recovery；
- 写回原始GPT后重新计算并验证CRC。

#### Adaptive Layout

适用于不同容量目标磁盘：

- 保留分区类型和逻辑身份；
- 根据已用空间调整分区；
- 保证对齐；
- 必要时移动Recovery；
- 更新GPT和BCD引用；
- 目标比源小时必须先完成可缩减性检查。

#### Standardized UEFI Layout

适用于异机或源布局异常：

~~~text
ESP
MSR
Windows
Recovery
Optional Data
~~~

优先遵循目标Windows版本的当前推荐布局。Recovery建议位于Windows分区之后，以利于后续WinRE扩容。

### 11.14 恢复模式与GUID策略

| 模式 | Disk/Partition GUID |
|---|---|
| Disaster Replace | 在无冲突时允许保留 |
| Same Machine New Disk | 可保留或按策略重建，随后重建BCD |
| Dissimilar Hardware | 优先生成目标安全布局并重建引用 |
| Side-by-Side Clone | 必须避免GUID冲突 |
| P2V | 根据虚拟硬件生成目标布局和启动配置 |

Restore Planner必须明确记录是否保留GUID，不能在代码中隐式决定。

### 11.15 UEFI BMR完整恢复流程

~~~mermaid
flowchart TD
    A["WinPE以UEFI启动"] --> B["识别目标硬件和磁盘"]
    B --> C["创建或恢复GPT"]
    C --> D["恢复Windows/Data/WinRE"]
    D --> E["恢复或重建ESP"]
    E --> F["重建BCD和NVRAM"]
    F --> G["驱动/BitLocker处理"]
    G --> H["Bootability Verification"]
~~~

详细步骤：

1. 确认恢复介质启动模式；
2. 解析源Boot Environment Profile；
3. 枚举目标UEFI、Secure Boot、TPM和存储控制器；
4. 按序列号、Location Path、Bus Type和容量确认目标盘；
5. 禁止仅凭Disk 0自动选择；
6. 创建Exact/Adaptive/Standardized布局；
7. 恢复Windows、数据和Recovery分区；
8. 依据模式恢复或重建ESP；
9. 从恢复后的Windows版本生成匹配的Boot Files；
10. 创建或修正BCD；
11. 创建目标NVRAM Windows Boot Manager；
12. 建立Fallback Boot Path；
13. 注入Boot-critical Storage Driver；
14. 修正离线Registry；
15. 重新注册WinRE；
16. 处理BitLocker恢复策略；
17. 校验GPT、ESP、BCD、EFI签名和驱动；
18. 生成Bootability Report；
19. 首次启动后执行Post-Restore Validation。

### 11.16 恢复介质启动模式检查

恢复UEFI系统时，第一版默认要求恢复介质以UEFI模式启动。

若用户以Legacy模式启动：

~~~text
Source boot mode: UEFI
Recovery media mode: Legacy BIOS
Action: Stop before destructive disk operations
User instruction: Reboot and select UEFI: Nüwa Recovery Media
~~~

禁止在模式不匹配时继续写盘后留下不可启动系统。

### 11.17 目标磁盘选择安全

在WinPE和多磁盘环境中，Disk 0不一定是目标盘。Provider必须使用：

- Device Serial；
- Model；
- Capacity；
- Bus Type；
- NVMe Namespace；
- Location Path；
- 用户确认；
- 源目标排除规则；
- 包含恢复介质的磁盘排除。

微软提示在UEFI、多ESP和WinPE环境中自动“系统磁盘”选择可能指向错误磁盘。[Microsoft Configure Multiple Hard Drives](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/configure-multiple-hard-drives?view=windows-11)

任何Clean/Partition操作前必须生成不可逆操作摘要并再次确认目标身份。

### 11.18 ESP与Windows Boot Manager重建

异机恢复推荐：

~~~text
创建FAT32 ESP
→ 从恢复后的Windows复制匹配版本启动文件
→ 创建BCD Store
→ 更新Windows Loader
→ 建立Fallback路径
→ 创建NVRAM启动项
~~~

可使用目标恢复环境中的BCDBoot，例如：

~~~text
bcdboot W:\Windows /s S: /f UEFI
~~~

但Provider必须理解：

- /s明确写入哪个ESP；
- 使用/s时通常不会创建NVRAM项；
- 还需要显式NVRAM注册或安全的第二阶段；
- 需要确保默认路径 \EFI\Boot\bootx64.efi 可用；
- 多ESP环境必须先隔离非目标磁盘，避免写错分区。

### 11.19 Secure Boot

保存：

~~~text
SECURE_BOOT_PROFILE
├── enabled/disabled
├── setup/user mode
├── architecture
├── Boot Manager Signature
├── EFI File Signature Summary
├── Policy Digest
└── Capture Time
~~~

原则：

- 默认不把源机器PK/KEK/db/dbx写入目标固件；
- 使用目标UEFI自己的信任数据库；
- 验证Windows Boot Manager签名；
- 验证Boot-critical Driver签名；
- 无法满足Secure Boot时明确提示；
- 禁止静默关闭Secure Boot并报告成功；
- Recovery Media自身必须具备可在Secure Boot环境启动的可信链。

### 11.20 TPM与BitLocker

TPM封装密钥与源硬件绑定，不能直接迁移。BitLocker必须明确两种模式。

#### 解锁后的逻辑备份

~~~text
BitLocker卷已解锁
→ 读取明文逻辑数据
→ 使用NWB加密保护
→ 恢复后在目标系统重新启用BitLocker
→ 绑定目标TPM
~~~

这是异机还原优先模式。

#### Raw Encrypted Volume备份

~~~text
保存原BitLocker密文扇区
→ 恢复后需要恢复密码/恢复密钥
→ 重新建立目标TPM保护器
~~~

保存：

~~~text
BITLOCKER_PROFILE
├── protection_status
├── encryption_method
├── volume_identity
├── protector_types
├── recovery_key_required
├── recovery_key_reference
└── backup_capture_mode
~~~

恢复密钥策略：

- Do Not Store；
- External Escrow Reference；
- Encrypted Dedicated Key Slot。

禁止把48位恢复密码以明文写入Catalog。微软说明BitLocker恢复可通过恢复密钥或恢复密码解锁，不要求原TPM继续可用。[Microsoft BitLocker FAQ](https://learn.microsoft.com/en-us/windows/security/operating-system-security/data-protection/bitlocker/faq)

### 11.21 WinRE

保存：

~~~text
WINRE_PROFILE
├── Recovery Partition Identity
├── winre.wim Path
├── ReAgent Configuration
├── Recovery Sequence
├── Windows Build
├── Image Hash
└── Partition Requirements
~~~

恢复后：

- 验证Recovery分区；
- 恢复winre.wim；
- 更新BCD Recovery Sequence；
- 使用目标Windows环境重新注册WinRE；
- 验证Windows RE可以启动；
- Recovery分区默认放在Windows分区之后。

微软建议将Windows RE置于专用分区并紧邻Windows分区之后。[Microsoft Windows RE reference](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/windows-recovery-environment--windows-re--technical-reference?view=windows-11)

### 11.22 Boot-critical Driver与异机注入

保存：

~~~text
BOOT_CRITICAL_DRIVER_MANIFEST
├── Service Name
├── Driver INF
├── SYS/CAT Files
├── Version
├── Architecture
├── Hardware IDs
├── Signature State
├── Start Type
└── Source Hash
~~~

异机还原阶段：

- 检测目标AHCI/NVMe/RAID/VirtIO控制器；
- 匹配内置或用户提供Driver Package；
- 验证架构和签名；
- 使用DISM等受支持方式离线注入；
- 修正SYSTEM Hive中的Boot-critical服务；
- 记录所有注入、跳过和失败项；
- Secure Boot开启时拒绝不合规的未签名Boot Driver。

### 11.23 原机、换盘、异机与P2V

| 场景 | 核心策略 |
|---|---|
| 原机原盘 | 优先精确布局；必要时修复BCD/NVRAM |
| 原机新盘 | 重建GPT、ESP和BCD；保留逻辑系统身份 |
| 更大磁盘 | 恢复后按策略扩容 |
| 更小磁盘 | 先检查已用空间、不可移动区和文件系统缩减能力 |
| 异机还原 | 标准化布局、驱动注入、BCD/NVRAM重建 |
| P2V | 注入虚拟控制器驱动，生成虚拟平台启动配置 |

“恢复数据完成”不等于“BMR成功”。BMR成功必须以目标系统实际启动和Post-Restore Validation为准。

### 11.24 Bootability Verification

写盘完成后至少验证：

- 目标以GPT标记；
- Primary/Backup GPT及CRC；
- ESP存在、FAT32可读；
- 必需EFI文件存在；
- BCD Store可解析；
- Loader指向正确Windows；
- NVRAM项或Fallback Path存在；
- Boot Manager签名状态；
- Boot-critical Driver存在；
- Windows分区和SYSTEM Hive可读；
- WinRE配置一致；
- BitLocker恢复条件明确；
- 未解决阻塞项为零。

输出：

~~~text
BootabilityReport
├── Ready / ReadyWithWarnings / Blocked
├── Firmware Mode
├── GPT Status
├── ESP Status
├── BCD Status
├── NVRAM Status
├── Secure Boot Status
├── Driver Status
├── BitLocker Status
├── WinRE Status
└── Required User Actions
~~~

### 11.25 Recovery Media第一版要求

第一版Recovery Media必须：

- UEFI x64启动；
- Secure Boot兼容；
- Legacy BIOS启动用于旧设备；
- 识别GPT/MBR；
- 识别SATA/AHCI/NVMe和常见RAID；
- 支持加载用户驱动；
- 读取本地盘、USB和SMB上的NWB；
- 解锁NWB；
- 浏览Internal Catalog；
- 执行Full/Diff恢复；
- 运行BCDBoot、DISM和WinRE修复；
- 访问UEFI变量；
- 输出可保存的恢复报告；
- 在破坏性操作前明确显示目标磁盘。

第一版建议基于Windows PE构建，以最大化Windows文件系统、驱动、BCD和恢复工具兼容性。

### 11.26 BIOS与UEFI转换边界

第一版必须稳定支持：

~~~text
UEFI/GPT → UEFI/GPT
BIOS/MBR → BIOS/MBR
~~~

以下路径必须独立标记能力等级：

~~~text
BIOS/MBR → UEFI/GPT
UEFI/GPT → BIOS/MBR
~~~

若未完成完整实验矩阵，不得在产品中宣称自动支持。BIOS转UEFI至少涉及：

- 目标转换为GPT；
- 创建ESP/MSR；
- 创建UEFI BCD；
- 驱动与OS版本兼容；
- Windows版本与架构限制；
- Secure Boot；
- BitLocker；
- Recovery分区重建。

### 11.27 UEFI与BMR错误模型

| 错误 | 处理 |
|---|---|
| 恢复介质启动模式错误 | 写盘前阻止 |
| 目标不支持UEFI | 阻止UEFI恢复或进入明确转换向导 |
| 找不到正确目标盘 | 阻止 |
| GPT范围重叠/CRC错误 | 重建计划或阻止 |
| ESP空间不足 | 调整布局或阻止 |
| BCD重建失败 | BMR失败，不报告成功 |
| NVRAM写入失败 | 尝试Fallback并报告警告/阻塞 |
| Secure Boot签名不合规 | 阻止或明确要求用户策略 |
| BitLocker缺少恢复密钥 | 阻止Raw加密卷恢复后的访问 |
| Boot Driver缺失 | 阻止首次启动或要求驱动 |
| WinRE注册失败 | 系统可启动时可警告，但不得隐藏 |
| Bootability Verification失败 | BMR状态为Blocked |

### 11.28 BMR与异机还原的不变量

1. BMR Provider负责启动语义，NWB Core只负责可靠存取；
2. 原始磁盘数据和语义Metadata同时保存；
3. UEFI NVRAM只作源诊断，目标端按硬件重建；
4. Secure Boot信任数据库默认不跨硬件覆盖；
5. TPM密钥不作为普通可复制数据；
6. BitLocker恢复策略在备份时即明确；
7. 驱动注入必须受签名和架构检查；
8. BMR完成标准是可启动性验证，不是写盘结束；
9. 恢复过程必须产出完整审计报告；
10. 第一版必须通过真实UEFI硬件和虚拟UEFI环境验收。

---

## 12. 崩溃一致性与原子提交

### 12.1 写入状态

```text
CREATING → CAPTURING → WRITING → INDEXING → VERIFYING → COMMITTING → COMMITTED
```

### 12.2 单文件提交流程

1. 在目标目录创建随机命名的 `.nwb.tmp`；
2. 写入双 Header 和初始 Metadata；
3. 顺序写入并逐个封闭 Segment；
4. 写入 Catalog、Extent Map 和 Chunk Index；
5. 写入两份 Recovery Manifest；
6. 写入 Commit Footer A；
7. Flush 用户态缓冲并执行文件 `fsync/FlushFileBuffers`；
8. 写入 Commit Footer B和单成员Volume Set Manifest；
9. 再次 Flush；
10. 原子重命名为最终 `.nwb`；
11. Flush 父目录元数据（平台支持时）。

只有 Footer B、Manifest Root、单成员Set Manifest和最终文件名全部有效，才显示为成功恢复点。

### 12.3 多分卷提交流程

1. 依次创建并Seal普通Volume临时文件；
2. 每个Volume独立Flush并记录Root Hash；
3. 在Final Volume中写入Catalog Root、Volume Set Manifest和SET_COMMITTED；
4. Flush Final Volume；
5. 先发布所有普通Volume；
6. 校验普通Volume已经位于最终位置；
7. 最后发布Final Volume；
8. Flush父目录元数据；
9. 重新打开Final Volume并验证Set Commit。

Final Volume是多分卷Archive的提交权威。Final Volume缺失时，即使普通Volume已经发布，Archive仍然是Incomplete/Uncommitted。

### 12.4 中断恢复

- `.tmp` 不自动作为恢复点；
- 工具可扫描其中已封闭 Segment，生成“可恢复数据报告”；
- 不允许在原 `.tmp` 上继续写入后伪装为原事务；
- Resume 如需支持，应创建新归档，并从源快照重新确认一致性。

---

## 13. 完整性、验证与可修复性

### 13.1 校验层级

| 层级 | 内容 | 用途 |
|---|---|---|
| Quick | Header、Footer、Manifest、索引页和链身份 | 秒级发现结构问题 |
| Standard | Quick + 全部 Segment/Record 密文认证 + 抽样解压哈希 | 定期健康检查 |
| Full | 解密、解压并验证全部 Chunk 明文 SHA-256，重放 Catalog/Extent | 关键备份验收 |
| Boot Test | 在隔离环境验证引导或执行自动恢复演练 | BMR 可用性验证 |

仅在写入过程中计算哈希，不足以证明介质已正确持久化。关键任务应支持备份完成后的读回验证。

### 13.2 Merkle 完整性树

```text
Chunk Hashes
    ↓
Segment Merkle Roots
    ↓
Data Root + Metadata Root
    ↓
Archive Root
    ↓
Commit Footer / Optional Signature
```

Merkle Root 用于快速定位损坏范围和验证归档结构，不能替代逐块读回。

### 13.3 可选恢复冗余

为“高可靠模式”提供归档内 Reed-Solomon 冗余，例如每组 32 个数据分片配 2 个校验分片，空间开销约 6.25%。它可修复有限的局部扇区或分片损坏。

必须明确：

- 冗余不能恢复被整体删除或严重截断的文件；
- 冗余不能替代 3-2-1 备份；
- 同一物理文件内的校验分片不能抵抗整盘故障；
- 默认“标准模式”可只检测，高可靠模式才启用修复冗余。

### 13.4 Salvage 原则

损坏恢复工具应：

1. 只读打开原文件；
2. 定位有效 Header；
3. 顺序扫描 Segment Magic 和 Trailer；
4. 验证每条 Record；
5. 从局部目录重建 Chunk Index；
6. 从 Catalog 页恢复可用命名空间；
7. 输出损坏区间和受影响对象清单；
8. 将可恢复内容写入新的 `.nwb` 或普通文件目录。

---

## 14. 安全威胁模型

### 14.1 需要防御

- 恶意构造的 `.nwb` 导致越界、整数溢出或内存耗尽；
- 归档记录替换、重排和跨归档复制；
- 密码暴力破解；
- 备份过程中断电、磁盘写满和 SMB 连接中断；
- 位翻转、坏扇区和部分文件截断；
- 勒索软件修改或删除归档；
- 错误的基准 Full 与 Diff 配对；
- 旧读取器误读新强制特性。

### 14.2 格式能做与不能做的事

格式可以做到：

- 密文认证、内容哈希和签名；
- 只追加写和提交后不可变；
- 精确检测错误基准；
- 局部损坏隔离；
- 可选局部纠错。

格式本身不能阻止拥有目录删除权限的恶意程序删除 `.nwb`。产品还必须提供：

- 目标目录最小权限；
- 备份进程与 UI 权限分离；
- Windows 服务账户隔离；
- 可选离线盘、NAS 快照或 WORM 目标；
- 备份完成后只读/不可变策略；
- 第二副本与 3-2-1-1-0 策略。

---

## 15. 性能与资源控制

### 15.1 流水线

```text
Async Read → Chunk → Hash → Compress → Encrypt → Ordered Segment Write
```

- 读取、哈希、压缩和加密可并行；
- 最终写入保持有界重排后的顺序 I/O；
- 所有队列必须有上限，禁止随备份规模增长；
- 内存目标按并发度配置，而不是按源数据量配置；
- 索引使用外排排序或临时工作库，最终封装进归档；
- 临时文件丢失只影响当前备份，不影响已有 `.nwb` 恢复。

### 15.2 目标文件系统

- 推荐 NTFS、ReFS、exFAT、ext4、XFS 或可靠 SMB/NAS 文件系统；
- 现代大型文件系统默认使用单Volume模式；
- FAT32通过Automatic模式使用约3.75GiB的Segment对齐分卷；
- NoSplit模式下，如果预计Archive超过目标单文件上限，必须在备份开始前失败；
- 创建任务时必须检查目标最大文件、剩余空间、稀疏文件行为、Flush和rename能力；
- 网络目标必须处理断连、短写、缓存未落盘、能力误报和rename语义差异；
- 无法可靠判断单文件上限时，应采用管理员策略、Fixed Size或保守分卷基线。

---

## 16. Retention 与链管理

Retention 规则：

- 任意 Diff 可以独立删除；
- 保留任意 Diff 时不得删除其 Full；
- 删除 Full 必须同时删除或先导出所有依赖 Diff；
- 文件名不是链关系真相；Archive UUID 和 Root Hash 才是；
- 归档不做原位空间回收；
- 合并、导出或修复始终产生新 Full 文件；
- 删除采用“两阶段”：产品清单标记后删除文件；失败可重试，但本地清单不是恢复依赖。

推荐默认策略：

```text
每月或每 30 个差异恢复点创建新 Full
保留最近 N 组 Full + Diff
```

差异链深度不会影响数据依赖深度，因为每个 Diff 只依赖 Full；主要影响是 Full 与当前数据差异越来越大，Diff 文件会逐渐接近 Full 大小。

---

## 17. 格式兼容与长期可恢复性

- `major` 变化表示不兼容格式；`minor` 只增加可安全忽略能力；
- 强制与可选 Feature Bitmap 分离；
- 所有枚举拒绝未知强制值，不允许通配符静默降级；
- 保留公开的 NWB Format Specification；
- Boot Media 固化经过验证的 Reader，不依赖已安装桌面程序；
- 提供只读 `nwb inspect/list/verify/extract` 工具；
- 新版本持续保留旧 Major Reader，迁移通过“读旧写新”完成；
- 归档中保存创建器版本，但不得以产品版本号代替格式版本。

---

## 18. 故障场景与预期行为

| 场景 | 预期结果 |
|---|---|
| 备份过程中断电 | 只留下 `.tmp`；已有归档不受影响。 |
| 最终 Footer 损坏 | 使用 Footer A 和 Manifest 副本打开。 |
| 全局 Chunk Index 损坏 | 扫描封闭 Segment 局部目录重建。 |
| 单个 Chunk 损坏 | 精确报告受影响文件/LBA；其他内容继续恢复。 |
| 高可靠模式少量分片损坏 | 使用纠删码写出修复后的新归档。 |
| Diff 文件丢失 | 不影响 Full 和其他 Diff。 |
| Full 文件丢失 | 所有依赖 Diff 无法完整恢复；必须明确报告缺少的 UUID。 |
| Full 文件被错误替换 | `base_root_hash` 不匹配，拒绝恢复。 |
| UI 历史数据库丢失 | 扫描 `.nwb` 重建全部恢复点。 |
| 文件被重命名 | 通过 Header UUID 重新识别。 |
| 文件尾部被截断 | 已提交归档判坏；Salvage 尽量恢复完整 Segment。 |
| 密码错误 | AEAD/Key Slot 验证失败，不泄露元数据细节。 |
| 目标空间不足 | 不提交；保留或清理 `.tmp`，不影响旧备份。 |

---

## 19. 建议的核心接口

```rust
trait ArchiveWriter {
    fn begin(spec: ArchiveSpec) -> Result<Self>;
    fn write_source_metadata(&mut self, metadata: SourceMetadata) -> Result<()>;
    fn write_extent(&mut self, extent: RawExtent) -> Result<ChunkRef>;
    fn write_catalog_entry(&mut self, entry: CatalogEntry) -> Result<()>;
    fn seal_segment(&mut self) -> Result<()>;
    fn finalize(self, manifest: RestoreManifest) -> Result<CommittedArchive>;
    fn abort(self) -> Result<()>;
}

trait ArchiveReader {
    fn open(path: &Path, credential: Option<Credential>) -> Result<Self>;
    fn identity(&self) -> &ArchiveIdentity;
    fn resolve_base(&mut self, candidates: &[PathBuf]) -> Result<()>;
    fn list_sources(&self) -> Result<Vec<SourceInfo>>;
    fn browse(&self, selector: BrowseSelector) -> Result<Vec<CatalogEntry>>;
    fn read_logical(&self, source: SourceId, range: Range<u64>) -> Result<Bytes>;
    fn verify(&self, level: VerifyLevel) -> Result<VerifyReport>;
}
```

Collector、Archive Writer、Restore Engine 和 BMR Engine 必须通过模型接口解耦，不能让文件系统扫描逻辑直接操作容器偏移。

---

## 20. 分阶段实施计划

本节保留架构层Wave摘要；实际工作包、依赖、Gate、退出标准和证据要求以`Nuwa_NWB_Implementation_Plan_v1.0.md`为准。两者发生粒度差异时，不得据此改变本架构的冻结原则。

### Wave 0：契约冻结

- NWB terminology；
- Full/Diff 语义；
- 二进制字段规范；
- 错误模型；
- 安全解析规则；
- 测试向量。

### Wave 1：只追加单文件容器

- Header/Record/Segment/Footer；
- 临时文件、单文件提交与多Volume Set提交；
- Automatic/NoSplit/FixedSize策略；
- Volume Header/Footer与Set Manifest；
- FAT32约3.75GiB分卷基线；
- SHA-256；
- Zstd；
- 基础 Reader/Inspector；
- 故障注入测试。

### Wave 2：文件完整备份与恢复

- File Catalog；
- NTFS 元数据；
- 归档内去重；
- 单文件和整目录恢复；
- Full Verify；
- UI 缓存丢失后重新导入。

### Wave 3：文件差异备份

- Full 基准索引；
- Diff Overlay；
- Tombstone/Rename；
- Base UUID/Root Hash 校验；
- `Full + Diff` 恢复；
- Diff 独立删除测试。

### Wave 4：卷与磁盘

- VSS；
- 文件系统分配位图；
- Volume/Disk Extent Map；
- MBR/GPT；
- 裸盘回退；
- 异常扇区策略。

### Wave 5：BMR、UEFI与异机还原 — 第一版强制发布门

- BMR_PROFILE与BOOT_ENVIRONMENT_PROFILE；
- UEFI x64与Legacy BIOS Recovery Media；
- GPT/MBR分区重建；
- ESP Raw Image与File Manifest；
- BCD、NVRAM与Fallback Boot Path；
- Secure Boot、TPM与BitLocker策略；
- WinRE恢复与重新注册；
- Boot-critical Driver注入；
- 原机原盘、原机新盘和异机还原；
- 不同容量磁盘恢复；
- P2V最小验收场景；
- Bootability Verification；
- BIOS/UEFI和物理机/虚拟机实验矩阵。

**发布约束：** Wave 5完成并通过质量门之前，不得发布Nüwa Backup第一版正式产品。

### Wave 6：加密与高可靠

- AES-256-GCM；
- Argon2id Key Slot；
- Recovery Key；
- 元数据副本和 Salvage；
- 可选 Reed-Solomon；
- 恶意归档模糊测试。

---

## 21. 必须通过的质量门

### 正确性

- Full 文件在无任何本地配置和数据库时可恢复；
- 每个 Diff 只用对应 Full 即可恢复；
- 逐文件/逐块 SHA-256 与源快照一致；
- 删除其他 Diff 不影响目标 Diff；
- 文件名和目录变化不影响 UUID 配对。

### 崩溃一致性

- 在每个写入阶段注入进程终止；
- 模拟短写、磁盘写满、Flush 失败和 rename 失败；
- 不得出现被 UI 识别为成功但无法打开的归档；
- 已存在的 `.nwb` 不得被失败任务修改。

### 多分卷Archive

- Automatic模式正确识别FAT32并自动分卷；
- NoSplit在目标不兼容时写入前失败；
- Fixed Size严格对齐Segment边界；
- 单文件模式与多Volume模式使用同一Reader；
- Chunk、Segment和Catalog Page不跨Volume；
- Volume Index重复、乱序、替换和混入必须拒绝；
- 缺失普通Volume必须报告精确Index和Segment范围；
- 缺失Final Volume不得显示为已提交；
- Final Volume必须最后发布；
- Cache丢失后通过Set Manifest重建；
- Full和Diff都支持分卷；
- 加密Archive跨Volume nonce不得重复；
- 选择性文件恢复必须计算Required Volumes；
- BMR在写盘前验证所有Required Volumes；
- Retention、Move、Copy、Delete必须覆盖完整Set；
- 模拟1000个Volume的索引、发现和性能边界。

### 损坏恢复

- Header、Footer、Manifest、Index、Catalog、Data 分别注入位翻转；
- 精确定位受影响对象；
- 未受影响内容仍能恢复；
- Index 可从 Segment 局部目录重建；
- 修复从不覆盖原文件。

### 安全

- Fuzz Header/Record/Index；
- 所有 offset/length 算术使用 checked arithmetic；
- 密钥、密码和明文数据不进入日志；
- nonce 唯一性自动测试；
- 错误 Full/Diff 配对必须拒绝；
- 旧 Reader 遇到未知 required feature 必须拒绝。

### BMR、UEFI与异机还原

#### 备份采集

- GPT主/备Header和Entry Array；
- Protective MBR；
- ESP Raw Image和File Manifest；
- BCD原始文件与语义模型；
- BootOrder/Boot####诊断快照；
- Secure Boot状态与EFI签名摘要；
- BitLocker/TPM状态；
- WinRE Profile；
- Boot-critical Driver Manifest；
- Hardware Profile；
- 所有BMR Metadata独立Hash和版本验证。

#### 恢复场景

- 原机原盘；
- 原机更换磁盘；
- 更小目标盘（已用空间和缩减条件允许）；
- 更大目标盘；
- SATA/AHCI/NVMe/RAID；
- BIOS/MBR同模式恢复；
- UEFI/GPT同模式恢复；
- 物理机到虚拟机；
- 异机驱动注入；
- Secure Boot开启环境；
- BitLocker逻辑备份和Raw加密备份；
- 多磁盘、多ESP和Recovery Media共存环境。

#### UEFI恢复验证

- Recovery Media确实以UEFI模式启动；
- 目标磁盘身份二次确认；
- GPT主备结构和CRC；
- ESP FAT32；
- Windows Boot Manager；
- BCD可解析且指向正确Windows；
- NVRAM项或Fallback Path；
- Secure Boot签名；
- Boot-critical Driver；
- WinRE；
- BitLocker恢复条件；
- 实际首次启动；
- Post-Restore Validation。

#### 发布判定

以下任一条件未通过，第一版不得宣称具备产品级BMR：

- 只写回系统卷但未恢复启动环境；
- 只在虚拟机验证、未验证真实UEFI硬件；
- BCD或NVRAM失败仍报告成功；
- 未覆盖NVMe/RAID驱动注入；
- BitLocker行为不明确；
- 未进行实际启动测试；
- 没有可保存的恢复报告。

---

## 22. v2.0 性能路径、故障矩阵与架构决策

### 22.1 日常性能路径

| 操作 | 首选数据源 | 是否读取 Data Segment |
|---|---|---:|
| 打开 UI | Catalog Cache | 否 |
| 显示备份链 | Catalog Cache | 否 |
| 重新导入归档 | Header/Footer/Manifest | 否 |
| 展开目录 | Internal Catalog Page | 否 |
| 搜索当前归档 | Internal Catalog Index | 否 |
| 跨归档搜索 | Search Index | 否 |
| 恢复文件 | Extent + Chunk + Segment | 是 |
| Full Verify | 全部核心结构和数据 | 是 |

### 22.2 目录恢复读优化

恢复目录时不要按文件逐个产生随机读取。Restore Planner 应：

1. 遍历 Catalog 子树；
2. 生成全部 Extent 请求；
3. 按 Archive、Segment、Offset 排序；
4. 合并相邻 Range；
5. 批量顺序读取 Full/Diff；
6. 将解码结果分发给目标文件；
7. 最后应用文件系统元数据。

### 22.3 关键故障矩阵

| 场景 | 预期行为 |
|---|---|
| Catalog Cache 丢失 | Header-only Import 重建 |
| Search Index 丢失 | 后台重建，不影响恢复 |
| Full 文件重命名 | UUID/Root 重新识别 |
| Diff 找不到 Full | 可浏览 Catalog，不可恢复数据 |
| 错误 Full | Root 不匹配，拒绝绑定 |
| Page Directory 损坏 | 扫描 Page Header 重建 |
| Object Index 损坏 | 扫描 Catalog Entry 重建 |
| Chunk Index 损坏 | 扫描 Segment Local Directory |
| 单 Catalog Page 损坏 | 定位受影响对象，其他页面可用 |
| Extent Map 损坏 | 对应对象可能无法重组 |
| Chunk 损坏 | 定位所有引用对象 |
| Footer B 损坏 | 使用 Footer A 并严格校验 |
| 文件尾部截断 | 归档判坏，Salvage 完整 Segment |
| 备份时断电 | 留下 tmp，不发布恢复点 |
| Cache 写入失败 | NWB 成功则报告备份成功、缓存待重建 |

### 22.4 ADR 汇总

#### NWB-ADR-001 — Three-Tier Catalog

**状态：FROZEN**

~~~text
Internal Catalog = Recovery Truth
Catalog Cache     = Performance
Search Index      = Cross-Backup Search
~~~

#### NWB-ADR-002 — Full and Differential Only

**状态：FROZEN**

- 不实现增量；
- Diff 只依赖 Full；
- Diff 之间互不依赖。

#### NWB-ADR-003 — Single Logical Archive Per Run

**状态：FROZEN**

- 一次成功运行产生一个逻辑NWB Archive；
- Archive可以由一个或多个Physical Volume组成；
- 无恢复依赖型外部Sidecar；
- 临时文件只存在于构建和提交阶段。

#### NWB-ADR-004 — Immutable After Commit

**状态：FROZEN**

修复、合并、升级全部读旧写新。

#### NWB-ADR-005 — Complete Logical Catalog in Diff

**状态：FROZEN**

Diff 独立浏览，数据恢复仍需 Full。

#### NWB-ADR-006 — Layered Granularity

**状态：BASELINE**

~~~text
4KB–64KB Change Detection
256KB Chunk
128MB Segment
64KB Catalog Page
~~~

#### NWB-ADR-007 — In-Archive Dedupe Only

**状态：FROZEN for v2**

不做跨归档物理块共享。Diff 到 Full 使用显式 Base Extent。

#### NWB-ADR-008 — Modular Provider Architecture

**状态：FROZEN**

- NWB Core只处理通用数据、Record、Chunk、Segment、Catalog和完整性；
- Source/Consistency/Change/Catalog/Restore Provider负责领域语义；
- System/BMR Provider通过Boot Environment Provider扩展UEFI和Legacy BIOS；
- Provider不得直接写Header/Footer或绕过Core校验；
- 第一版使用Rust Trait和编译期内置Provider，不急于开放第三方DLL ABI。

#### NWB-ADR-009 — UEFI Boot Environment as First-Class BMR Capability

**状态：FROZEN**

1. UEFI是第一版BMR一级能力；
2. GPT、ESP、BCD、WinRE和Boot-critical Driver必须进入NWB；
3. ESP采用Raw Image与File Manifest双重保护；
4. NVRAM变量保存用于诊断，目标端按硬件重建；
5. Secure Boot信任数据库默认不跨硬件迁移；
6. TPM密钥不能作为普通数据复制；
7. BitLocker必须在备份时冻结恢复策略；
8. Recovery Media必须支持UEFI x64；
9. 异机还原优先重建ESP、BCD和NVRAM项；
10. Bootability Verification是BMR成功的必要条件。

#### NWB-ADR-010 — BMR and Dissimilar Hardware Restore Are v1 Release Gates

**状态：FROZEN**

- BMR、UEFI和异机还原不是未来增强；
- Wave 5是第一版强制发布门；
- 没有真实启动验证不得发布正式版；
- 不得用“数据写回成功”替代“系统可启动且恢复完整”。

#### NWB-ADR-011 — Logical Single Archive with Physical Multi-Volume Support

**状态：FROZEN**

1. 每次备份产生一个逻辑Archive；
2. 单文件模式是Volume Set只有一个成员的特例；
3. Automatic模式根据目标能力自动分卷；
4. FAT32默认采用约3.75GiB的Segment对齐分卷基线；
5. Chunk、Segment和Page不得跨Volume；
6. 每个Volume独立Seal和Hash；
7. Final Volume保存Set Manifest与SET_COMMITTED；
8. Final Volume必须最后发布；
9. Retention、移动、删除和验证以Logical Archive为单位；
10. BMR开始前必须验证全部Required Volumes。

### 22.5 v2.0 必须通过的额外质量门

Catalog：

- 百万对象有界内存构建；
- Catalog Page 随机访问；
- Diff 不打开 Full Catalog 即可浏览；
- Cache 删除后重新导入；
- Search Index 删除后重建；
- Page/Directory/Object/Chunk Index 分别损坏测试；
- 加密归档搜索隐私测试。

性能：

- UI 冷启动和热启动；
- Header-only Import；
- 目录展开 P50/P95；
- 256KB/512KB/1MB Chunk 对比；
- 64/128/256MB Segment 对比；
- 本地 SSD/HDD/USB/SMB；
- Full/Diff 和加密开关；
- Catalog/Search 重建时间。

### 22.6 配套二进制规范

本架构的配套二进制规范已经建立为：

~~~text
Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md
~~~

它负责冻结：

- 精确字段 Offset；
- 字节序；
- Magic；
- Header/Footer；
- Record Schema；
- Feature ID；
- Hash 覆盖范围；
- AEAD Additional Data；
- Parser 安全规则；
- Golden Archive 和 Test Vector。

在二进制规范、跨平台原型、Golden Corpus和基准测试完成前，不应让生产代码自行决定不可逆的永久格式，也不得把`format_major`标记为1。

---

## 23. 参考依据与设计证据

- [Acronis True Image 2026 User Guide](https://dl.acronis.com/u/pdf/ATI2026_userguidewindows_en-US.pdf)：当前 TIBX 模型将完整和差异备份版本分别保存为文件；增量版本合并到基准归档。Nüwa 采纳其中与产品原则一致的 Full/Diff 分文件模型，但不实现增量合并。
- [Acronis Cyber Backup 12.5 User Guide](https://dl.acronis.com/u/pdf/AcronisCyberBackup_12.5_userguide_en-US.pdf)：TIBX 的块级去重、压缩和备份验证等设计思想。
- [Veeam Health Check for Backup Files](https://helpcenter.veeam.com/docs/vbr/userguide/backup_health_check.html)：在备份文件中保存元数据 CRC 和数据块哈希，并通过 Health Check 验证恢复点涉及的数据块。
- [Restic Repository Format](https://restic.readthedocs.io/en/v0.15.0/100_references.html)：先写 Pack、再写 Index、最后发布 Snapshot 的提交顺序，为 NWB 的“数据先持久化、最后发布恢复点”提供参考。
- [Borg Data Structures](https://borgbackup.readthedocs.io/en/stable/internals/data-structures.html) 与 [Kopia Architecture](https://kopia.io/docs/advanced/architecture/)：Segment/Pack 与可重建索引思想，为 NWB 避免“一块一个文件”和实现损坏扫描提供参考。

- [Acronis Backup Advanced 11.5 User Guide](https://dl.acronis.com/u/pdf/AcronisBackupAdvanced_11.5_userguide_en-US.pdf)：公开的旧版磁盘级 4KB 与文件级最大 256KB 去重粒度；只作为技术参考，不视为当前 TIBX 参数。
- Acronis True Image 2026 文档同时说明产品维护 metadata information database，并允许通过 Add existing backup 重新加入 TIBX；Nüwa 据此采用“归档内恢复真相 + 本地可重建缓存”，但具体 NWB 结构为自主设计。



### UEFI与BMR补充参考

- [Microsoft UEFI/GPT-based hard drive partitions](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/configure-uefigpt-based-hard-drive-partitions?view=windows-11)：UEFI Windows默认分区布局。
- [Microsoft BCDBoot Command-Line Options](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/bcdboot-command-line-options-techref-di?view=windows-11)：ESP启动文件、BCD与NVRAM启动项行为。
- [Microsoft Windows and GPT FAQ](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/windows-and-gpt-faq?view=windows-11)：UEFI系统盘所需ESP、MSR和基本数据分区。
- [Microsoft Windows Recovery Environment](https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/windows-recovery-environment--windows-re--technical-reference?view=windows-11)：WinRE分区与部署要求。
- [Microsoft BitLocker FAQ](https://learn.microsoft.com/en-us/windows/security/operating-system-security/data-protection/bitlocker/faq)：恢复密钥、恢复密码和TPM边界。


### 多分卷补充参考

- [Acronis Cyber Backup 12.5 User Guide — Splitting](https://dl.acronis.com/u/pdf/AcronisCyberBackup_12.5_userguide_en-US.pdf)：Automatic按目标文件系统最大文件大小自动拆分，Fixed Size允许指定分卷大小。

---

## 24. 单机版跨平台产品架构定稿

### 24.1 产品边界

第一阶段只交付单机备份恢复软件。任务配置、本地Catalog Cache和Search Index可以提升体验，但任何恢复关键数据都必须存在于NWB逻辑Archive中。企业控制台、Repository、集中策略、数据库专用备份、虚拟化无代理备份、对象存储和跨Archive全局去重属于后续产品线。

首版核心用户路径是：

```text
安装单机代理
→ 选择文件/卷/磁盘
→ 创建Full或Diff NWB
→ 在本机或Recovery Media中打开
→ Verify
→ 文件/卷/磁盘恢复
→ 认证场景BMR或异机恢复
```

### 24.2 五层模块

```text
Desktop/CLI
    │
Job and Policy Engine
    │
Windows/Linux Providers
    │
NWB Core: Format + Chunk + Catalog + Crypto + Verify/Salvage
    │
Local/USB/SMB/NFS Physical Target

Recovery Media复用NWB Reader、Restore、Provider和Boot Repair能力，
但不依赖原系统、桌面UI、本地Cache或任务数据库。
```

NWB Core是长期稳定层；Provider是平台适配层；UI和任务调度是可替换产品层。

### 24.3 Windows与Linux实施关系

- Windows 7 SP1至当前Windows产品线使用同一NWB格式；Win7采用兼容构建和受控Legacy依赖；
- Windows由VSS、NTFS/ReFS、GPT/MBR、ESP/BCD/WinRE和Driver Provider组成；
- Linux面向主流x86-64发行版，由LVM/Btrfs/fsfreeze、ext/XFS/Btrfs、GPT/MBR、GRUB/systemd-boot和initramfs Provider组成；
- Windows与Linux可以互相读取Catalog和通用数据，但跨系统恢复权限语义时必须明确降级；
- 未来Windows/Linux ARM64只增加平台Provider和Recovery Media，不改变NWB 1.0。

## 25. NWB Format 1.0设计基线

### 25.1 规范布局

```text
4096-byte Bootstrap/Volume Header
Key Slot Records（加密时）
Sealed Data/Checkpoint/Provider Metadata Segments
Final Catalog Segments
Archive Manifest
Final Volume Set Manifest（单文件也有逻辑Set）
4096-byte Commit A
4096-byte Commit B
```

正常Reader从文件尾Commit直接定位Manifest和Catalog Root；只有Salvage才扫描Segment。Header不保存可变完成状态，只有合法`SET_COMMITTED`才表示成功。

### 25.2 固定编码规则

- Little Endian；
- Offset、Length和Count使用无符号64位；
- Header、Segment边界、Footer和Commit按4KiB对齐；
- Record使用Type、Version、Flags、Header Length、Payload Length和Record ID自描述；
- 禁止直接序列化语言内存对象；
- Required Feature未知时拒绝，Optional Feature未知时安全跳过；
- 所有解析使用checked arithmetic和资源上限。

### 25.3 基线粒度与算法

- 256KiB Logical Chunk；
- 约128MiB Target Segment；
- 64KiB Catalog Page；
- Zstandard逐Chunk压缩，不获益时保存NONE；
- AES-256-GCM逐Chunk认证加密；
- Argon2id密码KDF与可扩展Key Slot；
- CRC32C快速错误检测；
- SHA-256内容、身份和根哈希。

这些值在Format 0.x原型阶段通过基准测试确认，进入Format 1.0后不得改变既有语义。实际参数仍必须写入档案，Reader不能只依赖默认值。

### 25.4 双层Catalog耐久性

备份过程中周期性写Checkpoint Catalog，只引用已密封内容；结束时写完整Final Catalog。Checkpoint用于诊断、同一有效快照会话内续写和Salvage，不构成提交点。Diff仍保存当前恢复点的完整逻辑Catalog。

### 25.5 格式冻结原则

Format 0.x只供内部测试。Windows Writer/Linux Reader、Linux Writer/Windows Reader、独立格式检查器、Golden Corpus、Fuzz、分卷故障、Full/Diff恢复及Windows/Linux BMR原型全部通过后，才发布`format_major=1`。

## 26. Provider模块化架构定稿

### 26.1 职责

Provider负责Discovery、Snapshot、File/Block Capture、Metadata、Restore、Boot和Verify能力；Core负责归档字节。Provider只能提交公共对象、数据流、Extent、私有Metadata Envelope、依赖、错误和一致性结果。

### 26.2 生命周期

```text
Capture: Probe → Prepare → Snapshot → Enumerate/Capture → Finalize → Release
Restore: ProbeTarget → Plan → Validate → Confirm → Restore → Boot Repair → Verify
```

失败、取消和超时都必须释放Snapshot、句柄、挂载和临时资源。系统重启导致Snapshot消失时不得跨新时间点续写旧Archive。

### 26.3 能力协商

Provider运行时返回`SUPPORTED`、`SUPPORTED_WITH_LIMITATIONS`、`DATA_ONLY`、`DETECTED_UNSUPPORTED`或`UNKNOWN`。Core和UI只能按实际能力创建任务。未知拓扑不能自动声称支持。

### 26.4 版本与安全

Provider有稳定ID、实现版本、Schema版本、能力、最低Core/Reader版本、发布者、二进制哈希和签名。NWB只保存恢复依赖，不嵌入执行代码。首版只加载官方Provider；未来第三方Provider使用签名、进程隔离和Conformance Suite。

公共SDK不得把Rust Trait或C++对象ABI作为长期协议。本文第19节Trait仅为Core内部示意；对外Provider协议必须使用版本化、长度有界的消息Schema。

## 27. 首版支持承诺与降级策略

### 27.1 正式目标

- Windows x86-64，Windows 7 SP1及后续桌面与相应Server产品线；
- Linux x86-64主流LTS/企业发行版；
- Windows基本磁盘、NTFS、MBR/BIOS、GPT/UEFI；
- Linux普通分区、LVM2、ext4、XFS、Btrfs简单拓扑、MBR/BIOS、GPT/UEFI；
- 文件、卷、磁盘Full/Diff、Verify、Salvage、Recovery Media；
- Windows UEFI/NTFS与Linux UEFI/LVM/ext4认证BMR；
- 认证存储控制器范围内的异机恢复。

### 27.2 首版受限场景

Windows动态磁盘、Storage Spaces、复杂软件镜像、复杂硬件RAID、Linux复杂mdraid、Btrfs多设备、ZFS Pool、Multipath/SAN可进行识别和数据级备份，但在没有完整拓扑恢复证据前不得标记为BMR Ready。

BitLocker/LUKS区分解锁逻辑备份和锁定Raw Image。EFS可用性依赖证书私钥。XFS和ReFS不支持缩容。Windows/Linux ARM64、P2V/V2P、跨操作系统BMR属于未来能力。

### 27.3 支持状态来源

产品运行时的支持状态必须来自已批准的认证账本，不能由市场文案或Provider名称推断。发现严重缺陷时可以降级能力，并同步到任务创建、BMR预检、恢复UI和发布说明。

## 28. 实施、验收与测试治理

### 28.1 文档套件

本架构由以下执行规范补充：

- `Nuwa_NWB_Engineering_Document_Set_README_v1.0.md`；
- `Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md`；
- `Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md`；
- `Nuwa_NWB_Product_Support_Matrix_v1.0.md`；
- `Nuwa_NWB_Implementation_Plan_v1.0.md`；
- `Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md`；
- `Nuwa_NWB_Test_Result_Record_v1.0.md`。

### 28.2 九个工程质量门

```text
GATE-0 工程与契约
GATE-1 NWB容器
GATE-2 文件Full/Diff
GATE-3 分卷与密码学
GATE-4 Verify/Salvage
GATE-5 卷与磁盘
GATE-6 Windows BMR
GATE-7 Linux BMR
GATE-8 Format 1.0冻结
GATE-9 单机产品发布
```

每个工作包都必须有计划、设计、实现、单元测试、集成/故障测试、验收、证据和文档更新。缺少任意环节时，最多标记为IMPLEMENTED，不能标记为ACCEPTED。

### 28.3 测试结果真实性

在实际代码和测试执行前，所有结果是NOT_RUN。PASS必须绑定被测提交、环境、命令、退出码、日志、输入/输出Manifest、NWB SHA-256、恢复验证和复核人。BMR只有真实启动、登录并完成验证脚本才算PASS。

### 28.4 发布底线

以下任何一项存在时不得冻结Format或发布产品：

- 数据丢失或错误恢复P0；
- 未提交归档被识别为成功；
- Diff可能引用错误Full；
- 密钥或密码泄漏；
- 恶意NWB导致Reader越界、无限分配或路径逃逸；
- Recovery Media不能独立读取必需Feature；
- 支持矩阵的Certified能力没有实际恢复证据；
- Golden Corpus未建立或新版Reader不能读取。

## 29. 跨平台产品线参考

- [Microsoft Windows Release Health](https://learn.microsoft.com/en-us/windows/release-health/)：Windows客户端与服务器发布/问题信息，用于维护认证矩阵。
- [Microsoft Windows Server Release Information](https://learn.microsoft.com/en-us/windows/release-health/windows-server-release-info)：Windows Server版本线。
- [Ubuntu Release Cycle](https://ubuntu.com/about/release-cycle)：LTS版本与长期维护策略。
- [Red Hat Enterprise Linux Life Cycle](https://access.redhat.com/support/policy/updates/errata)：RHEL主版本生命周期。
- [SUSE Linux Enterprise Server Documentation](https://documentation.suse.com/sles/)：SLES版本与平台文档。
