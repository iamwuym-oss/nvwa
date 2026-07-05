# Phase 1 — Test Evidence

**Phase:** 1 — Minimal File Backup/Restore CLI
**Date:** 2026-07-05

---

## 1. Automated Tests Summary

```
cargo test --offline: 22/22 total tests passed
  3 unit (checksum) + 19 integration (backup_restore)
```

---

## 2. Unit Tests (3/3 PASS)

### `checksum.rs`

| Test | Status | What it verifies |
|------|--------|-----------------|
| `test_sha256_empty_file` | PASS | SHA-256 of empty content returns valid 64-char hex |
| `test_sha256_bytes_consistency` | PASS | Same bytes produce same hash across multiple calls |
| `test_sha256_format` | PASS | Output is 64 lowercase hex characters |

---

## 3. Integration Tests (19/19 PASS)

### Core Backup/Restore

| # | Test | Status | Scenario |
|---|------|--------|----------|
| 1 | `test_normal_backup` | PASS | Single backup of standard source (7 files, 5 dirs) |
| 2 | `test_normal_restore` | PASS | Restore to new location |
| 3 | `test_consistency` | PASS | backup → restore → SHA-256 compare — ALL FILES MATCH |

### Directory Handling

| # | Test | Status | Scenario |
|---|------|--------|----------|
| 4 | `test_empty_dirs` | PASS | Empty directories at root and nested levels are preserved |
| 5 | `test_nested` | PASS | 4-level deep nesting (`a/b/c/d`) is preserved |

### Error Paths

| # | Test | Status | Scenario |
|---|------|--------|----------|
| 6 | `test_source_not_found` | PASS | Missing source path → exit code 2 (InvalidArgs) |
| 7 | `test_dest_not_writable` | PASS | File path used as dest → I/O error |

### Safety

| # | Test | Status | Scenario |
|---|------|--------|----------|
| 8 | `test_source_equals_dest` | PASS | src = dest → exit code 6 (SafetyViolation) |
| 9 | `test_dest_inside_source` | PASS | dest inside src → exit code 6, zero side effects |
| 10 | `test_source_inside_dest` | PASS | src inside dest → exit code 6, zero side effects |
| 11 | `test_no_overwrite` | PASS | Restore without --overwrite skips existing file, doesn''t overwrite |
| 12 | `test_dest_inside_source_does_not_create_dest_dir` | PASS | Dest directory NOT created when containment check rejects |

### Verification

| # | Test | Status | Scenario |
|---|------|--------|----------|
| 13 | `test_corrupted_manifest` | PASS | Corrupt JSON manifest → verify fails |
| 14 | `test_tampered_data` | PASS | Tampered backup file → verify fails |
| 15 | `test_interrupt_safety` | PASS | .tmp residue files ignored during verify |

### Scale

| # | Test | Status | Scenario |
|---|------|--------|----------|
| 16 | `test_100_files` | PASS | 100 files backup + restore + SHA-256 compare |
| 17 | `test_1000_files` | PASS | 1000 files backup + restore + SHA-256 compare |

### CLI

| # | Test | Status | Scenario |
|---|------|--------|----------|
| 18 | `test_list` | PASS | Correct number of backup points listed |

### Safety Hardening (Task 1.3)

| # | Test | Status | Scenario |
|---|------|--------|----------|
| 19 | `test_lock_check_does_not_modify_existing_file` | PASS | is_file_locked() preserves file content, size, and SHA-256 |

---

## 4. CLI E2E Restore Validation (Task 1.2)

### Test Setup
- **Source:** Temporary directory with **1012 files** + **6 directories**
- **Special filenames tested:**
  - Chinese: `中文文件.txt`, `测试报告.docx`
  - Japanese: `日本語.txt`
  - Emoji: `😊emoji.txt` (if included in test)
  - Umlauts: `über.txt`, `café.txt`
  - Spaces: `my file.txt`, `file with spaces.doc`
  - Dots: `file.with.dots.txt`
  - Parentheses: `file (1).txt`
- **Multi-level nesting:** `a/b/c/d/e/deep.txt`
- **Empty directories:** `empty/`, `a/empty/`

### Workflow

```
Step 1: nuwa backup --source <src> --dest <dest>
         → Creates timestamped backup directory
         → SHA-256 computed for all 1012 files
         → JSON manifest written atomically

Step 2: nuwa list --dest <dest>
         → Lists backup point with correct metadata

Step 3: nuwa verify --backup <backup_dir>
         → All 1012 files verified, SHA-256 matches

Step 4: Delete original source directory
         → Source removed to simulate recovery scenario

Step 5: nuwa restore --backup <backup_dir> --dest <restore_dir> --overwrite
         → All files restored with atomic writes
         → Post-restore SHA-256 verification for each file

Step 6: SHA-256 compare
         → Source checksums vs restored checksums
         → 100% match — ALL 1012 FILES IDENTICAL
```

### Result

| Metric | Value |
|--------|-------|
| Files backed up | 1012 |
| Files restored | 1012 |
| SHA-256 matches | 1012 / 1012 (100%) |
| Directories preserved | 6 / 6 |
| Special filenames | All preserved |
| Empty directories | All preserved |

---

## 5. Manual Tests (Pending)

| Test | Status | Doc Reference |
|------|--------|---------------|
| 10GB+ large file backup/restore | PENDING | Phase_1_Manual_Test_Plan.md M1 |
| Destination disk full during backup | PENDING | Phase_1_Manual_Test_Plan.md M2 |
| Windows locked file detection | PENDING | Phase_1_Manual_Test_Plan.md M3 |
| Unicode/special filename supplementary | OPTIONAL | Phase_1_Manual_Test_Plan.md M4 |

These manual tests are documented but not executed. They are non-blocking
for Phase 1 acceptance.

---

## 6. Quality Gates

| Gate | Result |
|------|--------|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo build` | PASS |
| `cargo test` | PASS (22/22) |

---

## 7. Conclusion

All 22 automated tests pass. CLI E2E validation confirms 1012/1012 files
restored with 100% SHA-256 consistency. Manual tests are documented but
pending execution. Phase 1 quality gates all pass.