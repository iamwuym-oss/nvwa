# Codex Identity — Nüwa Backup (女娲备份)
# Engineering Execution Rules

**注意：** 本文档包含产品身份信息和工程执行规范两部分。如果本文档与 `00_Codex_Working_Guardrails.md`、`09_MVP_Boundary_and_Risk_Correction.md`、`02_Development_Plan.md`、`07_Checklist.md` 或 `08_Design_Decision_Log.md` 冲突，以那些文档为准。

---

## PART I: PRODUCT IDENTITY

## 1. Product Identity

This is a brand-new project.

The product is a **local-first, single-machine backup and disaster recovery product for Windows Workstation, Windows Server, power users, small offices, PC repair shops, edge nodes, and future Linux/domestic OS scenarios.**

The product is inspired by the backup and recovery capabilities of products such as Acronis True Image, but it is **not intended to clone all features** of Acronis True Image.

Key positioning anchors:
- **Not cloud-based** — local-first architecture
- **Not enterprise centralized management** — single-machine focus
- **Not a security suite** — no antivirus, anti-malware, or ransomware protection
- **Staged delivery** — start with file-level CLI, then expand phase by phase

## 2. Target Users

1. Personal Windows users
2. Power users with important local data
3. Freelancers, creators, developers, and small office users
4. PC repair shops and system migration users
5. Users who need to recover from Windows system failure, disk failure, accidental deletion, or disk replacement
6. Windows Server operators (edge nodes, standalone servers)
7. Future: Linux and domestic OS (Kylin, UOS) users

The product is not designed for enterprise centralized backup management.

## 3. Core Product Goal

To help users protect and recover their local systems and data when the OS fails, disks fail, files are deleted, or hardware is replaced.

The product should eventually allow users to:

1. Back up files and folders locally
2. Back up Windows system partitions locally
3. Restore files and folders from local backups
4. Restore system partitions from local images
5. Create bootable recovery media
6. Recover Windows when the operating system cannot boot
7. Clone disks for SSD upgrade or disk replacement

## 4. Strict Product Boundaries

This product must only focus on backup and recovery.

The following features are **permanently out of scope** unless the authoritative product documents are formally revised and the user explicitly approves the scope change:

1. Cloud backup
2. Cloud sync
3. Cloud storage
4. Mobile phone backup
5. Microsoft 365 backup
6. Antivirus
7. Anti-malware
8. Ransomware protection
9. AI threat detection
10. Enterprise centralized management
11. Multi-device cloud dashboard
12. SaaS account system
13. Remote management
14. Security suite functionality

Do not design, implement, or assume these features. They are permanently excluded.

## 5. Engineering Principles

The product must prioritize:

1. Reliability over feature quantity
2. Restore success over backup speed
3. Data integrity over convenience
4. Clear user experience over complex configuration
5. Local-first architecture
6. Simple workflow for non-expert users
7. Safe defaults
8. Explicit warnings before destructive operations
9. Verifiable backup images
10. Recoverability in real disaster scenarios

---

## PART II: AUTHORITY AND CURRENT PHASE

## 6. Authority Documents

Product scope and boundary decisions are governed by the following authoritative Phase-0 documents. If AGENTS.md conflicts with them on scope or boundary matters, those documents take precedence:

| Priority | Document | Purpose |
|----------|----------|---------|
| 1 | `00_Codex_Working_Guardrails.md` | Codex work boundary rules |
| 2 | `09_MVP_Boundary_and_Risk_Correction.md` | MVP scope freeze and risk list |
| 3 | `02_Development_Plan.md` | Task breakdown by phase |
| 4 | `07_Checklist.md` | Feature acceptance checklist by phase |
| 5 | `08_Design_Decision_Log.md` | All confirmed design decisions |

## 7. Current Phase

**Current Phase: Phase 2.5 — Tauri Desktop GUI (CLOSED) + Phase S — Repository Engine (CLOSED)**

Phase 1 (file-level backup CLI), Phase 2 (CLI usability, egui GUI), and Phase 2.5 (Tauri Desktop GUI) are **CLOSED frozen baselines**.
Phase S (Repository Engine — unified storage foundation) is CLOSED. All 13 tasks (S-01 through S-13) complete. Architecture v1.1 baseline frozen.

### Completed Baselines

| Phase | Scope | Status |
|-------|-------|--------|
| Phase 1 | File-level backup/restore CLI (Rust) | CLOSED |
| Phase 2 | CLI usability: config, history, scheduler, SMB, UNC, egui GUI | CLOSED |
| Phase 2.5 | Tauri 2.0 desktop GUI, React frontend, Application Layer | CLOSED |
| Phase S | Repository Engine (unified storage foundation) | CLOSED |

### Phase 1 Implemented Scope (Frozen Baseline)

Phase 1 implemented and is now frozen:

- Rust CLI binary
- File/folder recursive traversal
- Flat-file backup storage (directory + JSON manifest, NOT `.nwb`)
- SHA-256 per-file checksum
- Optional single-thread zstd compression
- Backup / Restore / Verify / List CLI commands
- Restore to original location or alternate location
- Restore-time checksum validation
- Atomic write safety (.tmp → rename)
- Destination space check before backup
- Automated tests

### Historical Phase 1 Forbidden Scope (Reference Only)

During Phase 1 development, the following were forbidden:

- Scheduled backup (daily/weekly/monthly triggers)
- Desktop GUI (egui or any other framework)
- Background daemon / system service
- IPC (ZeroMQ or any other mechanism)
- VSS snapshot integration
- `.nwb` image format (any version)
- Partition-level backup
- Disk-level backup
- System volume backup
- Bootable recovery media (WinPE / Alpine)
- Disk cloning
- Differential backup
- Incremental backup
- AES encryption or any encryption
- XOR parity / erasure coding
- Performance pipeline optimization (500MB/s goals)
- Linux / domestic OS support (Kylin, UOS, LoongArch)
- NFS network target
- SMTP email notifications
- Multi-language / i18n

---

## PART III: ENGINEERING EXECUTION RULES

## 8. Task Intake Rules

Before starting any new task, Codex must read:

1. **AGENTS.md** — This file.
2. **docs/project/PROJECT_ENGINEERING_MEMORY.md** — Project state, phase boundaries, task-before-read checklist.
3. **docs/project/DOCUMENT_INDEX.md** — Document index with authority levels.
4. **docs/phase-1/Phase_1_Closing_Report.md** — Phase 1 closing decision and freeze status.
5. **docs/phase-1/Phase_1_Final_Acceptance_Report.md** — What was implemented and tested in Phase 1.
6. **docs/phase-1/Phase_1_Technical_Baseline.md** — Module structure, data flow, safety rules, error codes.
7. **docs/phase-1/Phase_1_Known_Limitations_and_Risks.md** — All known limitations and risks.
8. **docs/phase-1/Phase_1_to_Phase_2_Handoff.md** — Phase handoff boundary and Phase 2 boundaries.
9. **docs/phase-s/Nuwa_Repository_Engine_Architecture_v1.0.md** — Phase S Architecture Frozen Baseline.
10. **docs/phase-s/Nuwa_Repository_Engine_Architecture_v1.1.md** — Enterprise Readiness Revision (additive to v1.0).
11. **docs/phase-s/Nuwa_Repository_Engine_Implementation_Plan_v1.1.md** — Phase S Implementation Baseline.

After reading, Codex must output and confirm:

1. **Current phase** — Must match section 7 above.
2. **Task goal** — One clear sentence.
3. **Files expected to change** — Exact file paths.
4. **Features explicitly out of scope** — What this task does NOT do.
5. **Tests expected to run** — At minimum the Phase 1 test suite.
6. **Restore validation method** — How restore correctness will be proven.

If a task request exceeds the current phase, Codex must stop, state which future phase it belongs to, and refuse to implement it.

## 9. Task Lifecycle Rules

Every coding task must move through the following states:

1. **PLANNED** — Scope defined, intake check complete
2. **IMPLEMENTED** — Code written
3. **FORMATTED** — `cargo fmt --check` passes
4. **LINTED** — `cargo clippy --all-targets -- -D warnings` passes
5. **TESTED** — `cargo test` passes
6. **RESTORE-VALIDATED** — Restore validation completes successfully (backup/restore tasks only)
7. **REPORTED** — Task report delivered with evidence
8. **DONE** — All states completed

### Rules

- A task may only be marked DONE if all states are completed.
- If formatting fails, status is FAIL.
- If clippy fails, status is FAIL.
- If tests fail, status is FAIL.
- If restore validation fails or is missing, status is PARTIAL or FAIL.
- Codex must not continue to the next feature until the current task reaches DONE or the user explicitly accepts PARTIAL.

## 10. One Task, One Scope Rule

Each task must complete one bounded engineering goal.

### Allowed single-task scopes

- Initialize project skeleton
- Implement one CLI subcommand
- Implement the manifest module
- Implement the checksum module
- Implement restore validation
- Add one category of tests

### Forbidden patterns

- Implementing backup + restore + verify + compression + UI + daemon in one task
- Creating complex abstractions for future features
- Writing `.nwb`, VSS, daemon, or GUI code in Phase 1
- Introducing future-phase code "because it will be needed later"

## 11. Repository Verification Rules

Before every modification, Codex must check:

1. Current working directory
2. Git repository remote (confirm it is the Nüwa Backup repo)
3. Current branch
4. Whether the working tree is clean
5. That the project is Nüwa Backup — NOT CyberBackup, UrBackup, or any other project

If the repository is wrong or the working tree is dirty with unrelated changes, stop and report.

## 12. Phase 1 Code Organization Rules

Phase 1 must use a simple Rust module structure:

```
src/
  main.rs          # program entry
  cli.rs           # CLI argument parsing
  backup.rs        # backup execution
  restore.rs       # restore execution
  verify.rs        # backup verification
  list.rs          # backup point listing
  manifest.rs      # JSON manifest model and read/write
  checksum.rs      # SHA-256 helpers
  storage.rs       # flat-file storage layout and atomic writes
  errors.rs        # error types and user-facing messages

tests/
  backup_restore_tests.rs
```

### Rules

- Do not put all logic in `main.rs`.
- Do not create future modules for `.nwb`, VSS, daemon, WinPE, GUI, disk, volume, Linux, or clone.
- Keep Phase 1 modules small, testable, and focused.
- Do not create abstractions only needed by future phases.

## 13. Phase 1 CLI Contract

### Minimum commands

```
nuwa backup --source <path> --dest <path>
nuwa restore --backup <backup_id_or_path> --dest <path>
nuwa verify --backup <backup_id_or_path>
nuwa list --dest <path>
```

### Optional later within Phase 1

```
nuwa backup --source <path> --dest <path> --compress
nuwa restore --backup <backup_id_or_path> --dest <path> --overwrite
```

### Exit codes

| Code | Meaning |
| ---- | ------- |
| 0 | Success |
| 1 | General failure |
| 2 | Invalid arguments |
| 3 | I/O error |
| 4 | Checksum or verification failure |
| 5 | Restore validation failure |
| 6 | Safety rule violation |
| 7 | Manifest error |

### Output rules

- Human-readable output is acceptable in Phase 1.
- Errors must clearly state what failed and what the user can do next.
- Do not print stack traces to normal users.
- Do not print sensitive information.

## 14. Phase 1 Manifest Schema Rules

Phase 1 uses flat-file storage plus JSON manifest only.

### Minimum manifest fields

```json
{
  "schema_version": "1.0",
  "backup_id": "uuid-or-generated-id",
  "created_at": "utc timestamp",
  "source_root": "original source path",
  "storage_format": "flat-file",
  "compression": {
    "enabled": false,
    "algorithm": null
  },
  "files": [
    {
      "relative_path": "path/from/source",
      "size_bytes": 0,
      "modified_time": "timestamp",
      "sha256": "hex string",
      "stored_path": "relative path in backup storage"
    }
  ],
  "directories": [
    {
      "relative_path": "path/from/source"
    }
  ],
  "summary": {
    "file_count": 0,
    "directory_count": 0,
    "total_bytes": 0
  }
}
```

### Rules

- Always include `schema_version`.
- Never change manifest schema silently.
- Any schema change must be documented.
- `verify` must fail if manifest is missing, malformed, or incompatible.
- Do not store absolute target paths inside file records unless needed.
- Do not store passwords, tokens, or credentials.

## 15. Dependency Rules

### All dependencies must satisfy

1. Has a clear, immediate use in the current task.
2. Not introduced for a future phase.
3. Does not pull in cloud, networking, telemetry, accounts, encryption, GUI, or daemon dependencies.
4. Must be justified in the task report.
5. If the standard library can do it, do not add a crate.

### Phase 1 allowed dependency categories

- CLI argument parsing (e.g., `clap`)
- JSON serialization (e.g., `serde_json`)
- SHA-256 hashing (e.g., `sha2`)
- zstd compression (only when implementing optional compression)
- Temporary directory / test helpers (test dependencies only)

## 16. Test Rules

After every code change, the following commands must be run:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build
```

Enforcement starts from the first project initialization. If any command fails, task status must be FAIL. Do not declare completion.

### Phase 1 required test coverage

| # | Test scenario | Type |
|---|--------------|------|
| 1 | Normal file backup | Unit |
| 2 | Normal file restore | Unit |
| 3 | Backup → restore → SHA-256 consistency | Integration |
| 4 | Empty directories | Unit |
| 5 | Multi-level nested directories | Unit |
| 6 | 1000+ small files | Integration |
| 7 | 10GB+ large file (manual or ignored) | Manual |
| 8 | Chinese filename | Unit |
| 9 | Unicode filename | Unit |
| 10 | Spaces in filename | Unit |
| 11 | Destination disk full | Error path |
| 12 | Destination path not writable | Error path |
| 13 | Source path does not exist | Error path |
| 14 | Source path = destination path | Safety |
| 15 | Backup interrupted mid-way — existing backup preserved | Recovery |
| 16 | Restore target file already exists | Safety |
| 17 | Restore target file locked by another process | Error path |
| 18 | Manifest corrupted → verify must fail | Verification |
| 19 | Data file tampered → verify must fail | Verification |
| 20 | List command shows expected backup points | CLI |

## 17. Test Data Isolation Rules

All automated tests must:

- Use temporary directories (e.g., `std::env::temp_dir()` or `tempfile` crate).
- Never write to real user directories.
- Never write to Desktop, Documents, system root, Program Files, Windows, or user profile root.
- Clean up test data automatically where safe.
- Keep large-file tests marked as ignored/manual unless explicitly requested.
- Simulate failure where possible instead of damaging real files.
- Never require administrator permissions for Phase 1 tests.

## 18. Restore Validation Rules

Codex must never report only "backup success". Phase 1 minimum completion validation:

1. Create a test source directory with known content.
2. Execute backup.
3. Delete or move the original source directory.
4. Execute restore to a new location.
5. Compare SHA-256 of source files against restored files.
6. Confirm manifest matches restored result.
7. Output the validation result.

If restore validation is not complete, the feature is NOT done.

## 19. No Mocked Restore Validation Rule

Restore validation must exercise real backup and restore behavior.

### Rules

- Do not replace restore validation with mocks.
- Do not only test manifest generation.
- Do not only test checksum helper functions.
- A valid restore test must write real files, back them up, restore them, and compare SHA-256.
- Mocking is allowed only for isolated error handling tests, not for final restore validation.

## 20. Definition of Done

A coding task is DONE only when **all** of the following are satisfied:

1. Task scope has not exceeded the current phase boundary.
2. Code is implemented.
3. Relevant tests are added or updated.
4. `cargo fmt --check` passes.
5. `cargo clippy --all-targets -- -D warnings` passes.
6. `cargo test` passes.
7. `cargo build` passes.
8. Backup/restore tasks have completed restore validation.
9. No forbidden-scope features were introduced.
10. Task report is complete.

Otherwise the task status must be PARTIAL or FAIL.

## 21. Failure Handling Rules

If build, test, or validation fails:

1. Do not continue developing new features.
2. Fix the failure first.
3. Explain the failure reason.
4. Re-run the failed command.
5. If the failure cannot be fixed, report current state and blocking issue.
6. Do not say "it should work" or "theoretically it works" without evidence.

## 22. Evidence-Based Reporting Rules

Every task must end with this report structure:

```
Final Status:    PASS / PARTIAL / FAIL
Current Phase:   Phase 1
Scope Implemented:
  - ...
Files Changed:
  - ...
Features Not Implemented (intentionally):
  - ...
Commands Run:
  - cargo fmt --check
  - cargo clippy --all-targets -- -D warnings
  - cargo test
  - cargo build
Test Results:
  - <summary of test output>
Restore Validation Result:
  - <PASS/FAIL + evidence>
Forbidden Scope Check:
  - No forbidden features introduced
Remaining Risks:
  - ...
Next Recommended Task:
  - ...
```

No output, test results, or restore validation evidence → no PASS.

## 23. Traceability Rules

Every Phase 1 coding task must reference:

- Development Plan task ID, if available
- Checklist item ID, if available
- Related acceptance criteria
- Related test case

Task reports must show:

```
Traceability:
  Development Plan: <task id or section>
  Checklist: <checklist id>
  Acceptance Criteria: <summary>
  Tests: <test names>
```

This ensures every code change maps back to the approved documents.

## 24. Risk Register Update Rule

If Codex discovers a new technical risk while coding, it must:

1. Stop expanding scope.
2. Document the risk in the task report.
3. Classify it as Low / Medium / High.
4. Recommend whether to update `09_MVP_Boundary_and_Risk_Correction.md`.
5. Not silently work around the risk with unapproved architecture changes.

## 25. Manual Test Rules

Some Phase 1 tests may be manual or ignored by default:

- 10GB+ large file backup/restore
- Interruption during long backup
- Locked file behavior on Windows
- Disk-full simulation (if hard to automate safely)

### Each manual test must include

- Test goal
- Test environment
- Exact commands
- Expected result
- Actual result
- Restore validation result

Do not claim manual tests passed unless evidence is recorded.

## 26. Git Rules

Codex may commit only when the user explicitly asks for a commit.

Before committing:

1. Review the working tree diff.
2. Confirm no unrelated files are included.
3. Confirm no temporary files, test output, or backup data is staged.
4. Run the full test suite.
5. Use a clear commit message.

### Commit message format

```
<type>: <short summary>

- What changed
- Why it changed
- Tests run
- Restore validation result
```

### Allowed types

- `feat` — New feature
- `fix` — Bug fix
- `test` — Test addition or update
- `docs` — Documentation change
- `refactor` — Code restructuring
- `chore` — Build, CI, tooling

## 27. Security and Safety Rules

Phase 1 does not involve system-level recovery, but must still follow:

1. Never delete user source data.
2. Never overwrite unconfirmed target data.
3. Never write test files to system directories.
4. Never write test data to real user directories.
5. Tests must use temporary directories.
6. Restore-to-original must detect whether the target already exists.
7. All destructive operations must have explicit safety guards.
8. Error messages must be clear, but must not leak sensitive information.

## 28. Documentation Update Rules

If code behavior changes, the following documents must be updated:

1. README or CLI usage documentation
2. Development Plan current task status
3. Checklist corresponding entries
4. Known limitations section
5. Test documentation

Code behavior and documentation must never be inconsistent.


## 29. Phase 1 Baseline Freeze Rule

The Phase 1 file-level backup/restore CLI is now a frozen baseline.

Codex must not rewrite, restructure, or expand the Phase 1 file backup core
unless the user explicitly approves a task that modifies it.

Any change to Phase 1 core must include:
- Reason for change
- Affected files
- Compatibility impact
- Data integrity risk
- Restore validation plan
- Rollback plan

## 30. Phase 2 Planning Rule

Phase 2 planning is allowed.

Phase 2 coding is NOT allowed until the user explicitly approves the start
of Phase 2 implementation.

Planning may include:
- PRD updates
- Task breakdown
- UI/UX discussion
- Scheduling design
- Retention policy design
- Backup history design
- SMB design

Planning must not create Phase 2 production code.

## 31. Future Phase Boundary Rule

Phase 2 must not implement Phase 3/4/5/6+ capabilities.

Specifically forbidden during Phase 2 unless formally approved:
- .nwb format (Phase 3)
- VSS snapshot integration (Phase 3)
- Volume-level backup (Phase 3)
- Disk-level backup (Phase 5)
- System volume backup / system restore (Phase 4)
- WinPE recovery media (Phase 4)
- Disk cloning (Phase 5)
- Universal restore / heterogeneous restore (Future)
- Driver injection (Future)
- Differential backup (Phase 6+)
- Incremental backup (Excluded / Not planned)
- AES encryption (Phase 6+)
- Linux / domestic OS support (Phase 6+)

## 32. Document Authority Levels

- AUTHORITATIVE documents must be followed. If conflicting, higher-priority
  authoritative documents win.
- REFERENCE documents provide context but do not authorize implementation.
- HISTORICAL documents are superseded and must never be used as current
  authority.
- AGENTS.md is the highest-priority operational execution document. It governs how Codex performs work — phase rules, code organization, testing, reporting, and task lifecycle.
- Phase 0 guardrails (00_Codex_Working_Guardrails.md, 09_MVP_Boundary_Risk.md) govern product scope and boundaries. If AGENTS.md conflicts with Phase 0 documents on scope or boundary matters, Phase 0 documents take precedence.

## 33. Phase Closing Rule

Every phase must end with a Phase X Closing task.

A Phase Closing task must:
1. Update PROJECT_ENGINEERING_MEMORY.md with phase status.
2. Update DOCUMENT_INDEX.md with new documents.
3. Create Phase_X_Closing_Report.md describing completed scope.
4. Summarize known limitations for the phase.
5. Freeze the phase baseline (no unauthorized changes to core).
6. State whether next-phase planning is allowed.
7. State whether next-phase coding is allowed.
8. Run all quality gates if code exists.
---

## Revision History

| Version | Date | Reason for change |
|---------|------|-------------------|
| v1.0 | 2026-07-01 | Initial AGENTS.md — product identity + PRD mission |
| v2.0 | 2026-07-05 | Added Authority Documents, Current Phase, and 14 Engineering Execution Rules. Rewrote Phase scope, removed outdated "PRD v1.0" mission. Added restore validation Definition of Done. |
| v3.0 | 2026-07-05 | Broadened product positioning per expert review. Tightened permanent boundary language. Added: Task Lifecycle Rules, Code Organization Rules, CLI Contract, Manifest Schema Rules, Test Data Isolation Rules, No Mocked Restore Validation Rule, Traceability Rules, Risk Register Update Rule, Manual Test Rules. Renumbered sections 8-28. |
| v4.0 | 2026-07-05 | Phase 1 CLOSED and frozen as baseline. Added: Phase 1 Baseline Freeze Rule (§29), Phase 2 Planning Rule (§30), Future Phase Boundary Rule (§31), Document Authority Levels (§32), Phase Closing Rule (§33). Updated required reading list. |
| v5.0 | 2026-07-05 | Added Technology Stack Clarification. Confirmed Rust CLI as the only authorized tech stack. FastAPI/Vanilla JS explicitly excluded. Superseded template text rule added. |
| v5.1 | 2026-07-05 | Added Phase 2 GUI Direction. Confirmed Phase 2 includes local desktop GUI coding (egui+eframe). Acronis True Image-like UI. Clone page as disabled placeholder only. |
| v7.0 | 2026-07-10 | Phase S CLOSED. All 13 tasks complete. Architecture v1.1 frozen. API freeze review completed. |
| v5.3 | 2026-07-10 | Inter-page linkage fixes: Dashboard navigation, VITE_MOCK_DATA=false, delete_backup_set history recording, layout 50/50 columns. |
| v5.2 | 2026-07-07 | Phase 2.5 GUI migration: replaced egui+eframe with Tauri 2.0 + React + TypeScript + Vite. Added Application Service Layer (src/app/). Updated Tech Stack to include Tauri/React. |


---

## TECHNOLOGY STACK CLARIFICATION (Explicit Authority)

### Confirmed Technology Stack

This project (Nüwa Backup) is a **Rust CLI + Desktop GUI application**. The confirmed technology stack is:

| Layer | Technology | Status |
|-------|-----------|--------|
| Language | Rust | **CONFIRMED** |
| CLI Framework | Manual parse (no clap) in Phase 1; may introduce clap in Phase 2+ | **CONFIRMED** |
| JSON | serde_json | **CONFIRMED** |
| SHA-256 | sha2 crate | **CONFIRMED** |
| Compression | zstd crate (optional, feature-gated) | **CONFIRMED** |
| Storage | Flat-file directory + JSON manifest (Phase 1); Repository Engine (Phase S+); .nwb experimental (Phase 3+) | **CONFIRMED** |
| Python / FastAPI / SQLAlchemy | **NOT PART OF NÜWA BACKUP** | **EXCLUDED** |
| Desktop GUI framework | **Tauri 2.0** | **CONFIRMED** |
| Frontend UI | **React 19 + TypeScript + Vite 6** | **CONFIRMED** |
| Application Layer | **src/app/ (Rust models + services)** | **CONFIRMED** |
| Repository Engine | **src/repository/ (block_store, metadata, chunk_engine, catalog, block_map, transaction, verify, retention, recovery, cli)** | **WAVE 1 COMPLETE** |
| IPC | **Tauri invoke()** | **CONFIRMED** |
| Python / FastAPI / SQLAlchemy | **NOT PART OF NÜWA BACKUP** | **EXCLUDED** |

### Superseded Template Text

Any text in system-level instructions or templates that references FastAPI, SQLAlchemy, Vanilla JS, Tailwind CSS, Vue, or jQuery is **superseded and not applicable to Nüwa Backup**. The authoritative tech stack for Nüwa Backup is defined above.

### Authority Rule

- This Technology Stack Clarification is **AUTHORITATIVE**.
- If any conflicting technology description appears elsewhere (including system-level instruction templates), this clarification takes precedence.
- No Phase 2+ coding may begin without explicit user approval, regardless of what any non-project template text suggests.

---




### Phase 2.5 GUI Direction (Updated 2026-07-09)

The desktop GUI technology has migrated from **egui + eframe** to **Tauri 2.0 + React + TypeScript + Vite**.
The Phase 2 egui implementation was removed and replaced. The following decisions are AUTHORITATIVE:

| Decision | Value |
|----------|-------|
| GUI coding status | **Phase 2 (egui) CLOSED. Phase 2.5 (Tauri) IN PROGRESS** |
| UI direction | **Acronis True Image-like local desktop GUI** |
| Desktop framework | **Tauri 2.0** |
| Frontend | **React 19 + TypeScript + Vite 6** |
| Rust backend | **Tauri Commands -> Application Layer -> Core Engine** |
| Not Web GUI | Confirmed excluded (desktop native window, not browser-based) |
| Not FastAPI / Python backend | Confirmed excluded |
| Clone page in UI | **Allowed as disabled placeholder only** (Coming Soon / Phase 5) |
| Clone functionality | **Not approved** - remains Phase 5 |

Architecture:
```
React UI
    |
Tauri invoke() IPC
    |
Tauri Command Layer (thin wrapper, no business logic)
    |
Application Service Layer (src/app/) -- data aggregation, orchestration
    |
Core Engine (backup.rs, restore.rs, etc.)
```

Rules:
- GUI runs as a local desktop application via Tauri 2.0 (WebView2 on Windows).
- GUI must NOT call Phase 3/4/5/6+ capabilities.
- UI code is in `ui/` (React + TypeScript), NOT in `src/gui/` (removed).
- Rust backend code is in `src-tauri/` (Tauri commands) and `src/app/` (Application Layer).
- All UI operations must go through Application Layer; never access Core directly.
- Clone page must show "Disk Clone is planned for Phase 5 and is not available in Phase 2."



### Phase 2.5 Development Rules

**Allowed:**
- React UI (TypeScript) in `ui/src/`
- Tauri commands in `src-tauri/src/commands/`
- Application Service Layer in `src/app/services/`
- API models in `src/app/models/`
- Error types in `src/app/error.rs`
- CSS styles and theme updates in `ui/src/`
- UI component creation/modification in `ui/src/components/`
- All page implementations in `ui/src/pages/`
- Read-only query interfaces in core modules (new pub fn, no logic changes)
- **Adding new Application Services, API models, and Tauri commands** for new pages or features is **allowed and expected**. The architecture rule (UI -> Command -> Service -> Core) is frozen; the service catalog is extensible.

**Forbidden:**
- Modifying frozen core modules: backup.rs, restore.rs, verify.rs, manifest.rs, checksum.rs, storage.rs, prune.rs
- Bypassing Application Layer: UI must never call core modules directly
- VSS snapshot integration (Phase 3)
- .nwb image format implementation (Phase 3)
- Volume-level backup (Phase 3)
- System restore / WinPE recovery media (Phase 4)
- Disk cloning implementation (Phase 5)
- Differential/incremental backup (Phase 6+)
- Encryption (Phase 6+)
- Daemon/system service/IPC (future)
- Cloud backup, enterprise management, multi-device (permanent excluded)

### Phase S — Repository Engine (CLOSED BASELINE)

**Status:** CLOSED (All 13 tasks complete)

**Allowed in Phase S Wave 2+:**
- Chunk Engine (ChunkEngine trait + FixedChunkPolicy)
- Catalog Engine (CatalogEngine trait + SqliteCatalog)
- Block Map Engine (BlockMapEngine trait + SqliteBlockMap)
- Crash Consistency Manager (state machine + transaction journal)
- Verify Engine (three-level verification)
- Retention Engine (logical deletion + orphan candidates)
- Recovery module (repo.db scan-based rebuild)
- Legacy adapter (flat-file read-only)
- Repository CLI (repo init/check/verify/rebuild)

**Forbidden in Phase S:**
- VSS snapshot integration (Phase 3)
- Volume-level backup (Phase 3)
- System restore / WinPE recovery media (Phase 4)
- Disk cloning (Phase 5)
- Global dedup index / reference counting / GC (Phase 6+)
- Encryption implementation (Phase 6+)
- Object storage backend (Enterprise)
- Cloud tiering (Enterprise)
- Small file packing / container block format (future optimization)

**Architecture Compliance (v1.1 Enterprise Readiness):**
- Repository identity (UUID + repository.json) — implemented
- Repository capabilities (compression, encryption, dedup flags) — implemented
- Asset abstraction (asset_id/asset_type fields) — implemented
- Version migration (format_version + min_compatible_version) — implemented
- Block Map logical_address semantics — documented

### Phase 2.5 Current State

**Latest Commit:** `717ba3c` — feat: add backup content browser with three-column restore layout

**T2.5-04D — Backup Content Browser (COMMITTED)**

This task was committed on 2026-07-09. See commit `8ea7355`. The working tree has no further uncommitted changes.

#### Application Services (src/app/services/)

| Service | File | API |
|---------|------|-----|
| backup_service | src/app/services/backup_service.rs (271 lines) | list_jobs, get_job_detail, run_backup |
| config_service | src/app/services/config_service.rs (219 lines) | list/create/update/delete job config |
| dashboard_service | src/app/services/dashboard_service.rs (199 lines) | get_overview |
| file_browser_service | src/app/services/file_browser_service.rs (191 lines) | list_roots, list_directory |
| restore_service | src/app/services/restore_service.rs (264 lines) | list_restore_points, get_preview, execute_restore |

#### UI Page State

| Page | Status | Notes |
|------|--------|-------|
| Dashboard | ✅ Complete | Real data flow from Core through Application Layer |
| Settings | ✅ Complete | Backup Job CRUD via config_service |
| Backup | ✅ Complete | Reads jobs from Settings; run/empty states |
| Restore | ✅ 3-column layout | Plan grouping + BackupTreeView (committed in 717ba3c) |
| History | ✅ Complete | Filterable table, real HistoryDb backend, shows delete_backup_set operations |
| Schedule | ✅ Complete | Full CRUD with enable/disable toggle, Windows Task Scheduler integration |
| Clone | ❌ Disabled | Future phase notice only |

#### Configuration Model

- **JobConfig** (in src/app/models/config_job.rs) is the factual backup job configuration model
- **Do NOT create a BackupPlan or BackupPlanService abstraction** — Settings already provides full CRUD via config_service
- UI components (Backup page, Settings page) read from the same config_service

#### File Browser Policy

- **Do NOT use OS native dialog** (tauri-plugin-dialog was removed in 04C.1)
- Browse button must open **Nüwa In-App FileBrowserModal** (ui/src/components/common/FileBrowserModal.tsx)
- FileBrowserService provides list_roots() and list_directory() for navigation
- Users may also type paths manually
- Restore destination is NOT restricted by a system directory blacklist
- Restore path traversal protection is handled in the Core restore.rs layer

#### Guardrails for Future Codex

The following are common mistakes that must be avoided:

1. Phase 2.5 and Phase S are **CLOSED** frozen baselines. Phase 3 (Volume Backup Foundation) is the next phase for planning.
2. **VITE_MOCK_DATA** must be `false` for real backend testing. When true, all API layers return isolated mock data and inter-page data flow will not work.
3. **delete_backup_set** operations are recorded in History as a new `delete_backup_set` record. The backend reads manifest.json metadata before deleting.
4. **Dashboard navigation** — "Run Backup Now" and "Restore Files" navigate to respective pages via App.tsx -> Dashboard -> HeroCard prop chain.
5. **Do NOT restore egui** direction — it was superseded by Tauri 2.0 + React
6. **Do NOT re-introduce OS native dialog** — FileBrowserModal is the current solution
7. **Do NOT create a BackupPlan abstraction** — JobConfig is the factual model
8. **Do NOT claim mock restore data represents real backend catalog** capability
9. **Do NOT re-create History/Schedule** — they are already implemented and committed
10. **Do NOT bypass the Application Layer** — UI must never call Core Engine modules or SQLite directly
11. **Do NOT modify frozen core modules**: backup.rs, restore.rs, verify.rs, manifest.rs, checksum.rs, storage.rs, prune.rs

**Architecture Rule:**
```
React UI -> Tauri invoke() -> Tauri Command -> Application Service -> Core Engine
```
UI must never directly call Core Engine modules or access SQLite.

**Core Frozen Modules (do not modify):**
| Module | File |
|--------|------|
| Backup engine | src/backup.rs |
| Restore engine | src/restore.rs |
| Verification | src/verify.rs |
| Manifest | src/manifest.rs |
| Checksum | src/checksum.rs |
| Storage | src/storage.rs |
| Prune | src/prune.rs |



### Product Runtime Language Policy

**Effective from Phase 2 Task T2-LANG-01.**

The current product version is **English-only at runtime**.

All product code, CLI output, GUI text, error messages, warning messages, success messages, logs, generated config templates, JSON keys/values, test expected strings, and code comments must be **English**.

Chinese is allowed **only** in:
- Documentation files (docs/)
- Planning materials and reports
- PRD, technical design, and architecture documents
- Task reports and user discussion records

Chinese is **not allowed** in:
- src/
- tests/
- Cargo.toml
- Generated runtime config templates
- CLI runtime output
- GUI runtime strings
- Test assertions for product output

Compliance:
- No coding task may be marked PASS if Chinese characters remain in src/, tests/, Cargo.toml, or runtime-generated product text.
- Use ASCII-safe English in runtime output to avoid Windows PowerShell/console encoding issues.
- Use \"Nuwa Backup\" (without umlaut) in code and runtime output; \"Nüwa Backup / 女娲备份\" may be used in documentation.




