# Nüwa Repository Engine Implementation Plan v1.0

**Product:** Nüwa Backup
**Version:** v1.0
**Status:** Implementation Baseline
**Date:** 2026-07-10
**Prerequisite:** Architecture v1.0 (frozen)
**Scope:** Phase S — Repository Engine

---

## 1. Rust Module Architecture

### 1.1 Source Tree

Phase S introduces a new `src/repository/` module tree. The existing Phase 1/2.5 code in `src/` (backup.rs, restore.rs, storage.rs, manifest.rs) remains untouched.

```
src/
│
├── main.rs                    # CLI entry (existing, extended with repo commands)
├── ...
│
└── repository/                # NEW — Phase S Repository Engine
    ├── mod.rs                 # Public API re-exports
    │
    ├── repo_manager.rs        # Repository init, open, verify, rebuild
    │
    ├── block_store/           # Block Storage
    │   ├── mod.rs
    │   ├── store.rs           # BlockStore trait + LocalFsBlockStore
    │   ├── block_header.rs    # 64B header encode/decode
    │   └── block_id.rs        # SHA-256(raw) identity helpers
    │
    ├── chunk_engine/          # Chunk Engine
    │   ├── mod.rs
    │   ├── engine.rs          # ChunkEngine: stream → chunks
    │   └── policy.rs          # ChunkPolicy trait + FixedChunkPolicy
    │
    ├── metadata/              # Metadata Engine
    │   ├── mod.rs
    │   ├── models.rs          # Job, RestorePoint, BackupInstance structs
    │   ├── metadata_store.rs  # backup-metadata.json read/write
    │   └── chain.rs           # Backup chain tracking
    │
    ├── catalog/               # Catalog Engine
    │   ├── mod.rs
    │   ├── engine.rs          # CatalogEngine trait
    │   └── sqlite_catalog.rs  # PerBackupSqliteCatalog implementation
    │
    ├── block_map/             # Block Map Engine
    │   ├── mod.rs
    │   ├── engine.rs          # BlockMapEngine trait
    │   └── sqlite_block_map.rs # SqliteBlockMap implementation
    │
    ├── transaction/           # Crash Consistency Manager
    │   ├── mod.rs
    │   ├── manager.rs         # Transaction lifecycle: BEGIN → COMMIT/FAIL
    │   └── journal.rs         # Transaction Journal for crash recovery
    │
    ├── verify/                # Verify Engine
    │   ├── mod.rs
    │   └── engine.rs          # Three-level verification
    │
    ├── retention/             # Retention Engine
    │   ├── mod.rs
    │   └── engine.rs          # Logical deletion + orphan candidates
    │
    ├── recovery/              # Repository Recovery
    │   ├── mod.rs
    │   ├── rebuild.rs         # repo.db scan-based rebuild
    │   └── integrity_check.rs # cross-component consistency check
    │
    ├── legacy/                # Legacy Compatibility
    │   ├── mod.rs
    │   └── adapter.rs         # flat-file read-only adapter
    │
    ├── cli/                   # Repository CLI
    │   ├── mod.rs
    │   └── commands.rs        # repo init/check/verify/rebuild/orphans
    │
    └── error.rs               # RepositoryError enum
```

### 1.2 Module Dependency Direction

```
CLI (main.rs)
    │
    ▼
repo_manager
    │
    ├── metadata
    ├── block_store
    ├── chunk_engine
    ├── catalog (trait)
    ├── block_map (trait)
    ├── transaction (journal + state machine)
    ├── verify
    ├── retention
    ├── recovery
    └── legacy

No circular dependencies.
Traits (catalog, block_map) are defined in their modules;
SQLite implementations depend on the traits.
```

---

## 2. Phase S Task Breakdown

### S-01: Repository Layout & Initialization

**File:** `repo_manager.rs`

```rust
/// Initialize a new Repository
fn init_repo(dest: &Path, block_size: u32) -> Result<()>;
    // Creates .nuwarepo/, repo.db, block-store/ skeleton
    // Writes chunk_policy metadata to repo.db
    // block_size: fixed, 262144 default

/// Open an existing Repository
fn open_repo(path: &Path) -> Result<RepoHandle>;
    // Validates directory structure
    // Checks repo.db integrity
    // Returns handle for subsequent operations

/// Validate Repository structure
fn check_repo(handle: &RepoHandle) -> Result<RepoStatus>;
    // Checks: .nuwarepo/ exists, repo.db readable, block-store/ accessible
```

### S-02: Block Store Engine

**Files:** `block_store/store.rs`, `block_header.rs`, `block_id.rs`

```rust
/// BlockStore trait — storage backend abstraction
trait BlockStore: Send + Sync {
    fn put_block(&self, block: &Block) -> Result<BlockId>;
    fn get_block(&self, id: &BlockId) -> Result<Block>;
    fn exists(&self, id: &BlockId) -> Result<bool>;
    fn verify_block(&self, id: &BlockId) -> Result<bool>;
        // Read header → decompress → SHA-256(raw) == id
}

/// Phase S implementation — LocalFsBlockStore
struct LocalFsBlockStore { root: PathBuf }

struct Block {
    header: BlockHeader,   // 64B
    data: Vec<u8>,         // compressed data
}

struct BlockHeader {
    magic: [u8; 4],        // b"NWBL"
    version: u16,
    hash_algorithm: u16,   // 0 = SHA-256
    compression: u16,      // 0 = none, 1 = zstd
    raw_size: u64,
    stored_size: u64,
    header_crc: u32,
}

struct BlockId([u8; 32]);  // SHA-256(raw data)
```

**Atomic write:** .tmp → rename within block-store/

### S-03: Metadata Engine

**Files:** `metadata/models.rs`, `metadata_store.rs`, `chain.rs`

```rust
/// Data models
struct BackupJob {
    job_id: String,
    job_name: String,
    source_type: SourceType,
    source_path: String,
    created_at: DateTime<Utc>,
    status: JobStatus,
}

struct RestorePoint {
    point_id: String,
    job_id: String,
    chain_id: String,
    chain_position: u32,    // 0 = Full, 1+ = Incremental
    created_at: DateTime<Utc>,
    status: PointStatus,    // CREATING → WRITING → VERIFYING → COMMITTED / FAILED
}

/// backup-metadata.json: write on COMMIT, read for listing
fn write_metadata(instance_dir: &Path, metadata: &BackupInstanceMetadata) -> Result<()>;
    // Uses .tmp → rename atomic write

fn read_metadata(instance_dir: &Path) -> Result<BackupInstanceMetadata>;
```

**repo.db schema:**

```sql
CREATE TABLE backup_jobs (
    job_id          TEXT PRIMARY KEY,
    job_name        TEXT NOT NULL,
    source_type     INTEGER NOT NULL,   -- 0=file, 1=volume, 2=disk
    source_path     TEXT,
    created_at      TEXT NOT NULL,
    status          TEXT NOT NULL DEFAULT 'active'
);

CREATE TABLE restore_points (
    point_id        TEXT PRIMARY KEY,
    job_id          TEXT NOT NULL,
    chain_id        TEXT NOT NULL,
    chain_position  INTEGER NOT NULL,
    created_at      TEXT NOT NULL,
    status          TEXT NOT NULL,       -- CREATING / WRITING / VERIFYING / COMMITTED / FAILED
    instance_path   TEXT NOT NULL,
    block_count     INTEGER NOT NULL DEFAULT 0,
    total_raw_bytes INTEGER NOT NULL DEFAULT 0,
    parent_point_id TEXT,
    FOREIGN KEY (job_id) REFERENCES backup_jobs(job_id)
);
```

### S-04: Block Map Engine

**Files:** `block_map/engine.rs`, `sqlite_block_map.rs`

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

struct BlockMapIntegrity {
    block_count: u64,
    first_offset: u64,
    last_offset: u64,
    sha256: String,
}
```

**block-map.db schema:**

```sql
CREATE TABLE block_mappings (
    row_id          INTEGER PRIMARY KEY AUTOINCREMENT,
    logical_offset  INTEGER NOT NULL UNIQUE,
    block_id        TEXT NOT NULL,
    raw_size        INTEGER NOT NULL,
    is_sparse       INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_block_offset ON block_mappings(logical_offset);
```

**Integrity manifest (in backup-metadata.json):**

```json
{ "block_map": { "block_count": N, "first_offset": 0, "last_offset": X, "sha256": "hex" } }
```

Purpose: corruption DETECTION only. NOT for reconstruction.

### S-05: Catalog Engine

**Files:** `catalog/engine.rs`, `sqlite_catalog.rs`

```rust
trait CatalogEngine {
    fn add_file(&mut self, entry: &FileEntry) -> Result<()>;
    fn search_file(&self, pattern: &str) -> Result<Vec<FileEntry>>;
    fn get_file_extents(&self, file_id: i64) -> Result<Vec<FileExtent>>;
    fn verify(&self) -> Result<CatalogIntegrity>;
    fn close(self: Box<Self>) -> Result<()>;
}

struct FileEntry {
    path: String,
    file_size: u64,
    sha256: Option<String>,
}

struct FileExtent {
    logical_offset: u64,
    length: u64,
}
```

**catalog.db schema:**

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

CREATE INDEX idx_file_path ON file_entries(path);
CREATE INDEX idx_extent_file ON file_extents(file_id);
```

**Catalog → BlockMap bridge:** Extent's logical_offset → BlockMapEngine.lookup()

### S-06: Backup Chain Engine

**File:** `metadata/chain.rs`

```rust
struct BackupChain {
    chain_id: String,
    restore_points: Vec<RestorePointRef>,
}

struct RestorePointRef {
    point_id: String,
    chain_position: u32,      // 0=Full, 1,2,3=Incremental
    parent_point_id: Option<String>,  // NULL for Full
}

/// Resolve a chain position to its set of required blocks
fn resolve_chain(chain: &BackupChain, position: u32) -> Result<Vec<String>>;
    // Walk from position back to 0, collecting all block IDs
    // Used to determine which block-map.db entries to read
```

### S-07: Crash Consistency Manager

**Files:** `transaction/manager.rs`, `journal.rs`

**Name changed per review: "Transaction Manager" → "Crash Consistency Manager"**

This module guarantees crash consistency, not distributed ACID.

```rust
/// State machine
enum TransactionState {
    CREATING,
    WRITING,
    VERIFYING,
    COMMITTED,
    FAILED,
}

/// Lifecycle
let txn = CrashConsistencyManager::begin(repo, &restore_point)?;
    // INSERT restore_point (status=CREATING)
    // Create transaction journal

txn.write_blocks(block_stream)?;
    // Write blocks to block-store
    // Write block-map.db entries
    // Write catalog.db entries
    // Update journal

txn.verify()?;
    // Sample-verify blocks
    // Verify block-map integrity
    // Verify catalog integrity
    // Update journal

txn.commit()?;
    // Atomic rename: metadata.json.tmp → metadata.json
    // UPDATE restore_point (status=COMMITTED)
    // Remove journal file

// On any failure:
txn.fail()?;
    // UPDATE restore_point (status=FAILED)
    // Keep journal for orphan tracking
```

**Transaction Journal (`transactions/txn-{restore_point_id}.log`):**

```json
{
  "restore_point_id": "uuid",
  "state": "WRITING",
  "components": {
    "block-store": "completed",
    "block-map":   "completed",
    "catalog":     "in_progress",
    "metadata":    "pending"
  },
  "started_at": "2026-07-10T10:30:00Z"
}
```

**Startup recovery process:**

```
Repository opens:
  1. Scan transactions/ for incomplete journals
  2. For each journal:
     a. Read component statuses
     b. Check which components have actual data
     c. If all components complete → auto-transition to COMMITTED
     d. If any component incomplete → mark FAILED
     e. Remove processed journal
  3. Report any recovered or failed transactions
```

### S-08: Verify Engine

**File:** `verify/engine.rs`

```rust
/// Three-level verification
fn verify(handle: &RepoHandle, level: VerifyLevel) -> Result<VerifyReport>;

enum VerifyLevel {
    Metadata,          // Level 1: repo.db + schema validation
    Sampling(u32),     // Level 2: random N blocks, full verify
    Full,              // Level 3: all blocks, full verify
}

struct VerifyReport {
    total_restore_points: u64,
    corrupted_points: Vec<String>,
    total_blocks: u64,
    verified_blocks: u64,
    failed_blocks: u64,
    data_loss_detected: bool,
}
```

### S-09: Retention Engine

**File:** `retention/engine.rs`

```rust
fn apply_retention(handle: &RepoHandle, policy: &RetentionPolicy) -> Result<RetentionResult>;

struct RetentionPolicy {
    keep_days: u32,               // Delete points older than N days
    keep_full_count: u32,         // Keep at least N full backups
    // Phase S: simple time-based only
}

struct RetentionResult {
    deleted_points: Vec<String>,
    orphan_candidate_count: u64,
}

// Retention uses a two-phase delete for crash safety:
//   Phase 1: repo.db status = DELETING
//   Phase 2: delete backup-instances/{point_id}/
//   Phase 3: repo.db status = DELETED
//   Phase 4: record orphan_candidate_count
//
// Why two-phase:
//   If delete-instance succeeds but repo.db update fails,
//   the Restore Point would appear in listing but have no data.
//   Phase 1 mark prevents this: DELETING → rollback is detectable.
//
// Does NOT delete block files from block-store/
```

### S-10: Legacy Compatibility

**File:** `legacy/adapter.rs`

```rust
struct LegacyAdapter {
    legacy_root: PathBuf,   // points to legacy/ directory
}

impl LegacyAdapter {
    fn list_backups(&self) -> Result<Vec<LegacyBackupInfo>>;
        // Scan subdirectories for manifest.json

    fn restore_point(&self, point_id: &str, target: &Path) -> Result<()>;
        // Original flat-file restore logic
        // Reads manifest.json + files/ directory

    fn verify_point(&self, point_id: &str) -> Result<bool>;
        // Re-run SHA-256 verification on legacy flat-file backup
}
struct LegacyBackupInfo {
    point_id: String,
    source: String,
    created_at: String,
    file_count: u64,
    total_bytes: u64,
}
```

### S-11: Repository CLI

**File:** `cli/commands.rs`

| Command | Action |
|---------|--------|
| `nuwa repo init --dest <path>` | Initialize new Repository |
| `nuwa repo check` | Repository structure + integrity check |
| `nuwa repo verify [--full]` | Three-level verification |
| `nuwa repo rebuild` | Rebuild repo.db from backup-instances/ |
| `nuwa repo orphans` | List orphan block candidates |
| `nuwa repo list` | List all Restore Points |

### S-12: Scale Benchmark Framework

Independent benchmark tool to verify metadata scaling:

```
# Benchmark scenarios
nuwa-repo-bench --blocks 1M     # 1 million blocks
nuwa-repo-bench --blocks 10M    # 10 million blocks
nuwa-repo-bench --blocks 100M   # 100 million blocks (if feasible)

# Metrics measured:
#   - Block Map insert throughput (ops/sec)
#   - Block Map lookup latency (p50, p95, p99)
#   - Catalog insert throughput
#   - Catalog search latency
#   - Verify Engine throughput (blocks/sec)
#   - Directory distribution uniformity
```

Benchmark is a separate binary or test, NOT included in production builds.

### S-13: Repository Recovery

**Files:** `recovery/rebuild.rs`, `integrity_check.rs`

```rust
/// Cross-component integrity check
fn check_integrity(handle: &RepoHandle) -> Result<IntegrityReport>;

/// Rebuild repo.db from backup-instances/
fn rebuild_repo(repo_path: &Path) -> Result<()>;
    // Scan backup-instances/*/backup-metadata.json
    // Rebuild repo.db tables
    // Verify block-store accessibility

/// Recovery boundary:
///   repo.db → rebuildable
///   block-map.db → NOT rebuildable
///   catalog.db → NOT rebuildable
///   These boundaries are enforced in code.
```

---

## 3. Error Model

```rust
pub enum RepositoryError {
    // ======== Repository Initialization ========
    #[error("Repository already exists at {0}")]
    AlreadyExists(PathBuf),

    #[error("Not a valid repository: {0}")]
    InvalidRepository(String),

    // ======== Block Store ========
    #[error("Block not found: {0}")]
    BlockNotFound(BlockId),

    #[error("Block checksum mismatch: expected {expected}, got {actual}")]
    BlockCorrupted { expected: String, actual: String },

    #[error("Block header invalid: {0}")]
    InvalidBlockHeader(String),

    // ======== Metadata ========
    #[error("Metadata file not found: {0}")]
    MetadataMissing(PathBuf),

    #[error("Metadata schema version unsupported: {0}")]
    UnsupportedSchemaVersion(String),

    // ======== Block Map ========
    #[error("Block map index corrupted: {0}")]
    BlockMapCorrupted(String),

    #[error("Logical offset not found: {0}")]
    OffsetNotFound(u64),

    // ======== Catalog ========
    #[error("Catalog database corrupted: {0}")]
    CatalogCorrupted(String),

    #[error("File not found in catalog: {0}")]
    FileNotFound(String),

    // ======== Transaction / Consistency ========
    #[error("Transaction in progress: {0}")]
    TransactionInProgress(String),

    #[error("Transaction consistency check failed: {0}")]
    TransactionInconsistent(String),

    // ======== Recovery ========
    #[error("Component not rebuildable: {0} (block-map/catalog cannot be rebuilt from block-store)")]
    NotRebuildable(String),

    // ======== Repository Structural ========
    #[error("I/O error: {context} ({source})")]
    IoError { context: String, source: std::io::Error },

    #[error("JSON error: {0}")]
    JsonError(serde_json::Error),

    #[error("SQLite error: {0}")]
    SqliteError(rusqlite::Error),
}
```

**Error handling rule:** Every RepositoryError must be:
1. User-visible (Display impl with Chinese hint)
2. Distinguishable by CLI error code
3. Traced back to component source

---

## 4. Test Strategy

### 4.1 Unit Tests (per module)

| Module | Test scenarios | Count (min) |
|--------|---------------|-------------|
| block_header | Encode/decode round-trip, CRC validation, reserved fields zero | 5 |
| block_id | SHA-256 consistency, hex formatting | 3 |
| chunk_policy | Fixed 256KB split, small file exact size, large file multi-chunk | 5 |
| metadata models | Serialize/deserialize, field validation | 5 |
| chain | Full→Inc chain resolution, orphan detection | 4 |

### 4.2 Integration Tests

| Test | Description |
|------|-------------|
| repo_init_create | Init new repo, verify directory structure |
| repo_init_existing | Init on existing repo → error |
| block_write_read | Write block → read back → SHA-256 match |
| block_verify_corrupted | Tamper block file → verify fails |
| block_map_insert_lookup | Insert 10K mappings → lookup each |
| block_map_range_scan | Insert with non-contiguous offsets → range query |
| catalog_add_search | Add 1000 file entries → search by pattern |
| catalog_extent_bridge | Add file extents → lookup via logical_offset → match block |
| transaction_full_cycle | BEGIN → WRITE → VERIFY → COMMIT → repo.db has COMMITTED |
| transaction_crash_recovery | Simulate crash at WRITING → restart → auto-recover |
| retention_logical_delete | Delete Restore Point → metadata gone, blocks present |
| chain_resolve_full | Full → verify all blocks returned |
| chain_resolve_incremental | Full + Inc + Inc → resolve position 2 → 3 sets of blocks |
| repo_rebuild | Delete repo.db → rebuild → listing matches original |

### 4.3 Crash Tests (mandatory for Phase S)

Crash tests simulate process termination at critical points:

```
Test A: Kill after block write (before block-map)
  → repo check detects incomplete transaction
  → blocks present, no mapping → FAILED

Test B: Kill after block-map commit (before catalog)
  → blocks mapped, catalog missing → FAILED
  → block-store blobs are orphan candidates

Test C: Kill after catalog commit (before metadata rename)
  → .tmp metadata exists → recovery COMMIT
  → system auto-detects and completes transaction

Test D: Kill after metadata rename (before repo.db update)
  → metadata present, repo.db status still WRITING
  → recovery: components complete → auto COMMIT
```

**Crash test implementation:** Use an injection point callback that exits the process at a specific phase. No hacky `std::process::exit()` — use a controlled `TestCrashGuard`:

```rust
// Used ONLY in test builds
struct TestCrashGuard {
    injection_point: Option<&'static str>,
}

impl TestCrashGuard {
    fn check(&self) {
        if let Some(point) = self.injection_point {
            panic!("Simulated crash at: {}", point);
        }
    }
}
```

### 4.4 Legacy Compatibility Tests

| Test | Description |
|------|-------------|
| legacy_list_flatfile | List Phase 1 flat-file backups through LegacyAdapter |
| legacy_restore_flatfile | Restore from old flat-file format |
| legacy_verify_flatfile | Verify SHA-256 on old format |
| co_existence | New repo + legacy dir → both accessible |

---

## 5. Benchmark Plan

### 5.1 Block Map Metadata Scale

```
Setup: Generate N block mappings, insert into SqliteBlockMap

Metrics:
  - Insert throughput (ops/sec)
  - Lookup latency (single offset): p50, p95, p99
  - Range lookup (1K offsets): p50, p95, p99
  - Database file size on disk

Targets:
  1M blocks:  insert < 5s, lookup < 1ms p99
  10M blocks: insert < 60s, lookup < 5ms p99
  100M blocks: insert < 15min, lookup < 50ms p99
```

### 5.2 Verify Engine Throughput

```
Setup: Repository with N verified blocks

Metrics:
  - Level 1 (metadata verify): duration
  - Level 2 (sampling 1%): duration
  - Level 3 (full verify): throughput (blocks/sec)

Note: Full verify on 100TB / 256KB = 400M blocks.
  If verify speed = 10,000 blocks/sec → ~11 hours.
  This is acceptable for scheduled patrol (run weekly).
```

### 5.3 Block Distribution Uniformity

```
Setup: Generate N blocks, verify distribution across 65,536 directories

Metric: Chi-square test on directory file count
Pass: p > 0.05 (no significant clustering)
```

---

## 6. Implementation Order (Dependency-Aware)

```
Phase S Task Dependency Graph:

S-01 (repo init)
  └── prerequisite: nothing
  └── provides: directory structure, repo.db
        │
        ▼
S-02 (block_store) ────────────────────────┐
  └── prerequisite: S-01                   │
  └── provides: block file I/O             │
        │                                  │
        ▼                                  │
S-03 (metadata)                             │
  └── prerequisite: S-01                   │
  └── provides: Job/RestorePoint models    │
        │                                  │
        ▼                                  ▼
S-04 (block_map) ←──── S-05 (catalog)
  └── prerequisite: S-01       └── prerequisite: S-01
  └── provides: offset→hash    └── provides: file→extent
        │                          │
        └──────────┬───────────────┘
                   ▼
S-07 (crash consistency)
  └── integrates: S-02, S-03, S-04, S-05
  └── provides: BEGIN→WRITE→VERIFY→COMMIT
        │
        ▼
S-06 (backup chain) ←── depends on S-07 (for actual points)
S-08 (verify)
S-09 (retention)
S-13 (recovery)
        │
        ▼
S-10 (legacy) ←── independent, no dependency on S-02~S-09
S-11 (CLI)   ←── depends on all above
S-12 (bench) ←── depends on S-02, S-04, S-05
```

**Recommended development order:**

```
Wave 1: S-01 → S-02 → S-03
  Foundation: repo init, block I/O, metadata models

Wave 2: S-04 → S-05
  Indexing: block map, catalog (no transaction yet)

Wave 3: S-07
  Crash Consistency Manager (journals + state machine)
  This is the most critical module — invest in tests here

Wave 4: S-06 → S-08 → S-09 → S-13
  Application logic: chain, verify, retention, recovery

Wave 5: S-10 → S-11 → S-12
  Integration: legacy, CLI, benchmarks
```

---

## 7. Phase S Acceptance Criteria

| # | Criteria | Verification |
|---|----------|-------------|
| 1 | `nuwa repo init` creates valid repository | Directory structure check |
| 2 | Block write → read → SHA-256 match | Integration test |
| 3 | Block tampered → verify detects corruption | Integration test |
| 4 | Block Map: insert 1M → lookup each | Benchmark |
| 5 | Catalog: add 10K files → search by path | Integration test |
| 6 | Transaction: full cycle → COMMITTED status | Integration test |
| 7 | Crash at 4 injection points → recovery correct | Crash test |
| 8 | Retention: logical delete → blocks remain | Integration test |
| 9 | repo.db deleted → rebuild → listing restored | Integration test |
| 10 | Legacy flat-file: list + restore | Integration test |
| 11 | Benchmark: 10M blocks meets latency targets | Benchmark |
| 12 | `cargo fmt --check` passes | CI |
| 13 | `cargo clippy --all-targets -- -D warnings` passes | CI |
| 14 | Phase S scope audit: no VSS, no dedup, no encryption | Manual review |

---

## 8. Explicit Forbidden Patterns

The following implementation patterns are explicitly forbidden in Phase S:

```
1. Scanning block-store to reconstruct block-map
   Reason: block-store has no logical_offset → cannot recreate mapping

2. Scanning block-store to reconstruct catalog
   Reason: block-store has no file path → cannot recreate catalog

3. Implementing physical block deletion without reference counting
   Reason: content-addressed blocks may be shared; deletion breaks other points

4. Implementing reference counting in Phase S
   Reason: reserved for Phase 6+ global dedup engine

5. Embedding block file path in block-map or catalog
   Reason: block location is a BlockStore implementation detail

6. Using a single global catalog for all backup-instances
   Reason: per-Backup-Instance catalog provides isolation and portability

7. Hard-coding block-size in ChunkEngine (must use ChunkPolicy trait)
   Reason: future CDC support requires trait abstraction
```

---

## Revision History

| Version | Date | Change | Author |
|---------|------|--------|--------|
| v1.0 | 2026-07-10 | Initial implementation plan. All architecture decisions from Architecture v1.0 are preserved. | Codex |

