# Nüwa Backup — Phase 2.5 Codex Handoff

**Purpose:** Quick reference for future Codex/AI sessions. Read this first before starting any Phase 2.5 work.

---

## Current Phase

**Phase 2.5 — Tauri Desktop GUI + Application Layer (IN PROGRESS)**

Phase 2.5 is **NOT closed**. History and Schedule pages implemented (committed 0a8ed93).

---

## Latest Commit

`0a8ed93 — feat: add backup content browser with three-column restore layout`

---

## Current Uncommitted Work

T2.5-04D — Restore 3-column layout + BackupTreeView (3 files):
- ui/src/pages/Restore.tsx
- ui/src/api/restoreApi.ts
- ui/src/components/common/BackupTreeView.tsx

---

## Architecture (Do NOT Bypass)

```
React UI → API bridge → Tauri Command (thin) → Application Service → Core Engine
```

- UI must NEVER call Core Engine modules directly
- UI must NEVER access SQLite directly
- All operations go through Application Layer

---

## Key Facts (Avoid Common Mistakes)

| Topic | Truth |
|-------|-------|
| GUI framework | **Tauri 2.0 + React + TypeScript + Vite** — NOT egui (removed) |
| File browser | **Nüwa in-app FileBrowserModal** — NOT OS native dialog (tauri-plugin-dialog was removed) |
| Backup plan model | **JobConfig / config_job.rs** — NOT BackupPlan (no such abstraction exists) |
| Settings | Creates/edits/deletes JobConfigs via config_service |
| Backup page | Reads and runs existing jobs. Depends on Settings-created plans |
| Restore page | 3-column layout with BackupTreeView. Committed in 0a8ed93 |
| History page | **Implemented** — 4 lines |
| Schedule page | **Implemented** — 4 lines |
| Clone page | **Disabled** — "future phase" notice only |
| Backup Content Browser | Frontend-only enhancement using mock data. Not a full catalog browser |
| Core Engine | **Frozen.** backup/restore/verify/manifest/checksum/storage/prune — do not modify |
| Restore path safety | Path traversal protection added (03D.1). No system directory blacklist |
| Phase 2.5 | **IN PROGRESS** — do not declare CLOSED |

---

## Testing Baseline

- Rust: 116 lib tests + 19 config service tests, all pass
- TypeScript: 0 errors
- Vite build: pass

---

## Shortboards

1. History page — implemented
2. Schedule page — implemented
3. Backup Content Browser — mock data only, not real catalog
4. No Tauri command integration tests
5. (None currently — working tree is clean)
