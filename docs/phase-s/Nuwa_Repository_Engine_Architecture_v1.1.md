# Nüwa Repository Engine Architecture v1.1 - Enterprise Readiness Revision

**Product:** Nüwa Backup  
**Version:** v1.1 (Enterprise Readiness Revision)  
**Status:** Architecture Amendment - Frozen on top of v1.0  
**Date:** 2026-07-10  
**Scope:** Phase S - Repository Engine  
**Prerequisite:** Architecture v1.0 (frozen baseline)

---

## Revision Purpose

Architecture v1.0 established the core Repository Engine design for single-machine file backup.
v1.1 adds enterprise-level extensibility constraints without changing v1.0 core architecture.

### Why v1.1 Exists

Nüwa long-term goal is not merely a single-machine backup tool.
The product roadmap targets enterprise-class backup software (similar to Acronis):

- Local file backup
- Volume backup
- Disk backup
- System image backup
- Unified management
- Unified storage

Phase S must be designed from the start as the **Enterprise Repository Core**,
not retrofitted later.

### Relationship to v1.0

- v1.0 **remains the frozen core architecture baseline**
- v1.1 **adds** enterprise readiness constraints on top
- All v1.0 sections not explicitly revised by v1.1 remain in full effect
- No v1.0 design decision is reversed

---

## Revision 1: Repository Identity Model

### Problem

v1.0 Repository is identified by its filesystem path only.
In an enterprise deployment, a Repository must have a stable identity.

### Constraints

1. Every Repository MUST have a globally unique repository_id (UUID v4) at init time
2. Repository MUST carry a human-readable name
3. Repository MUST carry a type classifier:
   - filesystem (Phase S)
   - Future: object_store, cloud, tape
4. Repository MUST carry format_version and min_compatible_version
5. Repository Identity is stored in two places:
   - repo.db - repository_meta table
   - .nuwarepo/repository.json - human-readable manifest

### repository.json Schema

```json
{
  "schema_version": "1.0",
  "repository_id": "uuid-v4-string",
  "name": "Local Backup Repository",
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

### Capabilities Model

```rust
pub struct RepositoryCapabilities {
    pub compression: bool,
    pub encryption: bool,
    pub dedup: bool,
    pub immutable_storage: bool,
}
```

### Code Impact

| File | Change |
|------|--------|
| repo_manager.rs | Add repository_id (UUID), name, type, capabilities |
| repo_manager.rs | Write repository.json at init time |
| repo_manager.rs | Read repository.json at open time |
| error.rs | New error variant |
| Cargo.toml | Add uuid dependency |

---

## Revision 2: Protected Asset Abstraction

### Problem

Enterprise products use: Protected Asset -> Backup Policy -> Restore Point -> Backup Instance
v1.0 is missing the Asset layer abstraction.

### Constraints

1. BackupInstanceMetadata MUST carry optional asset_id and asset_type fields
2. Phase S does NOT implement the Asset layer (reserved for future)
3. Single-machine mode: asset_id defaults to empty string
4. asset_type: "" (default), "workstation", "server", "volume", "disk"

### Code Impact

| File | Change |
|------|--------|
| models.rs | Add asset_id and asset_type to BackupInstanceMetadata |

---

## Revision 3: logical_offset -> logical_address

### Problem

v1.0 uses logical_offset for file positions only.
Enterprise Repository must handle multiple address spaces:

| Source Type | Address Space Unit |
|-------------|-------------------|
| File backup | Byte offset in file |
| Volume backup | Byte offset from volume start |
| Disk backup | Byte offset from disk start |

### Key Principle

> logical_offset represents a position in the source data logical address space.
> The Block Map Engine does not know what the address space represents.
> It only maps logical_address -> block_id.

### No Code Change Required

- Database field name logical_offset stays unchanged
- This is a specification clarification

---

## Revision 4: CatalogEngine Interface Abstraction

### Status: Already Correct

v1.0 defines CatalogEngine trait (not FileCatalogEngine).
Lock this decision as an enterprise constraint.

### Key Principle

> CatalogEngine is the first implementation, not the only implementation.
> Future VolumeCatalog, DiskCatalog implement the same trait.

### No Code Change Required

---

## Revision 5: Repository Capability Model

### Constraints

1. Capabilities stored in repository.json and repo.db at init time
2. READ-ONLY after creation (except via explicit upgrade)
3. Phase S: compression=true, encryption=false, dedup=false, immutable=false

### Code Impact

| File | Change |
|------|--------|
| repo_manager.rs | Capabilities in RepositoryInfo, write/read |

---

## Revision 6: Repository Version Migration

### Constraints

1. Two version numbers: format_version, min_compatible_version
2. Phase S: format_version=1, min_compatible_version=1
3. On open: reject if software < min_compatible_version
4. Migration is explicit (repo upgrade), never automatic
5. Migration must backup repo.db first

### Code Impact

| File | Change |
|------|--------|
| repo_manager.rs | Version metadata read/write and validation |
| errors.rs | UnsupportedRepositoryVersion error |

---

## Compliance Checklist

| # | Requirement | Status |
|---|-------------|--------|
| E-01 | Repository has UUID identity | PENDING |
| E-02 | Repository has capabilities manifest | PENDING |
| E-03 | BackupInstanceMetadata carries asset fields | PENDING |
| E-04 | Block Map uses logical_address semantics (doc) | PENDING |
| E-05 | CatalogEngine trait named correctly | PENDING |
| E-06 | Repository has format_version + min_compatible_version | PENDING |
| E-07 | repository.json written at init | PENDING |
| E-08 | open_repo() validates compatibility | PENDING |
