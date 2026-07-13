# Phase 2.5 Codex Handoff

**Product:** Nüwa Backup
**Status:** Phase 2.5 GUI CLOSED | Storage Engine being redesigned

## Current State

- Tauri 2.0 + React + TypeScript desktop GUI is frozen and functional
- Application Layer (src/app/) provides clean service API
- Tauri command layer bridges UI to services
- Backup/restore operations are temporarily unavailable during storage engine redesign

## Architecture

`
React UI
    |
Tauri invoke() IPC
    |
Tauri Command Layer
    |
Application Service Layer (src/app/)
    |
New Storage Engine (under design)
`

## GUI Pages

All GUI pages compile and render but backup/restore operations return "unavailable" until the new storage engine is designed and implemented.

## Next Steps

1. Design new .nwb-based single-file storage engine
2. Implement storage engine in new module
3. Reconnect backup_service and restore_service to new engine
