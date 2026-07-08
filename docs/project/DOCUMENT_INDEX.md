# Nüwa Backup — Document Index

**Last Updated:** 2026-07-09

---

## Overview

This index lists every document in the `docs/` hierarchy with its purpose,
authority level, and applicability.

### Authority Levels

| Level | Meaning |
|-------|---------|
| **AUTHORITATIVE** | Must be followed. If conflicted, higher priority documents win. |
| **REFERENCE** | Informational. Provides context but does not override authoritative docs. |
| **DRAFT** | Work in progress. Subject to change. |
| **HISTORICAL** | Superseded. Kept for traceability only. |

---

## docs/project/ — Cross-Phase, Long-Term Valid

| Document | Authority | Purpose |
|----------|-----------|---------|
| `PROJECT_ENGINEERING_MEMORY.md` | AUTHORITATIVE | Long-term project state, phase boundaries, task-before-read checklist |
| `DOCUMENT_INDEX.md` | AUTHORITATIVE | This file — document index and navigation |
| `PROPOSAL_FOR_NEW_PROJECT.md` | REFERENCE | Original project proposal |
| `01_Product_Requirements_Document.md` | REFERENCE | Original PRD |
| `02_Development_Plan.md` | REFERENCE | Task breakdown by phase |
| `03_Nuwa_Architecture_Design.md` | REFERENCE | Architecture design document |
| `04_Functional_Specification.md` | REFERENCE | Functional specification |
| `05_Pipeline_Optimization_Design.md` | REFERENCE | Pipeline optimization (future phase) |
| `06_Nuwa_Image_Format_Spec.md` | REFERENCE | .nwb image format spec (future phase) |
| `07_Checklist.md` | REFERENCE | Feature acceptance checklist by phase |

---

## docs/phase-0/ — Phase 0: Project Setup & Guardrails

| Document | Authority | Purpose |
|----------|-----------|---------|
| `00_Codex_Working_Guardrails.md` | AUTHORITATIVE (highest) | Codex work boundary rules and behavior constraints |
| `08_Design_Decision_Log.md` | AUTHORITATIVE | All confirmed design decisions |
| `09_MVP_Boundary_and_Risk_Correction.md` | AUTHORITATIVE | MVP scope freeze and risk list |
| `10_Document_Correction_Report.md` | HISTORICAL | Document correction audit report |

---

## docs/phase-1/ — Phase 1: File-Level Backup/Restore CLI (CLOSED)

| Document | Authority | Purpose |
|----------|-----------|---------|
| `Phase_1_Final_Acceptance_Report.md` | AUTHORITATIVE | Phase 1 final acceptance report (upgraded from Draft) |
| `Phase_1_Technical_Baseline.md` | AUTHORITATIVE | Module structure, data flow, safety rules, error code system |
| `Phase_1_Known_Limitations_and_Risks.md` | AUTHORITATIVE | All known limitations and risks from Phase 1 |
| `Phase_1_Code_Map.md` | REFERENCE | Source code file-by-file map |
| `Phase_1_to_Phase_2_Handoff.md` | AUTHORITATIVE | Phase handoff boundary: what Phase 2 can build on |
| `Phase_1_Manual_Test_Plan.md` | REFERENCE | Manual test procedures |
| Phase_1_Closing_Report.md | AUTHORITATIVE | Phase 1 closing decision, freeze status, evidence summary, accepted limitations |
| Phase_1_Test_Evidence.md | AUTHORITATIVE | All test results, E2E validation evidence, quality gate results |

---

## docs/phase-2/ — Phase 2: CLI Usability + GUI Dashboard (CLOSED)

| Document | Authority | Purpose |
|----------|-----------|---------|
| `Phase_2_PRD.md` | AUTHORITATIVE | Phase 2 product requirements and approved coding basis for task-specific implementation |
| `Phase_2_Technical_Design.md` | AUTHORITATIVE | Phase 2 technical design and approved engineering basis for task-specific implementation |
| `Phase_2_Revised_Plan.md` | AUTHORITATIVE | Phase 2 revised plan: 6 usability features + local desktop GUI implementation + Clone disabled placeholder. .nwb moved to Phase 3. |
| `Phase_2_Planning_Source_Baseline.md` | AUTHORITATIVE | Phase 2 planning source baseline: confirmed/optional/forbidden scope, contamination cleanup record, required user decisions |
| `Phase_2_UI_Direction_Decision.md` | HISTORICAL | Phase 2 UI direction (superseded). egui/eframe replaced by Tauri 2.0+React. See Phase_2_5_Tauri_Migration_Decision.md. |

Phase 2 implementation has started under task-specific approval.

Current task status:
- T2-01 — Configuration system / backup job: DONE / PASS
- T2-02 — Backup history / SQLite schema: DONE / PASS
- T2-03 — CLI output enhancement: DONE / PASS
- T2-04 — Retention policy / prune: DONE / PASS
- T2-05 — Windows Task Scheduler: DONE / PASS
- T2-06 — SMB / UNC path support: DONE / PASS (manual SMB validation completed on \\localhost\C$ admin share; backup/list/verify/restore all confirmed working)
- T2-07 — GUI dependency + scaffold: DONE / PASS
- T2-08 — GUI 7 pages: NOT STARTED / requires user approval
- T2-09 — GUI Clone placeholder: NOT STARTED / requires user approval

Language compliance:
- Product runtime language is English-only (T2-LANG-01 enforced).
- Chinese is allowed only in documentation.
- Language compliance check is required before PASS for coding tasks.
- All src/, tests/, and Cargo.toml files are English-only as of T2-LANG-01.

Phase 2 implementation has started.
Coding approval is task-specific.
T2-01 through T2-04 have been completed and passed.
No further Phase 2 coding task, starting from T2-05, is approved until the user explicitly approves it.

Phase 2 must NOT implement:
- `.nwb` image format
- VSS snapshot integration
- Volume-level backup
- Disk-level backup
- System restore
- WinPE recovery media
- Real disk cloning functionality (Clone UI placeholder is allowed and required, but must be disabled and clearly marked as Phase 5)
- Differential backup
- Incremental backup
- Encryption
- Daemon / system service
- Web GUI / browser-based admin console
- FastAPI / Python backend
- Vanilla JS / Tailwind / React / Vue / jQuery

No clone engine, clone CLI command, disk access, partition access, PhysicalDrive access, VSS, or .nwb code may be introduced in Phase 2.

---


## docs/phase-2.5/ — Phase 2.5: Tauri Desktop GUI + Application Layer

| Document | Authority | Purpose |
|----------|-----------|---------|
| Phase_2_5_Tauri_Migration_Decision.md | AUTHORITATIVE | GUI technology migration from egui/eframe to Tauri 2.0 + React + TypeScript |
| Phase_2_5_T2_5_01_Tauri_Scaffold_Report.md | AUTHORITATIVE | T2.5-01 Tauri 2.0 scaffold + command bridge completion report |
| Phase_2_5_T2_5_02_Dashboard_UI_Architecture_Report.md | AUTHORITATIVE | T2.5-02 Dashboard UI architecture, mock data, and page structure |
| Phase_2_5_T2_5_03A_Application_Layer_Report.md | AUTHORITATIVE | T2.5-03A Application API Layer foundation |
| Phase_2_5_T2_5_03A_1_Dashboard_Polish_Report.md | AUTHORITATIVE | T2.5-03A.1 Dashboard product polish |
| Phase_2_5_Closing_Report.md | AUTHORITATIVE | Phase 2.5 closing report: completed tasks, architecture summary, frozen modules, next phase


### Missing Report Documents

The following tasks were completed but do not have individual report files. Their key details are captured in Phase_2_5_Current_Status.md and Phase_2_5_Codex_Handoff.md:

- T2.5-03D (Restore Service + UI)
- T2.5-03D.1 (Restore Safety Hardening)
- T2.5-04A (Config Job CRUD Service)
- T2.5-04B (Settings Backup Plan UI)
- T2.5-04C (Native Path Picker — superseded)
- T2.5-04C.1 (In-App File Browser)

### Phase 2.5 Task Status

| Task | Status |
|:----:|:------:|
| T2.5-00 — Remove egui GUI & Cleanup | ✅ DONE / PASS |
| T2.5-01 — Tauri 2.0 Scaffold + Command Bridge | ✅ DONE / PASS |
| T2.5-02 — Dashboard UI Architecture | ✅ DONE / PASS |
| T2.5-03A — Application API Layer Foundation | ✅ DONE / PASS |
| T2.5-03A.1 — Dashboard Product Polish | ✅ DONE / PASS |
| T2.5-03B — Backup Application Service Foundation | ✅ DONE / PASS |
| T2.5-03C — Backup UI Integration | ✅ DONE / PASS |
| T2.5-03D — Restore Service + UI | ✅ DONE / PASS |
| T2.5-03D.1 — Restore Safety Hardening | ✅ DONE / PASS |
| T2.5-04A — Config Job CRUD Service | ✅ DONE / PASS |
| T2.5-04B — Settings Backup Plan UI | ✅ DONE / PASS |
| T2.5-04C — Native Path Picker | ✅ DONE / PASS (superseded by 04C.1) |
| T2.5-04C.1 — In-App File Browser | ✅ DONE / PASS |
| T2.5-04D — Backup Content Browser | 🔄 UNCOMMITTED — 3 frontend files in working tree |
| T2.5-DOC-01A — Documentation sync | ✅ DONE / PASS |
| T2.5-DOC-02 — AGENTS + README alignment | ✅ DONE / PASS |

## docs/phase-3/ — Reserved: NTFS Volume Image, VSS, Block Backup

*(reserved)*

---

## docs/phase-4/ — Reserved: WinPE Recovery Media, System Restore

*(reserved)*

---

## docs/phase-5/ — Reserved: Disk Cloning

*(reserved)*

---

## docs/phase-6-plus/ — Reserved: Differential, Encryption, Cross-Platform

*(reserved)*

---

## Navigation Rules

1. Start at `PROJECT_ENGINEERING_MEMORY.md` for every new task.
2. Use this index to find the correct document.
3. Authoritative documents take precedence over Reference documents.
4. If documents conflict, higher-priority documents (lower number) win.
5. AGENTS.md at project root is the highest-priority operational document.
## docs/phase-2/ — Phase 2: CLI Usability + GUI Dashboard (CLOSED)

| Document | Authority | Purpose |
|----------|-----------|---------|
| `Phase_2_PRD.md` | AUTHORITATIVE | Phase 2 product requirements and approved coding basis |
| `Phase_2_Technical_Design.md` | AUTHORITATIVE | Phase 2 technical design |
| `Phase_2_Revised_Plan.md` | AUTHORITATIVE | Phase 2 revised plan: 6 usability features + GUI Dashboard |
| `Phase_2_Planning_Source_Baseline.md` | AUTHORITATIVE | Phase 2 planning baseline, confirmed/forbidden scope |
| `Phase_2_UI_Direction_Decision.md` | HISTORICAL | Phase 2 UI direction (superseded). egui/eframe replaced by Tauri 2.0+React. See Phase_2_5_Tauri_Migration_Decision.md. |
| `Phase_2_Closing_Report.md` | AUTHORITATIVE | Phase 2 closing report, final validation evidence |

### Phase 2 Task Status (Final)

| Task | Status |
|:----:|:------:|
| T2-01 — Config / Job | ✅ DONE / PASS |
| T2-02 — Backup History / SQLite | ✅ DONE / PASS |
| T2-03 — CLI Output Enhancement | ✅ DONE / PASS |
| T2-04 — Retention / Prune | ✅ DONE / PASS |
| T2-05 — Windows Task Scheduler | ✅ DONE / PASS |
| T2-06 — SMB / UNC Path | ✅ DONE / PASS |
| T2-07 — GUI Scaffold | ✅ DONE / PASS |
| T2-08 — GUI Dashboard | ✅ DONE / PASS |
| T2-09 — GUI Clone Placeholder | ✅ DONE / PASS |

### Phase 2 Deferred Items

| Item | Reason |
|------|--------|
| GUI Backup page (stub) | User decision — stop at Dashboard baseline |
| GUI Restore page (stub) | User decision — stop at Dashboard baseline |
| GUI History page (stub) | User decision — stop at Dashboard baseline |
| GUI Schedule page (stub) | User decision — stop at Dashboard baseline |
| GUI Settings page (stub) | User decision — stop at Dashboard baseline |

**Phase 2: CLOSED / ACCEPTED WITH KNOWN LIMITATIONS**
**Phase 3 coding is NOT authorized without explicit user approval.**

## docs/phase-3/ — Phase 3: NTFS Non-System Volume Image MVP (PLANNING)

| Document | Authority | Purpose |
|----------|-----------|---------|
| `Phase_3_Plan.md` | AUTHORITATIVE | Phase 3 scope, safety boundary, task chain, deferred items |

### Phase 3 Task Status

| Task | Status |
|:----:|:------:|
| T3-00 — Scope Reset & Documentation | ✅ DONE / PASS |
| T3-01 — .nwb v0.2 Format + Block SHA-256 | ⏳ NOT STARTED |
| T3-02 — VSS Snapshot Lifecycle Proof | ⏳ NOT STARTED |
| T3-03 — Non-System NTFS Volume Backup CLI | ⏳ NOT STARTED |
| T3-04 — Non-System NTFS Volume Restore CLI | ⏳ NOT STARTED |
| T3-CLOSE — Phase 3 Final Validation | ⏳ NOT STARTED |

### Phase 3.5/4 Status

| Phase | Scope | Status |
|-------|-------|:------:|
| Phase 2.5 | Tauri desktop GUI + Application Layer | ✅ IN PROGRESS (152 tests, 14 Tauri commands, 5 services, 4 active pages) |
| Phase 3.5 | GUI volume pages, GPT/MBR, boot partition, dynamic disk/RAID, BCD design | ❌ NOT AUTHORIZED |
| Phase 4 | System recovery / WinPE / BMR | ❌ NOT AUTHORIZED |
| Phase 5 | Disk clone | ❌ NOT AUTHORIZED |
| Phase 6+ | Differential, encryption, cross-platform | ❌ NOT AUTHORIZED |

**Phase 3 is in PLANNING. T3-00 complete. Coding not started.**
