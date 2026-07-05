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
