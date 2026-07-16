# Nüwa Backup — Project Engineering Memory

**Version:** 0.7.0
**Last Updated:** 2026-07-15
**Status:** CURRENT REFERENCE SNAPSHOT
**Baseline Reviewed:** 4b8e747
**Latest Code-Bearing Commit:** e1f1adb

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

**Architecture authority:** Contract documents #1–6 of the NWB Engineering Document Set, in the authority order defined by README §2, define the current storage architecture. The Test Result Record (#7) is evidence only and is currently STALE / PENDING_CORRECTION.
**Code baseline `518f9fe`:** Implements an initial **`crates/nwb-format` Registry** - the format-registration and type-system foundation.
**Important:** `crates/nwb-format` is **not** a complete NWB Storage Engine. It is one component (the Registry) of the larger NWB architecture, which also requires Writer, Reader, Catalog, Chunk engine, Crypto, Verify/Salvage, and Provider abstractions.

### Key design properties

- Each successful **Full** or **Differential** backup produces an **immutable, self-describing logical NWB archive**
- The archive is the unit of restore, verify, retention, and transfer
- The system does **not** depend on the superseded Repository architecture (Phase S: repo.db, BlockStore, block-map.db, backup-objects, external transaction journal)

### Code reality

- `crates/nwb-format` exists and contains the format registry, RecordType identifiers, feature-bit constants, header enums
- registry TOML files
- registry tests
- No generic Chunk structure, Chunk codec, or complete archive format implementation exists
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
| GATE-0 | Engineering and contract baseline: workspace/CI, registry, traceability, errors/logging, fixtures, and draft version policy | IN_PROGRESS |

### Implementation Package Status

| IMP | Title | Status |
|-----|-------|--------|
| IMP-000 | Workspace (build, CI, scaffolding) | CLOSED / PASS / ACCEPTANCE MET |
| IMP-001 | Format Registry (nwb-format crate) | Implementation/evidence remediation pending |
| IMP-002 | Requirements–Test Traceability Matrix | NOT_RUN / not authorized |
| IMP-003 through IMP-005 | Defined remaining GATE-0 work packages | NOT_RUN - execute per dependency and authorization |
| IMP-006 through IMP-009 | Not defined in current implementation plan | NOT_STARTED - must not start until defined |
| IMP-100 and later | Post-GATE-0 work | FORBIDDEN |

IMP-000 evidence chain is now closed (CLOSED / PASS / ACCEPTANCE MET). IMP-001 evidence remains under remediation; its documents are classified as STALE / PENDING_CORRECTION.

---

## 6. Build and Test Evidence Status

### Rust build

- CI pipeline established: `.github/workflows/nwb-workspace-ci.yml` (Windows + Ubuntu, 5 quality gates)
- Final CI run: 29426443433 on commit e1f1adb — ALL PASS
- Local Windows: all 5 quality gates pass, 130 tests pass

### Linux CI

- GitHub Actions ubuntu-latest: fmt / clippy / test / debug build / release build — ALL PASS

### Test count

Do not read a fixed number from this document. The authoritative test count and results must come from a corrected `Nuwa_NWB_Test_Result_Record_v1.0.md` and associated EVDs. As of this writing those documents are **STALE / PENDING_CORRECTION**.

Static source audit reveals approximately 130 Rust `#[test]` / `#[tokio::test]` annotations. This is not evidence of 130 passing tests.

### pnpm / UI build

The frontend `pnpm build` is **known to fail** at this baseline:

- `ui/src/components/common/BackupTreeView.tsx` contains garbled characters that break syntax
- `ui/src/pages/Backup.tsx` contains JSX structural errors

These are pre-existing defects, not introduced by this work package.

---

## 7. Known Limitations

1. Complete NWB write/read/restore closed loop not yet implemented
2. IMP-001 evidence chain still under remediation (IMP-000 is CLOSED / PASS / ACCEPTANCE MET)
3. Linux CI established (ubuntu-latest via GitHub Actions) — RESOLVED
4. Frontend fails production build (BackupTreeView.tsx, Backup.tsx)
5. Old Repository semantic residuals remain in UI/API layer
6. Volume, disk, BMR, and system restore capability not yet authorized or implemented
7. CI workflow established (.github/workflows/nwb-workspace-ci.yml) — RESOLVED
8. Provider SDK scope (file, volume, network) not implemented

---

## 8. Next Authorized Actions (ordered)

1. Fix root README.md (BASELINE-CONSISTENCY-001-C)
2. Fix Manifest, Test Result Record, and EVD documents
3. Fill IMP-000 CI / cross-platform evidence gap — COMPLETED (IMP-000-EVIDENCE-CLOSURE-1)
4. Resolve format contract issues and rework IMP-001
5. Fix UI build and clean Repository residuals
6. Re-accept IMP-000 and IMP-001 with current verified evidence — IMP-000 accepted (IMP-000-EVIDENCE-CLOSURE-1)
7. Plan and authorize IMP-002 (Requirements–Test Traceability Matrix)

- IMP-002 remains NOT_RUN, waiting for explicit authorization
- IMP-003-005 within GATE-0, execute per dependency and authorization
- IMP-006-009 not defined and must not start until defined
- Only IMP-100 and later must wait for GATE-0 closure
