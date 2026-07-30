# Nüwa Backup — Document Index

**Last Updated:** 2026-07-25 (IMP-003 evidence closure)

---

## Overview

This index lists every Markdown file in the docs/ hierarchy with its purpose and authority level. It is the single source of truth for document classification across all roles.

---

## Authority Levels

| Level | Meaning |
|-------|---------|
| **AUTHORITATIVE** | Must be followed. Active contract for implementation, architecture, or governance. |
| **DERIVED IMPLEMENTATION STANDARD** | Binding implementation rule for its approved work-package scope; subordinate to the authoritative contract documents and cannot change them. |
| **TRACEABILITY AUTHORITY** | Authoritative only for requirement/test IDs, priorities, source bindings, mappings and traceability dispositions. Cannot override implementation contracts. |
| **REFERENCE** | Informational context. Not binding for implementation decisions. |
| **HISTORICAL** | Superseded. Entries marked **ACCEPTED BASELINE** record formally accepted phase outcomes. |
| **SUPERSEDED** | Replaced by later decisions or documents. Do not cite as current authority. |
| **STALE / PENDING_CORRECTION** | Previously authoritative but known to be outdated. Must be corrected before reuse. |

### Classification Rules

1. **AGENTS.md** at project root is the highest-priority general engineering governance document. It governs execution discipline, safety boundaries, and multi-agent collaboration. It does not define storage format, implementation plans, or phase-specific contracts.
2. **NWB Engineering Document Set** (docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/) is the current storage engine implementation authority. Its Architecture, Format Specification, Provider SDK, Product Support Matrix, Implementation Plan, and Verification/Acceptance/Test Plan are **AUTHORITATIVE**. Approved derived implementation standards are binding only inside their named work-package scope and remain subordinate to documents #1–6.
3. `Nuwa_NWB_Traceability_Registry_v1.0.toml` is **TRACEABILITY AUTHORITY** only for ID, priority, test, source and mapping governance. It cannot change the meaning of contract documents #1–6. Its generated Markdown Matrix is **REFERENCE**.
4. **Test Result Record** and **Evidence Documents** (EVD) record actual test evidence only. They cannot modify architecture contracts.
5. **Phase closing reports and their test evidence** are classified as **HISTORICAL / ACCEPTED BASELINE** — formally accepted phase outcomes that serve as traceability anchors.
6. Documents for directories or files that no longer exist on disk must not be listed.

---

## Repository-Level Documents

| Document | Authority | Purpose |
|----------|-----------|---------|
| AGENTS.md (repo root) | AUTHORITATIVE | Highest general engineering governance, safety rules, multi-agent roles |
| README.md (repo root) | REFERENCE | Current repository overview and navigation entry. Derived from authoritative contracts and repository state; cannot override them |

---

## docs/ — Root Level

| Document | Authority | Purpose |
|----------|-----------|---------|
| docs/Nüwa_NWB_存储引擎_SubAgent_中文使用手册_v1.0.md | REFERENCE | Chinese user manual for storage engine Sub-Agent |

---

## docs/project/ — Cross-Phase

| Document | Authority | Purpose |
|----------|-----------|---------|
| docs/project/DOCUMENT_INDEX.md | AUTHORITATIVE | This file |
| docs/project/PROJECT_ENGINEERING_MEMORY.md | REFERENCE | Current cross-phase operational snapshot. Derived from authoritative contracts, evidence, and repository state; cannot override them |
| docs/project/PROPOSAL_FOR_NEW_PROJECT.md | HISTORICAL | Original project proposal. Retained for traceability |
| docs/project/01_Product_Requirements_Document.md | REFERENCE | Original PRD |
| docs/project/02_Development_Plan.md | REFERENCE | Original task breakdown by phase |
| docs/project/03_Nuwa_Architecture_Design.md | REFERENCE | Original architecture design |
| docs/project/04_Functional_Specification.md | REFERENCE | Original functional specification |
| docs/project/05_Pipeline_Optimization_Design.md | REFERENCE | Pipeline optimization design (future reference) |
| docs/project/06_Nuwa_Image_Format_Spec.md | REFERENCE | .nwb image format specification |
| docs/project/07_Checklist.md | REFERENCE | Feature acceptance checklist |

---

## docs/phase-0/ — Phase 0: Project Setup & Guardrails

All Phase 0 documents are **HISTORICAL**. They guided initial project setup but have been superseded by later phase documents and the NWB Engineering Document Set.

| Document | Authority | Purpose |
|----------|-----------|---------|
| docs/phase-0/00_Codex_Working_Guardrails.md | HISTORICAL | Previously claimed highest authority. Superseded by AGENTS.md and NWB Engineering Document Set |
| docs/phase-0/08_Design_Decision_Log.md | HISTORICAL | Early design decisions, superseded by subsequent architecture work |
| docs/phase-0/09_MVP_Boundary_and_Risk_Correction.md | HISTORICAL | MVP scope freeze from Phase 0, superseded by later phase scoping |
| docs/phase-0/10_Document_Correction_Report.md | HISTORICAL | Document correction record from Phase 0 |

---

## docs/phase-1/ — Phase 1: File-Level Backup CLI (CLOSED)

| Document | Authority | Purpose |
|----------|-----------|---------|
| docs/phase-1/Phase_1_Closing_Report.md | HISTORICAL / ACCEPTED BASELINE | Phase 1 closing decision and acceptance record |
| docs/phase-1/Phase_1_Final_Acceptance_Report.md | HISTORICAL / ACCEPTED BASELINE | Phase 1 final acceptance outcome |
| docs/phase-1/Phase_1_Test_Evidence.md | HISTORICAL / ACCEPTED BASELINE | Phase 1 test evidence and log |
| docs/phase-1/Phase_1_Signed_Acceptance_by_Stakeholder.md | HISTORICAL / ACCEPTED BASELINE | Phase 1 stakeholder acceptance |
| docs/phase-1/Phase_1_User_Manual.md | HISTORICAL | Phase 1 user manual; superseded by later documentation |
| docs/phase-1/Restore_Phase_1_Test_Report_and_Code_Review.md | HISTORICAL / ACCEPTED BASELINE | Phase 1 restore test + code review |

---

## docs/phase-2/ — Phase 2: CLI Usability + egui GUI (CLOSED)

| Document | Authority | Purpose |
|----------|-----------|---------|
| docs/phase-2/Phase_2_Design_and_Acceptance_v2.md | HISTORICAL / ACCEPTED BASELINE | Phase 2 design and acceptance record |
| docs/phase-2/Phase_2_Tauri_Upgrade_Design_and_Acceptance.md | HISTORICAL / ACCEPTED BASELINE | Phase 2 Tauri acceptance record |

---

## docs/phase-2.5/ — Phase 2.5: Tauri 2 Desktop GUI (CLOSED)

| Document | Authority | Purpose |
|----------|-----------|---------|
| docs/phase-2.5/Phase_2.5_Closing_Report.md | HISTORICAL / ACCEPTED BASELINE | Phase 2.5 closing decision and acceptance record |
| docs/phase-2.5/Phase_2.5_Component_Redesign_Report.md | HISTORICAL | Phase 2.5 component redesign |
| docs/phase-2.5/Phase_2.5_Security_Audit_Checklist.md | HISTORICAL | UI surface audit checklist |

---

## docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/ — Current NWB Storage Engine Authority

This directory contains the **current storage engine implementation authority**. Contract documents #1–6 follow the authority order defined by the Engineering Document Set README §2. Traceability authority, generated references, evidence and metadata are listed afterward and cannot override implementation contracts.

### AUTHORITATIVE Documents (implementation contracts)

| # | Document | Purpose |
|---|----------|---------|
| 1 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Storage_Engine_Architecture_v2.0.md | Current storage engine architecture — highest NWB architecture authority |
| 2 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Binary_Format_Specification_v1.0_Draft.md | NWB binary format specification (Draft) |
| 3 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Provider_SDK_Specification_v1.0_Draft.md | Provider SDK specification (Draft) |
| 4 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Product_Support_Matrix_v1.0.md | Product support matrix for NWB format |
| 5 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Implementation_Plan_v1.0.md | Implementation plan and Gate roadmap |
| 6 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Verification_Acceptance_and_Test_Plan_v1.0.md | Verification, acceptance, and test plan |

### TRACEABILITY AUTHORITY

| Document | Scope of authority |
|---|---|
| docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Traceability_Registry_v1.0.toml | Canonical requirement/test IDs, priorities, authority-source bindings, mappings and `MAPPED` / `SOURCE_SCOPED` dispositions. Cannot override documents #1–6 |

### DERIVED IMPLEMENTATION STANDARDS

| Document | Scope and boundary |
|---|---|
| docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Structured_Error_and_Logging_Specification_v1.0.md | IMP-003 structured error, secret handling and JSON logging contract. Subordinate to documents #1–6; cannot alter ErrorId values, binary format, recovery semantics or support scope |
| docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/ADR-IMP-004_Deterministic_Fixture_Contract_v1.0.md | IMP-004 deterministic ordinary-file Fixture contract. Excludes NWB archives, Writer/Reader, platform metadata, block data and BMR |

### REFERENCE Documents (evidence records, metadata)

| # | Document | Classification | Purpose |
|---|----------|----------------|---------|
| 7 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Requirements_Test_Traceability_Matrix_v1.0.md | GENERATED REFERENCE | Deterministic rendering of the Traceability Registry; do not edit manually |
| 8 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Test_Result_Record_v1.3.md | REFERENCE / CURRENT | Current result record; IMP-003 closes at d08a92b / run 30184529945 |
| 9 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/IMP-000_EVD_Build_Evidence_v1.0.md | REFERENCE / CURRENT FOR IMP-000 | IMP-000 evidence; closed, acceptance met |
| 10 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/IMP-001_EVD_Test_Result_Evidence_v1.2.md | REFERENCE / CURRENT FOR IMP-001 | IMP-001 evidence; CLOSED / PASS / ACCEPTANCE MET at 3bceb34 / run 29684903853 |
| 11 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/IMP-002_EVD_Requirements_Traceability_Evidence_v1.0.md | REFERENCE / CURRENT FOR IMP-002 | IMP-002 evidence; CLOSED / PASS / ACCEPTANCE MET at 8b2a68b / run 29691731514 |
| 12 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/IMP-003_EVD_Structured_Diagnostics_Evidence_v1.0.md | REFERENCE / CURRENT FOR IMP-003 | IMP-003 evidence; CLOSED / PASS / ACCEPTANCE MET at d08a92b / run 30184529945 |
| 13 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Engineering_Document_Set_README_v1.0.md | REFERENCE / CURRENT | Document set README and reading order |
| 14 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Engineering_Document_Set_Manifest_v1.0.md | REFERENCE / CURRENT | Current 20-file SHA-256 inventory; manifest excludes itself |
| 15 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Test_Result_Record_v1.2.md | HISTORICAL / SUPERSEDED | Replaced by Test Result Record v1.3; retains the accepted IMP-002 snapshot |
| 16 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Test_Result_Record_v1.1.md | HISTORICAL / SUPERSEDED | Replaced by Test Result Record v1.2; retains the accepted IMP-001 snapshot |
| 17 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/Nuwa_NWB_Test_Result_Record_v1.0.md | HISTORICAL / SUPERSEDED | Malformed source retained for traceability; contains control bytes and old d550907 state; never cite as current |
| 18 | docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0/IMP-001_EVD_Test_Result_Evidence_v1.0.md | HISTORICAL / SUPERSEDED | Malformed source retained for traceability; stale state; never cite as current |

---

## Current Project State

| Dimension | Status |
|-----------|--------|
| **Gate** | GATE-0 — IN_PROGRESS |
| **IMP-000** (Workspace) | CLOSED / PASS / ACCEPTANCE MET |
| **IMP-001** (Format Registry) | CLOSED / PASS / ACCEPTANCE MET — Generator remediation closed at 3bceb34, run 29684903853, 172/172 on Windows and Ubuntu |
| **IMP-002** (Requirements–Test Traceability Matrix) | CLOSED / PASS / ACCEPTANCE MET — final tested commit 8b2a68b, run 29691731514, 188/188 on Windows and Ubuntu |
| **IMP-003** (Structured Diagnostics) | CLOSED / PASS / ACCEPTANCE MET — final PR head d08a92b, run 30184529945, 195/195 plus Canary 2/2 on Windows and Ubuntu |
| **IMP-004** (Deterministic Fixture Generator) | CLOSED / PASS / ACCEPTANCE MET — commit `f89cad8`, run `30430862143` |
| **IMP-005** (Format 0.x Version Policy) | CLOSED / PASS / ACCEPTANCE MET — commit 718a096, run 30456312056 |
| **IMP-100 and later** | NOT_STARTED / FORBIDDEN UNTIL GATE-0 — must not be initiated before GATE-0 is closed |

### Key Constraints

1. All new storage functionality must follow the NWB Storage Engine architecture: each successful Full or Differential backup produces an immutable, self-describing logical NWB archive. It must not depend on the superseded Repository architecture.

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
- **Task:** IMP-003 structured diagnostics evidence closure
- **Purpose:** Register the traceability authority, generated Matrix, current evidence and actual Gate status
- **Previous update:** 2026-07-19 (IMP-001 Generator Remediation evidence closure)
- **Closure IMP-000:** IMP-000-EVIDENCE-CLOSURE-1 closed IMP-000 as PASS/ACCEPTANCE MET with full CI evidence (e1f1adb, run 29426443433)
- **Closure IMP-001 (v1.0):** IMP-001 evidence closed as PASS/ACCEPTANCE MET at d550907 (CI run 29471690977)
- **Historical reopen checkpoint (v1.1, superseded):** IMP-001 was reopened by Generator Remediation (RIR-005, 2026-07-18). That intermediate snapshot is retained only in the historical EVD and is not a current status source.
- **Closure IMP-001 (v1.2):** Generator remediation closed at 3bceb34 with Code Review APPROVED, Validation PASS (172/172 on Windows and Ubuntu), Recovery Integrity APPROVED and CI run 29684903853 SUCCESS. Current evidence is EVD v1.2 and TRR v1.1. Old malformed v1.0 records are HISTORICAL / SUPERSEDED.
- **Closure IMP-002:** Requirements–test traceability closed at 8b2a68b with final Code Review APPROVED, dual-platform CI 188/188 PASS, Recovery Integrity APPROVED and CI run 29691731514 SUCCESS. Current evidence is IMP-002 EVD v1.0 and TRR v1.2; TRR v1.1 is historical/superseded.
- **Closure IMP-003:** Structured diagnostics closed at PR head d08a92b with independent remediation review APPROVED, final rustfmt delta review APPROVED, dual-platform CI 195/195 plus Canary 2/2 PASS and run 30184529945 SUCCESS. Current evidence is IMP-003 EVD v1.0 and TRR v1.3; TRR v1.2 is historical/superseded.
- **Maintenance:** Update when documents are added, removed, reclassified, or when project gate status changes
