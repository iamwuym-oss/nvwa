# Nüwa NWB Binary Format Specification v1.0 Draft

**状态：** DRAFT 0.9 — `GATE-8`通过后升级为FORMAT FROZEN 1.0  
**日期：** 2026-07-12  
**规范标识：** `NWB-FORMAT-1`  
**默认扩展名：** `.nwb`

---

## 1. 规范目标

本规范定义NWB逻辑归档及其物理卷的规范编码。目标是：

- 单机、离线、自描述恢复；
- 顺序追加写入，不修改已密封内容；
- 快速随机浏览和恢复；
- 断电、截断、缺卷和局部损坏可准确识别；
- Windows与Linux使用同一格式；
- Reader能够跳过未知可选记录并拒绝未知强制特性；
- 格式不绑定Rust、C/C++或任何序列化框架。

本文使用“必须”“不得”表示规范性要求，使用“建议”表示默认实现。

## 2. 格式不变量

| ID | 不变量 |
|---|---|
| `FMT-001` | 所有整数使用Little Endian。 |
| `FMT-002` | 所有偏移、长度和逻辑数据计数使用无符号64位整数。 |
| `FMT-003` | Writer只能追加；已密封Segment和已提交Archive不得原位修改。 |
| `FMT-004` | Full自包含；Diff仅依赖一个通过ID和根哈希绑定的Full。 |
| `FMT-005` | Chunk、Record、Catalog Page和Segment不得跨物理卷。 |
| `FMT-006` | 只有合法Commit副本引用的Manifest才构成已提交归档。 |
| `FMT-007` | Reader不得信任文件名、外部数据库或未经校验的偏移。 |
| `FMT-008` | 未知Required Feature必须导致拒绝打开；未知Optional Feature允许跳过。 |
| `FMT-009` | 加密先于持久化，解密认证成功前不得向恢复端释放明文。 |
| `FMT-010` | 每个结构必须有显式版本、长度和校验，不得直接落盘内存结构。 |

## 3. 逻辑与物理模型

一个备份运行产生一个Logical Archive。Logical Archive物理上是：

- 单个Physical Volume；或
- 按顺序编号的Volume Set。

单文件是Volume Set只有一个成员的特例，两者使用相同Record、Segment、Catalog和Commit协议。

```text
Logical Archive
├── Volume 1
│   ├── Bootstrap Header
│   ├── Key Slot Records（可选）
│   ├── Data/Catalog Segments
│   └── Volume Footer
├── Volume 2（可选）
│   ├── Volume Header
│   ├── Segments
│   └── Volume Footer
└── Final Volume
    ├── Final Catalog Segments
    ├── Archive Manifest
    ├── Volume Set Manifest
    └── Commit A + Commit B
```

## 4. 对齐与基本尺寸

| 项目 | v1默认值 | 规则 |
|---|---:|---|
| Physical alignment | 4096 bytes | Header、Segment起点、Footer、Commit必须4KiB对齐 |
| Logical Chunk | 256 KiB | 允许最后一个Chunk缩短 |
| Segment target | 128 MiB | 可因卷边界提前密封 |
| Catalog Page | 64 KiB | 固定页长，页面独立校验 |
| Commit copy | 4096 bytes | 最终物理卷尾部连续保存两份 |

参数写入档案，不得让Reader只依赖编译时默认值。Chunk和Segment目标值在`GATE-8`前仍是BASELINE；若基准测试调整，必须修改本规范、Golden Corpus和ADR。

## 5. Bootstrap Header

每个物理卷从4096字节Header开始。Volume 1使用Archive Bootstrap Header，后续卷使用相同布局并设置卷序号。

### 5.1 固定字段

| Offset | Size | Field | 说明 |
|---:|---:|---|---|
| 0 | 8 | `magic` | 字节`4E 55 57 41 4E 57 42 00`（`NUWANWB\0`） |
| 8 | 2 | `format_major` | Draft使用0，冻结后为1 |
| 10 | 2 | `format_minor` | 向后兼容扩展版本 |
| 12 | 4 | `header_size` | v1固定4096 |
| 16 | 4 | `byte_order_marker` | `0x01020304` |
| 20 | 4 | `header_crc32c` | 计算时本字段置零 |
| 24 | 8 | `required_features` | Reader必须支持的Feature Bitmap |
| 32 | 8 | `optional_features` | 可安全忽略的Feature Bitmap |
| 40 | 16 | `archive_id` | 当前备份UUID |
| 56 | 16 | `chain_id` | Full/Diff链UUID |
| 72 | 16 | `base_full_id` | Full为全零；Diff为基准Full UUID |
| 88 | 16 | `volume_set_id` | 物理卷集合UUID |
| 104 | 4 | `volume_number` | 从1开始 |
| 108 | 4 | `volume_count_hint` | 未知时为0；不得作为最终事实 |
| 112 | 8 | `created_utc_ns` | Unix Epoch UTC纳秒 |
| 120 | 4 | `backup_kind` | 1=FULL，2=DIFFERENTIAL |
| 124 | 4 | `platform_hint` | 0=MIXED/UNKNOWN，1=WINDOWS，2=LINUX |
| 128 | 8 | `first_record_offset` | v1固定4096 |
| 136 | 8 | `keyslot_area_offset` | 未加密时为0 |
| 144 | 8 | `keyslot_area_length` | 未加密时为0 |
| 152 | 32 | `header_sha256` | 计算时本字段和CRC字段置零 |
| 184 | 3912 | `reserved` | v1 Writer必须写零；Reader忽略 |

Header只用于识别、Feature协商和找到解密入口，不保存可变进度或“完成”状态。

## 6. 通用Record Envelope

除固定Header/Footer/Commit外，所有内容使用64字节Record Envelope。

Record起点按8字节对齐；`total_length`包含Record内部Padding。Segment结束时再补齐到4KiB边界。Reader必须验证Padding位于所属Record/Segment内，但不得依赖Padding内容承载语义；v1 Writer写零。

| Offset | Size | Field |
|---:|---:|---|
| 0 | 4 | `record_magic = NWBR` |
| 4 | 2 | `record_type` |
| 6 | 2 | `record_version` |
| 8 | 4 | `record_flags` |
| 12 | 4 | `header_length`，v1最小64 |
| 16 | 8 | `total_length`，含Header、Payload和Padding |
| 24 | 8 | `sequence_number` |
| 32 | 16 | `record_id` |
| 48 | 4 | `header_crc32c` |
| 52 | 4 | `reserved`，必须为0 |
| 56 | 8 | `payload_length` |

Reader必须验证：

- `header_length >= 64`；
- `payload_length <= total_length - header_length`；
- 加法和对齐不存在整数溢出；
- Record完全位于当前物理卷和所属Segment内；
- Required Record的类型和版本受支持。

## 7. Record类型注册表

| 范围 | 用途 |
|---|---|
| `0x0001–0x00FF` | 容器、Manifest、Commit辅助记录 |
| `0x0100–0x01FF` | Chunk、Extent、Zero/Hole/Unreadable |
| `0x0200–0x02FF` | Catalog Page与Index |
| `0x0300–0x03FF` | Snapshot、一致性和错误记录 |
| `0x0400–0x04FF` | 磁盘、分区、文件系统和BMR公共记录 |
| `0x0500–0x05FF` | 加密、Key Slot和签名记录 |
| `0x8000–0xBFFF` | 注册Provider私有记录 |
| `0xC000–0xFFFF` | 实验记录；Format 1.0正式档案不得使用 |

具体数值由单独的Format Registry源码生成。相同数值不得复用为不同语义。

## 8. Segment

### 8.1 结构

```text
4096-byte Segment Header
Record 1
Record 2
...
Alignment Padding
4096-byte Segment Footer
```

Segment Header至少包含：`segment_id`、序号、类型、预期Chunk大小、codec/cipher profile、记录数量提示和Header校验。

Segment Footer至少包含：

- Segment ID与序号；
- 实际Payload长度；
- Record数量；
- 存储字节CRC32C；
- 存储字节SHA-256；
- Record ID Merkle Root；
- `SEALED`标记；
- Footer CRC32C与SHA-256。

Footer不存在或校验失败时，该Segment不得进入正常Catalog。Salvage可以在严格边界检查后分析其中的完整Record，但结果必须标记为Recovered/Unverified，不得改写原档案。

### 8.2 Segment类型

- `DATA`
- `CATALOG`
- `CHECKPOINT`
- `MANIFEST`
- `PROVIDER_METADATA`
- `ERROR_MAP`

一个Segment只采用一个加密Profile，但内部Chunk可以逐个选择`NONE`或指定压缩算法。

## 9. Chunk与Extent

### 9.1 Chunk处理顺序

```text
原始数据
→ 原始SHA-256
→ 零块/空洞识别
→ Zstandard压缩或NONE
→ AES-256-GCM加密或NONE
→ 存储CRC32C与长度
→ Chunk Record
```

### 9.2 Chunk Record必要字段

- `chunk_id`：归档内唯一；
- `plaintext_length`；
- `stored_length`；
- `plaintext_sha256`（加密归档中位于加密Payload或加密Catalog）；
- `compression_id`；
- `encryption_id`；
- `nonce`或Nonce派生计数；
- `authentication_tag`；
- `stored_crc32c`；
- `chunk_flags`。

### 9.3 特殊范围

以下范围不得伪造成普通零数据：

| 类型 | 含义 |
|---|---|
| `ZERO` | 已确认逻辑内容全部为零 |
| `HOLE` | 文件系统稀疏空洞 |
| `UNALLOCATED` | 文件系统未分配空间 |
| `UNREADABLE` | 源介质读取失败，必须记录范围和错误 |
| `BASE_REF` | Diff引用Full中的Chunk或Extent |

恢复器不得将`UNREADABLE`静默转换为正常零数据。用户明确选择填零时，结果必须标记为Degraded Restore。

## 10. Catalog

### 10.1 Catalog Page

Catalog采用固定64KiB不可变页。每页包含：

- Page Magic、版本和类型；
- Page ID；
- Tree ID；
- Level；
- Entry Count；
- Lower/Upper边界；
- Next/Previous Page ID（适用时）；
- Payload CRC32C；
- Page SHA-256；
- 规范排序的Entry区。

Reader必须验证页内偏移、Key排序、重复Key策略和树层级。任何循环引用、越界Child或深度超过实现上限都必须拒绝。

### 10.2 必需Catalog树

1. `OBJECT_TREE`：Machine、Disk、Partition、Volume、Directory、File等对象；
2. `PATH_TREE`：规范路径键到Object ID；
3. `EXTENT_TREE`：对象逻辑范围到Chunk/Base Ref；
4. `CHUNK_TREE`：Chunk ID/哈希到物理位置；
5. `BOOT_TREE`：启动与BMR对象；
6. `PROVIDER_TREE`：Provider Schema、能力和恢复依赖；
7. `ERROR_TREE`：不可读范围、一致性降级和警告；
8. `VOLUME_TREE`：物理分卷与Segment位置。

### 10.3 Windows名称与时间

Windows路径组件必须保存原始UTF-16LE字节序列，并可附带显示用UTF-8和比较键。不得因无效代理项而丢失原名。

文件时间保存原始文件系统值、UTC规范值和精度。Linux名称保存原始字节序列，同时提供可选显示编码；不得假设所有Linux文件名都是合法UTF-8。

## 11. Full与Differential

Full的`base_full_id`全零，所有可恢复数据均在自身逻辑Archive内。

Diff必须保存：

- 与该时间点一致的完整逻辑Catalog；
- `chain_id`；
- `base_full_id`；
- Full最终Manifest SHA-256；
- 新增和变化Chunk；
- 未变化范围的`BASE_REF`；
- 删除、重命名和元数据变化；
- 当前快照与一致性记录。

打开Diff时必须先验证Full身份和根哈希。不得根据文件名或“最接近的Full”自动替代。

## 12. Checkpoint与最终Catalog

Writer应周期性写入Checkpoint Segment，记录已密封Segment、已完成对象边界、当前错误和Chunk摘要。Checkpoint用于诊断、同一有效快照会话内的续写和Salvage，不构成已提交恢复点。

备份结束时写入完整Final Catalog。正常Reader从Commit定位Manifest和Catalog Root，不扫描全部数据区。

## 13. 压缩注册表

| ID | 算法 |
|---:|---|
| 0 | NONE |
| 1 | ZSTD |

Writer对压缩收益不足的Chunk写`NONE`。Reader按记录中的算法解压，不依赖全局默认压缩等级。压缩库版本不进入可恢复语义，但算法ID和必要参数必须进入记录。

## 14. 加密与Key Slot

### 14.1 v1密码学Profile

- 数据加密：AES-256-GCM；
- 密码KDF：Argon2id；
- 快速错误检测：CRC32C；
- 内容和身份哈希：SHA-256；
- 随机数：操作系统CSPRNG。

算法使用数字ID，禁止通过自由文本选择算法。参数必须保存到Key Slot。

### 14.2 密钥层次

每个Archive生成随机Archive Master Key（AMK）。用户密码经KDF产生KEK，KEK只包装AMK。数据、Metadata和Nonce派生子密钥由AMK通过HKDF-SHA-256按不同Context分离。Key Slot包含：KDF ID与参数、Salt、Wrap算法、Wrapped AMK、认证标签和Slot用途。

可以有用户密码Slot和恢复密钥Slot。Provider不得获得AMK或任何派生子密钥。密码变更不得原位改写已提交归档；首版通过输出新归档完成。

### 14.3 Nonce

同一数据加密子密钥下Nonce绝不允许重复。v1使用Archive随机Nonce前缀加单调Record计数派生，派生输入同时绑定`archive_id`、`segment_id`和`record_id`。Writer必须在崩溃续写时避免计数回退；无法证明唯一时创建新Archive而不是续写。

## 15. Physical Volume与Volume Set

每个Volume Header保存`volume_set_id`、卷序号和前卷Footer哈希。每个Volume Footer保存当前卷大小、Segment清单摘要、前卷哈希、当前卷SHA-256和密封状态。

Final Volume Set Manifest保存：

- 总卷数；
- 每卷ID、序号、大小和SHA-256；
- 首尾Segment序号；
- Archive Manifest位置；
- 所需卷与可选卷标记；
- Volume Set Root Hash。

默认分片模式：

- `AUTO`：探测文件系统限制，FAT32默认安全上限约3.75GiB；
- `NO_SPLIT`：目标不能容纳时在写入前失败；
- `FIXED_SIZE`：按用户策略分卷，但不得小于实现的最小安全值。

最终卷最后发布。缺少Final Volume时，整个集合不是已提交Archive。

## 16. Manifest与Commit

### 16.1 Archive Manifest

Manifest至少包含：

- Archive、Chain、Base Full身份；
- Format与Required/Optional Features；
- 所有Segment ID、类型、位置、长度和SHA-256；
- 所有Catalog Root；
- 所有Provider及最低恢复能力；
- Snapshot与Consistency结果；
- Volume Set Root；
- 数据总量、对象总量和错误摘要；
- Manifest自身SHA-256。

### 16.2 Commit副本

最终物理卷尾部保存两个独立4096字节Commit Block。每个Block包含：

- Commit Magic与版本；
- Archive ID；
- Commit Sequence；
- Manifest物理卷号、Offset和Length；
- Manifest SHA-256；
- Volume Set Root Hash；
- Commit UTC时间；
- `SET_COMMITTED`标记；
- Block CRC32C和SHA-256。

Reader读取文件尾的两份副本：

1. 两份均合法且一致：正常打开；
2. 一份合法：使用合法副本并报告冗余降级；
3. 两份合法但不一致：拒绝正常打开，进入诊断；
4. 两份均无效：归档未提交或尾部损坏，只允许Salvage。

### 16.3 哈希覆盖范围

- Header CRC/SHA覆盖完整4096字节；计算时两个校验字段置零；
- Record Header CRC覆盖`header_length`字节；计算时自身CRC字段置零；
- Segment Stored SHA-256覆盖规范化Segment Header加实际Record/Padding字节，不包含Footer；
- Catalog Page SHA-256覆盖完整64KiB Page，计算时Page校验字段置零；
- Physical Volume SHA-256覆盖从Volume起点到Volume Footer起点的全部字节；Footer自身单独校验；
- Final Volume尾部Commit A/B不进入Physical Volume SHA-256，分别由Commit Block校验并共同绑定Volume Set Root；
- Manifest SHA-256覆盖规范编码后的完整Manifest，计算时自身哈希字段置零；
- Commit SHA-256覆盖完整4096字节Block，计算时CRC和SHA字段置零。

任何实现不得使用“结构体内存字节”计算规范哈希，必须先产生规范编码。

## 17. 提交协议

单文件：

1. 写Header；
2. 写并密封Data/Checkpoint Segment；
3. 写Final Catalog；
4. 写Manifest；
5. Flush数据和元数据；
6. 写Commit A并Flush；
7. 写Commit B并Flush；
8. 原子发布最终文件名；
9. 更新可重建的Local Catalog Cache。

多卷：先逐卷密封和发布非最终卷，最后写Final Volume Set Manifest和两份Commit，再发布最终卷。Cache失败不影响Archive提交，任务结果使用`COMMITTED_BUT_CACHE_FAILED`。

## 18. Reader安全限制

Reader必须有可配置的硬上限，至少覆盖：

- 最大Record和Segment长度；
- 最大Catalog页数和树深度；
- 最大路径组件、路径深度和对象数量；
- 最大解压比例和单Chunk明文长度；
- 最大Provider Metadata长度；
- 最大Volume数量；
- 最大错误记录数量；
- 最大内存和并行任务数。

所有乘加、Offset和Length运算使用checked arithmetic。恢复路径必须经过包含性检查，符号链接/Reparse Point最后创建，不能将后续文件写出恢复根。

## 19. 版本兼容

- Major变化表示存在不兼容语义；
- Minor只允许向后兼容扩展；
- Record和Provider Metadata独立版本；
- Required Feature未知时拒绝；
- Optional Feature未知时跳过并报告；
- 新Writer不得改变旧字段含义；
- 发布NWB 2.0后仍必须保留NWB 1.0 Reader；
- Draft 0.x档案不承诺长期兼容，不得向正式用户发布。

## 20. 格式一致性测试

Format 1.0冻结前必须：

1. Windows Writer生成的档案由Linux Reader读取；
2. Linux Writer生成的档案由Windows Reader读取；
3. 独立格式检查器重新计算Header、Segment、Page、Manifest和Commit校验；
4. Full/Diff、单文件/分卷、明文/加密Golden Files建立；
5. 截断每个关键边界时Reader均返回确定状态；
6. Fuzz测试不崩溃、不越界、不无限分配；
7. Golden Corpus在后续版本CI中永久回归。

## 21. Format 1.0冻结条件

以下全部满足后，才允许把`format_major`从0改为1：

- 字段注册表生成并审查；
- Reference Writer、Reader、Verify和Salvage通过；
- Full/Diff文件恢复闭环通过；
- Windows UEFI/NTFS与Linux UEFI/LVM/ext4 BMR原型启动通过；
- 分卷、断电、错链、错密、篡改和损坏测试通过；
- Golden Corpus及其SHA-256清单封存；
- 不存在需要修改Header、Record Envelope、Segment、Catalog或Commit既有语义的P0缺陷；
- 架构、开发、测试和安全负责人共同签署`GATE-8`。

在此之前，本文件是实现基线，不是已发布兼容承诺。
