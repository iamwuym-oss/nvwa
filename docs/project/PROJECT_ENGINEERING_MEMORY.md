# Nüwa Backup — Project Engineering Memory

**Version:** 1.3.0
**Last Updated:** 2026-07-29
**Status:** CURRENT REFERENCE SNAPSHOT
**Latest verified code commit:** `f89cad8faad574fe27a2709522776c31307dc52a`

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
- the IMP-003 `nwb-diagnostics` crate with generated ErrorId identity, fixed structured fields, deterministic JSON Lines, sanitized writer failures and redacted `Secret`;
- CI enforcement of Format Registry freshness, requirements traceability and dedicated Secret Canary tests before the remaining Rust quality gates.

IMP-004 committed as `f89cad8`. The committed traceability inventory contains 38 requirements, 150 formal tests and 153 mappings (35 P0). Of the formal tests, 35 are `IMPLEMENTED` and 115 are `PLANNED`. The 115 planned tests have not been executed merely because they are registered.

The repository still does not contain a complete NWB Writer, Reader, Catalog, Chunk engine, Crypto subsystem, Verify/Salvage implementation, Provider implementation or recovery loop. Existing `src-tauri/` and `ui/` code contains old Repository-semantic residuals and remains outside the accepted IMP-000–003 storage-engine scope.

## 4. Gate and Work Package Status

| Gate / IMP | Status | Evidence |
|---|---|---|
| GATE-0 | IN_PROGRESS | IMP-004 is implemented locally but not accepted; IMP-005 remains NOT_RUN |
| IMP-000 | CLOSED / PASS / ACCEPTANCE MET | `e1f1adb`, run `29426443433` |
| IMP-001 | CLOSED / PASS / ACCEPTANCE MET | `3bceb34`, run `29684903853`, EVD v1.2 |
| IMP-002 | CLOSED / PASS / ACCEPTANCE MET | `8b2a68b`, run `29691731514`, EVD v1.0 |
| IMP-003 | CLOSED / PASS / ACCEPTANCE MET | `d08a92b`, run `30184529945`, EVD v1.0 |
| IMP-004 | CLOSED / PASS / ACCEPTANCE MET | `f89cad8`, run `30430862143` |
| IMP-005 | NOT_RUN | Execute only with bounded scope and role authorization |
| IMP-006–009 | NOT_DEFINED / NOT_STARTED | Must be defined before work starts |
| IMP-100+ | FORBIDDEN UNTIL GATE-0 CLOSES | Post-GATE-0 work |

## 5. IMP-003 Evidence Snapshot

| Item | Result |
|---|---|
| Initial implementation | `539b796110782629cefa6d5680da2dab2c61d7ff` |
| Review remediation | `2e262394d2410387e5878b3d98e4cb9687654f39` |
| Final tested PR head | `d08a92baf289a9ae4eb1cb6cb668fed474ccdc5c` |
| GitHub Actions | run `30184529945` — SUCCESS |
| Ubuntu | job `89746726215` — all gates PASS |
| Windows | job `89746726158` — all gates PASS |
| Toolchain | rustc/cargo 1.97.1 |
| Workspace tests | 195 passed / 0 failed / 0 ignored on both platforms |
| Secret Canary | 2 passed / 0 failed in a dedicated step on both platforms |
| Checker result | 37 requirements / 145 tests / 148 mappings / 34 P0 |
| IMP-003 contracts | `TST-ERR-001`–`TST-ERR-007` PASS |
| Code review | CHANGES_REQUIRED, remediated, final APPROVED |
| Final format delta review | APPROVED; no behavior, Schema, assertion or traceability change |
| Independent validation | Final-commit dual-platform CI evidence verified |
| Recovery integrity | N/A; no recovery execution or persistent format change |
| Pull Request | PR #1 remains Draft and unmerged |


## 5b. IMP-004 Evidence Snapshot

| Item | Result |
|---|---|
| Implementation commit | `f89cad8faad574fe27a2709522776c31307dc52a` |
| Commit title | `feat(storage): add deterministic fixture generator` |
| GitHub Actions Workflow | `NWB Workspace CI` |
| Implementation CI Run | `30430862143` |
| Implementation CI URL | https://github.com/iamwuym-oss/nvwa/actions/runs/30430862143 |
| Windows Job | `90507558164` — all gates PASS (4m00s) |
| Ubuntu Job | `90507558201` — all gates PASS (2m03s) |
| Toolchain (Windows) | rustc/cargo 1.97.1, target x86_64-pc-windows-msvc |
| Toolchain (Ubuntu) | rustc/cargo 1.97.1, target x86_64-unknown-linux-gnu |
| Workspace tests | 200 passed / 0 failed / 0 ignored on both platforms |
| Secret Canary | 2 passed / 0 failed in a dedicated step on both platforms |
| Checker result | 38 requirements / 150 tests / 153 mappings / 35 P0 |
| Fixed fixture root hash | `8ce810bd57473b4af044855996291c7ba34860606b6f6ae9a3e98aa774bc7222` |
| TST-FIX-001 (determinism) | PASS on both platforms |
| TST-FIX-002 (approved dataset) | PASS on both platforms |
| TST-FIX-003 (content tamper) | PASS on both platforms |
| TST-FIX-004 (extra files) | PASS on both platforms |
| TST-FIX-005 (path traversal + co-tamper) | PASS on both platforms |
| Format check | PASS on both platforms |
| Format Registry check | PASS on both platforms |
| Requirements traceability check | PASS on both platforms |
| Secret canary check | PASS on both platforms |
| Clippy with `-D warnings` | PASS on both platforms |
| Build (debug) | PASS on both platforms |
| Build (release) | PASS on both platforms |
| Independent code review | APPROVED at IMP-004-LOCAL-VALIDATION-1 |
| Independent closure review | APPROVED_FOR_CLOSURE at IMP-004-FINAL-CLOSURE-REVIEW-1 |
| Fixture CLI hash evidence | Pre-commit Windows local validation; verified by TST-FIX-001/002 in CI |
| PR #1 | Remains Draft and unmerged |
| IMP-005 | NOT_RUN |
| Closure evidence commit | `4f9b5b7929985906b6c82c8a12d730a69a9ee8d5` |
| Closure evidence title | `docs(storage): close IMP-004 evidence` |
| Closure evidence CI Run | `30437421409` |
| Closure evidence CI URL | https://github.com/iamwuym-oss/nvwa/actions/runs/30437421409 |
| Closure evidence Windows Job | `90528335193` — all gates PASS (3m38s) |
| Closure evidence Ubuntu Job | `90528335403` — all gates PASS (2m10s) |
| Final metadata closure | Final Metadata Commit at `<<FINAL_SHA>>` completes the bounded loop. This commit's own CI is recorded in the final execution report and final independent closure review, and does NOT trigger further repository metadata write-back. |

## 6. Current Evidence Sources

- `Nuwa_NWB_Test_Result_Record_v1.3.md` — current result and acceptance record;
- `IMP-003_EVD_Structured_Diagnostics_Evidence_v1.0.md` — current IMP-003 evidence;
- Implementation commit `f89cad8` with IMP-004 CI run `30430862143` — current IMP-004 evidence (no separate EVD created; evidence is in committed CI, ENGINEERING_MEMORY, Manifest and closed DOCUMENT_INDEX);
- `IMP-002_EVD_Requirements_Traceability_Evidence_v1.0.md` — current IMP-002 evidence;
- `IMP-001_EVD_Test_Result_Evidence_v1.2.md` — current IMP-001 evidence;
- Test Result Record v1.2 and v1.1 are historical/superseded;
- the v1.0 IMP-001 EVD and v1.0 Test Result Record are historical, superseded malformed source files retained only for traceability.

## 7. Known Boundaries and Risks

1. Traceability `PASS` proves registry consistency, coverage policy and CI enforcement; it does not prove planned product tests passed.
2. 115 formal tests remain `PLANNED / NOT_RUN` (IMP-004 only added 5 IMPLEMENTED Fixture tests; the 115 figure is the same denominator from IMP-002/003).
3. Real Backup/Restore, fault injection, Verify/Salvage, block restore and BMR startup remain `N/A / NOT_RUN` for IMP-003.
4. Bare `cargo build` does not parse Registry TOML; the CI sequence runs both dedicated checkers first.
5. RIR-001 (`BackupKind Invalid=0`) and RIR-003 (non-atomic Registry `generate`) remain deferred under their prior rulings.
6. `Secret` evidence does not cover crash dumps, swap, process memory scans or future real key hierarchies; `TST-CRY-007` remains planned.
7. Support certification, Format Freeze and product release remain `NOT_RUN / NOT_APPROVED`.
8. PR #1 remains Draft and has not been merged.

## 8. Next Authorized Planning Point

IMP-003 and IMP-004 are CLOSED. GATE-0 remains IN_PROGRESS because IMP-005 is NOT_RUN. The next work package must be selected and authorized with a bounded scope. IMP-005 and all post-GATE-0 implementation remain unauthorized.
