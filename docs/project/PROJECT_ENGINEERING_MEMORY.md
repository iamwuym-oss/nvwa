# Nüwa Backup — Project Engineering Memory

**Version:** 0.3.0
**Last Updated:** 2026-07-11 (Phase S GUI Integration Complete)
**Current Phase:** Phase 2.5 — Tauri Desktop GUI + Application Layer + Phase S GUI Integration — CLOSED
**Phase 1 Status:** CLOSED (PARTIAL — non-Windows space check limitation accepted)

---

## 1. Purpose

This document is Codex's long-term engineering memory entry point. It records
the essential project state — what phase we are in, what has been done, what
is allowed and forbidden, and which documents must be read before starting
any new task.

---

## 2. Every New Task Must Read

Before any coding task begins, Codex MUST read these documents in order:

1. **AGENTS.md** — Product identity, phase rules, task lifecycle,
   code organization rules, CLI contract, test rules, Definition of Done.

2. **docs/project/PROJECT_ENGINEERING_MEMORY.md** — This file.

3. **docs/project/DOCUMENT_INDEX.md** — Document index with authority levels.

4. **docs/phase-1/Phase_1_Closing_Report.md** — Phase 1 closing decision,
   freeze status, and evidence summary.

5. **docs/phase-1/Phase_1_Final_Acceptance_Report.md** — What was implemented
   and tested in Phase 1.

6. **docs/phase-1/Phase_1_Technical_Baseline.md** — Module structure, data
   flow, safety rules, error codes.

7. **docs/phase-1/Phase_1_Known_Limitations_and_Risks.md** — All known
   limitations and risks from Phase 1.

8. **docs/phase-1/Phase_1_to_Phase_2_Handoff.md** — Phase handoff boundary.
9. **docs/phase-s/P-00_File_Backup_Repository_Data_Contract.md** — P-00 File Backup Repository Data Contract v0.7. Defines Restore Point state machine, Catalog Path Security Contract, crash recovery rules, Gate 1-6 definitions.
10. **docs/phase-s/P-02_Repository_Restore_Plan.md** — P-02 Repository Restore Reader implementation plan.
11. **docs/project/DOCUMENT_INDEX.md** — See also full document index with authority levels.

---

## 3. Current Phase 1 Facts

- Nüwa Backup is a **local-first, single-machine file backup CLI**.
- Phase 1 is **file-level backup/restore only**.
- Phase 1 is **NOT** volume backup, disk image, or system recovery.
- Phase 1 uses flat-file storage + JSON manifest (NOT `.nwb`).
- Phase 1 has 22 passing automated tests (3 unit + 19 integration).
- Phase 1 has 3 manual tests (10GB+ large file, disk full, locked file).
- Phase 1 **destination space check** is PARTIAL on non-Windows.
- Phase 1 **lock detection** is best-effort.

---

## 4. Phase 1 Baseline Freeze

**Phase 1 is now CLOSED and frozen as a project baseline.**

### Phase 1 Baseline Documents

| Document | Location | Authority |
|----------|----------|-----------|
| Phase 1 Closing Report | `docs/phase-1/Phase_1_Closing_Report.md` | AUTHORITATIVE |
| Phase 1 Final Acceptance Report | `docs/phase-1/Phase_1_Final_Acceptance_Report.md` | AUTHORITATIVE |
| Phase 1 Technical Baseline | `docs/phase-1/Phase_1_Technical_Baseline.md` | AUTHORITATIVE |
| Phase 1 Test Evidence | `docs/phase-1/Phase_1_Test_Evidence.md` | AUTHORITATIVE |
| Phase 1 Known Limitations and Risks | `docs/phase-1/Phase_1_Known_Limitations_and_Risks.md` | AUTHORITATIVE |
| Phase 1 Code Map | `docs/phase-1/Phase_1_Code_Map.md` | REFERENCE |
| Phase 1 → Phase 2 Handoff | `docs/phase-1/Phase_1_to_Phase_2_Handoff.md` | AUTHORITATIVE |

### Phase 2.5 Status

- **Phase 2 (egui) CLOSED. Phase 2.5 (Tauri) is the current active phase.**
- **Phase 2.5 coding is approved and in progress.**
- Phase 2.5 must NOT implement Phase 3/4/5/6+ features.
- See docs/phase-2.5/Phase_2_5_Tauri_Migration_Decision.md for migration details.

#
## Phase S — Repository Engine

**Status:** CLOSED (All 13 tasks complete)
**Started:** 2026-07-10
**Wave 1 Commit:** e50fbb4 (plus uncommitted Wave 1 fixes)
**Prerequisite:** None (independent from Phase 2.5)

### Scope

Phase S (Storage Foundation) establishes Nüwa's unified backup storage engine.
It is independent from all feature phases. Phase 3 (Volume Backup) and Phase 3.1 (File Backup v2) depend on Phase S.

### Architecture Documents

- docs/phase-s/Nuwa_Repository_Engine_Architecture_v1.0.md — Architecture Frozen Baseline
- docs/phase-s/Nuwa_Repository_Engine_Implementation_Plan_v1.1.md — Implementation Plan

### Wave 1 Implementation Status (2026-07-10)

| Module | S-ID | Status | Tests |
|--------|------|--------|-------|
| Repository Init (repo_manager.rs) | S-01 | Complete | 8 tests |
| Block Store (block_store/) | S-02 | Complete | 7 tests |
| Metadata Engine (metadata/) | S-03 | Complete | 7 tests |

Remaining Waves (not started): Chunk Engine, Catalog, Block Map, Crash Consistency Manager, Verify Engine, Retention, Recovery, Legacy Adapter, Repository CLI

### Core Design Decisions

| Decision | Value |
|----------|-------|
| Block Size | 256KB (fixed, repository-level) |
| Block Identity | SHA-256(raw data) |
| Block Header | 64 bytes fixed |
| Storage Layout | .nuwarepo/ + backup-instances/ + block-store/ |
| Data Model | Backup Job → Restore Point → Backup Instance |
| Catalog | Per-Backup-Instance SQLite (trait) |
| Block Map | Per-Backup-Instance SQLite (trait) |
| Transaction | Crash Consistency Manager (state machine + journal) |
| Retention | Logical deletion only (orphan candidates, no GC) |
| Content Addressing | NOT Deduplication (Phase 6+ reserves dedup) |

### Key Boundaries

- Repository Engine does NOT know data source types (File/Volume/Disk)
- block-map.db is NOT rebuildable from block-store
- catalog.db is NOT rebuildable (full-point restore only if lost)
- Phase S implements: Repository Init, Block Store, Metadata Engine, Block Map, Catalog, Transaction/Consistency, Verify, Retention, Recovery, Legacy Compat, CLI
- Phase S does NOT implement block GC, encryption, CDC, or cloud tier

### Phase S Final Audit (5 Gates, All PASS)

| Gate | Audit | Result | Report |
|------|-------|--------|--------|
| Stage 1 | Architecture Re-Audit | PASS (5 fixes) | Phase_S_Stage1_Audit_Report.md |
| Stage 2 | Integrity + Crash + Retention | PASS (337 tests) | Phase_S_Stage2_Audit_Report.md |
| Stage 3 | API Freeze + Scale + Volume Compat | PASS | Phase_S_Stage3_API_Freeze_Report.md |
| A-08 | Scale Review | PASS | Phase_S_Scale_Analysis.md |
| A-09 | Volume Backup Compatibility | PASS | Phase_S_Volume_Compatibility.md |

**Total: 337 tests passed, 0 failures, 0 clippy warnings, cargo fmt PASS**

### Frozen Baseline (Do Not Modify Without Approval)

- BlockStore, BlockMapEngine, CatalogEngine, ChunkPolicy traits
- All data models: BlockHeader, BlockId, FileEntry, FileExtent, BlockMapEntry
- All entry points: init_repo, open_repo, check_repo, verify_repo, rebuild_repo, apply_retention, CrashConsistencyManager
## Phase 1 Core Freeze

The Phase 1 file-level backup/restore CLI is frozen. Codex must not rewrite,
restructure, or expand the Phase 1 file backup core unless the user explicitly
approves a task that modifies it.

---

## 5. Phase Boundaries

| Phase | Scope | Status |
|-------|-------|--------|
| Phase 0 | Project setup, MVP boundary, guardrails | COMPLETE |
| Phase 1 | File-level backup/restore CLI | CLOSED |
| Phase 2 | CLI usability: config, history, scheduler, SMB, GUI (egui) | CLOSED |
| Phase 2.5 | Tauri 2.0 desktop GUI + Application Layer | CLOSED |
| Phase S | Repository Engine (unified storage foundation) | CLOSED |
| Phase 3 | NTFS non-system volume image, VSS, block backup | NOT STARTED |
| Phase 4 | WinPE recovery media, system restore, BCD repair | NOT STARTED |
| Phase 5 | Disk cloning | NOT STARTED |
| Phase 6+ | Differential backup, encryption, cross-platform | NOT STARTED |

### Forbidden in Phase 1
- Scheduled backup, GUI, daemon, IPC, VSS, `.nwb`, partition/disk backup,
  bootable media, cloning, differential/incremental, encryption.

### Forbidden in ALL Phases
- Cloud backup, cloud sync, antivirus, ransomware protection, AI threat
  detection, enterprise centralized management, multi-device dashboard.

---

## 6. Conflict Resolution

If a new task request conflicts with any of the above boundaries:
1. Codex MUST stop and report the conflict.
2. State which phase the requested feature belongs to.
3. Do not implement cross-phase features silently.

---

## 7. Restore Validation = Definition of Done

For backup/restore tasks, the minimum validation is:
1. Create test source with known content.
2. Execute backup.
3. Delete or move original source.
4. Execute restore to new location.
5. Compare SHA-256 of every file.
6. Confirm manifest matches restored result.

Do not replace this with mocks.

---

## 8. Quality Gates

Every task must pass these before marking DONE:
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build`
- `cargo test`


## 9. AGENTS.md Contamination Cleanup (Task 2.0B)

During Task 2.0B (2026-07-05), the following was found and addressed:

| Finding | Detail |
|---------|--------|
| AGENTS.md disk file | **Clean** — No FastAPI / SQLAlchemy / Vanilla JS / Tailwind CSS text present |
| Contamination source | System-level instruction template (not part of any project file) |
| Action taken | Added explicit Technology Stack Clarification to AGENTS.md stating Rust CLI is the only confirmed stack |
| Superseded text | Any FastAPI/Vanilla JS references are superseded and not applicable to Nüwa Backup |
| Phase 2 planning must use | `docs/phase-2/Phase_2_Planning_Source_Baseline.md` as the authoritative planning baseline |
| Phase 2 coding still requires | Explicit user approval |

### Updated Every New Task Must Read List

Before any coding task begins, Codex MUST now also read:

9. **docs/phase-s/P-02_Repository_Restore_Plan.md** — P-02 Repository Restore Reader implementation plan. 14-step preflight, restore algorithm, error semantics, Gate 2-3 test cases.
10. **docs/phase-2/Phase_2_Planning_Source_Baseline.md** — Phase 2 planning authority baseline, confirmed/optional/forbidden scope.


## 10. Phase 2 UI Decision (Task 2.0B Updated)

> **NOTE: This section is HISTORICAL. The Phase 2 egui GUI direction was superseded by Phase 2.5 (Tauri 2.0 + React). See Section 15 and docs/phase-2.5/Phase_2_5_Tauri_Migration_Decision.md.**


**Date:** 2026-07-05
**Status:** User confirmed — Phase 2 includes local desktop GUI coding.

### Key Decisions

| Decision | Value |
|----------|-------|
| Phase 2 GUI coding | **Approved** (as part of Phase 2 scope, after PRD + Technical Design) |
| UI direction | **Acronis True Image-like local desktop GUI** |
| Candidate technology | **egui + eframe** (pure Rust) |
| Not Web GUI | Confirmed excluded |
| Not FastAPI / Vanilla JS | Confirmed excluded |
| Clone page in UI | **Allowed as disabled placeholder only** |
| Clone functionality | **Not approved** — remains Phase 5 |

### Phase 2 Planning Must Use

1. `docs/phase-2/Phase_2_Revised_Plan.md` (v2) — Confirmed scope and timeline
2. `docs/phase-2/Phase_2_UI_Direction_Decision.md` — UI decisions record
3. `docs/phase-2/Phase_2_Planning_Source_Baseline.md` — Audit trail and contamination cleanup


## 11. Product Runtime Language Policy

**Status:** Active (enforced from Task T2-LANG-01)

The current product version is English-only at runtime.

| Context | Language | Example |
|---------|----------|---------|
| Code (src/, tests/) | English only | println!(\"Backup complete\") |
| Cargo.toml | English only | description = \"Nuwa Backup\" |
| CLI output | English only | [OK] Backup completed |
| GUI text | English only | Dashboard, Backup Now |
| Error messages | English only | Source path not found |
| Generated config | English only | # Nuwa Backup config file |
| Documentation (docs/) | Chinese allowed | Planning reports, PRD, design docs |

### Compliance Check

Before marking any coding task PASS, verify:
1. Select-String -Path src,tests -Recurse -Include *.rs,*.toml -Pattern '[\\u4e00-\\u9fff]' returns no matches.
2. All runtime-visible strings are English.
3. Generated config templates are English-only.

Chinese characters in src/, tests/, or Cargo.toml are a blocking defect.

---

## 12. T2-08 — Dashboard Page Completion Record

**Date:** 2026-07-06
**Task:** T2-08 — Implement Dashboard page for Nuwa Backup GUI
**Status:** ? COMPLETE (user confirmed "T2-08就到这里了")

### Scope Implemented

**New file:** `src/gui/pages/dashboard.rs` (~670 lines)
**Modified:**
- `src/gui/app.rs` — Added navigation handling + `use dashboard`
- `src/gui/theme.rs` — Added light content colors, increased font sizes, `TOP_BAR_HEIGHT` 40→48

### Dashboard Layout (Confirmed with User)

| Row | Content | Details |
|-----|---------|---------|
| Row 1 | Top Banner | Green "Your data is protected" + last backup time (or "No backup configured" empty state) |
| Row 2 | 4 Stat Cards | Protected Jobs (count) / Last Backup (latest time+status) / Storage Usage (default job's dest disk: total/free/percentage bar) / Scheduled Tasks (count) |
| Row 3 (3 cols) | Recent Activity | Up to 10 records (success/fail/start time) |
| | Backup Jobs | Only the single most recent job's full details (type, source, dest, schedule, last/next run, last status) |
| | Quick Actions | Backup Now, Restore, Verify Backup, View Reports |


> **NOTE: The following design decisions are from the Phase 2 egui implementation which was REMOVED in T2.5-00. They are kept for traceability only.**

### Key Design Decisions

| Decision | Value |
|----------|-------|
| Multiple jobs decision | Protected Jobs shows **total count**; Storage Usage uses **default job's dest** (or first job); Last Backup is **globally most recent** across all jobs |
| Backup Jobs row | Shows only the **most recent** backup job's details |
| Navigation signal | Quick Actions buttons signal page switches via `egui::Id::new("nav_page")` constants |
| Theme | Sidebar/top bar: dark tech; Content area: white bg, black text (as user requested) |
| Language | All runtime strings English-only (per T2-LANG-01) |

### Files Changed

| File | Change |
|------|--------|
| `src/gui/pages/dashboard.rs` | New — Full Dashboard implementation |
| `src/gui/app.rs` | Modified — Nav handling + use dashboard |
| `src/gui/theme.rs` | Modified — Light content colors, larger fonts, TOP_BAR_HEIGHT 40→48 |

### Quality Gates

| Gate | Result |
|------|--------|
| `cargo build --features gui` | ? PASS |
| GUI launched successfully | ? PASS (process started, Dashboard UI visible) |
| Phase 1 core frozen | ? No Phase 1 core files modified |
| Forbidden scope | ? No .nwb, VSS, encryption, daemon, clone |

### Remaining Phase 2 GUI Pages

| Page | File | Status |
|------|------|--------|
| Backup | `src/gui/pages/backup.rs` | ? Stub |
| Restore | `src/gui/pages/restore.rs` | ? Stub |
| History | `src/gui/pages/history.rs` | ? Stub |
| Schedule | `src/gui/pages/schedule.rs` | ? Stub |
| Settings | `src/gui/pages/settings.rs` | ? Stub |
| Clone | `src/gui/pages/clone.rs` | ? Done (disabled placeholder) |

**Next recommended task:** Backup page (`backup.rs`) — core functionality enabling actual backup operations from GUI.

---

## 13. Phase 2 Closing Status

**Date:** 2026-07-06
**Status:** CLOSED / ACCEPTED WITH KNOWN LIMITATIONS

### Phase 2 Final Scope

| Area | Status |
|------|:------:|
| T2-01 Config/Job | ? PASS |
| T2-02 History/SQLite | ? PASS |
| T2-03 CLI Output/JSON | ? PASS |
| T2-04 Retention/Prune | ? PASS |
| T2-05 Scheduler | ? PASS |
| T2-06 SMB/UNC | ? PASS |
| T2-07 GUI Scaffold | ? PASS |
| T2-08 GUI Dashboard | ? PASS |
| T2-09 Clone Placeholder | ? PASS |

### Deferred (User Decision)

| Item | Status |
|------|:------:|
| GUI Backup/Restore/History/Schedule/Settings | DEFERRED — stubs only |

### Quality Gates (Final)

| Gate | Result |
|------|:------:|
| `cargo fmt --check` | ? PASS |
| `cargo clippy --all-targets -- -D warnings` | ? PASS |
| `cargo build` | ? PASS |
| `cargo test` (105 tests) | ? ALL PASS |
| `cargo build --features gui` | ? PASS |
| GUI launch | ? PASS |
| Forbidden scope audit | ? No violations |
| Phase 1 core protection | ? Intact |
| English-only / mojibake | ? Clean |

### Phase 3 Permission

Phase 3 coding is NOT authorized without explicit user approval.

### Key Documents

| Document | Location |
|----------|----------|
| Phase 2 Closing Report | `docs/phase-2/Phase_2_Closing_Report.md` |
| Phase 2 PRD | `docs/phase-2/Phase_2_PRD.md` |
| Phase 2 Technical Design | `docs/phase-2/Phase_2_Technical_Design.md` |
| Phase 2 Revised Plan | `docs/phase-2/Phase_2_Revised_Plan.md` |

---

## 14. Phase 3 — Scope Reset (T3-00)

**Date:** 2026-07-06
**Status:** PLANNING — T3-00 documentation complete, coding not started

### Phase 3 Theme

NTFS non-system volume image backup and restore MVP.

### Phase 3 Approved Scope

| # | Item |
|:-:|------|
| 1 | .nwb v0.2 minimal image format |
| 2 | Block-level SHA-256 verification |
| 3 | VSS snapshot lifecycle integration |
| 4 | Non-system NTFS volume backup CLI |
| 5 | Non-system NTFS volume restore CLI |
| 6 | Phase 3 closing validation |

### Phase 3 Safety Rules

- Windows only, Administrator mode required
- Local fixed disk, NTFS only, non-system/non-boot volumes only
- CLI-first (no GUI in Phase 3)
- Volume restore is destructive — requires explicit confirmation
- Must reject system/boot/ESP/Recovery/FAT32/exFAT/dynamic disks/RAID
- See `docs/phase-3/Phase_3_Plan.md` for full safety boundary

### Phase 3 Excluded

GPT/MBR parser, boot partition detection, BCD repair, WinPE, bare metal recovery, daemon, GUI volume pages, dynamic disk, RAID, encryption, differential/incremental, compression, deduplication.

### Phase 3.5 / Phase 4

**Phase 3.5 is NOT AUTHORIZED. Phase 4 is NOT AUTHORIZED.** Requires explicit user approval after Phase 3 closing.

### Task Status

| Task | Status |
|:----:|:------:|
| T3-00 — Phase 3 Scope Reset & Documentation | ? DONE / PASS |
| T3-01 — .nwb v0.2 Format + Block SHA-256 | ? NOT STARTED |
| T3-02 — VSS Snapshot Lifecycle Proof | ? NOT STARTED |
| T3-03 — Non-System NTFS Volume Backup CLI | ? NOT STARTED |
| T3-04 — Non-System NTFS Volume Restore CLI | ? NOT STARTED |
| T3-CLOSE — Phase 3 Final Validation | ? NOT STARTED |

### Key Documents

| Document | Location |
|----------|----------|
| Phase 3 Plan | `docs/phase-3/Phase_3_Plan.md` |
- - - 
 
 
 
 # #   1 5 .   P h a s e   2 . 5   -   G U I   T e c h n o l o g y   M i g r a t i o n   ( T 2 . 5 - 0 0 ) 
 
 
 
 * * D a t e : * *   2 0 2 6 - 0 7 - 0 6 
 
 * * S t a t u s : * *   D O N E   -   e g u i   G U I   r e m o v e d ,   T a u r i   2 . 0   d i r e c t i o n   c o n f i r m e d ,   s c a f f o l d i n g   p e n d i n g 
 
 
 
 # # #   D e c i s i o n 
 
 
 
 T h e   d e s k t o p   G U I   t e c h n o l o g y   r o u t e   h a s   c h a n g e d   f r o m   * * e g u i   +   e f r a m e * *   t o   * * T a u r i   2 . 0   +   R e a c t   +   T y p e S c r i p t   +   V i t e * * . 
 
 
## 15. Phase 2.5 — Tauri Desktop GUI & Application Layer

### Phase 2.5 Status

| Task | Status |
|:----:|:------:|
| T2.5-00 — Remove egui GUI & Cleanup | ? DONE / PASS |
| T2.5-01 — Tauri 2.0 Scaffold + Command Bridge | ? DONE / PASS |
| T2.5-02 — T2.5-04F — Full UI Pages & Services (Dashboard, Backup, Restore, History, Schedule, Settings) | ? ALL DONE / PASS |

### T2.5-00 — Remove egui GUI, Clean Up, Document Migration

**Date:** 2026-07-06
**Status:** DONE — egui GUI removed, Tauri 2.0 direction confirmed

#### Decision

The desktop GUI technology route has changed from **egui + eframe** to **Tauri 2.0 + React + TypeScript + Vite**.

| Previous | Current | Rationale |
|----------|---------|----------|
| egui + eframe (Rust immediate-mode) | Tauri 2.0 + React + TypeScript + Vite | egui visual quality ceiling too low for professional Acronis-like backup product |

#### What Was Done (T2.5-00)

| Action | Detail |
|--------|--------|
| Removed src/gui/ directory | All egui page files deleted |
| Removed src/gui_main.rs | egui binary entry deleted |
| Cleaned Cargo.toml | Removed egui/eframe/gui feature/nuwa-gui binary target |
| Cleaned src/lib.rs | Removed #[cfg(feature = "gui")] pub mod gui |
| Core library protection | Verified all Phase 1 + T2-07 core files unchanged |
| Quality gates | fmt/clippy/build/test all PASS (105 tests) |
| Documentation | Created Phase 2.5 migration decision doc |

### T2.5-01 — Tauri 2.0 Scaffold + Command Bridge

**Date:** 2026-07-07
**Status:** DONE / PASS — full Tauri 2.0 scaffold with React + TypeScript frontend and Rust command bridge

#### What Was Done (T2.5-01)

| Action | Detail |
|--------|--------|
| Created src-tauri/ | Tauri 2.0 project scaffold with tauri.conf.json, build.rs, Cargo.toml |
| Created src-tauri/src/main.rs | Tauri binary entry point (calls nuwa_tauri_lib::run()) |
| Created src-tauri/src/lib.rs | Tauri commands (get_version, list_backup_jobs), AppState (Mutex config_path) |
| Created src-tauri/capabilities/ | Default capability manifest for WebView permissions |
| Created src-tauri/icons/ | App icons (32x32, 128x128, 128x128@2x, .icns, .ico) |
| Created ui/ | React 19 + TypeScript + Vite 6 frontend scaffold |
| Created ui/src/App.tsx | Page router with 7 pages + Sidebar/TopBar layout |
| Created ui/src/components/ | Sidebar.tsx (nav), TopBar.tsx (page title + version) |
| Created ui/src/pages/ | All 7 pages (Dashboard, Backup, Restore, History, Schedule, Settings full implementation; Clone placeholder) |
| Created ui/src/styles.css | Base app styles (dark tech theme, flex layout) |
| Connected nuwa-backup core | src-tauri Cargo.toml depends on nuwa-backup = { path = ".." } |

#### Tauri Commands Registered

| Command | Signature | Purpose |
|---------|-----------|---------|
| get_version | fn get_version() -> String | Returns "Nuwa Backup vX.Y.Z (GUI)" from CARGO_PKG_VERSION |
| list_backup_jobs | fn list_backup_jobs() -> Result<Vec<(String, JobConfig)>, String> | Reads nuwa.toml and returns all configured backup jobs |

#### Architecture

Tauri Frontend (React + TypeScript) -> invoke() IPC -> Tauri Rust Commands (src-tauri/src/lib.rs) -> nuwa-backup core library (src/lib.rs)

#### Quality Gates

| Gate | Result |
|------|:------:|
| cargo fmt --check | ? PASS |
| cargo clippy --all-targets -- -D warnings | ? PASS |
| cargo build | ? PASS |
| cargo test | ? PASS (105 tests) |
| Forbidden scope audit | ? No forbidden features introduced |
| Phase 1 core protection | ? Intact |
| English-only / mojibake | ? Clean |

#### Key Documents

| Document | Location |
|----------|----------|
| Tauri Migration Decision | docs/phase-2.5/Phase_2_5_Tauri_Migration_Decision.md |
| T2.5-01 Completion Report | docs/phase-2.5/Phase_2_5_T2_5_01_Tauri_Scaffold_Report.md |

### Phase 2.5 Remaining Scope

### Desktop Application Architecture

The current architecture for the Tauri desktop GUI follows a strict layering:

`
React UI
    |
    | Tauri invoke() IPC
    v
Tauri Command Layer (src-tauri/src/commands/)
    | - Parameter validation only
    | - No business logic
    v
Application Service Layer (src/app/services/)
    | - Data aggregation and orchestration
    | - Product-level model mapping
    v
Core Engine (src/backup.rs, restore.rs, verify.rs, etc.)
    | - Backup/restore engine, storage, history, scheduler
    v
Storage / SQLite / File System
`

### Layer Rules

| Layer | Responsibility | Forbidden |
|-------|---------------|-----------|
| React UI | Display, interaction, state rendering | Direct core access, SQLite queries |
| Tauri Command | Parameter validation, invoke handling, error conversion | Business logic |
| Application Service | Business orchestration, data aggregation, model mapping | Core module modification |
| Core Engine | Backup/restore, verification, storage | UI coupling, web access |

### Core Protection Rule

The following core engine modules are **frozen stable modules**. They must NOT be modified,
restructured, or bypassed by any UI or Application Layer code:

| Module | File | Status |
|--------|------|--------|
| Backup engine | src/backup.rs | FROZEN |
| Restore engine | src/restore.rs | FROZEN |
| Verification engine | src/verify.rs | FROZEN |
| Manifest | src/manifest.rs | FROZEN |
| Checksum | src/checksum.rs | FROZEN |
| Storage | src/storage.rs | FROZEN |
| Prune | src/prune.rs | FROZEN |

**Violation example (FORBIDDEN):**

```
React UI -> invoke() -> Tauri command -> backup.rs  (WRONG)
```

**Correct pattern:**

```
React UI -> invoke() -> Tauri command -> app::services::backup_service -> backup.rs
```



| Task | Scope | Status |
|:----:|-------|:------:|
| T2.5-02 | Dashboard UI Architecture (React pages, mock data, layout components) | ? DONE / PASS |
| T2.5-03A | Application API Layer Foundation (models, services, error, Tauri bridge) | ? DONE / PASS |
| T2.5-03A.1 | Dashboard Product Polish (skeleton, empty/error states, micro-interactions) | ? DONE / PASS |
| T2.5-03B | Backup Application Service | ? DONE / PASS |
| T2.5-03C | Backup UI Integration | ? DONE / PASS |
| T2.5-03D | Restore Application Service + UI | ? DONE / PASS |
| T2.5-03D.1 | Restore Safety Hardening (path validation, rename reject) | ? DONE / PASS |
| T2.5-04A | Config Job CRUD Service (plan CRUD) | ? DONE / PASS |
| T2.5-04B | Settings Backup Plan UI | ? DONE / PASS |
| T2.5-04C | Native Path Picker (tauri-plugin-dialog) | ? DONE / PASS (superseded by 04C.1) |
| T2.5-04C.1 | In-App File Browser (FileBrowserModal) | ? DONE / PASS |
| T2.5-04D | Backup Content Browser (Restore 3-column + BackupTreeView) | ? DONE / PASS (8ea7355) |
| T2.5-04E | Schedule Consistency Hardening (lifecycle sync, two-phase delete, tests) | ? DONE / PASS (0a8ed93) |
| T2.5-04F | History Page + Service + Tauri Commands | ? DONE / PASS (0a8ed93) |
| T2.5-DOC-01A | Documentation synchronization | ? DONE / PASS |
| T2.5-DOC-02 | AGENTS + README alignment | ? DONE / PASS |



## 16. P-00C — Crash Consistency Manager Audit + Fix

**Status:** ? COMPLETE

**Contract:** `docs/phase-s/P-00_File_Backup_Repository_Data_Contract.md` v0.7

### Changes to Phase S Frozen Types (Section 4 of P-00 Contract)

| Type | Field | Change | File |
|------|-------|--------|------|
| `CatalogEntryType` | (new enum) | Added `File`/`Directory` variants | `catalog/engine.rs` |
| `FileExtent` | `file_offset: u64` | Added per-file offset | `catalog/engine.rs` |
| `FileEntry` | `entry_type: CatalogEntryType` | Added entry type | `catalog/engine.rs` |
| `FileEntry` | `sha256: Option<String>` | Added per-file hash | `catalog/engine.rs` |
| `CatalogEngine` | `add_directory()` | Added trait method | `catalog/engine.rs` |
| `RecoveryReport` | `auto_committed` removed | Replaced by `journal_cleaned`, `state_mismatch`, `orphans_cleaned` | `transaction/manager.rs` |

### CrashConsistencyManager Changes

| Requirement | Implementation | Status |
|-------------|----------------|--------|
| §3.2: `begin()` sets repo.db=WRITING, journal=CREATING | `create_restore_point(status="WRITING")` + `TransactionJournal(state=Creating)` | ? |
| §3.2: `enter_writing()` transitions journal CREATING→WRITING | `journal.state = TransactionState::Writing` + persist | ? |
| §5.3#1: No auto-commit from non-terminal states | `recover_at_startup()` marks non-terminal as FAILED only | ? |
| §5.3#2: commit reorder (repo.db before journal) | `set_restore_point_status(COMMITTED)` before journal write | ? |
| §5.3#3: Use RepoHandle API, not raw SQL | All calls through `repo.set_restore_point_status()` etc. | ? |

### RepoHandle API (New)

| Method | Purpose |
|--------|---------|
| `create_restore_point()` | Insert restore_points row |
| `update_restore_point_stats()` | Update block_count/total_raw_bytes |
| `set_restore_point_status()` | Update status field |
| `get_restore_point_status()` | Read status (returns `Result<Option<String>>`) |
| `list_non_terminal_restore_points()` | Find incomplete transactions |

### Files Changed (P-00C)

- `src/repository/transaction/manager.rs` — CrashConsistencyManager rewrite
- `src/repository/repo_manager.rs` — RepoHandle CRUD API
- `src/repository/catalog/engine.rs` — CatalogEntryType, file_offset, sha256, add_directory
- `src/repository/catalog/mod.rs` — Module sync
- `src/repository/catalog/sqlite_catalog.rs` — SQLite implementation update
- `tests/repository_integration_tests.rs` — Gate 3/4 tests, retention fix

### Quality Gates

| Gate | Result |
|------|--------|
| `cargo fmt --check` | ? |
| `cargo clippy --features repository --all-targets -- -D warnings` | ? |
| `cargo build --features repository` | ? |
| `cargo test --features repository --lib` | 292/292 ? |

---

## 17. P-01 — Repository File Backup Writer + CLI

**Status:** ? COMPLETE

### Scope

- P-01a: RepositoryBackupWriter lifecycle (new → begin → write → finalize → fail)
- P-01b: Directory traversal + file processing (backup_directory, write_file, write_directory)
- P-01c: Metadata with real SHA-256 per-file hashes
- P-01d: CLI integration (`nuwa backup --source <path> --repo <path>`)

### New Files

| File | Description |
|------|-------------|
| `src/repository/backup_writer.rs` | RepositoryBackupWriter (397 lines) |

### Changed Files

| File | Change |
|------|--------|
| `src/repository/mod.rs` | Added `pub mod backup_writer` + re-exports |
| `src/cli.rs` | Added `--repo` param to `Command::Backup`, Mode 3 in help text |
| `src/main.rs` | Added `execute_repo_backup()` function |

### CLI Usage

```
nuwa backup --source <path> --repo <path> [--compress] [--json]
```

### Quality Gates

| Gate | Result |
|------|--------|
| `cargo fmt --check` | ? |
| `cargo clippy --features repository --all-targets -- -D warnings` | ? |
| `cargo build --features repository` | ? |
| `cargo test --features repository --lib` | 292/292 ? |

### Remaining (P-02+)

- P-02: Repository Restore Reader (See Sect 18)
- P-03: Repository Verify CLI
- Gate 5: Application Layer + Tauri UI integration
- Gate 6: Flat File code removal

## 18. P-02 — Repository Restore Reader

**Status:** PLANNED
**Plan Document:** docs/phase-s/P-02_Repository_Restore_Plan.md

### Scope

- P-02a: Repository Preflight + Restore Point Identity & Integrity Validation (14-step)
- P-02b: Catalog Entry Validation (entry_type strict, extent verification)
- P-02c: Streaming File Restore (sequential write_all, SHA-256, atomic rename)
- P-02d: Directory Restore (per-ancestor dir guard, empty dir preservation)
- P-02e: Full & Selective Restore + RestoreOutcome (Complete/Partial)
- P-02f: Gate 2 & 3 Tests (27 test cases)

### Key Design Decisions

| Decision | Detail |
|----------|--------|
| Only COMMITTED Restore Points eligible | Any other status → reject |
| 14-step Preflight before any data read | Identity + 4-way metadata integrity check |
| Integrity errors = hard Err | Path escape, hash mismatch, block corruption |
| IO errors = soft fail (RestoreOutcome::Partial) | Single-file failure recorded in summary |
| Sequential write_all (no write_at) | Cross-platform safety, no sparse assumption |
| Windows atomic replace via ReplaceFileW | Overwrite safety on Windows |
| Per-ancestor dir guard (no create_dir_all) | Reject symlink/junction/reparse at every level |
| Shared path module (path_security.rs) | Writer + Reader use same security rules |

### New Modules

| Module | File | Purpose |
|--------|------|---------|
| M1 | src/repository/path_security.rs | validate_catalog_relative_path, prepare_restore_file_target, prepare_restore_directory |
| M2 | src/repository/repo_manager.rs | RestorePointRecord struct, get_restore_point() API |
| P-02 main | src/repository/file_restore_reader.rs | FileRestoreReader |

### Dependencies

| ID | Dependency | Status |
|----|-----------|--------|
| F1 | SqliteCatalog WAL checkpoint | DEFERRED (P-01 bug) |
| F2 | CatalogEntryType strict parsing | DEFERRED (P-01 bug) |
| M1 | path_security.rs | NOT STARTED |
| M2 | get_restore_point() | NOT STARTED |

