# Phase 2.5 UI Surface Audit Checklist

**Objective:** Validate end-user backup and restore workflows in real desktop environment.
**Scope:** User journey validation only.
**Out of Scope:** UI polish, new features, architecture changes, refactoring.
**Estimated Time:** 30–45 minutes.

---

## Preparation

1. Back up or note your existing `~/.nuwa/` config (if any)
2. Delete `~/.nuwa/` to simulate first-time user
3. Run `cd ui && pnpm tsc && vite build && cd .. && cargo tauri dev`
4. Confirm app window opens to Dashboard

---

## 1. First Launch — Directory Initialization

| Check | Expected | Result |
|-------|----------|--------|
| `~/.nuwa/` directory created automatically | ✅ Created | ☐ |
| Dashboard renders without crash/blank page | ✅ Renders | ☐ |
| No React console errors (open DevTools: F12) | ✅ No errors | ☐ |
| Dashboard shows empty state guidance | ✅ Guidance visible | ☐ |

## 2. Create Backup Job

| Check | Expected | Result |
|-------|----------|--------|
| Navigate to Settings → "Create Plan" | ✅ Form opens | ☐ |
| Fill: name="TestData", source=`C:\test_data`, dest=`D:\nuwa_test_backup` | ✅ Saved | ☐ |
| Create `C:\test_data` with 10+ small files + 1 with Chinese filename | ✅ Created | ☐ |
| Verify job appears in Settings list | ✅ Visible | ☐ |
| **Safety:** Try source=C:\test_data, dest=C:\test_data | ❌ Should be rejected | ☐ |

## 3. Execute Backup

| Check | Expected | Result |
|-------|----------|--------|
| Navigate to Backup page | ✅ Loads | ☐ |
| Select "TestData" job, click Run | ✅ Starts | ☐ |
| Backup completes without error | ✅ Success | ☐ |
| Dashboard updated (protection status, recent activity) | ✅ Updated | ☐ |

## 4. History

| Check | Expected | Result |
|-------|----------|--------|
| Navigate to History page | ✅ Loads | ☐ |
| Table shows the backup operation | ✅ Visible | ☐ |
| Operation type, status, timestamp all correct | ✅ Correct | ☐ |
| **Consistency:** Dashboard shows matching latest activity | ✅ Matches | ☐ |

## 5. Restore Browse

| Check | Expected | Result |
|-------|----------|--------|
| Navigate to Restore page | ✅ 3-column layout | ☐ |
| Left column: "TestData" plan visible | ✅ Visible | ☐ |
| Middle column: backup point visible with timestamp | ✅ Visible | ☐ |
| Right column: file tree expands, shows all backed-up files | ✅ Tree ok | ☐ |
| **Integrity:** Files in tree match what was backed up | ✅ Match | ☐ |

## 6. Execute Restore + SHA-256 Verify

| Check | Expected | Result |
|-------|----------|--------|
| Delete original C:\test_data | ✅ Deleted | ☐ |
| In Restore: select destination (e.g., C:\test_data_restored) | ✅ Set | ☐ |
| Click "Start Restore" | ✅ Completes | ☐ |
| **SHA-256 Verify (manual):** `certutil -hashfile C:\test_data_restored\<file> SHA256` vs original | ✅ Match | ☐ |
| **Critical:** 100% file content integrity | ✅ PASS | ☐ |

## 7. Schedule Config

| Check | Expected | Result |
|-------|----------|--------|
| Navigate to Schedule page | ✅ Loads | ☐ |
| Create: name="Nightly", Daily, 02:00 | ✅ Saved | ☐ |
| Edit: change name or time | ✅ Updated | ☐ |
| Toggle enable/disable | ✅ Toggles | ☐ |
| Delete schedule | ✅ Removed | ☐ |

## 8. Delete Job — Data Protection

| Check | Expected | Result |
|-------|----------|--------|
| Settings → Delete "TestData" job | ✅ Deleted | ☐ |
| **Verify backup data still exists** in D:\nuwa_test_backup | ✅ Preserved | ☐ |
| **Verify history still accessible** from History page | ☐ Check | ☐ |

## 9. Exception Handling

| Check | Expected | Result |
|-------|----------|--------|
| Settings: create job with non-existing source path, try Run | ❌ Error shown, no crash | ☐ |
| Settings: corrupt ~/.nuwa/nuwa.toml manually, open app | ❌ Graceful handling | ☐ |
| Restore: try restoring to a read-only USB / protected path | ❌ Error shown | ☐ |

## 10. Restart Persistence

| Check | Expected | Result |
|-------|----------|--------|
| Close Tauri app completely | ✅ Closed | ☐ |
| Re-launch with `cargo tauri dev` | ✅ Opens | ☐ |
| Settings page: "TestData" job still listed? | ✅ Still there | ☐ |
| History page: backup record still visible? | ✅ Still there | ☐ |
| Dashboard: protection status still correct? | ✅ Correct | ☐ |
| Backup data on disk still intact? | ✅ Intact | ☐ |

---

## Summary

| # | Item | PASS / FAIL |
|---|------|-------------|
| 1 | First Launch | ☐ |
| 2 | Create Backup Job + Safety | ☐ |
| 3 | Execute Backup | ☐ |
| 4 | History | ☐ |
| 5 | Restore Browse | ☐ |
| 6 | Execute Restore + SHA-256 | ☐ |
| 7 | Schedule Config | ☐ |
| 8 | Delete Job — Data Protection | ☐ |
| 9 | Exception Handling | ☐ |
| 10 | Restart Persistence | ☐ |

**Blocking Issues Found:** (list if any)

**Final Verdict:** ☐ PASS / ☐ PASS WITH NOTES / ☐ BLOCKERS FOUND