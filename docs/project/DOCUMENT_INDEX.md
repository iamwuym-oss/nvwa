# Nüwa Backup — Document Index

**Last Updated:** 2026-07-15 (BASELINE-CONSISTENCY-001-A)

---

## Overview

This index lists every Markdown file in the `docs/` hierarchy with its purpose and authority level. It is the single source of truth for document classification across all roles.

---

## Authority Levels

| Level | Meaning |
|-------|---------|
| **AUTHORITATIVE** | Must be followed. Active contract for implementation, architecture, or governance. |
| **REFERENCE** | Informational context. Not binding for implementation decisions. |
| **HISTORICAL** | Superseded. Entries marked **ACCEPTED BASELINE** record formally accepted phase outcomes. |
| **SUPERSEDED** | Replaced by later decisions or documents. Do not cite as current authority. |
| **STALE / PENDING_CORRECTION** | Previously authoritative but known to be outdated. Must be corrected before reuse. |

### Classification Rules

1. **AGENTS.md** at project root is the highest-priority general engineering governance document. It governs execution discipline, safety boundaries, and multi-agent collaboration. It does not define storage format, implementation plans, or phase-specific contracts.
2. **NWB Engineering Document Set** (`docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/`) is the current storage engine implementation authority. Its Architecture, Format Specification, Provider SDK, Product Support Matrix, Implementation Plan, and Verification/Acceptance/Test Plan are **AUTHORITATIVE**.
3. **Test Result Record** and **Evidence Documents** (EVD) record actual test evidence only. They cannot modify architecture contracts.
4. **Phase closing reports and their test evidence** are classified as **HISTORICAL / ACCEPTED BASELINE** — formally accepted phase outcomes that serve as traceability anchors.
5. Documents for directories or files that no longer exist on disk must not be listed.

---

## Repository-Level Documents

| Document | Authority | Purpose |
|----------|-----------|---------|
| `AGENTS.md` (repo root) | AUTHORITATIVE | Highest general engineering governance, safety rules, multi-agent roles |
| `README.md` (repo root) | STALE / PENDING_CORRECTION | Not in `docs/`. Known to be outdated, must be corrected in a future task |

---

## docs/ — Root Level

| Document | Authority | Purpose |
|----------|-----------|---------|
| `docs/Nüwa_NWB_存储引擎_SubAgent_中文使用手册_v1.0.md` | REFERENCE | Chinese user manual for storage engine Sub-Agent |

---

## docs/project/ — Cross-Phase

| Document | Authority | Purpose |
|----------|-----------|---------|
| `docs/project/DOCUMENT_INDEX.md` | AUTHORITATIVE | This file |
| `docs/project/PROJECT_ENGINEERING_MEMORY.md` | STALE / PENDING_CORRECTION | Previously authoritative project state tracker. Known to contain outdated phase/status references; must be corrected in a dedicated work package before reuse |
| `docs/project/PROPOSAL_FOR_NEW_PROJECT.md` | HISTORICAL | Original project proposal. Retained for traceability |
| `docs/project/01_Product_Requirements_Document.md` | REFERENCE | Original PRD |
| `docs/project/02_Development_Plan.md` | REFERENCE | Original task breakdown by phase |
| `docs/project/03_Nuwa_Architecture_Design.md` | REFERENCE | Original architecture design |
| `docs/project/04_Functional_Specification.md` | REFERENCE | Original functional specification |
| `docs/project/05_Pipeline_Optimization_Design.md` | REFERENCE | Pipeline optimization design (future reference) |
| `docs/project/06_Nuwa_Image_Format_Spec.md` | REFERENCE | .nwb image format specification |
| `docs/project/07_Checklist.md` | REFERENCE | Feature acceptance checklist |

---

## docs/phase-0/ — Phase 0: Project Setup & Guardrails

All Phase 0 documents are **HISTORICAL**. They guided initial project setup but have been superseded by later phase documents and the NWB Engineering Document Set.

| Document | Authority | Purpose |
|----------|-----------|---------|
| `docs/phase-0/00_Codex_Working_Guardrails.md` | HISTORICAL | Previously claimed highest authority. Superseded by `AGENTS.md` and NWB Engineering Document Set |
| `docs/phase-0/08_Design_Decision_Log.md` | HISTORICAL | Early design decisions, superseded by subsequent architecture work |
| `docs/phase-0/09_MVP_Boundary_and_Risk_Correction.md` | HISTORICAL | MVP scope freeze from Phase 0, superseded by later phase scoping |
| `docs/phase-0/10_Document_Correction_Report.md` | HISTORICAL | Document correction record from Phase 0 |

---

## docs/phase-1/ — Phase 1: File-Level Backup CLI (CLOSED)

| Document | Authority | Purpose |
|----------|-----------|---------|
| `docs/phase-1/Phase_1_Closing_Report.md` | HISTORICAL / ACCEPTED BASELINE | Phase 1 closing decision and acceptance record |
| `docs/phase-1/Phase_1_Final_Acceptance_Report.md` | HISTORICAL / ACCEPTED BASELINE | Phase 1 final acceptance outcome |
| `docs/phase-1/Phase_1_Test_Evidence.md` | HISTORICAL / ACCEPTED BASELINE | Phase 1 test results and evidence |
| `docs/phase-1/Phase_1_Technical_Baseline.md` | HISTORICAL | Phase 1 module structure and safety rules |
| `docs/phase-1/Phase_1_Known_Limitations_and_Risks.md` | HISTORICAL | Phase 1 known limitations |
| `docs/phase-1/Phase_1_Code_Map.md` | HISTORICAL | Phase 1 source code map |
| `docs/phase-1/Phase_1_Manual_Test_Plan.md` | HISTORICAL | Phase 1 manual test plan |
| `docs/phase-1/Phase_1_to_Phase_2_Handoff.md` | HISTORICAL | Phase 1 to Phase 2 handoff |

---

## docs/phase-2/ — Phase 2: CLI Usability (CLOSED)

| Document | Authority | Purpose |
|----------|-----------|---------|
| `docs/phase-2/Phase_2_Closing_Report.md` | HISTORICAL / ACCEPTED BASELINE | Phase 2 closing decision and acceptance record |
| `docs/phase-2/Phase_2_PRD.md` | HISTORICAL | Phase 2 product requirements |
| `docs/phase-2/Phase_2_Technical_Design.md` | HISTORICAL | Phase 2 technical design |
| `docs/phase-2/Phase_2_Revised_Plan.md` | HISTORICAL | Phase 2 revised plan |
| `docs/phase-2/Phase_2_Planning_Source_Baseline.md` | HISTORICAL | Phase 2 planning baseline |
| `docs/phase-2/Phase_2_UI_Direction_Decision.md` | HISTORICAL | Phase 2 UI direction decision |

---

## docs/phase-2.5/ — Phase 2.5: Tauri Desktop GUI (CLOSED)

| Document | Authority | Purpose |
|----------|-----------|---------|
| `docs/phase-2.5/Phase_2_5_Closing_Report.md` | HISTORICAL / ACCEPTED BASELINE | Phase 2.5 closing decision and acceptance record |
| `docs/phase-2.5/T2.5-05_Final_Acceptance_Report.md` | HISTORICAL / ACCEPTED BASELINE | Phase 2.5 GUI integration acceptance |
| `docs/phase-2.5/Phase_2_5_Codex_Handoff.md` | HISTORICAL | Phase 2.5 GUI handoff |
| `docs/phase-2.5/Phase_2_5_Current_Status.md` | HISTORICAL | Phase 2.5 current state |
| `docs/phase-2.5/Phase_2_5_T2_5_01_Tauri_Scaffold_Report.md` | HISTORICAL | Tauri scaffold implementation report |
| `docs/phase-2.5/Phase_2_5_T2_5_03A_1_Dashboard_Polish_Report.md` | HISTORICAL | Dashboard polish report |
| `docs/phase-2.5/Phase_2_5_T2_5_03A_Application_Layer_Report.md` | HISTORICAL | Application layer implementation report |
| `docs/phase-2.5/Phase_2_5_T2_5_03B_Backup_Service_Report.md` | HISTORICAL | Backup service implementation report |
| `docs/phase-2.5/Phase_2_5_T2_5_03C_Backup_UI_Report.md` | HISTORICAL | Backup UI implementation report |
| `docs/phase-2.5/Phase_2_5_T2_5_04D_Backup_Content_Browser_Draft.md` | HISTORICAL | Backup content browser draft |
| `docs/phase-2.5/Phase_2_5_Tauri_Migration_Decision.md` | HISTORICAL | Tauri migration decision record |
| `docs/phase-2.5/Phase_2_5_UI_Surface_Audit_Checklist.md` | HISTORICAL | UI surface audit checklist |

---

## docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/ — Current NWB Storage Engine Authority

This directory contains the **current storage engine implementation authority**. Contract documents #1–6 follow the authority order defined by the Engineering Document Set README §2. Evidence and metadata documents are listed afterward and cannot override implementation contracts.

### AUTHORITATIVE Documents (implementation contracts)

| # | Document | Purpose |
|---|----------|---------|
| 1 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Storage_Engine_Architecture_v2.0.md` | Current storage engine architecture — highest NWB architecture authority |
| 2 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md` | NWB binary format specification (Draft) |
| 3 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md` | Provider SDK specification (Draft) |
| 4 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Product_Support_Matrix_v1.0.md` | Product support matrix for NWB format |
| 5 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md` | Implementation plan and Gate roadmap |
| 6 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md` | Verification, acceptance, and test plan |

### REFERENCE / STALE Documents (evidence records, metadata)

| # | Document | Classification | Purpose |
|---|----------|----------------|---------|
| 7 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Test_Result_Record_v1.0.md` | STALE / PENDING_CORRECTION | Records actual test results. Cannot modify architecture contracts |
| 8 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/IMP-000_EVD_Build_Evidence_v1.0.md` | STALE / PENDING_CORRECTION | IMP-000 build evidence |
| 9 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/IMP-001_EVD_Test_Result_Evidence_v1.0.md` | STALE / PENDING_CORRECTION | IMP-001 test result evidence |
| 10 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Engineering_Document_Set_README_v1.0.md` | REFERENCE | Document set README and reading order |
| 11 | `docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Engineering_Document_Set_Manifest_v1.0.md` | STALE / PENDING_CORRECTION | Document set manifest with SHA-256 hashes |

---



## Current Project State

| Dimension | Status |
|-----------|--------|
| **Gate** | GATE-0 — IN_PROGRESS |
| **IMP-000** (Workspace) | Consistency and evidence remediation pending — not yet a closed evidence loop |
| **IMP-001** (Format Registry) | Consistency and evidence remediation pending — not yet a closed evidence loop |
| **IMP-002** (Requirements–Test Traceability Matrix) | NOT_RUN — not yet authorized to start |
| **IMP-100 and later** | NOT_STARTED / FORBIDDEN UNTIL GATE-0 — must not be initiated before GATE-0 is closed and IMP-002 is planned and authorized |

### Key Constraints

1. All new storage functionality must follow the NWB Storage Engine architecture: each successful Full or Differential backup produces an immutable, self-describing logical NWB archive. It must not depend on the superseded Repository architecture.
2. `PROJECT_ENGINEERING_MEMORY.md` must be corrected in a dedicated work package before it can be reused as authoritative.
3. `README.md` at repo root is **STALE / PENDING_CORRECTION** — its content does not reflect the current project status or NWB storage engine direction.

---

## Navigation Rules

1. Start at this index to identify the correct authority level for each document.
2. **AUTHORITATIVE** documents take precedence over all lower levels.
3. In case of conflict between AUTHORITATIVE documents, the NWB Storage Engine Architecture v2.0 takes precedence for storage engine matters; AGENTS.md takes precedence for governance and execution discipline.
4. **STALE / PENDING_CORRECTION** documents must be corrected before reuse as authority.
5. Documents classified as **HISTORICAL** or **SUPERSEDED** must not be cited as current implementation authority.
6. When a document on disk is not listed in this index, treat it as unclassified and do not rely on it for authoritative guidance until classified.

---


## Revision

- **Version:** 1.0
- **Task:** BASELINE-CONSISTENCY-001-A
- **Purpose:** Rebuild document index to reflect real filesystem state, correct authority levels, and current project status
- **Previous version:** 2026-07-13 (contained NUL bytes, listed deleted documents, incorrect authority levels)
- **Maintenance:** Update when documents are added, removed, reclassified, or when project gate status changes
