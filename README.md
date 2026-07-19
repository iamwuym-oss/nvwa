# Nüwa Backup

**Status:** GATE-0 IN_PROGRESS

Nüwa Backup is a local-first, single-machine backup and disaster recovery application currently under active development. The existing desktop and application code is primarily Windows-oriented. The NWB product architecture targets Windows x86-64 and Linux x86-64 as defined by the Product Support Matrix; these are release targets, not claims of current certified support.

---

## Authority Boundary

This README is a **REFERENCE** project entry point. It cannot override:

- AGENTS.md — general engineering governance and safety rules
- docs/project/DOCUMENT_INDEX.md — authoritative document classification (start here)
- NWB Engineering Document Set contract documents #1–6 — current storage architecture and implementation authority; evidence records cannot override these contracts

Start navigating at docs/project/DOCUMENT_INDEX.md rather than relying on this file.

---

## Current Architecture

The current implementation target is the **NWB Storage Engine**:

- Each successful Full or Differential backup produces an immutable, self-describing logical NWB Archive
- The system does not depend on the superseded Repository architecture (Phase S: repo.db, BlockStore, block-map.db, backup-objects, external transaction journal)
- Old Phase S Repository track is abandoned

**Note:** Phase 1, Phase 2, and Phase 2.5 are HISTORICAL / ACCEPTED BASELINE. Their old storage designs are not the current implementation target.

---

## Current Implementation Status

The current verified GATE-0 baseline contains the IMP-001 Format Registry/Generator and the IMP-002 requirements–test traceability controls:

- RecordType identifiers (18 variants)
- Feature-bit constants (5 constants)
- Header enums (BackupKind 5, PlatformHint 3)
- ErrorId identifiers (9 variants)
- Registry TOML files (4)
- `format-registry-generator` with `check` and `generate` paths
- Registry contract tests, Generator negative tests, and trybuild compile-fail coverage
- a machine-readable Traceability Registry with 36 requirements, 138 formal tests and 141 mappings
- a deterministic generated Requirements–Test Traceability Matrix
- `traceability-checker` validation of IDs, authority sources, P0 coverage, implementation locators and Matrix drift
- Windows and Ubuntu CI enforcement of both dedicated checkers

Of the 138 formal traceability tests, 23 are `IMPLEMENTED` and 115 remain `PLANNED`; registration does not mean those 115 tests have run. Nineteen planned tests are explicitly `SOURCE_SCOPED` to authoritative clauses rather than mapped to the current 36 Requirement IDs.

It is not a complete NWB Storage Engine. The following have not yet been implemented as a complete closed loop:

- NWB Writer
- NWB Reader / Restore Reader
- Archive production (Full / Differential)
- Catalog engine targeting NWB archives
- Chunk engine for the NWB path
- Crypto (encryption / signing)
- Verify / Salvage
- Provider abstraction (file, volume, network)

src/ still contains app/, main.rs, cli.rs, config.rs, history.rs, scheduler.rs and other modules. The backup and restore CLI arguments can still be parsed, but main.rs returns a clear "temporarily unavailable during storage engine redesign" message.

src-tauri/ and ui/ exist but contain old Repository semantic residuals that have not yet been cleaned. The frontend is not yet wired to a complete NWB engine.

---

## Quality Status

- IMP-000: CI evidence complete (e1f1adb, run 29426443433) — PASS / ACCEPTANCE MET
- IMP-001: Generator remediation evidence complete (3bceb34, run 29684903853) — PASS / ACCEPTANCE MET; 172/172 workspace tests passed on Windows and Ubuntu
- IMP-002: requirements–test traceability evidence complete (8b2a68b, run 29691731514) — PASS / ACCEPTANCE MET; 188/188 workspace tests passed on Windows and Ubuntu
- pnpm build is known to fail:
  - ui/src/components/common/BackupTreeView.tsx — garbled characters
  - ui/src/pages/Backup.tsx — JSX structural errors
- Current acceptance record: `Nuwa_NWB_Test_Result_Record_v1.2.md`
- Current IMP evidence: `IMP-001_EVD_Test_Result_Evidence_v1.2.md` and `IMP-002_EVD_Requirements_Traceability_Evidence_v1.0.md`
- PR #1 remains Draft and unmerged; product release is NOT_APPROVED

---

## Project State

| IMP | Title | Status |
|-----|-------|--------|
| IMP-000 | Workspace (build, CI, scaffolding) | CLOSED / PASS / ACCEPTANCE MET |
| IMP-001 | Format Registry (nwb-format crate) | CLOSED / PASS / ACCEPTANCE MET |
| IMP-002 | Requirements-Test Traceability Matrix | CLOSED / PASS / ACCEPTANCE MET |
| IMP-003-005 | Defined remaining GATE-0 work packages | NOT_RUN |
| IMP-006-009 | Not defined in current plan | NOT_STARTED |
| IMP-100+ | Post-GATE-0 work | FORBIDDEN until GATE-0 closes |

---

## Repository Structure (top-level)

```
crates/nwb-format/   - Format Registry (initial)
tools/               - Format Registry generator and traceability checker
src/                 - Rust application services
src-tauri/           - Tauri 2 desktop shell
ui/                  - React + TypeScript frontend
docs/                - Project documentation
```

---

## Development Commands

These commands execute the corresponding tools. They are not evidence of passing results.

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo run --locked -p format-registry-generator -- check
cargo run --locked -p traceability-checker -- check
cd ui && pnpm build
cd src-tauri && cargo check
```
