# Phase S Volume Backup Compatibility (A-09)

**Date:** 2026-07-10  **Status: PASS**

---

## 1. Compatibility Matrix

| Component | File Backup | Volume Backup | Compat? |
|-----------|------------|--------------|---------|
| BlockStore | put 256KB blocks | put 4MB blocks | YES (content agnostic) |
| BlockMapEngine | file offsets | volume LBA offsets | YES (logical offset is abstract) |
| CatalogEngine | file list + extents | partition metadata as files | YES (new provider impl only) |
| ChunkEngine | FixedChunkPolicy(256KB) | FixedChunkPolicy(4MB) | YES (configurable) |
| Transaction | per-file backup | per-volume backup | YES (same state machine) |
| Retention | per-RP lifecycle | per-RP lifecycle | YES (identical) |
| Recovery | integrity + rebuild | integrity + rebuild | YES (identical) |

## 2. Data Flow Simulation

Volume backup data flow through existing Phase S APIs:

  Volume Source (partition)
    |
    v
  ChunkEngine.process() -- chunks at 4MB block boundary
    |                      |
    v                      v
  BlockStore           BlockMapEngine
  put_block()           insert_mapping()
    |                      |
    v                      v
  block-store/        block-map.db
    |
    v
  Catalog + Transaction -- same lifecycle

**All APIs consume the volume data without modification.**

## 3. File vs Volume Catalog Comparison

| Aspect | File Backup Catalog | Volume Backup Catalog |
|--------|-------------------|---------------------|
| File paths | Present | N/A (no files) |
| File metadata | size, modified, extents | N/A |
| Volume metadata | N/A | partition size, fs type, disk layout |
| Block references | Via extents logical_offset | Via partition logical_offset |

**Conclusion:** Volume backup needs a different catalog provider (storing
partition metadata instead of file metadata). The CatalogEngine trait itself
is compatible; only the provider impl differs.

## 4. Gap Summary

| Gap | Impact | Resolution |
|-----|--------|-----------|
| Catalog provider | Volume metadata vs file metadata | New VolumeCatalog impl |
| RepositoryCapabilities | No VOLUME_BACKUP flag | Add flag at Volume Backup time |
| Chunk size | Different default | Phase S already configurable via FixedChunkPolicy |

**A-09 Status: PASS.** No Phase S API changes required for Volume Backup compatibility.