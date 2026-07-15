# Nüwa Backup — Project Engineering Memory

**Version:** 0.7.0
**Last Updated:** 2026-07-15
**Status:** CURRENT REFERENCE SNAPSHOT
**Baseline Reviewed:** 4b8e747
**Latest Code-Bearing Commit:** 518f9fe

---

## 1. Authority Boundary

This document is a **current cross-phase operational snapshot**. It is derived from authoritative contracts, evidence records, and repository state. It cannot override:

- `AGENTS.md` — general engineering governance and safety rules
- `docs/project/DOCUMENT_INDEX.md` — authoritative document classification
- NWB Engineering Document Set (contract documents #1–6 per README §2)
- Real source code, real test results, and committed repository state

In case of conflict, resolve by the authority order defined in `docs/project/DOCUMENT_INDEX.md`.

---

## 2. Project Identity

Nüwa Backup is a **local-first, single-machine** backup and disaster recovery product for Windows Workstation, Windows Server, power users, small offices, PC repair shops, and edge nodes.

### Core engineering priority (immutable)

1. Data recoverability
2. Crash consistency and data integrity
3. User data safety and misoperation prevention
4. Format compatibility and verifiability
5. Clear error semantics and observability
6. Maintainability and testability
7. Performance, convenience, feature count

---

## 3. Current Product Baseline: NWB Storage Engine

The current storage engine implementation authority is the **NWB Storage Engine** (crates/nwb-format).

### Key design properties

- Each successful **Full** or **Differential** backup produces an **immutable, self-describing logical NWB archive**
- The archive is the unit of restore, verify, retention, and transfer
- The system does **not** depend on the superseded Repository architecture (Phase S: repo.db, BlockStore, block-map.db, backup-objects, external transaction journal)

### Code reality

- `crates/nwb-format` exists and contains the format registry, chunk type definitions, header enums, and a test suite
- The following have **not yet been implemented** as a complete closed loop:
  - NWB Writer
  - NWB Reader / Restore Reader
  - Archive production (Full / Differential)
  - Catalog engine targeting NWB archives
  - Chunk engine for the NWB path
  - Crypto (encryption / signing)
  - Verify / Salvage
  - Provider abstraction (file, volume, network)
- `src/` still contains `app/`, `main.rs`, `cli.rs`, `config.rs`, `history.rs`, `scheduler.rs` and other modules; they are **not yet wired** to a complete NWB engine
- `src-tauri/` and `ui/` exist but contain **old Repository semantic residuals** that must be cleaned in a dedicated task
- The following **root-level modules no longer exist**: `backup.rs`, `restore.rs`, `verify.rs`, `manifest.rs`, `storage.rs`, `prune.rs`. They must not be described as "frozen baselines" or "current implementations".

---

## 4. Historical Baselines

| Phase | Scope | Authority Classification |
|-------|-------|--------------------------|
| Phase 0 | Project setup, MVP boundary, guardrails | HISTORICAL |
| Phase 1 | File-level backup/restore CLI | HISTORICAL / ACCEPTED BASELINE |
| Phase 2 | CLI usability + egui GUI | HISTORICAL / ACCEPTED BASELINE |
| Phase 2.5 | Tauri 2 desktop GUI + React frontend + Application Layer | HISTORICAL / ACCEPTED BASELINE |

These phases are **closed**. Their closing reports, acceptance records, and test evidence are retained for traceability. Their architecture (flat-file backup, old Repository engine) is **not** the current implementation target.

The old Phase 3–6 roadmap (NTFS volume image, VSS, system recovery, BMR, disk clone, encryption, differential backup) is **superseded**. The execution model is now **GATE-0 through GATE-9** as defined in the NWB Implementation Plan v1.0.

---

## 5. Current Execution Model: Gates and IMPs

### Gate Roadmap

| Gate | Purpose | Status |
|------|---------|--------|
| GATE-0 | Foundation: Workspace, Format Registry, Provider SDK, Verification | IN_PROGRESS |

### Implementation Package Status

| IMP | Title | Status |
|-----|-------|--------|
| IMP-000 | Workspace (build, CI, scaffolding) | Evidence remediation pending |
| IMP-001 | Format Registry (nwb-format crate) | Implementation/evidence remediation pending |
| IMP-002 | Requirements–Test Traceability Matrix | NOT_RUN / not authorized |
| IMP-003 through IMP-009 | Planned for GATE-0 scope | NOT_STARTED / FORBIDDEN until GATE-0 closes |
| IMP-100 and later | Post-GATE-0 work | FORBIDDEN |

All evidence, test results, and EVD documents for IMP-000 and IMP-001 are currently classified as **STALE / PENDING_CORRECTION** in the document index. No IMP-000 or IMP-001 closure claim is supported by current verified evidence.

---

## 6. Build and Test Evidence Status

### Rust build

- Static audit of the repository shows a Rust workspace with `crates/nwb-format` and root-level `src/` modules.
- No current CI pipeline exists in the repository (no GitHub Actions workflows).
- Previous Windows build/test reports exist in documents marked STALE / PENDING_CORRECTION. They are not current PASS evidence.

### Linux CI

- No Linux CI evidence exists at this baseline.

### Test count

Do not read a fixed number from this document. The authoritative test count and results must come from a corrected `Nuwa_NWB_Test_Result_Record_v1.0.md` and associated EVDs. As of this writing those documents are **STALE / PENDING_CORRECTION**.

Static source audit reveals approximately 130 Rust `#[test]` / `#[tokio::test]` annotations. This is not evidence of 130 passing tests.

### pnpm / UI build

The frontend `pnpm build` is **known to fail** at this baseline:

- `ui/src/components/BackupTreeView.tsx` contains garbled characters that break syntax
- `ui/src/pages/Backup.tsx` contains JSX structural errors

These are pre-existing defects, not introduced by this work package.

---

## 7. Known Limitations

1. Complete NWB write/read/restore closed loop not yet implemented
2. IMP-000 and IMP-001 evidence chains not yet remediated
3. Linux CI absent
4. Frontend fails production build (BackupTreeView.tsx, Backup.tsx)
5. Old Repository semantic residuals remain in UI/API layer
6. Volume, disk, BMR, and system restore capability not yet authorized or implemented
7. No GitHub Actions or any CI workflow configured in this repository
8. Provider SDK scope (file, volume, network) not implemented

---

## 8. Next Authorized Actions (ordered)

1. Fix root README.md (BASELINE-CONSISTENCY-001-C)
2. Fix Manifest, Test Result Record, and EVD documents
3. Fill IMP-000 CI / cross-platform evidence gap
4. Resolve format contract issues and rework IMP-001
5. Fix UI build and clean Repository residuals
6. Re-accept IMP-000 and IMP-001 with current verified evidence
7. Plan and authorize IMP-002 (Requirements–Test Traceability Matrix)

IMP-002 and all later packages must not start before GATE-0 foundational evidence is closed.
