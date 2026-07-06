# Nüwa Backup — Phase 2.5 GUI Technology Migration Decision

**Version:** 1.0
**Date:** 2026-07-06
**Status:** AUTHORITATIVE — supersedes Phase 2 GUI technology decisions

---

## 1. Decision Summary

| Item | Value |
|------|-------|
| Technology | **Tauri 2.0 + React + TypeScript + Vite** |
| Previous technology | egui + eframe (Rust immediate-mode GUI) |
| Decision type | Technology route change |
| Decision ID | ADR-T2.5-001 |

## 2. Rationale

The previous egui/eframe GUI technology was evaluated during Phase 2 development and found insufficient for producing a professional, Acronis-like desktop backup software UI.

| Factor | egui + eframe | Tauri 2.0 + React + TypeScript |
|--------|---------------|-------------------------------|
| Visual quality ceiling | Low (immediate-mode, limited widget set) | Unlimited (CSS/HTML full control) |
| Component reusability | Low (no component model) | High (React component system) |
| Type safety | Compile-time only (Rust) | Compile-time on both sides (Rust + TypeScript) |
| Dashboard/complex layouts | Difficult, manual layout code | Natural, CSS grid/flexbox |
| Ecosystem maturity | Small, niche | Large, mainstream |
| Long-term maintainability | Poor (as UI grows) | Good (structured components) |

## 3. Architecture

`
+--------------------------------------------------+
|               Tauri 2.0 Desktop Window             |
|  +--------------------------+  +-----------------+ |
|  |   WebView2 (Windows)     |  |  Rust Backend   | |
|  |   React + TypeScript     |  |  Tauri Commands  | |
|  |   + HTML/CSS             |  |  invoke backup   | |
|  |                          |  |  restore verify  | |
|  |   Pages:                 |  |  list config     | |
|  |   Dashboard              |  |  history         | |
|  |   Backup                 |  |  scheduler       | |
|  |   Restore                |  |                  | |
|  |   History                |  |  lib.rs API      | |
|  |   Schedule               |  |                  | |
|  |   Settings               |  |                  | |
|  |   Clone (placeholder)    |  |                  | |
|  +--------------------------+  +-----------------+ |
+--------------------------------------------------+
`

- Frontend: React + TypeScript, rendered in OS WebView
- Backend: Rust Tauri commands calling existing 
uwa-backup core library
- Communication: Tauri invoke() IPC, no localhost HTTP server
- CLI and GUI share the same lib.rs core library

## 4. What Changed

### Removed
- src/gui/ — entire egui GUI layer
- src/gui_main.rs — egui binary entry point
- Cargo.toml — egui/eframe dependencies, gui feature, nuwa-gui binary target
- src/lib.rs — #[cfg(feature = "gui")] pub mod gui; export

### Preserved (Core Library Enhancements from T2-07)
- src/config.rs — save(), save_to(), pub config_paths() methods
- src/list.rs — #[derive(Clone)] on BackupPointSummary
- All Phase 1 core modules and CLI

## 5. What DID NOT Change

- Phase 1 core backup/restore/verify/list/manifest/checksum/storage modules
- CLI binary (
uwa) — unchanged, still the same commands and output
- All tests — preserved and passing (75 unit + 19 integration = 94 tests)
- English-only runtime policy
- All product boundaries (no cloud, no enterprise, etc.)

## 6. Non-Goals (Forbidden Scope)

This migration does not introduce:
- Tauri full GUI pages (to be implemented in later T2.5 tasks)
- .nwb image format
- VSS snapshot integration
- Volume/disk backup
- System restore
- WinPE recovery media
- Disk cloning functionality
- Daemon/system service
- Web GUI
- Localhost HTTP server

## 7. Frontend Technology

**React + TypeScript + Vite** is selected as the frontend stack.

Rationale:
- Nüwa Backup is a production desktop application, not a small utility
- Component-based architecture suits the multi-page dashboard layout
- TypeScript provides compile-time safety for IPC data types
- Vite provides fast dev experience and optimized production builds
- The ~200MB Node.js toolchain cost is acceptable for long-term maintainability

## 8. Quality Gates at Migration Time

| Gate | Result |
|------|:------:|
| cargo fmt --check | ✅ PASS |
| cargo clippy --all-targets -- -D warnings | ✅ PASS |
| cargo build | ✅ PASS |
| cargo test (94 tests) | ✅ ALL PASS |
| Old egui residue scan | ✅ Clean — no egui/eframe references remain |
| Core file protection scan | ✅ No core files modified |

## 9. Next Recommended Task

**T2.5-01 — Tauri 2.0 project scaffold + command bridge setup**

