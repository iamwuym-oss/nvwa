# Phase S Scale Analysis (A-08)

**Date:** 2026-07-10  **Status: PASS**

---

## 1. Block Store Scale

Layout: block-store/{hash[0:2]}/{hash[2:4]}/{full_hash}.block

| Parameter | Value |
|-----------|-------|
| Leaf directories | 256 x 256 = 65,536 |
| Files per leaf (practical) | ~10,000 |
| Total blocks (practical) | 655M |
| At 256KB (file backup) | ~160 TB |
| At 4MB (volume backup) | ~2.5 PB |
| Single file lookup | O(1) |
| Atomic write | .tmp rename |

**Verdict:** Horizontally scalable. Same pattern as git object store.

## 2. SQLite Query Complexity

### repo.db (shared)

| Table | Primary Query | Complexity | Risk |
|-------|--------------|------------|------|
| repository_meta | SELECT by key | O(1) | None |
| backup_jobs | SELECT by job_id | O(1) | None |
| restore_points | SELECT by status | O(n) | Medium (100K+ RPs) |

### block-map.db (per instance, naturally sharded)

| Table | Primary Query | Complexity | Risk |
|-------|--------------|------------|------|
| block_map | lookup by logical_offset (PK) | O(log n) | None |
| block_map | range query | O(log n + m) | Low |

### catalog.db (per instance, naturally sharded)

| Table | Primary Query | Complexity | Risk |
|-------|--------------|------------|------|
| file_entries | lookup by path (indexed) | O(log n) | None |
| file_extents | lookup by file_id (indexed) | O(log n) | None |

## 3. Risk: restore_points status query

SELECT COUNT(*) FROM restore_points WHERE status = 'COMMITTED'
is used by retention and check_repo. Full table scan without status index.

**Mitigation:** Typical usage is 30-365 RPs. At 100K+ RPs, add index.

## 4. Memory Estimates

| Component | Growth | At 1M blocks |
|-----------|--------|-------------|
| repo.db | ~1KB per RP | Negligible |
| block-map.db | ~40B per block | ~40MB |
| catalog.db | ~200B per file | ~200MB |
| block-store | Block size | Configurable |

## 5. Conclusion

| Area | Assessment |
|------|-----------|
| Block Store | Horizontally scalable to PB |
| SQLite (instance) | Sharded, no single-DB bottleneck |
| SQLite (repo.db) | Adequate; minor index risk documented |
| Directory layout | 2-2-64 proven pattern |

**A-08 Status: PASS with note.** Add index on restore_points.status if scale exceeds 100K RPs.