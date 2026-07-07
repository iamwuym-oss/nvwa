# Nuwa Backup — Phase 2.5 T2.5-01: Tauri 2.0 Scaffold + Command Bridge

**Date:** 2026-07-07
**Status:** DONE / PASS
**Traceability:**
  - Phase 2.5 — GUI Technology Migration
  - ADR: Phase_2_5_Tauri_Migration_Decision.md

---

## 1. Task Goal

Scaffold the Tauri 2.0 desktop GUI project skeleton, establish the Rust command bridge from frontend to the nuwa-backup core library, and verify that the full chain (cargo build, cargo test) passes cleanly.

## 2. Scope Implemented

### Backend (src-tauri/)

| File | Purpose |
|------|---------|
| src-tauri/tauri.conf.json | Window config (1200x800, min 900x600, centered, resizable), frontendDist=../ui/dist, devUrl=localhost:1420 |
| src-tauri/Cargo.toml | 
uwa-tauri crate, depends on 
uwa-backup = { path = ".." }, tauri v2 with tray-icon, serde, serde_json |
| src-tauri/build.rs | Standard tauri-build |
| src-tauri/src/main.rs | Binary entry: 
uwa_tauri_lib::run() |
| src-tauri/src/lib.rs | Tauri commands + AppState + .run() |
| src-tauri/capabilities/default.json | Default WebView capability manifest |
| src-tauri/icons/ | App icons (32x32, 128x128, 128x128@2x, .icns, .ico) |

### Frontend (ui/)

| File | Purpose |
|------|---------|
| ui/package.json | React 19, @tauri-apps/api v2, Vite 6, TypeScript 5 |
| ui/vite.config.ts | Vite config with @vitejs/plugin-react |
| ui/tsconfig.json | TypeScript config |
| ui/index.html | App root HTML |
| ui/src/main.tsx | React DOM entry |
| ui/src/App.tsx | Page router (7 pages), Sidebar + TopBar layout |
| ui/src/components/Sidebar.tsx | Navigation sidebar |
| ui/src/components/TopBar.tsx | Page title bar + version |
| ui/src/pages/Dashboard.tsx | Dashboard stub |
| ui/src/pages/Backup.tsx | Backup page stub |
| ui/src/pages/Restore.tsx | Restore page stub |
| ui/src/pages/History.tsx | History page stub |
| ui/src/pages/Schedule.tsx | Schedule page stub |
| ui/src/pages/Settings.tsx | Settings page stub |
| ui/src/pages/Clone.tsx | Clone placeholder (Phase 5 Coming Soon) |
| ui/src/styles.css | Base styles (dark theme, flex layout) |

### Tauri Commands

| Command | Signature |
|---------|-----------|
| get_version | n get_version() -> String — returns "Nuwa Backup vX.Y.Z (GUI)" |
| list_backup_jobs | n list_backup_jobs() -> Result<Vec<(String, JobConfig)>, String> — reads nuwa.toml config |

## 3. Features NOT Implemented (Intentionally)

- GUI page content (Dashboard/Backup/Restore etc. are stubs only)
- Clone functionality (Clone page shows disabled Phase 5 placeholder only)
- .nwb image format
- VSS snapshot integration
- Volume/disk backup
- System restore
- WinPE recovery media
- Disk cloning
- Daemon/system service
- Tray icon functionality (tray-icon feature added but not wired)
- Cloud backup, sync, enterprise features (permanent boundaries)

## 4. Quality Gates

| Gate | Result |
|------|:------:|
| cargo fmt --check | ✅ PASS |
| cargo clippy --all-targets -- -D warnings | ✅ PASS |
| cargo build | ✅ PASS |
| cargo test (94 tests) | ✅ ALL PASS |
| Forbidden scope audit | ✅ No forbidden features introduced |
| Phase 1 core protection | ✅ Intact — no src/ changes |
| English-only / mojibake | ✅ Clean |

## 5. Test Results

All 94 existing tests pass:
- 75 unit tests
- 19 integration tests
- No new tests added (Tauri scaffold is build-time verification only)

## 6. Forbidden Scope Check

- No cloud, no enterprise, no VSS, no .nwb, no daemon, no encryption introduced
- Clone page shows a disabled placeholder with Phase 5 notice — no clone functionality
- Phase 1 core files untouched

## 7. Remaining Risks

| Risk | Level | Mitigation |
|------|-------|------------|
| npm dependency supply chain | Low | Lock file (package-lock.json) committed |
| Tauri v2 API stability | Low | Pinned to v2 stable |
| Windows WebView2 availability | Medium | Most Windows 10/11 have WebView2 pre-installed; SxS install via bundle on older systems |

## 8. Next Recommended Tasks

- T2.5-02 — GUI Dashboard page with real data display
- T2.5-03 — GUI Backup page with job list and trigger
- T2.5-04 — GUI Restore page with backup point browser
