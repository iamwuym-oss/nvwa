# Phase S API Freeze Report (Stage 3)

**Date:** 2026-07-10  **Status: PASS**

---

## 1. Public API Inventory

### 1.1 Core Traits (Freeze Critical)

| Trait | Location | Methods | Volume Compatible | Freeze Status |
|------|----------|---------|-------------------|--------------|
| BlockStore | block_store/store.rs | put_block, get_block, exists, verify_block, root_path | YES - content agnostic | FROZEN |
| BlockMapEngine | block_map/engine.rs | insert_mapping, get_block, get_range, block_count, close | YES - logical offset agnostic | FROZEN |
| CatalogEngine | catalog/engine.rs | add_file, get_file, list_files, file_count, close | YES - file-centric; volume needs new provider impl | FROZEN |
| ChunkPolicy | chunk_engine/policy.rs | chunk_size, chunk_data | YES - pure data transform | FROZEN |

### 1.2 Core Structs

| Struct | File | Key Fields | Freeze Status |
|--------|------|------------|--------------|
| RepoHandle | repo_manager.rs | root, block_store_dir, instances_dir, repo_db() | FROZEN |
| RepositoryInfo | repo_manager.rs | repository_id, format_version, min_compatible_version, capabilities | FROZEN |
| Block | block_store/store.rs | header, data | FROZEN |
| BlockHeader | block_store/block_header.rs | compression, raw_size, stored_size, reserved | FROZEN |
| BlockId | block_store/block_id.rs | SHA-256 hash | FROZEN |
| FileEntry | catalog/engine.rs | path, size, modified, extents | FROZEN |
| FileExtent | catalog/engine.rs | logical_offset, length | FROZEN |
| BlockMapEntry | block_map/engine.rs | logical_offset, block_id, raw_size | FROZEN |
| ChunkResult | chunk_engine/engine.rs | blocks, total_raw, block_size | FROZEN |
| ChunkEngine | chunk_engine/engine.rs | process(), policy(), compression_enabled() | FROZEN |
| FixedChunkPolicy | chunk_engine/policy.rs | chunk_size() | FROZEN |
| RetentionPolicy | retention/engine.rs | keep_days, keep_full_count | FROZEN |
| RetentionResult | retention/engine.rs | deleted_points, orphan_candidate_count | FROZEN |
| OrphanSummary | retention/engine.rs | deleted_points, total_orphan_blocks | FROZEN |
| TransactionJournal | transaction/journal.rs | restore_point_id, state, components, started_at | FROZEN |
| ComponentStatus | transaction/journal.rs | block_store, block_map, catalog, metadata | FROZEN |
| RecoveryReport | transaction/manager.rs | auto_committed, marked_failed, total_incomplete, errors | FROZEN |
| CrashConsistencyManager | transaction/manager.rs | begin, complete_*, commit, fail, recover_at_startup | FROZEN |
| IntegrityReport | recovery/integrity_check.rs | is_healthy() | FROZEN |
| BackupInstanceSummary | metadata/models.rs | restore_point_count, latest, total_blocks, total_raw_bytes | FROZEN |
| RepositoryCapabilities | metadata/models.rs | phase_s_default() | FROZEN |

### 1.3 Public Enums

| Enum | File | Variants | Freeze Status |
|------|------|---------|--------------|
| RepositoryError | error.rs | 15 variants | FROZEN |
| Compression | block_store/block_header.rs | None, Zstd | FROZEN |
| TransactionState | transaction/journal.rs | Creating, Writing, Verifying, Committed, Failed | FROZEN |
| ComponentPhase | transaction/journal.rs | Pending, Completed | FROZEN |
| CheckStatus | recovery/integrity_check.rs | Pass, Warning, Error | FROZEN |
| VerifyLevel | verify/engine.rs | Quick, Full | FROZEN |

### 1.4 Public Functions (Entry Points)

| Function | Module | Purpose |
|----------|--------|--------|
| init_repo() | repo_manager | Create new repository |
| open_repo() | repo_manager | Open existing repository |
| is_repository() | repo_manager | Check if path is a repo |
| check_repo() | repo_manager | Run consistency checks |
| verify_repo() | verify/engine | Verify blocks and metadata |
| check_integrity() | recovery/integrity_check | Check data integrity |
| rebuild_repo() | recovery/rebuild | Rebuild repo from metadata |
| apply_retention() | retention/engine | Apply retention policy |
| recover_incomplete_deletions() | retention/engine | Recover deletion crash |
| count_orphan_candidates() | retention/engine | Count orphan blocks |
| list_orphan_candidates() | retention/engine | List orphan details |
| write_journal() | transaction/journal | Write transaction journal |
| read_journal() | transaction/journal | Read transaction journal |
| remove_journal() | transaction/journal | Remove transaction journal |
| scan_journals() | transaction/journal | Scan for journals |

---

## 2. Volume Backup Compatibility Analysis

### 2.1 Data Flow: Volume Backup through Phase S API

`
Volume Source (partition)
  |
  v
ChunkEngine.process() -- splits 4MB volume blocks into chunks
  |                              |
  v                              v
BlockStore.put_block()    BlockMapEngine.insert_mapping()
  |                              |
  v                              v
block-store/xxxx.block      block-map.db
  |
  v
CatalogEngine.add_file() -- volume metadata as file entries
  |
  v
CrashConsistencyManager -- same transaction model
`

### 2.2 Compatibility Results

| Phase S API | Volume Backup Compat? | Notes |
|------------|----------------------|-------|
| BlockStore | YES | Content-agnostic; 4MB volume blocks same API |
| BlockMapEngine | YES | Logical offset maps to block; position-agnostic |
| CatalogEngine | YES | Volume metadata as file entries works |
| ChunkPolicy | YES | FixedChunkPolicy(4MB) for volume blocks |
| Transaction | YES | Same state machine; components abstract |
| Retention | YES | Same RestorePoint lifecycle |
| Recovery | YES | Same integrity model |
| Verify | YES | Same block verification |

### 2.3 Non-Blocking Gaps

1. CatalogEngine API uses file paths/metadata. Volume Backup may need:
   - Volume metadata store (partition info, filesystem, disk layout)
   - A separate volume_catalog provider (future extension)
2. RepositoryCapabilities lacks volume-backup capability flag currently.
   - Add VOLUME_BACKUP when Volume Backup is implemented.

---

## 3. Freeze Conclusion

| Assessment | Result |
|------------|--------|
| API Stability | FROZEN - All 4 core traits, data models, entry points stable |
| Volume Backup Compat | COMPATIBLE - No API changes needed |
| Future Extension | 3 extension points documented |
| Breakage Risk | LOW - Traits are cohesive, loosely coupled |

## 4. Extension Points for Volume Backup

| Component | Extension Strategy |
|-----------|-------------------|
| Catalog | New provider impl for volume metadata |
| Capabilities | Add VOLUME_BACKUP flag |
| Block Map | No change needed |
| Chunk Policy | FixedChunkPolicy(4MB) for volume blocks |
| Transaction | No change needed |
| Retention | No change needed |
| Block Store | No change needed |

---

**Stage 3 Status: PASS. API freeze validated. Volume Backup compatible.**