# Nüwa Repository Engine Implementation Plan v1.1

**Product:** Nüwa Backup  
**Version:** v1.1 (Enterprise Readiness Sync)  
**Status:** Implementation Baseline  
**Date:** 2026-07-10  
**Prerequisites:** Architecture v1.0 (frozen) + v1.1 Enterprise Readiness Revision  
**Scope:** Phase S — Repository Engine

---

## Revision Purpose

v1.1 synchronizes the Implementation Plan with Architecture v1.1 Enterprise Readiness Revision.
v1.0 task structure, module layout, test strategy, and acceptance criteria remain in full effect.
This document only adds and updates sections affected by v1.1.

For sections not listed here, v1.0 definitions apply unchanged.

---

## 1. Updated: S-01 Repository Init (Enterprise Readiness)

### init_repo() additions

```rust
fn init_repo(dest: &Path, block_size: u32) -> Result<RepoHandle>
    // v1.0 behavior (unchanged):
    //   Creates .nuwarepo/, repo.db, block-store/, backup-instances/, legacy/
    //   Writes chunk_policy metadata to repo.db
    //   block_size: 262144 default, 4KB-4MB range
    // v1.1 additions:
    //   Generates UUID v4 as repository_id
    //   Writes identity metadata to repo.db (repository_meta table):
    //     - repository_id, repository_name, repository_type
    //     - format_version (1), min_compatible_version (1)
    //     - cap_compression (true), cap_encryption (false)
    //     - cap_dedup (false), cap_immutable (false)
    //   Writes .nuwarepo/repository.json (human-readable identity manifest)
```

### open_repo() additions

```rust
fn open_repo(path: &Path) -> Result<RepoHandle>
    // v1.0 behavior (unchanged):
    //   Validates directory structure
    //   Checks repo.db integrity (PRAGMA integrity_check)
    // v1.1 additions:
    //   Reads identity: repository_id, name, type, format_version
    //   Reads capabilities: compression, encryption, dedup, immutable
    //   Validates format_version <= software max compatible
    //   Cross-validates repository.json matches repo.db
```

### RepositoryInfo Struct

```rust
pub struct RepositoryInfo {
    pub version: u32,
    pub chunk_policy: ChunkPolicyMetadata,
    pub restore_point_count: u64,
    pub total_raw_bytes: u64,
    // v1.1 additions:
    pub repository_id: String,
    pub repository_name: String,
    pub repository_type: String,
    pub format_version: u32,
    pub min_compatible_version: u32,
    pub capabilities: RepositoryCapabilities,
}
```

### repository.json

Written to `{repo_root}/.nuwarepo/repository.json` at init time:

```json
{
  "schema_version": "1.0",
  "repository_id": "uuid-v4-string",
  "name": "Nuwa Backup Repository",
  "type": "filesystem",
  "format_version": 1,
  "min_compatible_version": 1,
  "created_at": "2026-07-10T10:00:00Z",
  "capabilities": {
    "compression": true,
    "encryption": false,
    "dedup": false,
    "immutable_storage": false
  }
}
```

## 2. Updated: S-03 Metadata Models

### RepositoryCapabilities (NEW)

```rust
pub struct RepositoryCapabilities {
    pub compression: bool,
    pub encryption: bool,
    pub dedup: bool,
    pub immutable_storage: bool,
}

impl RepositoryCapabilities {
    pub fn phase_s_default() -> Self {
        Self { compression: true, encryption: false, dedup: false, immutable_storage: false }
    }
}
```

### BackupInstanceMetadata Extension

```rust
pub struct BackupInstanceMetadata {
    // v1.0 fields (unchanged)
    pub schema_version: String,
    pub restore_point_id: String,
    pub job_id: String,
    pub source_type: String,
    pub source_description: String,
    // v1.1 additions:
    pub asset_id: String,
    pub asset_type: String,
    pub created_at: String,
    pub status: String,
    pub block_chunk_policy: ChunkPolicyMetadata,
    pub block_map: BlockMapIntegrity,
    pub catalog: Option<CatalogIntegrity>,
    pub summary: BackupInstanceSummary,
}
```

Notes:
- `asset_id`: empty string in single-machine mode; reserved for enterprise managed assets
- `asset_type`: empty string (default), "workstation", "server", "volume", "disk"
- Phase S does NOT implement the Asset management layer

## 3. Updated: Error Model

```rust
pub enum RepositoryError {
    // v1.0 variants (unchanged)
    // ...
    // v1.1 additions:
    #[error("Repository format version {0} is not supported (min_compatible={1})")]
    UnsupportedRepositoryVersion(u32, u32),
    #[error("Repository identity mismatch: {field} ({v1} vs {v2})")]
    RepositoryIdentityMismatch { field: String, v1: String, v2: String },
}
```

## 4. Updated: repo.db Schema

Additional keys in `repository_meta` table (v1.1):

| Key | Type | Example | Description |
|-----|------|---------|-------------|
| `repository_id` | TEXT | uuid-v4 | Stable repository identity |
| `repository_name` | TEXT | "Nuwa Backup Repository" | Human-readable name |
| `repository_type` | TEXT | "filesystem" | Storage backend type |
| `format_version` | TEXT | "1" | Repository format version |
| `min_compatible_version` | TEXT | "1" | Minimum compatible version |
| `cap_compression` | TEXT | "true" | Compression capability |
| `cap_encryption` | TEXT | "false" | Encryption capability |
| `cap_dedup` | TEXT | "false" | Dedup capability |
| `cap_immutable` | TEXT | "false" | Immutable storage capability |

All values stored as TEXT for schema flexibility.

## 5. Updated: Block Map logical_address Semantics

Specification clarification — no database schema change.

```
Block Map: logical_offset -> block_id

logical_offset represents a position in the source data logical address space.
NOT limited to file offsets.
Block Map Engine does not know what the address space represents.
```

| Source Type | logical_offset Meaning |
|-------------|----------------------|
| File backup | Byte offset within file |
| Volume backup | Byte offset from volume start |
| Disk backup | Byte offset from disk start |

Block Map module code MUST NOT reference "file" in its module.
Comments must use "logical address" terminology.

## 6. Updated: Forbidden Patterns (v1.1 Additions)

In addition to v1.0 forbidden patterns:

8. **Auto-migrating repository format on open**
   - Reason: backup repos must never auto-upgrade
   - Migration is explicit: `nuwa repo upgrade --repo <path>`

9. **Hard-coding repository identity metadata**
   - Reason: identity fields are generated at init, not hard-coded

10. **Deleting repository.json independently from repo.db**
    - Reason: two-place identity requires coordinated lifecycle

## 7. Updated: Acceptance Criteria (v1.1 Additions)

| # | Criteria | Verification |
|---|----------|-------------|
| E1 | `repo init` generates UUID repository_id | Check repo.db + repository.json |
| E2 | `repo init` writes correct repository.json | Validate JSON schema |
| E3 | repository.json and repo.db identity match | Cross-validation test |
| E4 | Identity mismatch detected on open | Negative test (corrupt repository.json) |
| E5 | Unsupported format_version rejected on open | Negative test (set version=99) |
| E6 | BackupInstanceMetadata carries asset fields | Serialization/deserialization test |
| E7 | RepositoryCapabilities defaults correct | Unit test on phase_s_default() |

## 8. Updated: Test Strategy Additions

```rust
// Repository Identity Tests
#[test]
fn test_init_repo_creates_repository_json() {
    // Init repo, verify .nuwarepo/repository.json exists
    // Validate all required fields present
}

#[test]
fn test_repository_json_matches_repo_db() {
    // Cross-validate repository_id between repo.db and repository.json
}

#[test]
fn test_open_repo_rejects_unsupported_version() {
    // Set format_version=99 in repo.db
    // Verify UnsupportedRepositoryVersion error
}

#[test]
fn test_backup_instance_metadata_asset_fields() {
    // Verify asset_id and asset_type serialize/deserialize correctly
}
```

## 9. Updated: Phase S Scope Audit

### v1.1 In Scope (added)

- Repository identity (UUID, name, type)
- Repository capabilities (compression, encryption, dedup, immutable)
- Repository version tracking (format_version, min_compatible_version)
- Asset abstraction (asset_id, asset_type reserved fields)
- logical_address semantics (specification clarification)

### v1.1 Still Excluded

- Asset management layer (not implemented, fields only)
- Repository health state (future management layer)
- Repository ownership/location metadata (future management layer)
- Global dedup index (Phase 6)
- Reference counting / GC (Phase 6)
- Encryption implementation (Phase 6)
- Object storage backend (Enterprise)
- Cloud tiering (Enterprise)
- Small file packing (future optimization)
- Container block format (future optimization)

---

## Revision History

| Version | Date | Change |
|---------|------|--------|
| v1.0 | 2026-07-10 | Initial implementation plan |
| v1.1 | 2026-07-10 | Enterprise Readiness sync |
