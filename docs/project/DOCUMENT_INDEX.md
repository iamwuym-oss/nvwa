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

## docs/phase-2/ — Phase 2 Planning Phase (Coding Requires Approval)

| Document | Authority | Purpose |
|----------|-----------|---------|
| *(reserved)* | — | No documents yet |

Phase 2 is reserved for: scheduler, retention policy, backup history, SMB
network target, GUI initial prototypes.

Phase 2 must NOT implement: `.nwb`, VSS, volume-level backup, disk-level
backup, WinPE, system restore, cloning, differential/incremental backup,
encryption.

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


