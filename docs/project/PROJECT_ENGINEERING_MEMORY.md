# Nüwa Backup — Project Engineering Memory

**Version:** 1.0.0
**Last Updated:** 2026-07-19
**Status:** CURRENT REFERENCE SNAPSHOT
**Latest verified code commit:** `3bceb34b4697a7552bccf9863b821a5b9d63e4b4`

---

## 1. Authority Boundary

This file is a derived operational snapshot. It cannot override:

1. `AGENTS.md` for engineering governance;
2. `docs/project/DOCUMENT_INDEX.md` for document classification;
3. the NWB Engineering Document Set contract documents #1–6;
4. committed source code and current evidence records.

## 2. Product and Architecture Baseline

Nüwa Backup is a local-first, single-machine backup and disaster recovery product. The active implementation target is the NWB Storage Engine, not the superseded Repository architecture.

Each successful Full or Differential backup is intended to produce an immutable, self-describing logical NWB archive. This remains an architecture target; the complete archive write/read/restore loop has not yet been implemented.

## 3. Current Code Reality

The verified IMP-001 baseline contains:

- a Format Registry generator with `check` and `generate` paths;
- four authoritative TOML registries;
- generated Rust definitions for RecordType, FeatureBit, BackupKind, PlatformHint and ErrorId;
- exact runtime, TOML and generated-source contract tests;
- duplicate/out-of-range semantic validation with exit code 4;
- CI enforcement of generator freshness before Rust quality gates.

It does not yet contain a complete NWB Writer, Reader, Catalog, Chunk engine, Crypto subsystem, Verify/Salvage implementation, Provider implementation or recovery loop. Existing `src-tauri/` and `ui/` code still contains old Repository-semantic residuals and is outside IMP-001.

## 4. Gate and Work Package Status

| Gate / IMP | Status | Evidence |
|---|---|---|
| GATE-0 | IN_PROGRESS | Remaining GATE-0 work is not closed |
| IMP-000 | CLOSED / PASS / ACCEPTANCE MET | `e1f1adb`, run `29426443433` |
| IMP-001 | CLOSED / PASS / ACCEPTANCE MET | `3bceb34`, run `29684903853`, EVD v1.2 |
| IMP-002 | NOT_RUN / NOT_STARTED | Next planned GATE-0 package; not automatically authorized |
| IMP-003–005 | NOT_RUN | Execute only after dependency and scope authorization |
| IMP-006–009 | NOT_DEFINED / NOT_STARTED | Must be defined before work starts |
| IMP-100+ | FORBIDDEN UNTIL GATE-0 CLOSES | Post-GATE-0 work |

## 5. IMP-001 Evidence Snapshot

| Item | Result |
|---|---|
| Verified commit | `3bceb34b4697a7552bccf9863b821a5b9d63e4b4` |
| GitHub Actions | run `29684903853` — SUCCESS |
| Windows | job `88187397699` — all gates PASS |
| Ubuntu | job `88187397701` — all gates PASS |
| Toolchain | rustc/cargo 1.97.1 |
| Workspace tests | 172 passed / 0 failed / 0 ignored on both platforms |
| Code review | APPROVED |
| Independent validation | VALIDATION_PASS |
| Recovery integrity | RECOVERY_INTEGRITY_APPROVED |

The failed formatting run `29684808801` on `1033f9f` is retained as failure evidence. Commit `3bceb34` applied only the Rustfmt layout and then passed the full matrix.

## 6. Current Evidence Sources

- `IMP-001_EVD_Test_Result_Evidence_v1.2.md` — current IMP-001 evidence;
- `Nuwa_NWB_Test_Result_Record_v1.1.md` — current result and acceptance record;
- the v1.0 IMP-001 EVD and v1.0 Test Result Record are historical, superseded, malformed source files retained only for traceability.

## 7. Known Boundaries and Risks

1. Bare `cargo build` does not parse Registry TOML. The required engineering gate runs Generator `check` first.
2. RIR-001 (`BackupKind Invalid=0`) remains deferred by architecture ruling.
3. RIR-003 (non-atomic `generate` writes) remains deferred; `check` is read-only.
4. Writer/Reader and real backup/restore verification are not implemented or accepted.
5. UI production build defects are pre-existing and outside IMP-001.
6. Support certification, Format Freeze and product release remain NOT_RUN.

## 8. Next Authorized Planning Point

IMP-001 is closed. The next planned work package is IMP-002, the Requirements–Test Traceability Matrix. It must receive its own bounded scope and role assignments before implementation. Closing IMP-001 does not itself authorize changes for IMP-002.
