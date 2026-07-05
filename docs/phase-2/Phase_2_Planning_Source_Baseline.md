# Phase 2 — Planning Source Baseline

**Product:** Nüwa Backup (女娲备份)
**Task:** 2.0B — AGENTS.md Contamination Cleanup & Phase 2 Planning Authority Baseline
**Date:** 2026-07-05
**Status:** ACTIVE — Phase 2 planning baseline. Does NOT authorize Phase 2 coding.

---


> **NOTE: UI implementation status in this document has been superseded.**  
> Phase 2 now includes local desktop GUI implementation (egui + eframe, Acronis True Image-like).  
> The statement "GUI implementation not approved" in earlier sections of this document is no longer authoritative.  
> Refer to `Phase_2_UI_Direction_Decision.md` and `Phase_2_PRD.md` for the current UI decisions.

---

## 1. Purpose

This document establishes the authoritative source baseline for Phase 2 planning.
It records which documents were read, what information was extracted, and what
Phase 2 scope is confirmed vs. optional vs. forbidden.

This document does NOT authorize Phase 2 implementation.
Phase 2 coding requires explicit user approval.

---

## 2. Files Actually Read

| # | File Path | Authority Level | Full Read? | Phase 2 Information Extracted |
|:-:|-----------|:---------------:|:----------:|-------------------------------|
| 1 | `AGENTS.md` | AUTHORITATIVE (operational) | Yes (via system context + Get-Content) | Tech stack: Rust CLI. Phase 1 CLOSED. Phase 2 planning allowed, coding not allowed. §30 Planning Rule. §31 Future Phase Boundary. §32 Document Authority Levels |
| 2 | `docs/project/PROJECT_ENGINEERING_MEMORY.md` | AUTHORITATIVE | Yes | Phase 1 CLOSED (PARTIAL). Phase 2 NOT STARTED. Phase boundaries table. Phase 2 forbidden scope |
| 3 | `docs/project/DOCUMENT_INDEX.md` | AUTHORITATIVE | Yes | Document authority levels. Phase 2 "(reserved)". Phase 2 reserved for: scheduler, retention, history, SMB, GUI |
| 4 | `docs/phase-1/Phase_1_Closing_Report.md` | AUTHORITATIVE | Yes | Phase 1 CLOSED. §7 Phase 2 planning allowed/coding not allowed. Recommended Phase 2: scheduler, retention, history, SMB, GUI prototype, CLI output, config |
| 5 | `docs/phase-1/Phase_1_Final_Acceptance_Report.md` | AUTHORITATIVE | Yes | Phase 1 scope, PARTIAL status, 22 tests, "Phase 2 coding requires explicit user approval" |
| 6 | `docs/phase-1/Phase_1_Technical_Baseline.md` | AUTHORITATIVE | Yes | Module structure, data flow, manifest v1.0, error codes, path safety, atomic write, SHA-256 rules |
| 7 | `docs/phase-1/Phase_1_Known_Limitations_and_Risks.md` | AUTHORITATIVE | Yes | Non-Windows space check PARTIAL. Locked file best-effort. Large file manual-only |
| 8 | `docs/phase-1/Phase_1_to_Phase_2_Handoff.md` | AUTHORITATIVE | Yes | §3 Recommended tasks: scheduler, retention, history, SMB, "GUI initial prototype — Simple TUI or minimal desktop UI", config, CLI output. §4 Allowed/NOT Allowed boundaries |
| 9 | `docs/project/02_Development_Plan.md` | REFERENCE | Yes (full) | Phase 2 original plan: compression enhancement, checksum improvement, scheduler, retention, .nwb v0.1 experimental. GUI in Phase 6+ (egui). V2.1 removed GUI from Phase 2 |
| 10 | `docs/project/07_Checklist.md` | REFERENCE | Yes (full) | Phase 2 items: P2-01 scheduler, P2-02 combined schedule, P2-03 retention N copies, P2-04 non-destructive prune, P2-05 (truncated). **No GUI entry in Phase 2** |
| 11 | `docs/phase-0/00_Codex_Working_Guardrails.md` | AUTHORITATIVE (highest) | Yes | Phase locking rules, forbidden list, restore-validation-first, documentation change rules. GUI listed as Phase 2 后期 |
| 12 | `docs/phase-0/09_MVP_Boundary_and_Risk_Correction.md` | AUTHORITATIVE | Yes | MVP scope freeze. GUI explicitly marked "Phase 2". Desktop GUI (egui) listed in Phase 2 original plan then removed in v2.1 |
| 13 | `docs/phase-0/08_Design_Decision_Log.md` | AUTHORITATIVE | Yes | ADL-New-005: .nwb v0.1 experimental Phase 2 中期 (optional). Phase 划分 table: Phase 2 = scheduler + SMB + GUI (egui) |

---

## 3. Phase 2 Confirmed Planning Themes

These are planning themes only. They do not authorize implementation.

| Theme | Source Documents | Notes |
|-------|-----------------|-------|
| Configuration / backup jobs | Handoff §3.7, Closing Report §7.7 | TOML config file, `nuwa init` |
| Retention policy / prune | Handoff §3.2, Checklist P2-03, P2-04 | Keep N backups, keep N days |
| Scheduler planning | Handoff §3.1, Checklist P2-01, P2-02 | Windows Task Scheduler integration |
| Backup history / operation log | Handoff §3.3 | SQLite or log file |
| SMB / UNC destination planning | Handoff §3.4 | UNC path support |
| CLI output improvement | Handoff §3.6 | Progress bar, JSON output |
| Compression behavior review | 02_DevPlan §三, Handoff §5 | zstd multi-level, --compress feature fix |

---

## 4. Phase 2 Technical Design Candidates

These are design candidates for discussion. They are NOT approved for implementation.

| Candidate | Source | Status |
|-----------|--------|:------:|
| Windows Task Scheduler integration | Handoff §3.1, Checklist P2-01 | Planning candidate |
| SQLite for backup history | Handoff §3.3 | Planning candidate |
| TOML configuration format | Handoff §3.7 | Planning candidate |
| JSON CLI output mode | Handoff §3.6 | Planning candidate |
| Progress display (terminal) | Handoff §3.6 | Planning candidate |
| SMB UNC path handling | Handoff §3.4 | Planning candidate |
| zstd compression levels (1-22) | 02_DevPlan §三, Checklist P2-06 | Planning candidate |
| UI/UX form: TUI / Desktop / Web | Handoff §3.5 (TUI / minimal desktop UI) | **Undecided — user decision required** |

---

## 5. Phase 2 Explicitly Not Approved

The following are **not approved** for Phase 2:

| Item | Reason | Source |
|------|--------|--------|
| Web GUI is not approved | No authoritative Phase 2 document authorizes Web GUI. Task 2.0A audit confirmed this | AGENTS.md §30, Task 2.0A |
| FastAPI + Vanilla JS is not approved | Not part of Nüwa Backup tech stack. Contradicts confirmed Rust CLI stack | AGENTS.md Technology Stack Clarification |
| GUI implementation of any form is not approved | UI form (TUI/Desktop/Web) is undecided. **User decision required** | Handoff §3.5 |
| Phase 2 coding is not approved | Requires explicit user approval after Phase 1 acceptance | AGENTS.md §30 |
| .nwb v0.1 is not a Phase 2 candidate unless user formally revises authoritative docs | ADL-New-005 says Phase 2 中期 optional. 02_DevPlan v2.1 removed it. User must confirm | ADL-New-005, 02_DevPlan v2.1 |

---

## 6. Phase 2 Forbidden Scope

The following are forbidden during Phase 2 (belong to Phase 3+):

| Feature | Phase |
|---------|:-----:|
| `.nwb` image format (formal) | Phase 3 |
| VSS snapshot integration | Phase 3 |
| Volume-level backup | Phase 3 |
| Block-level backup | Phase 3 |
| System volume backup / system restore | Phase 4 |
| WinPE recovery media | Phase 4 |
| BCD boot repair | Phase 4 |
| Disk cloning | Phase 5 |
| Differential backup | Phase 6+ |
| Incremental backup | **Excluded (Not planned)** |
| Universal restore / heterogeneous restore | Phase 6+ |
| Driver injection | Phase 6+ |
| AES encryption | Phase 6+ |
| Linux / domestic OS support | Phase 6+ |
| Daemon / system service | Phase 3+ |
| Distributed / enterprise management | **Permanently excluded** |
| Cloud backup / sync / storage | **Permanently excluded** |
| Security suite / antivirus / ransomware | **Permanently excluded** |

---

## 7. Required User Decisions Before Phase 2 Coding

The following decisions must be made by the user before any Phase 2 coding begins:

| # | Decision | Options |
|:-:|----------|---------|
| 1 | Phase 2 first task | Configuration/job system first? Or scheduler first? |
| 2 | SQLite adoption | Should Phase 2 introduce SQLite for backup history? |
| 3 | TOML configuration | Should Phase 2 introduce TOML config? |
| 4 | Scheduler approach | Windows Task Scheduler integration via schtasks.exe or COM? |
| 5 | SMB support | Should Phase 2 include SMB UNC destination? |
| 6 | UI approach | If UI is needed, TUI (ratatui) / Desktop (egui) / Web? |
| 7 | Compression enhancement | Should zstd multi-level compression be in Phase 2? |
| 8 | Compression ordering | Should zstd be deferred to Phase 3? |

---

## 8. Contamination Cleanup Record

During Task 2.0B, the following contamination was found and addressed:

| Contamination | Location | Action Taken |
|--------------|----------|-------------|
| FastAPI + SQLAlchemy + Vanilla JS + Tailwind CSS tech stack | System-level instruction template (NOT in AGENTS.md on disk) | AGENTS.md now contains explicit Technology Stack Clarification stating this is **superseded** and **not part of Nüwa Backup** |
| Web GUI identified as Phase 2 target | Task 2.0A report (first version) | Corrected: Web GUI moved to Optional Candidates. User Decision Required |
| "GUI" vs "TUI" confusion | Handoff §3.5 says "Simple TUI or minimal desktop UI" | Clarified: UI form is undecided. TUI/Desktop/Web all require user decision |

### AGENTS.md Disk File Status

- **AGENTS.md on disk is clean**: No FastAPI, SQLAlchemy, Vanilla JS, Tailwind CSS, React, Vue, or jQuery text exists in the file.
- **Added**: Technology Stack Clarification section explicitly stating Rust CLI is the confirmed stack.
- **Added**: Revision History v5.0 documenting this clarification.
- **Contamination source**: The system-level instruction template block (`<INSTRUCTIONS>` before `--- project-doc ---`) contained the FastAPI/Vanilla JS references, but this is not part of the AGENTS.md file.

---

## 9. Document Authority Summary for Phase 2

| Priority | Document | Authority | Role in Phase 2 |
|:--------:|----------|:---------:|-----------------|
| 1 | `AGENTS.md` (Technology Stack Clarification) | AUTHORITATIVE | Defines confirmed tech stack (Rust CLI) |
| 2 | `00_Codex_Working_Guardrails.md` | AUTHORITATIVE (highest) | Phase locking, scope boundaries |
| 3 | `09_MVP_Boundary_and_Risk_Correction.md` | AUTHORITATIVE | MVP scope freeze, future phase boundaries |
| 4 | `Phase_1_to_Phase_2_Handoff.md` | AUTHORITATIVE | Phase 2 allowed scope, recommended tasks |
| 5 | `Phase_1_Closing_Report.md` | AUTHORITATIVE | Phase 1 close status, Phase 2 readiness |
| 6 | `Phase_1_Final_Acceptance_Report.md` | AUTHORITATIVE | Phase 1 acceptance, quality gates |
| 7 | `Phase_1_Technical_Baseline.md` | AUTHORITATIVE | Module structure, data flow, safety rules |
| 8 | `Phase_1_Known_Limitations_and_Risks.md` | AUTHORITATIVE | Known limitations for Phase 2 planning |
| 9 | `08_Design_Decision_Log.md` | AUTHORITATIVE | Design decisions including Phase 2 scope |
| 10 | `PROJECT_ENGINEERING_MEMORY.md` | AUTHORITATIVE | Long-term project state |
| 11 | `DOCUMENT_INDEX.md` | AUTHORITATIVE | Document navigation |
| 12 | `02_Development_Plan.md` | REFERENCE | Provides context but does not authorize coding |
| 13 | `07_Checklist.md` | REFERENCE | Feature acceptance checklist (reference only) |
| 14 | **This document** (`Phase_2_Planning_Source_Baseline.md`) | AUTHORITATIVE | Phase 2 planning source baseline |

---

## Revision History

| Version | Date | Reason for change |
|---------|------|-------------------|
| v1.0 | 2026-07-05 | Initial creation — Phase 2 planning source baseline after AGENTS.md contamination cleanup |

