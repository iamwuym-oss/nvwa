# Nüwa Repository Engine Architecture v1.0

**Product:** Nüwa Backup
**Version:** v1.0
**Status:** Architecture Frozen Baseline
**Date:** 2026-07-10
**Scope:** Phase S — Repository Engine

---

## 1. Overview

### 1.1 Purpose

Repository Engine is the unified storage management layer for Nüwa Backup. It is the shared data foundation for all future backup types (file, volume, disk, system image).

**Repository Engine owns:**

- Backup Instance management
- Restore Point management
- Metadata management
- Block Storage management
- Transaction consistency (cross-component atomic commit)
- Recovery assurance (three-level verification + index rebuild)

**Repository Engine does not own:**

- Data collection (Collector responsibility)
- Chunk generation (ChunkEngine responsibility)
- Scheduling (Scheduler responsibility)
- UI presentation (GUI layer responsibility)

### 1.2 Why Phase S Exists

Phase S (Storage Foundation) is independent from Phase 2.5 (Desktop GUI). Reasons:

1. Volume Backup is not the real challenge — building a reliable storage foundation is
2. Storage engine is the shared architecture base for all future features
3. Storage management should not couple with specific Collectors
4. Independent phase = independent testing, independent validation, independent delivery

### 1.3 Document Status

This document is the **sole architecture baseline for Phase S development**. All Phase S implementation must follow the definitions in this document. No new architecture decisions are introduced here.

---

## 2. Design Principles

### 2.1 Immutable Block Storage

block-store: **Write Once, Read Many, Never Modify**

- Block files, once written to block-store, are never modified
- Block filename = SHA-256(raw data) — content addressing guarantees immutability
- Atomic write (.tmp → rename) prevents partial-write corruption
- Immutability is the foundation of data integrity verification

### 2.2 Metadata First

**Metadata describes data. Block does not describe itself.**

- A block file contains only compressed data + header (identity, size, compression algorithm)
- A block file does not know which Restore Point, which Job, or which file it belongs to
- Metadata (repo.db, backup-metadata.json, catalog.db, block-map.db) is the sole logical view of data
- Losing Metadata leaves data uninterpretable, even when block-store is intact

### 2.3 Recovery Transparency

Recovery depends on three-layer data integrity:

```
Repository Metadata (repo.db)         →  Rebuildable
Logical Mapping (block-map + catalog) →  NOT Rebuildable
Physical Block (block-store)          →  Not self-describing
```

### 2.4 Content Addressing != Deduplication

```
Phase S:
  SHA-256(raw data) = Block Identity
  Purpose: integrity verification, immutable addressing
  NOT: cross-Restore-Point block sharing, reference counting, physical dedup

Phase 6+:
  Global deduplication engine
  Purpose: cross-backup-set block sharing
  Implementation: reference counting + GC
```

### 2.5 Data Source Agnostic

Repository Engine does not know any data source type. It only processes Chunks.

```
FileCollector  ─┐
VolumeCollector ─┼─ RawDataStream → ChunkEngine → Repository
DiskCollector   ┘
```

- Collector: "where to read data from"
- ChunkEngine: "how to organize storage granularity"
- Repository: "how to store blocks"

---

## 3. Repository Directory Layout

### 3.1 Full Layout

```
<repository_root>/
│
├── .nuwarepo/
│   ├── repo.db                     # Global backup catalog (SQLite)
│   └── repo.db.bak                 # Periodic backup of repo.db
│
├── backup-instances/
│   └── {restore_point_id}/
│       ├── backup-metadata.json    # This Restore Point's metadata
│       ├── block-map.db            # Block mapping (SQLite)
│       └── catalog.db              # File index (SQLite)
│
├── block-store/
│   └── {hash[0:2]}/
│       └── {hash[2:4]}/
│           └── {full_hash}.block   # Block data file
│
└── legacy/
    └── (Phase 1/2.5 flat-file backups, read-only)
```

### 3.2 Directory Specifications

| Path | Purpose | Protection |
|------|---------|-----------|
| .nuwarepo/repo.db | Global backup catalog | .bak automatic backup |
| backup-instances/{id}/ | Per-Restore-Point data | Transaction-dependent |
| block-store/ | Immutable block storage | Content-addressed self-verify |
| legacy/ | Old format backups, read-only | Compatibility layer |

### 3.3 Layout Rationale

- Two-level hash subdirectory (`ab/12/hash.block`) controls per-directory file count: 256 x 256 = 65,536 directories. At 100TB / 256KB ~ 400M blocks, ~6,100 files per directory — manageable.
- Hash prefix distribution is uniform by SHA-256 randomness, no rebalancing needed.
- Each Restore Point's data (metadata + block-map + catalog) in an isolated directory provides natural fault isolation.

---

## 4. Data Model

### 4.1 Three-Layer Model

```
Backup Job (user-defined, persists)
    │
    ├── Restore Point 001 (one execution)
    │   └── Backup Instance — the actual stored data
    │
    ├── Restore Point 002
    │   └── Backup Instance
    │
    └── Restore Point 003
        └── Backup Instance
```

### 4.2 Backup Job

A user-defined backup task. Persists until the user deletes it.

```
job_id, job_name, source_type, source_path, created_at, status
```

### 4.3 Restore Point

One execution result. Tracks chain relationships.

```
point_id, job_id, chain_id, chain_position, created_at, status
```

### 4.4 Backup Instance

The stored data for one Restore Point. Each Backup Instance is a self-contained directory:

```
backup-instances/{restore_point_id}/
├── backup-metadata.json
├── block-map.db
└── catalog.db
```

---

## 5. Block Storage Model

### 5.1 Block Definition

| Property | Value |
|----------|-------|
| Default Block Size | **256 KB** (262,144 bytes) |
| Hash Algorithm | **SHA-256** |
| Hash Target | **Raw data** (uncompressed) |
| Block Identity | block_id = SHA-256(raw_chunk_data) |
| Compression | zstd (configurable level) |
| Storage Format | Fixed header (64B) + compressed data |
| Immutability | Never modified after write |
| Sparse Handling | No padding; raw_size records exact size |

### 5.2 Block Header (64 bytes, fixed)

```
Offset  Size  Field              Value
────── ───── ─────────────────── ─────────────────
0-3     4    Magic               b"NWBL"
4-5     2    Version             1
6-15   10    Reserved            Zero (future flags)
16-17   2    Hash Algorithm      0 = SHA-256
18-19   2    Compression         0 = none, 1 = zstd
20-27   8    Raw Size            Original data size
28-35   8    Stored Size         Compressed data size
36-59  24    Reserved            Zero (future extension)
60-63   4    Header CRC32C       CRC32C of Header
```

### 5.3 Small File Policy

```
File size 4KB, block size 256KB:
  → ChunkEngine produces 1 chunk
  → raw_size = 4096 (not 262144)
  → Only 4KB is compressed and stored
  → block-map records raw_size = 4096

Key: Block size is a SPLIT THRESHOLD, not a PADDING TARGET.
```

---

## 6. Chunk Engine

### 6.1 Responsibility

ChunkEngine converts RawDataStream into uniformly-sized Chunks, compresses and hashes them.

**ChunkEngine does not know data source types.** It receives RawDataStream, outputs Chunks.

### 6.2 RawDataStream

```
DataCollector → RawDataStream → ChunkEngine

RawSegment {
    data: Vec<u8>,          # Raw data, uncompressed
    logical_offset: u64,    # Position in data source
}
```

### 6.3 ChunkPolicy

```rust
trait ChunkPolicy {
    fn name(&self) -> &str;
    fn nominal_size(&self) -> u32;
    fn split(&self, data: &[u8], offset: u64) -> Vec<Chunk>;
}
```

**Phase S implementation: FixedChunkPolicy { size: 262144 }**

**Phase 6+ extension: CDCChunkPolicy**

### 6.4 Processing Flow

```
RawSegment { data, logical_offset }
    │
    ▼
ChunkPolicy.split(data, logical_offset)
    │
    ▼
Vec<Chunk>
    │
    ▼
For each Chunk:
    ├── zstd compress(chunk.data)
    ├── SHA-256(chunk.data) = block_id
    ├── Build BlockHeader (64B)
    └── Write to BlockStore
```

### 6.5 Collector vs ChunkEngine

```
Collector: "where to read data from"
  → Returns RawDataStream
  → Handles IO batching internally
  → Does NOT know ChunkPolicy

ChunkEngine: "how to organize storage granularity"
  → Splits by ChunkPolicy
  → Compresses + hashes
  → Does NOT know data source type
```

---

## 7. Metadata Model

### 7.1 backup-metadata.json

Lightweight JSON, no block list:

```json
{
  "schema_version": "1.0",
  "restore_point_id": "uuid",
  "job_id": "uuid",
  "source_type": "file",
  "source_description": "C:\\Users\\Tony\\Documents",
  "created_at": "2026-07-10T10:30:00Z",
  "status": "committed",

  "block_chunk_policy": {
    "type": "fixed",
    "size": 262144
  },

  "block_map": {
    "database": "block-map.db",
    "block_count": 4000000,
    "sha256": "abc...",
    "first_offset": 0,
    "last_offset": 1048576000000
  },

  "catalog": {
    "database": "catalog.db",
    "file_count": 1234,
    "sha256": "def..."
  },

  "summary": {
    "total_raw_bytes": 1099511627776,
    "file_count": 1234
  }
}
```

**block_map.sha256**: corruption DETECTION only, NOT for reconstruction.

### 7.2 Repository Truth Hierarchy

```
repo.db                →  Rebuildable (scan backup-instances)
backup-metadata.json   →  NOT rebuildable
block-map.db           →  NOT rebuildable (lost = unrecoverable)
catalog.db             →  NOT rebuildable (lost = full-point restore only)
block-store            →  Not self-describing
```

---

## 8. Catalog Engine

### 8.1 Responsibility

CatalogEngine manages file-level indexing. It answers: "Which file does the user want to restore?"

**Catalog does NOT know about blocks.** It records file structure and extents only.

### 8.2 CatalogEngine Trait

```rust
trait CatalogEngine {
    fn add_file(&mut self, entry: &FileEntry) -> Result<()>;
    fn search_file(&self, pattern: &str) -> Result<Vec<FileEntry>>;
    fn get_file_extents(&self, file_id: i64) -> Result<Vec<FileExtent>>;
    fn verify(&self) -> Result<CatalogIntegrity>;
    fn close(self: Box<Self>) -> Result<()>;
}
```

### 8.3 PerBackupSqliteCatalog Schema

```sql
CREATE TABLE file_entries (
    file_id     INTEGER PRIMARY KEY AUTOINCREMENT,
    path        TEXT NOT NULL,
    file_size   INTEGER NOT NULL,
    sha256      TEXT,
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE file_extents (
    id              INTEGER PRIMARY KEY AUTOINCREMENT,
    file_id         INTEGER NOT NULL,
    logical_offset  INTEGER NOT NULL,
    length          INTEGER NOT NULL,
    FOREIGN KEY (file_id) REFERENCES file_entries(file_id)
);
```

### 8.4 Catalog → BlockMap Bridge

```
     catalog.db                      block-map.db
FileEntry                            BlockMapping
  path: "a.docx"                      logical_offset: 0
  extents: [(0,262144), ...]          block_id: abc...
           │                              │
           └── logical_offset ────────────┘
```

---

## 9. Block Map Engine

### 9.1 Responsibility

BlockMapEngine manages logical-address to physical-block mapping. It answers: "Which blocks are needed to reconstruct this Restore Point?"

### 9.2 BlockMapEngine Trait

```rust
trait BlockMapEngine {
    fn append_blocks(&mut self, blocks: &[BlockMapping]) -> Result<()>;
    fn lookup(&self, logical_offset: u64) -> Result<Option<BlockMapping>>;
    fn lookup_range(&self, start: u64, end: u64) -> Result<Vec<BlockMapping>>;
    fn iterate(&self) -> Box<dyn Iterator<Item = BlockMapping>>;
    fn verify(&self) -> Result<BlockMapIntegrity>;
    fn close(self: Box<Self>) -> Result<()>;
}

struct BlockMapping {
    logical_offset: u64,
    block_id: [u8; 32],
    raw_size: u32,
    is_sparse: bool,
}
```

### 9.3 SqliteBlockMap Schema

```sql
CREATE TABLE block_mappings (
    row_id          INTEGER PRIMARY KEY AUTOINCREMENT,
    logical_offset  INTEGER NOT NULL UNIQUE,
    block_id        TEXT NOT NULL,
    raw_size        INTEGER NOT NULL,
    is_sparse       INTEGER NOT NULL DEFAULT 0
);
```

---

## 10. Crash Consistency Model

### 10.1 Why a Crash Consistency Manager

Backup spans multiple components:

```
Block Store (filesystem)
Block Map (SQLite tx)
Catalog (SQLite tx)
Metadata (JSON file write)
repo.db (SQLite tx)
```

The Crash Consistency Manager guarantees crash consistency across all components via journal + state machine + recovery detection.

### 10.2 State Machine

```
        CREATING
            │
            ▼
        WRITING
            │
            ▼
        VERIFYING
            │
       ┌────┴────┐
       ▼         ▼
   COMMITTED   FAILED

Crash recovery:
  WRITING / VERIFYING at startup
    → Check component consistency
    → Consistent → auto COMMITTED
    → Inconsistent → FAILED
```

### 10.3 Transaction Flow

```
1. BEGIN TRANSACTION
     repo.db: INSERT (status=CREATING)

2. WRITE PHASE
     block-store:    write blocks (immutable, auto-atomic)
     block-map.db:   INSERT mappings
     catalog.db:     INSERT entries
     repo.db:        UPDATE (status=WRITING)

3. VERIFY PHASE
     Sample-verify block SHA-256
     Verify block-map consistency
     Verify catalog consistency
     repo.db:        UPDATE (status=VERIFYING)

4. COMMIT PHASE
     backup-metadata.json: .tmp → rename
     repo.db: UPDATE (status=COMMITTED)

5. On any failure:
     repo.db: UPDATE (status=FAILED)
```

---

## 11. Verify Engine

### 11.1 Three-Level Verification

```
Level 1: Metadata Verification
  repo.db: PRAGMA integrity_check
  backup-metadata.json: JSON schema validation
  block-map/catalog integrity manifest check

Level 2: Block Sampling
  Random sample N blocks
  Verify Magic + Version + CRC32C
  Decompress → SHA-256(raw) == filename
  N = 1% of total blocks (min 100, max 10,000)

Level 3: Full Verification
  Iterate all blocks in block-map
  Verify each block: read, decompress, SHA-256
  For scheduled patrol or pre-restore validation
```

### 11.2 CLI Commands

```
nuwa repo verify                    # Level 1 + Level 2
nuwa repo verify --full             # Level 1 + Level 3
nuwa repo verify --restore-point    # Single Restore Point
```

---

## 12. Retention Engine

### 12.1 Phase S Scope

Phase S Retention Engine performs **logical deletion only, no physical reclamation**.

```
Delete Restore Point:
  ├── Delete backup-instances/{point_id}/
  ├── repo.db: mark restore_point.status = deleted
  └── Record orphan_candidate_count += block_count
```

### 12.2 Orphan Candidates

```
Phase S has no reference counting.
Deleting a Restore Point does not prove blocks are unreferenced.

orphan_candidates = "blocks that MAY no longer be referenced"
NOT = "blocks confirmed safe to delete"

Phase S does NOT:
  Physical block deletion
  Reference counting
  Space reclamation
```

### 12.3 Phase 6+ Full GC

```
With reference counting:
  reference_count++ (on write)
  reference_count-- (on Restore Point deletion)
  When count == 0 → physical block deletion
  Periodic GC scan
```

---

## 13. Repository Recovery

### 13.1 Recovery Scenarios

| Component | Recoverable | Action |
|-----------|------------|--------|
| repo.db corrupted | Yes | Scan backup-instances, rebuild |
| backup-instance lost | No | That Restore Point is lost |
| catalog.db corrupted | Partial | Full-point restore works, no file search |
| block-map.db corrupted | No | Restore Point is unrecoverable |
| block file corrupted | Per-block | Affected files fail verification |

### 13.2 repo.db Rebuild

```
nuwa repo rebuild
  ├── Scan backup-instances/ for backup-metadata.json
  ├── Rebuild restore_points table
  ├── Rebuild backup_jobs table
  └── Generate complete repo.db
```

### 13.3 Recovery Boundaries

```
repo rebuild != restore rebuild

repo rebuild:
  Rebuild indexes. Repository becomes browsable.

restore rebuild:
  Recover corrupted backup data — NOT SUPPORTED in Phase S.
  If block-map.db or catalog.db is corrupted, the user must re-backup.
```

### 13.4 CLI Commands

```
nuwa repo check        # Detect component consistency
nuwa repo verify       # Verify Engine (three-level)
nuwa repo rebuild      # repo.db rebuild
nuwa repo orphans     # List orphan candidates
```

---

## 14. Legacy Compatibility

### 14.1 Policy

```
Phase 1/2.5 flat-file backups:
  Permanently readable (read-only)
  No new flat-file backups will be created
  No migration to new format
  No conversion
```

### 14.2 Legacy Directory

```
legacy/
├── 20260705_MyDocs/
│   ├── manifest.json
│   └── files/...
├── 20260706_WorkDocs/
└── ...
```

### 14.3 LegacyAdapter Interface

```rust
trait LegacyAdapter {
    fn list_restore_points() -> Vec<LegacyRestorePoint>;
    fn restore_full(point_id, target_path) -> Result<()>;
}
```

Note: Legacy backups are not managed by repo.db. They are accessed through the independent LegacyAdapter.

---

## 15. Phase S Scope Boundary

### 15.1 Phase S Includes

| Module | Description |
|--------|------------|
| S-01 | Repository Layout & Initialization |
| S-02 | Block Store Engine |
| S-03 | Metadata Engine |
| S-04 | Block Map Engine (trait + SqliteBlockMap) |
| S-05 | Catalog Engine (trait + PerBackupSqliteCatalog) |
| S-06 | Backup Chain Engine |
| S-07 | Crash Consistency Manager |
| S-08 | Verify Engine |
| S-09 | Retention Engine |
| S-10 | Legacy Compatibility |
| S-11 | Repository CLI |
| S-12 | Scale Benchmark Framework |
| S-13 | Repository Recovery |

### 15.2 Phase S Excludes

```
VSS integration (Phase 3)
Volume backup/restore (Phase 3)
File Backup migration to new engine (Phase 3.1)
Desktop GUI features (Phase 2.5 parallel)
Global Deduplication / reference counting (Phase 6+)
Physical block deletion / GC (Phase 6+)
Encryption (Phase 6+)
CDC ChunkPolicy (Phase 6+)
Cloud Tier / multi-repository replication
```

---

## 16. Future Extension Roadmap

```
Phase S (Current)
Repository Engine Foundation
  FixedChunkPolicy 256KB
  Immutable Block Store
  Content Addressing (hash raw)
  Crash Consistency Manager
  Retention (orphan candidates only)

Phase 3
Volume Backup (VSS)

Phase 3.1
File Backup Migration

Phase 4
System Image + BMR

Phase 5
Advanced Restore (FLR, Universal Restore)

Phase 6+
Enterprise Features
  Global Deduplication
  Encryption
  CDC ChunkPolicy
  Cloud Tier
  Scale-out Repository
```

---

## 17. Appendix A — Data Loss Scenarios

| # | Failure | Impact | Recovery | User Impact |
|---|---------|--------|----------|-------------|
| 1 | repo.db corrupted | Cannot list backups | Rebuildable | Run `nuwa repo rebuild` |
| 2 | backup-metadata.json lost | One Restore Point gone | NOT recoverable | Point disappears from list |
| 3 | catalog.db corrupted | No file search/restore | Partial | Full-point restore works |
| 4 | block-map.db corrupted | Restore Point unrecoverable | NOT recoverable | Point marked corrupted |
| 5 | Single block corrupted | Affected files fail | Per-skip | Verify failure warning |
| 6 | block-store directory deleted | All dependent points lost | NOT recoverable | Mass data loss |
| 7 | Repository disk physically damaged | All data lost | NOT recoverable | No recovery path — this is backup, not replication |
| 8 | Crash during WRITING phase | In-progress Restore Point | Auto-recoverable | Auto COMMIT or FAILED on restart |
| 9 | Crash during VERIFYING phase | Same as above | Auto-recoverable | Same as above |
| 10 | User mistakenly deletes Restore Point | That Point is gone | Phase 6+: soft-delete undo | Not supported in Phase S |

### Key Principle

Repository Engine cannot solve all data loss. Its goals are:
1. Prevent software-bug-induced data loss (via Crash Consistency Manager)
2. Communicate what is and is not recoverable (transparency)
3. Provide repair paths for recoverable corruption

What it cannot solve:
- User deletes wrong file (needs file history/versioning)
- Physical disk failure (needs off-site replication)
- Natural disaster (needs multi-site replication)

---

## Revision History

| Version | Date | Change | Author |
|---------|------|--------|--------|
| v1.0 | 2026-07-10 | Initial architecture baseline. All decisions frozen. | Codex + Expert Review |
