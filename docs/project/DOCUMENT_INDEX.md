# Nüwa Backup — Document Index

**Last Updated:** 2026-07-05

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

## docs/phase-2/ — Phase 2 Implementation Phase (Task-Specific Coding Approval)

| Document | Authority | Purpose |
|----------|-----------|---------|
| `Phase_2_PRD.md` | AUTHORITATIVE | Phase 2 product requirements and approved coding basis for task-specific implementation |
| `Phase_2_Technical_Design.md` | AUTHORITATIVE | Phase 2 technical design and approved engineering basis for task-specific implementation |
| `Phase_2_Revised_Plan.md` | AUTHORITATIVE | Phase 2 revised plan: 6 usability features + local desktop GUI implementation + Clone disabled placeholder. .nwb moved to Phase 3. |
| `Phase_2_Planning_Source_Baseline.md` | AUTHORITATIVE | Phase 2 planning source baseline: confirmed/optional/forbidden scope, contamination cleanup record, required user decisions |
| `Phase_2_UI_Direction_Decision.md` | AUTHORITATIVE | Phase 2 UI direction: local desktop GUI, Acronis True Image-like, egui+eframe. Clone page as disabled placeholder. Not Web GUI. Not FastAPI. |

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
