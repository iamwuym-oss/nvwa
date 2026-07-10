# Phase S — Known Limitations & Future Roadmap

**Date:** 2026-07-10  **Phase S Status:** CLOSED BASELINE

本文档记录 Phase S 中已知未实现但计划后续补充的功能，以及明确超出范围的能力。

---

## 一、已知未实现（计划后续 Phase 补充）

| # | 功能 | 描述 | 计划阶段 | 关键程度 |
|---|------|------|---------|--------|
| 1 | **物理块 GC（Garbage Collection）** | Retention 只产生 orphan candidates，不删除物理块。需要独立的 GC 扫描确认块是否被所有 Restore Point 引用，然后才物理删除 | Phase 6+ | ★★★★★ |
| 2 | **加密（Encryption）** | 块级别和数据传输加密，AES-256-GCM 或类似方案 | Phase 6+ | ★★★★☆ |
| 3 | **CDC 变长分块（Content-Defined Chunking）** | 目前只有 FixedChunkPolicy（固定块大小）。CDC 可提升重复数据删除率 | Phase 6+ | ★★★☆☆ |
| 4 | **增量备份（Incremental Backup）** | 目前只支持全量备份。增量备份需要块级别的 change tracking | 待规划 | ★★★★☆ |
| 5 | **去重（Deduplication）** | 块级别的全局去重。Phase S 的 Block Store 以 hash 寻址，为去重奠定了基础，但未实现去重逻辑 | Phase 6+ | ★★★☆☆ |
| 6 | **多 Repository 管理** | 目前每个 Repository 独立管理，不支持跨 repo 策略 | 待规划 | ★★☆☆☆ |
| 7 | **Repository 导出/备份** | 将整个 Repository 导出为可迁移格式 | 待规划 | ★★★☆☆ |
| 8 | **Repository 健康监控** | 自动定期 verify、告警机制 | 待规划 | ★★★☆☆ |
| 9 | **远程 Repository** | 通过网络备份到远程 Repository | 待规划 | ★★★☆☆ |
| 10 | **压缩算法扩展** | 当前仅支持 zstd。后续可扩展 lz4、lzma 等 | Phase 6+ | ★★☆☆☆ |

## 二、明确超出范围（不计划实现）

| 功能 | 原因 |
|------|------|
| 云端存储 | 产品定位本地优先，不是云备份 |
| 企业集中管理 | 单机产品，不是 Veeam/Commvault 式集中管理 |
| 多租户 | 同上 |
| 对象存储直接对接（S3） | 未来可能支持，但不在 Phase S 范围 |
| 实时同步/连续备份 | 非备份产品范畴 |

## 三、已确认的架构边界（设计约束）

| 约束 | 说明 | 文档引用 |
|------|------|---------|
| block-map.db 不可从 block-store 重建 | block-store 不保存 logical_offset 信息 | Architecture v1.1 §14 |
| Retention 不删除物理块 | 只管理 Restore Point 生命周期，产生 orphan candidates | S-09 Design |
| 每个 Backup Instance 有独立的 block-map.db 和 catalog.db | 天然分片，无单库瓶颈 | Architecture v1.1 §5 |
| Repository Engine 不感知数据源类型 | File/Volume/Disk 都由上层决定，Repository 只管块 | Architecture v1.1 §3 |

---
*本文档随 Phase S 基线冻结，未来功能实现后更新对应条目。*
