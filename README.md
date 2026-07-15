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

crates/nwb-format currently contains an initial Registry only:

- RecordType identifiers
- Feature-bit constants
- Header enums
- Registry TOML files
- Registry tests

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

- IMP-000 and IMP-001 evidence chains are under remediation — no PASS claim is supported
- No GitHub Actions or Linux CI evidence exists
- pnpm build is known to fail:
  - ui/src/components/common/BackupTreeView.tsx — garbled characters
  - ui/src/pages/Backup.tsx — JSX structural errors
- Do not read a fixed test count from this document; the authoritative test count must come from corrected evidence documents

---

## Project State

| IMP | Title | Status |
|-----|-------|--------|
| IMP-000 | Workspace (build, CI, scaffolding) | Evidence remediation pending |
| IMP-001 | Format Registry (nwb-format crate) | Implementation/evidence remediation pending |
| IMP-002 | Requirements-Test Traceability Matrix | NOT_RUN / not authorized |
| IMP-003-005 | Defined remaining GATE-0 work packages | NOT_RUN |
| IMP-006-009 | Not defined in current plan | NOT_STARTED |
| IMP-100+ | Post-GATE-0 work | FORBIDDEN until GATE-0 closes |

---

## Repository Structure (top-level)

```
crates/nwb-format/   - Format Registry (initial)
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
cd ui && pnpm build
cd src-tauri && cargo check
```