# Nüwa Backup — Project Engineering Memory

**Version:** 0.1.0
**Last Updated:** 2026-07-05 (Updated Task 2.0B)
**Current Phase:** Phase 1 — Minimal File Backup/Restore CLI (CLOSED)
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

### Phase 2 Status

- **Phase 2 planning is allowed.**
- **Phase 2 coding requires explicit user approval after Phase 1 final acceptance.**
- Phase 2 must NOT implement Phase 3/4/5/6+ features.

### Phase 1 Core Freeze

The Phase 1 file-level backup/restore CLI is frozen. Codex must not rewrite,
restructure, or expand the Phase 1 file backup core unless the user explicitly
approves a task that modifies it.

---

## 5. Phase Boundaries

| Phase | Scope | Status |
|-------|-------|--------|
| Phase 0 | Project setup, MVP boundary, guardrails | COMPLETE |
| Phase 1 | File-level backup/restore CLI | PARTIAL (current) |
| Phase 2 | Scheduler, retention, history, SMB, GUI | NOT STARTED |
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

9. **docs/phase-2/Phase_2_Planning_Source_Baseline.md** — Phase 2 planning authority baseline, confirmed/optional/forbidden scope.


## 10. Phase 2 UI Decision (Task 2.0B Updated)

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
**Status:** ✅ COMPLETE (user confirmed "T2-08就到这里了")

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
| `cargo build --features gui` | ✅ PASS |
| GUI launched successfully | ✅ PASS (process started, Dashboard UI visible) |
| Phase 1 core frozen | ✅ No Phase 1 core files modified |
| Forbidden scope | ✅ No .nwb, VSS, encryption, daemon, clone |

### Remaining Phase 2 GUI Pages

| Page | File | Status |
|------|------|--------|
| Backup | `src/gui/pages/backup.rs` | ⏳ Stub |
| Restore | `src/gui/pages/restore.rs` | ⏳ Stub |
| History | `src/gui/pages/history.rs` | ⏳ Stub |
| Schedule | `src/gui/pages/schedule.rs` | ⏳ Stub |
| Settings | `src/gui/pages/settings.rs` | ⏳ Stub |
| Clone | `src/gui/pages/clone.rs` | ✅ Done (disabled placeholder) |

**Next recommended task:** Backup page (`backup.rs`) — core functionality enabling actual backup operations from GUI.

---

## 13. Phase 2 Closing Status

**Date:** 2026-07-06
**Status:** CLOSED / ACCEPTED WITH KNOWN LIMITATIONS

### Phase 2 Final Scope

| Area | Status |
|------|:------:|
| T2-01 Config/Job | ✅ PASS |
| T2-02 History/SQLite | ✅ PASS |
| T2-03 CLI Output/JSON | ✅ PASS |
| T2-04 Retention/Prune | ✅ PASS |
| T2-05 Scheduler | ✅ PASS |
| T2-06 SMB/UNC | ✅ PASS |
| T2-07 GUI Scaffold | ✅ PASS |
| T2-08 GUI Dashboard | ✅ PASS |
| T2-09 Clone Placeholder | ✅ PASS |

### Deferred (User Decision)

| Item | Status |
|------|:------:|
| GUI Backup/Restore/History/Schedule/Settings | DEFERRED — stubs only |

### Quality Gates (Final)

| Gate | Result |
|------|:------:|
| `cargo fmt --check` | ✅ PASS |
| `cargo clippy --all-targets -- -D warnings` | ✅ PASS |
| `cargo build` | ✅ PASS |
| `cargo test` (94 tests) | ✅ ALL PASS |
| `cargo build --features gui` | ✅ PASS |
| GUI launch | ✅ PASS |
| Forbidden scope audit | ✅ No violations |
| Phase 1 core protection | ✅ Intact |
| English-only / mojibake | ✅ Clean |

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
| T3-00 — Phase 3 Scope Reset & Documentation | ✅ DONE / PASS |
| T3-01 — .nwb v0.2 Format + Block SHA-256 | ⏳ NOT STARTED |
| T3-02 — VSS Snapshot Lifecycle Proof | ⏳ NOT STARTED |
| T3-03 — Non-System NTFS Volume Backup CLI | ⏳ NOT STARTED |
| T3-04 — Non-System NTFS Volume Restore CLI | ⏳ NOT STARTED |
| T3-CLOSE — Phase 3 Final Validation | ⏳ NOT STARTED |

### Key Documents

| Document | Location |
|----------|----------|
| Phase 3 Plan | `docs/phase-3/Phase_3_Plan.md` |
- - -  
  
 # #   1 5 .   P h a s e   2 . 5   -   G U I   T e c h n o l o g y   M i g r a t i o n   ( T 2 . 5 - 0 0 )  
  
 * * D a t e : * *   2 0 2 6 - 0 7 - 0 6  
 * * S t a t u s : * *   D O N E   -   e g u i   G U I   r e m o v e d ,   T a u r i   2 . 0   d i r e c t i o n   c o n f i r m e d ,   s c a f f o l d i n g   p e n d i n g  
  
 # # #   D e c i s i o n  
  
 T h e   d e s k t o p   G U I   t e c h n o l o g y   r o u t e   h a s   c h a n g e d   f r o m   * * e g u i   +   e f r a m e * *   t o   * * T a u r i   2 . 0   +   R e a c t   +   T y p e S c r i p t   +   V i t e * * .  
 
## 15. Phase 2.5 — GUI Technology Migration (T2.5-00)

**Date:** 2026-07-06
**Status:** DONE — egui GUI removed, Tauri 2.0 direction confirmed

### Decision

The desktop GUI technology route has changed from **egui + eframe** to **Tauri 2.0 + React + TypeScript + Vite**.

| Previous | Current | Rationale |
|----------|---------|----------|
| egui + eframe (Rust immediate-mode) | Tauri 2.0 + React + TypeScript + Vite | egui visual quality ceiling too low for professional Acronis-like backup product |

### What Was Done (T2.5-00)

| Action | Detail |
|--------|--------|
| Removed src/gui/ directory | All egui page files deleted |
| Removed src/gui_main.rs | egui binary entry deleted |
| Cleaned Cargo.toml | Removed egui/eframe/gui feature/nuwa-gui binary target |
| Cleaned src/lib.rs | Removed #[cfg(feature = gui)] pub mod gui |
| Core library protection | Verified all Phase 1 + T2-07 core files unchanged |
| Quality gates | fmt/clippy/build/test all PASS (94 tests) |
| Documentation | Created Phase 2.5 migration decision doc |

### Architecture (Future)

`
Tauri Frontend (React + TypeScript)
  -> invoke() IPC -> Tauri Rust Commands -> nuwa-backup core library (lib.rs)
`

### Phase 2.5 Scope

| Task | Scope | Status |
|:----:|-------|:------:|
| T2.5-00 | Remove egui GUI, clean up, document migration | ✅ DONE |
| T2.5-01 | Tauri 2.0 scaffold + command bridge | ⏳ NOT STARTED |
| T2.5-02..08 | GUI pages (Dashboard, Backup, Restore, History, Schedule, Settings, Clone placeholder) | ⏳ NOT STARTED |

### Key Documents

| Document | Location |
|----------|----------|
| Tauri Migration Decision | docs/phase-2.5/Phase_2_5_Tauri_Migration_Decision.md |
