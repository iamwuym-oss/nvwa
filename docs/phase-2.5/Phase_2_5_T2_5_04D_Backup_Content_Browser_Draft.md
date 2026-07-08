# Nüwa Backup — T2.5-04D Backup Content Browser (DRAFT)

**Status:** DRAFT / UNCOMMITTED
**Date:** 2026-07-09
**Parent Commit:** 4962910 — fix: replace native dialog with in-app file browser

---

## 1. Summary

T2.5-04D upgrades the Restore page from a single-column flat list to a **3-column layout** with plan grouping and a file tree browser, and adds the BackupTreeView component that converts flat file lists into an expandable directory tree.

---

## 2. Files Changed

| File | Change |
|------|--------|
| ui/src/pages/Restore.tsx | Major rewrite: single column → 3-column layout |
| ui/src/api/restoreApi.ts | Enhanced mock data: 3 plans, multiple versions, nested directory paths |
| ui/src/components/common/BackupTreeView.tsx | **NEW** — converts RestoreFileEntry[] → directory tree |

---

## 3. What Changed (Restore.tsx)

### Before
- Single-column flat list of all restore points
- Each point shown as an independent card
- Preview replaces the list entirely

### After (3-column)
- **Left column (320px):** Restore points grouped by plan name (jobName)
  - Each plan is collapsible (click header to expand/collapse)
  - Each point shows: timestamp label + "Full Backup" badge
  - Selected point is highlighted
- **Right column (flex: 1):** Detail view for selected point
  - Summary header: plan name, status badge, source path, timestamp + file count + size
  - BackupTreeView showing file contents (fully collapsed by default)
  - RestoreForm at the bottom (destination path, overwrite option, start button)
  - When no point is selected: centered "Select a backup point" empty state
- **All original states preserved:** loading (skeleton), error (ErrorState), empty (EmptyState)

### States
- ✅ Loading — skeleton cards
- ✅ Error — ErrorState with retry
- ✅ Empty — "No Restore Points Available"
- ✅ No selection — "Select a backup point"
- ✅ Selected point — file tree + restore form
- ✅ Restore running — loading state
- ✅ Restore result — success/failure summary

---

## 4. BackupTreeView Component

- Converts flat RestoreFileEntry[] (from get_restore_preview) into a tree
- Directories first, alphabetical sorting
- All nodes start **fully collapsed** — user expands level by level
- Shows file sizes for files, item count for directories
- Supports path selection (highlighted when clicked)
- Empty state: "No files to restore in this backup point."
- **Not** a full enterprise catalog browser — it's a frontend display enhancement using mock data

---

## 5. Mock Data Changes (restoreApi.ts)

The mock data now reflects a more realistic restore scenario:

| Plan | Versions | Source |
|------|:--------:|--------|
| Documents | 3 | C:\Users\Tony\Documents |
| Projects | 2 | C:\Users\Tony\Projects |
| Server Configs | 1 | C:\Configs |

Mock preview files include nested directories: Projects/Source Code/, Photos/Vacation/family/, etc.

---

## 6. Current Limitations

- **Mock data only** — not connected to a real backup catalog backend
- **"Full Backup" badge is hardcoded** — no differential/incremental type data from backend
- **Tree is frontend-only** — backup_id → flat file list → tree conversion is done in the browser
- **No file search/filter** in the tree
- **No multi-select** for partial restore
- **History/Schedule pages still placeholders**

---

## 7. Verification

- `pnpm tsc`: 0 errors ✅
- `pnpm vite build`: pass ✅

---

## 8. Not Yet Committed

This work is **not yet committed**. It exists in the working tree:

```
 M ui/src/pages/Restore.tsx
 M ui/src/api/restoreApi.ts
?? ui/src/components/common/BackupTreeView.tsx
```
