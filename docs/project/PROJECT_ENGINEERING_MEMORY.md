# Nüwa Backup — Project Engineering Memory

**Version:** 1.1.0
**Last Updated:** 2026-07-19
**Status:** CURRENT REFERENCE SNAPSHOT
**Latest verified code commit:** `8b2a68b0528d37587636774eb76bc196b94dd59a`

---

## 1. Authority Boundary

This file is a derived operational snapshot. It cannot override:

1. `AGENTS.md` for engineering governance;
2. `docs/project/DOCUMENT_INDEX.md` for document classification;
3. the NWB Engineering Document Set contract documents #1–6;
4. `Nuwa_NWB_Traceability_Registry_v1.0.toml` for traceability IDs, priorities, sources, mappings and dispositions;
5. committed source code and current evidence records.

The Traceability Registry itself cannot override contract documents #1–6 or change product, format, recovery or support semantics.

## 2. Product and Architecture Baseline

Nüwa Backup is a local-first, single-machine backup and disaster recovery product. The active implementation target is the NWB Storage Engine, not the superseded Repository architecture.

Each successful Full or Differential backup is intended to produce an immutable, self-describing logical NWB archive. This remains an architecture target; the complete archive write/read/restore loop has not yet been implemented.

## 3. Current Code Reality

The verified baseline contains:

- the IMP-001 Format Registry generator, four TOML registries, generated Rust definitions and contract tests;
- the IMP-002 machine-readable Traceability Registry;
- a deterministic generated Requirements–Test Traceability Matrix;
- a Rust `traceability-checker` that validates identifiers, authority sources, mappings, P0 coverage, implementation locators and Matrix drift;
- CI enforcement of Format Registry freshness and requirements traceability before the remaining Rust quality gates.

The IMP-002 traceability inventory contains 36 requirements, 138 formal tests and 141 mappings. Of the formal tests, 23 are `IMPLEMENTED` and 115 are `PLANNED`; 119 are `MAPPED` and 19 are `SOURCE_SCOPED`. The 115 planned tests have not been executed merely because they are registered.

The repository still does not contain a complete NWB Writer, Reader, Catalog, Chunk engine, Crypto subsystem, Verify/Salvage implementation, Provider implementation or recovery loop. Existing `src-tauri/` and `ui/` code contains old Repository-semantic residuals and remains outside the accepted IMP-000–002 storage-engine scope.

## 4. Gate and Work Package Status

| Gate / IMP | Status | Evidence |
|---|---|---|
| GATE-0 | IN_PROGRESS | IMP-003–005 remain NOT_RUN |
| IMP-000 | CLOSED / PASS / ACCEPTANCE MET | `e1f1adb`, run `29426443433` |
| IMP-001 | CLOSED / PASS / ACCEPTANCE MET | `3bceb34`, run `29684903853`, EVD v1.2 |
| IMP-002 | CLOSED / PASS / ACCEPTANCE MET | `8b2a68b`, run `29691731514`, EVD v1.0 |
| IMP-003–005 | NOT_RUN | Execute only with bounded scope and role authorization |
| IMP-006–009 | NOT_DEFINED / NOT_STARTED | Must be defined before work starts |
| IMP-100+ | FORBIDDEN UNTIL GATE-0 CLOSES | Post-GATE-0 work |

## 5. IMP-002 Evidence Snapshot

| Item | Result |
|---|---|
| Implementation commit | `c61b17e4a6ade9499c36e77598c689240792109f` |
| Final tested commit | `8b2a68b0528d37587636774eb76bc196b94dd59a` |
| GitHub Actions | run `29691731514` — SUCCESS |
| Ubuntu | job `88205559857` — all gates PASS |
| Windows | job `88205559861` — all gates PASS |
| Toolchain | rustc/cargo 1.97.1 |
| Workspace tests | 188 passed / 0 failed / 0 ignored on both platforms |
| Checker result | 36 requirements / 138 tests / 141 mappings / 33 P0 |
| P0 coverage | 33 / 33 with positive and negative/fault/security roles |
| P1 coverage | 3 / 3 with at least one test |
| Code review | APPROVED after three rounds |
| Independent validation | Final-commit dual-platform CI evidence verified; local Cargo NOT_RUN / BLOCKED_BY_ENVIRONMENT |
| Recovery integrity | APPROVED; recovery execution N/A |
| Pull Request | PR #1 remains Draft and unmerged |

## 6. Current Evidence Sources

- `Nuwa_NWB_Test_Result_Record_v1.2.md` — current result and acceptance record;
- `IMP-002_EVD_Requirements_Traceability_Evidence_v1.0.md` — current IMP-002 evidence;
- `IMP-001_EVD_Test_Result_Evidence_v1.2.md` — current IMP-001 evidence;
- Test Result Record v1.1 is historical/superseded;
- the v1.0 IMP-001 EVD and v1.0 Test Result Record are historical, superseded malformed source files retained only for traceability.

## 7. Known Boundaries and Risks

1. Traceability `PASS` proves registry consistency, coverage policy and CI enforcement; it does not prove planned product tests passed.
2. 115 formal tests remain `PLANNED / NOT_RUN`; 19 of them are `SOURCE_SCOPED` rather than mapped to the current 36 requirements.
3. Real Backup/Restore, fault injection, Verify/Salvage, block restore and BMR startup remain `N/A / NOT_RUN` for IMP-002.
4. Bare `cargo build` does not parse Registry TOML; the CI sequence runs both dedicated checkers first.
5. RIR-001 (`BackupKind Invalid=0`) and RIR-003 (non-atomic Registry `generate`) remain deferred under their prior rulings.
6. UI production build defects are pre-existing and outside IMP-002.
7. Support certification, Format Freeze and product release remain `NOT_RUN / NOT_APPROVED`.
8. PR #1 remains Draft and has not been merged.

## 8. Next Authorized Planning Point

IMP-002 is closed. GATE-0 remains open because IMP-003–005 are still `NOT_RUN`. The next work package must be selected and authorized with a bounded scope; closing IMP-002 does not authorize IMP-003 or any post-GATE-0 implementation.
