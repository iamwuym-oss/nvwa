# Phase 3 — NTFS Non-System Volume Image MVP

**Product:** Nüwa Backup (女娲备份)
**Date:** 2026-07-06
**Status:** PLANNING — T3-00 documentation complete, coding not started

---

## 1. Phase 3 Theme

NTFS non-system volume image backup and restore MVP.

Phase 3 establishes the block-level image foundation (.nwb format + VSS) and delivers a working CLI for backing up and restoring non-system NTFS volumes.

---

## 2. Approved Scope (Phase 3 Only)

| # | Item | Description |
|:-:|------|-------------|
| 1 | .nwb v0.2 minimal image format | Block-level image format: Header, Block Group, Block Index, metadata section |
| 2 | Block-level SHA-256 verification | Per-block checksum at write time, verify at read time |
| 3 | VSS snapshot lifecycle integration | COM-based create/query/cleanup of volume snapshot sets |
| 4 | Non-system NTFS volume backup CLI | CLI command: validate volume → VSS snapshot → read blocks → write .nwb |
| 5 | Non-system NTFS volume restore CLI | CLI command: read .nwb → verify → validate target → restore blocks |
| 6 | Phase 3 closing validation | Full test gate, manual volume test, forbidden scope audit, closing report |

---

## 3. Explicitly Excluded from Phase 3

The following are NOT in Phase 3:

- GPT/MBR partition table parser
- Boot partition detection
- BCD repair
- WinPE recovery media
- Bare metal recovery
- System volume backup (C:)
- Boot volume backup
- Disk clone (real functionality)
- Dynamic disk support
- OS RAID support
- Formal hardware RAID validation
- Daemon / Windows service
- Startup/logon/USB trigger development
- GUI volume operation pages
- Web GUI
- Enterprise Server / Agent
- Differential / incremental backup
- Encryption
- Compression (beyond Phase 1 zstd)
- Deduplication
- NTFS $Bitmap optimization (unless explicitly approved later)

---

## 4. Phase 3 Safety Boundary

### Supported (Must Match ALL)

| Criteria | Value |
|----------|-------|
| OS | Windows only |
| Privilege | Administrator mode required |
| Disk type | Local fixed disk |
| File system | NTFS only |
| Volume type | Non-system, non-boot volumes (D:, E:, etc.) |
| Backup mode | Online via VSS (volume can be in use) |
| Restore mode | To explicitly selected non-system target volume |
| Interface | CLI-first (no GUI) |

### Must Reject

- C: drive (system volume)
- Boot volume / System Reserved partition
- EFI System Partition (ESP)
- MSR partition
- Windows Recovery partition
- OEM recovery partition
- Network paths / SMB / UNC as source volume
- Removable volumes (USB flash drives) by default
- FAT32 / exFAT volumes
- ReFS (unless explicitly approved later)
- Dynamic disks (unless explicitly approved later)
- OS RAID (unless explicitly approved later)
- Target volumes smaller than source volume data size
- Source = Target (same volume self-restore)

### Destructive Operation Rules

| Rule | Enforcement |
|------|-------------|
| Volume restore is destructive | Target volume data is permanently overwritten |
| Confirmation level | Explicit user confirmation (Y/N prompt, --force bypass available) |
| Target validation | Verify volume is not system/boot/ESP before restore |
| Abort on mismatch | If target volume changed since backup, abort and warn |

---

## 5. Phase 3 Task Chain

```
T3-00: Phase 3 Scope Reset & Documentation   ← YOU ARE HERE
  │
  └─ T3-01: .nwb v0.2 Format + Block SHA-256
      │   Format spec, Rust structs, ser/de, synthetic image writer/reader,
      │   block index, per-block SHA-256, image verification.
      │   No VSS, no real volume access.
      │
      └─ T3-02: VSS Snapshot Lifecycle Proof
      │   COM lifecycle, admin detection, create snapshot set,
      │   add non-system NTFS volume, query snapshot device,
      │   open read-only, cleanup (success + failure).
      │   No .nwb writing (optional smoke metadata only).
      │
      └─ T3-03: Non-System NTFS Volume Backup CLI
      │   CLI volume backup command, validate source volume,
      │   reject system/boot/unsupported, VSS → read blocks → .nwb,
      │   compute SHA-256, cleanup.
      │
      └─ T3-04: Non-System NTFS Volume Restore CLI
      │   CLI volume restore command, read .nwb, verify checksums,
      │   validate target, destructive confirmation,
      │   restore blocks, post-restore verification.
      │
      └─ T3-CLOSE: Phase 3 Final Validation & Closing
          Full test gate, manual volume validation,
          forbidden scope audit, English-only scan,
          documentation update, closing report.
```

---

## 6. Task Status

| Task | Status |
|:----:|:------:|
| T3-00 — Phase 3 Scope Reset & Documentation | ✅ DONE / PASS |
| T3-01 — .nwb v0.2 Format + Block SHA-256 | ⏳ NOT STARTED |
| T3-02 — VSS Snapshot Lifecycle Proof | ⏳ NOT STARTED |
| T3-03 — Non-System NTFS Volume Backup CLI | ⏳ NOT STARTED |
| T3-04 — Non-System NTFS Volume Restore CLI | ⏳ NOT STARTED |
| T3-CLOSE — Phase 3 Final Validation | ⏳ NOT STARTED |

---

## 7. Phase Boundaries

| Phase | Scope | Status |
|-------|-------|--------|
| Phase 1 | File-level backup/restore CLI | ✅ CLOSED |
| Phase 2 | CLI usability + GUI Dashboard | ✅ CLOSED |
| Phase 2.5 | GUI remaining pages (Backup/Restore/History/Schedule/Settings) | ⏳ DEFERRED |
| **Phase 3** | **NTFS non-system volume image MVP** | **🔵 PLANNING (T3-00 done)** |
| Phase 3.5 | Deferred: GUI volume pages, GPT/MBR, boot partition, dynamic disk/RAID, BCD design | ❌ NOT AUTHORIZED |
| Phase 4 | System recovery / WinPE / BMR | ❌ NOT AUTHORIZED |
| Phase 5 | Disk clone | ❌ NOT AUTHORIZED |
| Phase 6+ | Differential, encryption, cross-platform | ❌ NOT AUTHORIZED |

---

## 8. Phase 3.5 Deferred Scope

Not authorized for Phase 3. These items are deferred to a future Phase 3.5:

| Item | Reason for Deferral |
|------|---------------------|
| GUI volume operation pages | Phase 3 is CLI-first; GUI comes after CLI stabilizes |
| GPT/MBR partition table parser | Not needed for non-system volume access (Windows volume API suffices) |
| Boot partition detection | System recovery (Phase 4) prerequisite, not volume backup |
| Dynamic disk / RAID investigation | Basic disk first; dynamic/RAID after base is stable |
| BCD repair design | System recovery prerequisite |

---

## 9. Phase 4 Boundary

Phase 4 (system recovery / WinPE / BMR) is NOT authorized.

Phase 4 requires:
1. Phase 3 completed and accepted.
2. User explicitly approves Phase 4 planning.
3. Phase 4 planning documents reviewed and accepted.
4. User explicitly approves first Phase 4 coding task.

---

## Revision History

| Version | Date | Reason for change |
|---------|------|-------------------|
| v1.0 | 2026-07-06 | Initial Phase 3 plan — scope reset per user direction |
