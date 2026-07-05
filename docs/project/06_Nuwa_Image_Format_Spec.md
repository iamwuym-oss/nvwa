# Nüwa Backup 镜像格式规范 — .nwb 文件格式 v0.1（实验阶段）

**版本：** v0.1（实验阶段）
**日期：** 2026-07-05
**状态：** ⚠️ 实验阶段格式 — 仅用于 Phase 2 备份实验，不属于 Phase 1 开发范围

---

## 重要声明

> **本文档定义的 .nwb v0.1 格式仅供 Phase 2 实验使用，不属于 Phase 1（MVP）的开发范围。**
>
> Phase 1 仅使用**平文件存储**（目录 + JSON manifest），不产生任何 .nwb 文件。
>
> .nwb v0.1 是**最小实验格式**，仅支持完整备份（Full），不支持差异备份、XOR Parity、AES 加密等高级特性。
>
> 所有高级特性（Differential、XOR Parity、AES 加密、复杂 Block Group）已移入附录 B：Future Format Expansion，明确不在 v0.1 中实现。

---

## 1. 格式定位

### 1.1 v0.1 目标

.nwb v0.1 是块级镜像格式的**最小可行版本**，设计目标：

1. 验证块级备份/恢复的数据闭环
2. 验证块级压缩 + SHA-256 校验的完整性
3. 验证空洞检测（Sparse Block）的正确性
4. 为 v0.2+ 的复杂格式积累工程经验

### 1.2 v0.1 排除项

| 特性 | 状态 | 计划版本 |
|------|------|---------|
| 差异备份 (Differential) | ❌ 不实现 | v0.3+ |
| XOR Parity / Erasure Coding | ❌ 不实现 | v1.0+ |
| AES 加密 | ❌ 不实现 | v1.0+ |
| 块组 (Block Group) | ❌ 不实现（当前使用平坦索引） | v0.2+ |
| 文件级索引（备份内文件搜索） | ❌ 不实现（依赖独立 SQLite 索引） | v0.2+ |
| 自动损坏修复 | ❌ 不实现（仅检测损坏，不自动修复） | v1.0+ |

### 1.3 格式演进路线

```
v0.1 (当前)   →   v0.2        →   v1.0        →   v1.x
 实验阶段         Block Group      XOR Parity     差异备份
 平坦索引         文件索引          AES 加密        版本链
 Full only        性能优化         格式冻结         多平台
```

---

## 2. 整体布局

```
┌─ File Header (128 字节, 固定) ───────────────┐
│  魔数、版本、备份属性、块信息                   │
├─ Block Index Table (变长) ────────────────────┤
│  Block #0 Entry                               │
│  Block #1 Entry                               │
│  ...                                          │
├─ Block Data ──────────────────────────────────┤
│  Block #0 压缩数据                             │
│  Block #1 压缩数据                             │
│  ...                                          │
├─ File Trailer (64 字节, 固定) ────────────────┤
│  统计信息、全文件 SHA-256                      │
└───────────────────────────────────────────────┘
```

---

## 3. File Header (128 字节)

### 3.1 布局

| 偏移 | 大小 | 字段 | 说明 |
|------|------|------|------|
| 0x00 | 8 | magic | 魔数 "\x4E\x57\x42\x46\x00\x01\x00\x00" (NWBF v0.1) |
| 0x08 | 2 | version_major | 主版本号 v0 |
| 0x0A | 2 | version_minor | 次版本号 v1 |
| 0x0C | 1 | image_type | **仅允许 0 = Full**，其他值保留未来使用 |
| 0x0D | 1 | compression_type | 0=None, 3/5/12/19=Zstd (默认 5) |
| 0x0E | 1 | checksum_type | **仅允许 1 = SHA-256** |
| 0x0F | 1 | _reserved_flags | 保留标志位（当前全零） |
| 0x10 | 4 | block_size | 块大小(字节), 默认 65536 (64KB) |
| 0x14 | 4 | _reserved1 | 保留 |
| 0x18 | 8 | timestamp | Unix 纳秒 |
| 0x20 | 16 | source_volume_guid | 源卷 GUID |
| 0x30 | 8 | source_volume_size | 源卷总大小 |
| 0x38 | 16 | backup_uid | 备份 UUID |
| 0x48 | 8 | total_blocks | 总非空洞 Block 数 |
| 0x50 | 8 | total_raw_bytes | 原始数据总大小 |
| 0x58 | 8 | header_crc32c | Header 自身 CRC32C |
| 0x60 | 8 | _reserved2 | 扩展预留 |
| 0x68 | 8 | _reserved3 | 扩展预留 |
| 0x70 | 16 | _reserved4 | 扩展预留 |
| **0x80** | — | — | Block Index Table 从此开始 |

### 3.2 枚举值

```rust
#[repr(u8)]
enum ImageTypeV01 {
    Full = 0,   // 唯一允许值
    // 1..255 = Reserved / Future
}

enum CompressionType {
    None       = 0,
    ZstdLevel3 = 3,   // 快速
    ZstdLevel5 = 5,   // 正常(默认)
    ZstdLevel12 = 12, // 高压缩
    ZstdLevel19 = 19, // 最高
}

enum ChecksumTypeV01 {
    Sha256 = 1,   // 唯一允许值，必须使用 SHA-256
}
```

---

## 4. Block Index Entry

### 4.1 二进制布局 (32 字节/条)

```
#[repr(C, packed)]
struct BlockIndexEntryV01 {
    source_offset:      u64,   // 源卷偏移（字节）
    file_offset:        u64,   // 在 .nwb 文件中的数据偏移（字节）
    compressed_size:    u32,   // 压缩后数据大小（字节），0 = 空洞块
    decompressed_size:  u32,   // 解压后数据大小（字节）
    flags:              u8,    // 标志位
        bit 0: is_sparse       // 1 = 空洞块（隐式填零）
        bit 1: is_compressed   // 0 = 未压缩存储
        bit 2..7: reserved
    _padding:           u8,    // 对齐填充
    checksum:           [u8; 32], // SHA-256（32 字节）
}
```

### 4.2 空洞块处理

空洞块（Sparse Block）在索引中标记 `is_sparse = 1`：
- `compressed_size = 0`
- `file_offset = 0`
- `checksum = 全零（SHA-256 零值）`
- 恢复时隐式填零，不占用文件空间

这就是 Acronis 的方式：空洞完全不在数据区中出现，恢复时隐式填零。

---

## 5. Block Index Table

```
结构：
┌─────────────────────────────────┐
│ block_count: u64 (8 字节)       │ ← 索引条目总数
├─────────────────────────────────┤
│ BlockIndexEntryV01 [0] (32B)    │
│ BlockIndexEntryV01 [1] (32B)    │
│ ...                             │
│ BlockIndexEntryV01 [N-1] (32B)  │
├─────────────────────────────────┤
│ table_crc32c: u64 (8 字节)      │ ← 整个索引表的 CRC32C
└─────────────────────────────────┘
```

---

## 6. Block Data

Block Index Table 之后的连续数据区：

```
Block Data 区域：
┌─ Block #0 压缩数据 (compressed_size 字节) ──┐
│  如果 is_compressed = 1 → zstd 压缩数据      │
│  如果 is_compressed = 0 → 原始数据           │
├─ Block #1 压缩数据 ──────────────────────────┤
│  ...                                          │
└───────────────────────────────────────────────┘
```

空洞块在数据区中不占空间，Block Data 区域的偏移由 `file_offset` 字段跳跃定位。

---

## 7. File Trailer (64 字节)

### 7.1 布局

| 偏移 | 大小 | 字段 | 说明 |
|------|------|------|------|
| 0x00 | 8 | total_blocks | 总非空洞 Block 数 |
| 0x08 | 8 | total_raw_bytes | 原始数据总大小 |
| 0x10 | 8 | total_compressed_bytes | 压缩后总大小 |
| 0x18 | 4 | trailer_crc32c | Trailer 自身 CRC32C |
| 0x1C | 4 | _reserved | 保留 |
| 0x20 | 32 | file_sha256 | 全文件 SHA-256（仅校验 Block Index + Block Data） |
| **0x40** | — | — | Trailer 结束 |

---

## 8. 校验和验证体系

| 层级 | 算法 | 存储位置 | 验证时机 |
|------|------|---------|---------|
| 文件头 | CRC32C | header_crc32c | 打开文件时 |
| Block Index Table | CRC32C | table_crc32c | 加载索引时 |
| 逐块 | SHA-256 (32B) | BlockIndexEntry.checksum | 每次读取 Block 时 |
| 全文件 | SHA-256 | file_sha256 | 备份完成/定期巡检/手动 |

---

## 9. 格式版本兼容

```
major 变更 → 不向前兼容（格式重写）
minor 变更 → 向前兼容（利用 _reserved 字段扩展）

当前版本: major=0, minor=1（实验阶段）

v0.x 阶段：
  · 格式可向后不兼容变更
  · 每个实验版本之间的 .nwb 文件不保证兼容
  · 正式冻结从 v1.0 开始
```

---

## 10. 与 Phase 1 的关系

| 项目 | Phase 1 (MVP) | .nwb v0.1 (Phase 2) |
|------|---------------|---------------------|
| 备份格式 | 平文件存储（目录 + JSON manifest） | .nwb 二进制镜像文件 |
| 备份粒度 | 文件级 | 块级（卷级） |
| 压缩 | zstd（可选，单线程） | zstd（管线化） |
| 校验 | SHA-256 文件级 | SHA-256 块级 |
| 依赖项 | 无 | VSS 快照（Phase 3 才引入） |
| CLI 集成 | nuwa backup/restore 使用平文件 | nuwa image backup/restore 使用 .nwb |

---

## 附录 A：Phase 1 平文件存储格式（参考）

Phase 1 不使用 .nwb 格式，采用简单的目录 + JSON manifest 方式：

```
D:\Backup\
└── 20260705_MyDocs/
    ├── manifest.json（备份元数据 + 文件清单 + SHA-256）
    ├── file1.txt（或 file1.txt.zst — 压缩时）
    ├── folder/
    │   ├── file2.docx.zst
    │   └── ...
    └── ...
```

manifest.json 结构：

```json
{
  "backup_uid": "a1b2c3d4-...",
  "timestamp": "2026-07-05T10:00:00Z",
  "source_path": "C:\\Users\\MyDocs",
  "backup_type": "full",
  "compression": "zstd_level_5",
  "total_files": 1234,
  "total_size_bytes": 5242880000,
  "files": [
    {
      "rel_path": "Documents/report.docx",
      "size": 2048000,
      "sha256": "abc123...",
      "compressed": true
    },
    ...
  ]
}
```

---

## 附录 B：Future Format Expansion（明确不在 v0.1 实现）

以下内容仅作为未来格式扩展的技术记录，**不在 v0.1 版本中实现**：

### B.1 Differential 支持（v0.3+）
- Header 中 image_type = 1 预留
- 差异索引：与全量索引合并后定位块
- 依赖检查：差异 .nwb 必须引用一个全量 .nwb

### B.2 XOR Parity / Erasure Coding（v1.0+）
- Block Group 结构：每组固定 N 个数据块 + 1 个 XOR 块
- 损坏检测：发现损坏时用 XOR 恢复
- 开销：约 1.56% 存储开销

### B.3 AES 加密（v1.0+）
- Header 中 encryption_type = 1 预留
- 加密粒度：逐块加密
- Key 派生：用户密码 + salt → PBKDF2 → AES-256-GCM

### B.4 Block Group 结构（v0.2+）
- 组内 65,536 个块的连续索引
- 分组目的：降低索引内存占用 + 支持部分校验
- 组索引独立 CRC32C 校验

---

## Revision History

| 版本 | 日期 | 变更原因 |
|------|------|---------|
| v1.0 | 2026-07-04 | 初始完整格式规范 |
| v0.1 | 2026-07-05 | 灾备专家审查后：降级为 v0.1 实验阶段，移除 Differential/XOR/加密/Block Group，简化索引结构，明确 Phase 1 不使用 .nwb |
