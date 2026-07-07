# Nüwa Backup (女娲备份) — Product Requirements Document

**Nüwa Backup (女娲备份) — PRD v2.0**

| Field | Value |
|---|---|
| **Document Title** | Personal Windows Backup & Recovery Product Requirements Document |
| **Version** | v2.2（Phase mapping aligned） |
| **Status** | Revised — 阶段边界已收敛 |
| **Product Name** | Nüwa Backup (女娲备份) |
| **Target Platform** | Windows Workstation (Windows 7 SP1 ~ Windows 11) + Windows Server (2008 R2 ~ 2025) |
| **Product Type** | Local backup and disaster recovery software |
| **Date** | 2026-07-05（专家审查后修订） |
| **Author** | Codex |

---

## 1. Product Background

### 1.1 Why Personal Windows Users Need Local Backup

Personal Windows users store valuable data directly on their local disks: family photos, financial documents, work projects, development code, creative content, and years of personal records. Unlike enterprise environments with dedicated backup teams, most personal users have no reliable protection for this data.

### 1.2 Common Failure Scenarios

| Scenario | Impact |
|---|---|
| Windows cannot boot | Entire system inaccessible; data trapped on disk |
| Disk failure (HDD/SSD) | Partial or total data loss |
| Accidental file deletion | Lost documents, photos, projects |
| System corruption | Time-consuming OS reinstall, potential data loss |
| Malware or ransomware damage | Files encrypted or destroyed **(note: we do not design security protection features)** |
| Failed Windows update | System unstable or unbootable |
| SSD upgrade or disk replacement | Need to migrate data safely |
| Power failure during write | File system corruption |

### 1.3 Why Local-First and Cloud-Free

- Many personal users have limited or unreliable internet bandwidth.
- Backup data volume (multiple terabytes) makes cloud backup impractical.
- Privacy-sensitive users do not want personal files stored on third-party servers.
- Local backup is predictable: no subscription fees, no data caps, no vendor lock-in.
- Local restore is fast: no re-download from cloud during disaster recovery.
- The product solves real problems for users who already own external hard drives or secondary disks.

---

## 2. Product Positioning

**Personal Windows Backup & Recovery** is a **local-only backup and disaster recovery tool** for personal Windows workstation users.

It is explicitly **not**:

- A cloud backup product
- A cybersecurity product
- An antivirus product
- An enterprise backup platform
- A centralized management system
- A SaaS or subscription cloud service

The product's sole purpose is to help users **protect and recover their local system and data** using **local storage destinations** (internal secondary disks, external USB disks, external SSDs, NAS / SMB shares).

---

## 3. Target Users

### 3.1 Personal Windows Users

| Attribute | Description |
|---|---|
| Typical data | Family photos, personal documents, music, videos |
| Typical risk | Accidental deletion, disk failure |
| Expected value | Simple, set-and-forget file protection |

### 3.2 Power Users with Important Local Files

| Attribute | Description |
|---|---|
| Typical data | Large collections of documents, media, research data |
| Typical risk | Disk failure, file corruption |
| Expected value | Reliable scheduled backups, version history |

### 3.3 Freelancers and Content Creators

| Attribute | Description |
|---|---|
| Typical data | Design projects, video edits, code repositories, client files |
| Typical risk | Work loss = money loss; disk failure = missed deadlines |
| Expected value | Fast restore, backup verification, peace of mind |

### 3.4 Developers and Technical Users

| Attribute | Description |
|---|---|
| Typical data | Source code, development environments, databases, configurations |
| Typical risk | System corruption after update, disk upgrade migration |
| Expected value | System image backup, disk cloning, bootable recovery |

### 3.5 PC Repair Shops and System Migration Users

| Attribute | Description |
|---|---|
| Typical data | Multiple client machines, system images, replacement disks |
| Typical risk | Data loss during migration, incompatible recovery |
| Expected value | Disk cloning, partition restore, bootable media for offline recovery |

### 3.6 Small Office Users without Enterprise IT

| Attribute | Description |
|---|---|
| Typical data | Business documents, spreadsheets, email archives |
| Typical risk | No IT support, no backup system in place |
| Expected value | Simple setup, scheduled backup, easy restore |

---

## 4. Core User Problems

| ID | Problem |
|---|---|
| P-01 | Important files are lost or deleted with no way to recover |
| P-02 | Windows fails to boot and user cannot access data |
| P-03 | System disk fails — all data and OS are gone |
| P-04 | User needs to migrate from HDD to SSD but has no safe migration tool |
| P-05 | A bad Windows update causes system instability or boot failure |
| P-06 | User does not understand complex backup terminology (full, incremental, differential, chain) |
| P-07 | User has no confidence that backups are actually recoverable until disaster strikes |
| P-08 | Restore process is too complex or requires expert knowledge |
| P-09 | Backup process slows down the computer or interrupts work |

---

## 5. Product Goals

| Goal ID | Goal | Scope |
|---|---|---|
| G-01 | Provide simple local file and folder backup | Phase 1A |
| G-02 | Provide reliable local file and folder restore | Phase 1A |
| G-03 | Support manual backup only in initial release | Phase 1A |
| G-04 | Provide basic backup verification (catalog + per-file checksum) | Phase 1A |
| G-05 | Support scheduled backup with minimal user intervention | Phase 1B |
| G-06 | Support common Windows folder quick-select | Phase 1B |
| G-07 | Support restore to original location | Phase 1B |
| G-08 | Provide system image backup for full Windows recovery | V1.5 |
| G-09 | Provide partition-level backup and restore | V1.5 |
| G-10 | Provide bootable recovery media for non-bootable systems | V2 |
| G-11 | Provide disk cloning for HDD-to-SSD migration | V2 |
| G-12 | Ensure restore-oriented design — every backup has a defined restore path | All |
| G-13 | Implement safe handling of all destructive operations (confirmations, warnings, validation) | All |
| G-14 | Provide clear user workflow — no backup expertise required | All |
| G-15 | Keep all data local — no cloud dependency, no internet requirement | All |

---

## 6. Non-Goals

The following features are **explicitly out of scope** for this product (all phases):

| NG ID | Feature |
|---|---|
| NG-01 | Cloud backup — no storage on cloud servers |
| NG-02 | Cloud sync — no file sync across devices |
| NG-03 | Cloud storage — no built-in cloud storage service |
| NG-04 | Mobile phone backup — no iOS or Android backup |
| NG-05 | Microsoft 365 backup — no Exchange, SharePoint, Teams backup |
| NG-06 | Antivirus — no virus scanning engine |
| NG-07 | Malware scanning — no malware detection |
| NG-08 | Ransomware protection — no real-time ransomware detection |
| NG-09 | AI threat detection — no machine learning security features |
| NG-10 | Enterprise centralized management — no management console |
| NG-11 | Remote management — no remote monitoring or administration |
| NG-12 | SaaS account system — no user accounts |
| NG-13 | Multi-tenant management — no tenant separation |
| NG-14 | Enterprise RBAC — no role-based access control |
| NG-15 | Security suite features — no security product bundling |

---

## 7. Product Scope Overview

> **Note — Historical Phase Mapping (Updated 2026-07-07)**
> The phase numbering below reflects the original product roadmap (Phase 1A/1B → Phase 2 → Phase 3 → Phase 4).
> **Current development phase is Phase 2.5 — Tauri Desktop GUI + Application Layer.**
> See docs/phase-2.5/Phase_2_5_Closing_Report.md for the current phase status.
>
> | Original Roadmap | Current Status |
> |---|---|
> | Phase 1A — Minimal Backup CLI | ✅ **CLOSED** — Implemented as Phase 1 |
> | Phase 1B — Usable File Backup | ✅ **CLOSED** — Implemented as Phase 2 |
> | Phase 2 — Partition/System Image | 🔄 **Rescheduled** — Now Phase 3 |
> | **Phase 2.5 — Desktop GUI + App Layer** | 🟢 **ACTIVE** — Tauri 2.0 + React (replaced egui) |
> | Phase 3 — Bootable Recovery Media | 📅 **Future** — Phase 4 in new numbering |
> | Phase 4 — Disk Cloning | 📅 **Future** — Phase 5 in new numbering |

### 7.1 Phase 1A — Minimal Viable Backup Loop

Phase 1A is the **smallest reliable backup/restore loop**. Nothing ships until this loop is proven.

| FR ID | Requirement | Priority |
|---|---|---|
| FR-1A-01 | User can select local folders for backup | P0 |
| FR-1A-02 | User can select backup destination: internal secondary disk or external USB disk | P0 |
| FR-1A-03 | User can initiate a manual backup on demand | P0 |
| FR-1A-04 | User can restore selected files or folders to an alternate location | P0 |
| FR-1A-05 | System builds a basic backup catalog (file list, timestamps, sizes, checksums) | P0 |
| FR-1A-06 | System records basic operation logs with timestamps and results | P0 |
| FR-1A-07 | System performs basic checksum verification after each backup run | P0 |
| FR-1A-08 | System provides clear success or failure result for backup and restore | P0 |
| FR-1A-09 | System detects when backup destination is unavailable | P1 |
| FR-1A-10 | System detects insufficient disk space before starting backup | P1 |

### 7.2 Phase 1B — Usable File Backup

Phase 1B adds usability, scheduling, and completeness to the Phase 1A loop.

| FR ID | Requirement | Priority |
|---|---|---|
| FR-1B-01 | User can quick-select common Windows user folders (Desktop, Documents, Pictures, Videos, Downloads) | P0 |
| FR-1B-02 | User can configure scheduled backup (daily, weekly, custom interval) | P0 |
| FR-1B-03 | User can restore files to the original location | P0 |
| FR-1B-04 | User can view backup history (list of completed backup runs with versions) | P1 |
| FR-1B-05 | System supports configurable retention policy (number of versions to keep) | P1 |
| FR-1B-06 | User can select a network backup destination: NAS or SMB shared folder | P1 |
| FR-1B-07 | System provides improved UI workflow for all operations | P1 |
| FR-1B-08 | System handles file conflicts during restore (overwrite / keep both / skip) | P1 |

### 7.3 Phase 2 — Partition / System Image Backup (V1.5)

Phase 2 adds partition-level image backup. **System partition restore for a non-bootable Windows system requires WinPE or another offline recovery environment — this belongs to Phase 3, not Phase 2.**

| FR ID | Requirement | Priority |
|---|---|---|
| FR-2-01 | System detects Windows disks and partitions (physical layout) | P0 |
| FR-2-02 | System identifies system-related partitions: EFI, MSR, Windows, Recovery | P0 |
| FR-2-03 | User can back up a selected NTFS partition (non-system or system) | P0 |
| FR-2-04 | System uses Windows VSS for online consistent volume snapshot | P0 |
| FR-2-05 | System compresses partition image backup | P1 |
| FR-2-06 | System computes and stores checksums for image backup verification | P1 |
| FR-2-07 | User can restore a **non-system** partition from backup image (within Windows, when safe) | P0 |
| FR-2-08 | System detects partition size compatibility before restore | P1 |
| FR-2-09 | User can view partition backup history and logs | P1 |

**Important boundary**: V1.5 does not include full system disk recovery. System partition restore for a non-bootable system requires the bootable recovery media from Phase 3.

### 7.4 Phase 3 — Bootable Recovery Media + Full System Recovery (V2)

Phase 3 enables disaster recovery when Windows cannot boot.

| FR ID | Requirement | Priority |
|---|---|---|
| FR-3-01 | User can create a bootable WinPE recovery USB drive | P0 |
| FR-3-02 | Bootable environment detects local disks and backup images | P0 |
| FR-3-03 | Bootable environment supports system image restore | P0 |
| FR-3-04 | Bootable environment supports partition restore | P0 |
| FR-3-05 | Bootable environment supports basic network access (SMB) | P1 |
| FR-3-06 | Bootable environment supports boot configuration repair (BCD) | P1 |

### 7.5 Phase 4 — Disk Cloning (V2)

Phase 4 adds disk migration capability. **Do not implement disk cloning before image backup and recovery media design are validated.**

| FR ID | Requirement | Priority |
|---|---|---|
| FR-4-01 | User can clone a source disk to a target disk | P0 |
| FR-4-02 | Disk cloning supports HDD-to-SSD migration | P0 |
| FR-4-03 | System validates cloned disk bootability after clone | P1 |
| FR-4-04 | Target disk warning + double confirmation before destructive clone | P0 |

### 7.6 Future Scope (Phase 5+)

| FR ID | Requirement | Priority |
|---|---|---|
| FR-FUT-01 | Incremental block-level system backup | Future |
| FR-FUT-02 | Differential backup | Future |
| FR-FUT-03 | Image mounting (browse backup as virtual disk) | Future |
| FR-FUT-04 | Advanced retention policies (GFS: Grandfather-Father-Son) | Future |
| FR-FUT-05 | Hardware-independent restore (restore to dissimilar hardware) | Future |
| FR-FUT-06 | BitLocker-encrypted volume backup and restore | Future |
| FR-FUT-07 | ReFS partition support | Future |
| FR-FUT-08 | Email notification for backup results | Future |
| FR-FUT-09 | Backup image export and import for offline transport | Future |

---

## 8. Functional Requirements Detail

### 8.1 Backup Management

#### Backup Job Creation (Phase 1A)

- User creates named backup jobs.
- Each job has:
  - **Source selection**: folders (file/folder picker).
  - **Destination selection**: local path (internal secondary disk or external USB disk).
  - **Schedule**: manual only (Phase 1A); scheduled options added in Phase 1B.
  - **Retention**: not configurable in Phase 1A; added in Phase 1B.
- System validates destination accessibility and free space before saving job.
- **Design Rationale**: Validate destination accessibility and free space before writing any data, to prevent the risk of data inconsistency from a mid-backup failure.

#### Backup Execution (Phase 1A)

- On manual trigger, system performs:
  1. Scan selected folders recursively for files.
  2. Compare with previous backup catalog (if exists) for change detection.
  3. Copy new/changed files to destination with metadata.
  4. Build backup catalog (file list, timestamps, sizes, per-file content checksums).
  5. Validate catalog checksum after completion.
  6. Record backup run log.
- On failure:
  - Partial backup saves what was copied.
  - Error is recorded in log.
  - User is notified in UI.
- On destination unavailable:
  - Backup is skipped, error logged.
  - Next scheduled attempt proceeds normally (Phase 1B).

#### Scheduled Backup (Phase 1B)

- Schedule is defined per job: daily, weekly, custom (e.g., every 6 hours).
- Scheduler runs as Windows scheduled task or service.
- Missed schedules due to system shutdown run on next startup (configurable).

#### File-Level Incremental Backup Semantics (Phase 1A+)

This section defines how the system handles file versioning across multiple backup runs.

**Core Design Principle**: Users must not need to understand incremental chain concepts. Every backup version presents a complete, restorable file tree.

1. **Each BackupRun creates one BackupVersion**: Every backup execution, whether manual or scheduled, produces a distinct version entry in the backup history.
2. **Each BackupVersion presents a complete restorable file tree**: The UI and restore workflow always show every version as a self-contained snapshot. The user never sees "full" vs "incremental" labels.
3. **Internal optimization — avoid copying unchanged files**: The engine compares the current file state against the previous version. Files that have not changed (by size, modification time, and optionally checksum) are not physically copied again. The backup catalog references the existing file data from the previous version.
4. **Deleted files must be represented correctly**: If a file existed in version N but is missing in version N+1 (because the user deleted it), the catalog for version N+1 must reflect that the file is absent. The file data still exists in version N for restore purposes.
5. **Renamed files**: Initially, renamed files are treated as delete + add. The catalog records the deletion under the old name and the addition under the new name. Future optimization may detect renames via heuristics.
6. **Change detection criteria**:
    - Primary: file size + last modification timestamp.
    - Secondary (optional, configurable): partial or full file content checksum for files where size and timestamp match but content may differ.
7. **Restore simplicity**: When restoring from a specific version, the system resolves the full file list by walking the version chain internally. The user selects one version and sees the complete file tree as it existed at that point in time.
8. **Chain integrity**: Each version records its parent version ID. The chain is validated before each new backup run. If a gap or corruption is detected, the system warns the user and starts a fresh baseline.

#### Backup Verification (Phase 1A+)

**Design Rationale**: Metadata-only validation is insufficient. Backup verification must detect data corruption and ensure data integrity during restore.

- **Per-file content checksum**: Each file copied to the backup destination has its content checksum computed (SHA-256 or similar). The checksum is stored in the backup catalog alongside the file entry.
- **Catalog checksum**: The backup catalog file itself has a checksum recorded at the end of each backup run. This detects catalog corruption.
- **Per-file metadata verification**: File size and last modification timestamp are recorded in the catalog and verified on demand.
- **Verification status per BackupVersion**: Each version maintains a verification status field:
  - `unverified` — backup completed, no explicit verification run yet.
  - `passed` — all checksums match.
  - `failed` — one or more checksums do not match.
  - `suspect` — partial or interrupted backup; some files may be missing.
- **Restore-time checksum validation**: When files are restored, the system recomputes the content checksum of each restored file and compares it against the catalog entry. Mismatches are reported immediately.
- **Explicit user-initiated verification**: The user can trigger a full verification of any backup version. The system reads all files from the backup destination, recomputes checksums, and reports the result.
- **Warning on suspect versions**: If a BackupVersion is marked `incomplete`, `suspect`, or verification `failed`, the UI displays a clear warning before allowing restore from that version.

### 8.2 Restore Management

#### File Restore — Alternate Location (Phase 1A)

- User selects a backup version.
- User browses or searches files in the backup catalog.
- User selects files/folders to restore.
- User chooses a target directory (alternate location — **not** the original path in Phase 1A).
- System restores files from backup to destination.
- System validates each restored file by comparing content checksum against catalog.
- Restore result (success/partial/fail with per-file details) is recorded in log.

#### File Restore — Original Location (Phase 1B)

- Same workflow as Phase 1A, plus:
  - User can choose to restore to the original file path.
  - Conflict handling: prompt user with options — Overwrite / Keep both / Skip.
  - System preserves original file timestamps and attributes where possible.

#### Restore from Alternate Backup Location (Phase 1B)

- If original backup destination is unavailable, user can manually point to the backup storage location.
- System detects backup catalog and presents available restore points.

### 8.3 Partition / System Backup (Phase 2)

#### Disk and Partition Discovery (Phase 2)

- System enumerates physical disks and partitions via Windows API.
- Disk information includes: disk number, size, partition style (GPT/MBR), model.
- Partition information includes: partition type (EFI, MSR, Basic, Recovery), file system, size, used space.
- **Design Rationale**: Both GPT and MBR partition tables must be detected because Windows 10/11 systems use a mix of both formats.

#### Partition Image Backup (Phase 2)

- System reads partition block-by-block (used sectors only, with intelligent NTFS parsing).
- Image data is compressed and stored with block-level checksums.
- Backup metadata records original partition geometry, volume label, and file system type.
- **Design Rationale**: Block-level backup must record complete partition geometry information to correctly reconstruct the partition structure during restore.

#### VSS Integration (Phase 2)

- System creates VSS snapshot before partition backup.
- Backup reads from snapshot volume for consistent data.
- Snapshot is released immediately after backup completes.
- If VSS fails, the backup is aborted (not silently continued without consistency).

#### Non-System Partition Restore (Phase 2)

- User selects a partition backup image.
- User selects target partition (or unallocated space).
- System validates: target size >= image used size, target is not the system partition.
- User confirms destructive operation.
- System restores partition data from image.
- **Boundary**: System partition restore is NOT in Phase 2. It requires offline WinPE environment (Phase 3).

### 8.4 Bootable Recovery Media (Phase 3)

#### Media Creation (Phase 3)

- User inserts USB drive (minimum 8 GB recommended).
- System formats USB drive (FAT32 with UEFI support).
- System copies WinPE image with recovery application.
- Recovery application is a minimal Windows-based tool for:
  - Disk discovery.
  - Backup image detection (local disks, USB, SMB).
  - Partition restore.
  - System image restore.
  - BCD repair.

#### Recovery Workflow (Phase 3)

- User boots from USB.
- Recovery application launches automatically.
- User selects backup image location (local disk, USB, SMB).
- System scans for valid backup images.
- User selects a restore point.
- User selects target disk/partition.
- System validates target compatibility.
- User confirms destructive operation.
- System performs restore.
- BCD repair performed if needed.
- System reports result, prompts to remove USB and reboot.

### 8.5 Disk Cloning (Phase 4)

#### Clone Workflow (Phase 4)

- User selects source disk.
- User selects target disk (must be same size or larger, or same-or-larger usable capacity).
- System displays source and target disk details with warnings.
- User confirms:
  - Target disk will be completely overwritten.
  - All data on target will be lost.
- System performs bit-for-bit or intelligent sector copy.
- System adjusts partition layout for target disk size if needed.
- System validates target bootability.

### 8.6 Logs and Diagnostics (All)

| Feature | Scope |
|---|---|
| Backup operation logs | Phase 1A |
| Restore operation logs | Phase 1A |
| Verification logs | Phase 1A |
| Error details with timestamps | Phase 1A |
| Export diagnostics package (logs + system info) | Phase 1B |

---

## 9. Non-Functional Requirements

| NFR ID | Requirement | Scope |
|---|---|---|
| NFR-01 | **Reliability**: Backup must complete or report a clear failure. Partial backups must be detectable. | All |
| NFR-02 | **Data integrity**: Per-file content checksums computed on backup and verified on restore. | All |
| NFR-03 | **Restore success**: Every backup type must have a defined, validated restore path. Do not claim backup success unless restore validation exists. | All |
| NFR-04 | **Performance**: Backup should not saturate system resources. Configurable throttle. | Phase 1B |
| NFR-05 | **Usability**: A non-technical user must be able to perform first backup within 5 minutes. | Phase 1A |
| NFR-06 | **Safety**: Every destructive operation requires explicit confirmation with full disclosure of impact. | All |
| NFR-07 | **Compatibility**: Windows 7 SP1 ~ Windows 11, Windows Server 2008 R2 ~ 2025; NTFS, GPT/MBR, UEFI/BIOS. | All |
| NFR-08 | **Privacy**: All data stays local. No telemetry of file names, paths, or content. | All |
| NFR-09 | **Error handling**: All errors caught, logged, and displayed to user in clear language. | All |
| NFR-10 | **Logging**: All operations logged with timestamp, result, and error context. | All |
| NFR-11 | **Maintainability**: Modular architecture with clear separation between UI, engine, and storage layers. | All |
| NFR-12 | **Testability**: Core backup and restore logic must be testable without real disks. | All |

---

## 10. User Workflows

### 10.1 First-Time Setup (Phase 1A)

| Step | Actor | Action |
|---|---|---|
| Precondition | | Windows installed, application launched |
| 1 | User | Launches the application |
| 2 | System | Shows welcome screen with "Create First Backup" CTA |
| 3 | User | Clicks "Create First Backup" |
| 4 | System | Shows folder selection dialog |
| 5 | User | Selects folders to back up |
| 6 | System | Shows destination selection (local disk, external disk) |
| 7 | User | Selects destination |
| 8 | System | Validates destination (accessible, has space) |
| 9 | User | Clicks "Start Backup" |
| 10 | System | Runs backup, shows progress, reports result |
| **Success** | | Backup completed |
| **Failure** | | Error displayed with actionable guidance |

### 10.2 Create a File Backup Job (Phase 1A)

| Step | Actor | Action |
|---|---|---|
| Precondition | | Application installed, user on main dashboard |
| 1 | User | Clicks "New Backup Job" |
| 2 | System | Presents job configuration form |
| 3 | User | Enters job name |
| 4 | User | Selects source folders |
| 5 | User | Selects destination path |
| 6 | User | Clicks "Save" |
| 7 | System | Validates all inputs, saves job configuration |
| **Success** | | Job listed in backup job list |
| **Failure** | | Validation error displayed with field-level details |

### 10.3 Run Manual Backup (Phase 1A)

| Step | Actor | Action |
|---|---|---|
| Precondition | | Backup job exists |
| 1 | User | Selects job from list |
| 2 | User | Clicks "Back Up Now" |
| 3 | System | Checks destination availability and space |
| 4 | System | Runs backup with progress indicator |
| 5 | System | Records result in backup history |
| **Success** | | Green checkmark on job, new version in history |
| **Failure (no space)** | | Alert: "Destination is full. Free space or choose another destination." |
| **Failure (disconnected)** | | Alert: "Destination not available. Reconnect and try again." |

### 10.4 Restore Deleted File to Alternate Location (Phase 1A)

| Step | Actor | Action |
|---|---|---|
| Precondition | | Backup history exists for the file |
| 1 | User | Opens "Restore" tab |
| 2 | User | Selects backup version |
| 3 | System | Shows file tree from backup catalog |
| 4 | User | Browses / searches for deleted file |
| 5 | User | Selects file, clicks "Restore to..." |
| 6 | System | Shows folder picker for alternate location |
| 7 | User | Selects target folder |
| 8 | System | Restores file with checksum validation, reports result |
| **Success** | | "Restored file to [path]" |
| **Failure** | | Error details with guidance |

### 10.5 Configure Scheduled Backup (Phase 1B)

| Step | Actor | Action |
|---|---|---|
| Precondition | | Backup job exists |
| 1 | User | Opens job edit |
| 2 | User | Sets schedule type: Daily / Weekly / Custom |
| 3 | User | Configures time and frequency |
| 4 | User | Saves |
| 5 | System | Registers Windows scheduled task for job |
| **Success** | | Schedule appears on job details |
| **Failure** | | Error: "Could not register scheduled task. Check permissions." |

### 10.6 Restore Folder to Original Location (Phase 1B)

| Step | Actor | Action |
|---|---|---|
| Precondition | | Backup version exists |
| 1 | User | Opens "Restore" tab |
| 2 | User | Selects backup version |
| 3 | User | Selects folder |
| 4 | User | Clicks "Restore to Original" |
| 5 | System | Shows conflict resolution dialog if files exist |
| 6 | User | Chooses: Overwrite / Keep both / Skip |
| 7 | System | Restores files with checksum validation, reports result |
| **Success** | | "Restored X files to original location" |
| **Failure** | | Error details with guidance |

### 10.7 Verify Backup (Phase 1A)

| Step | Actor | Action |
|---|---|---|
| Precondition | | Backup version exists |
| 1 | User | Selects backup version in history |
| 2 | User | Clicks "Verify" |
| 3 | System | Reads all files from destination, recomputes checksums, compares catalog |
| 4 | System | Reports verification result per-file and overall |
| **Success** | | "Backup verified — integrity OK" with file count |
| **Failure** | | "Backup corrupted — X files failed checksum. Create a new backup." |

### 10.8 Create System Image Backup (Phase 2)

| Step | Actor | Action |
|---|---|---|
| Precondition | | User is on Partitions tab |
| 1 | System | Detects and displays disk/partition layout |
| 2 | User | Selects system partition (usually C:) |
| 3 | User | Optionally selects EFI and Recovery partitions |
| 4 | User | Selects image destination |
| 5 | User | Clicks "Back Up System" |
| 6 | System | Creates VSS snapshot |
| 7 | System | Reads partition from snapshot |
| 8 | System | Compresses and writes image with block-level checksums |
| 9 | System | Releases VSS snapshot |
| 10 | System | Records result |
| **Success** | | System image backup completed |
| **Failure (VSS)** | | "VSS snapshot failed. Try again or check VSS service." |

### 10.9 Create Bootable Recovery USB (Phase 3)

| Step | Actor | Action |
|---|---|---|
| Precondition | | USB drive inserted (8 GB+), app running as admin |
| 1 | User | Opens "Recovery Media" tab |
| 2 | System | Detects USB drives |
| 3 | User | Selects USB drive from list |
| 4 | System | Warns: "All data on [drive] will be erased" |
| 5 | User | Confirms |
| 6 | System | Formats USB (FAT32) |
| 7 | System | Copies WinPE + recovery application |
| 8 | System | Reports result |
| **Success** | | "Bootable USB created successfully" |
| **Failure** | | Error with troubleshooting steps |

### 10.10 Recover Windows from Bootable USB (Phase 3)

| Step | Actor | Action |
|---|---|---|
| Precondition | | Bootable USB created, system cannot boot normally |
| 1 | User | Inserts USB, boots from it (BIOS/UEFI boot menu) |
| 2 | System | Loads recovery environment |
| 3 | System | Scans disks for backup images |
| 4 | User | Selects backup image to restore |
| 5 | System | Shows target disk/partition options |
| 6 | User | Selects target disk |
| 7 | System | Warns: "All data on target will be overwritten" |
| 8 | User | Confirms |
| 9 | System | Restores partition(s) from image |
| 10 | System | Repairs BCD if needed |
| 11 | System | Reports result, prompts to remove USB and reboot |
| **Success** | | Windows boots normally after reboot |
| **Failure (boot)** | | "Boot repair failed. Use Windows Recovery Environment for advanced boot repair." |

### 10.11 Clone Disk to New SSD (Phase 4)

| Step | Actor | Action |
|---|---|---|
| Precondition | | New SSD installed, detected by Windows |
| 1 | User | Opens "Disk Clone" tab |
| 2 | System | Lists source disks and target disks |
| 3 | User | Selects source disk (e.g., old HDD) |
| 4 | User | Selects target disk (e.g., new SSD) |
| 5 | System | Displays comparison: source vs target |
| 6 | System | Warning: "Target disk will be completely overwritten" |
| 7 | User | Confirms twice (double confirmation) |
| 8 | System | Clones source to target |
| 9 | System | Adjusts partition sizes if target is larger |
| 10 | System | Validates cloned disk bootability |
| **Success** | | "Clone successful. Replace old disk with new disk and boot." |
| **Failure (small target)** | | "Target disk is too small. Select a larger target." |

---

## 11. Safety and Destructive Operation Rules

| Rule ID | Operation | Safety Rule |
|---|---|---|
| SAFE-01 | Partition restore | Require explicit confirmation with clear warning: "This will DESTROY all data on the target partition." |
| SAFE-02 | System restore | Require confirmation + warning that restored system may differ from original configuration. |
| SAFE-03 | Disk clone target | Require double confirmation. First: "Target disk [name] will be overwritten." Second: "Type CONFIRM to proceed." |
| SAFE-04 | Target disk overwrite | Always display source and target disk details side by side before confirmation. |
| SAFE-05 | Backup deletion | Require confirmation. If deleting the last available restore point for a job, show extra warning. |
| SAFE-06 | Backup chain deletion (Future) | Warn that deleting an incremental baseline invalidates all dependent increments. |
| SAFE-07 | Low disk space | Halt backup when free space is below configured threshold (default: 1 GB or calculated estimate). |
| SAFE-08 | Destination unavailable | Skip backup, do not retry indefinitely. Log explicitly. |
| SAFE-09 | Interrupted backup | Mark backup as incomplete on next startup. Offer resume or restart. |
| SAFE-10 | Interrupted restore | Warn user that partial restore may leave inconsistent state. Offer retry or manual cleanup. |

---


---

## 附录 A：阶段划分与当前范围

### A.1 总体阶段

| Phase | 名称 | 核心目标 |
|-------|------|---------|
| Phase 1 | MVP — 文件级备份/恢复 | 最小可验证的文件备份/恢复闭环 |
| Phase 2 | 可用文件备份 | 压缩、校验、计划任务、zstd 压缩 |
| Phase 3 | NTFS 非系统卷镜像 | .nwb 格式 + 卷级备份/恢复 |
| Phase 4 | 可启动恢复 + 系统恢复 | WinPE/VSS/系统卷恢复 |
| Phase 5 | 磁盘克隆 | 磁盘级克隆和迁移 |
| Phase 6+ | 高级功能 | 异机还原/Linux/国产OS/NFS/加密/差异 |

### A.2 当前阶段（Phase 1）范围

**包含：**
- 文件/文件夹级完整备份
- 文件/文件夹级恢复（原位置 / 新位置）
- 本地副盘 / USB 备份目标
- 手动触发备份
- zstd 压缩（可选，单线程）
- SHA-256 校验
- 备份清单管理

**明确不包含（Phase 1 禁止实现）：**
- 差异备份、增量备份
- .nwb 镜像格式
- VSS 快照
- 卷级、磁盘级备份
- 系统卷备份
- GPT/MBR 分区处理
- WinPE / 启动介质
- 裸机恢复 / 异机还原
- 磁盘克隆
- 桌面 GUI
- AES 加密
- XOR Parity
- NFS
- daemon / IPC 服务
- Linux / 国产 OS / LoongArch
- 性能管线优化

### A.3 文档一致性声明

本文档中涉及上述"明确不包含"功能的内容，仅作为产品终局能力描述，不作为当前开发目标。当前开发目标以  9_MVP_Boundary_and_Risk_Correction.md 为准。

## 12. Backup and Restore Data Model

This section defines **high-level entities** and their purpose. Database schema is defined separately.

| Entity | Purpose | Key Fields (PRD Level) |
|---|---|---|
| **BackupJob** | User-defined backup configuration | job_id, name, source_paths, destination_path, schedule_config, retention_policy, created_at, updated_at |
| **BackupRun** | A single execution of a BackupJob | run_id, job_id, started_at, completed_at, status (running/completed/failed/partial), total_files, total_bytes, errors |
| **BackupDestination** | Storage location for backups | destination_id, path, type (local/usb/smb), total_space, free_space, is_available |
| **BackupItem** | A file or folder included in a BackupJob | item_id, job_id, path, type (file/folder), is_common_folder, common_folder_type |
| **BackupVersion** | A snapshot of files at a point in time | version_id, run_id, parent_version_id, timestamp, catalog_path, catalog_checksum, total_size, file_count, verification_status (unverified/passed/failed/suspect) |
| **BackupCatalog** | Index of files in a BackupVersion, with per-file metadata and checksums | catalog_id, version_id, file_path, file_size, modified_at, checksum_algorithm, content_checksum, is_deleted (for incremental tracking) |
| **RestoreJob** | User-initiated restore operation | restore_id, version_id, source_paths, target_path, started_at, completed_at, status, errors |
| **RestoreRun** | A single file restore execution | restore_run_id, restore_id, file_path, status (restored/skipped/failed), checksum_validated (yes/no/mismatch), bytes_restored |
| **VerificationRun** | A backup verification execution | verification_id, version_id, started_at, completed_at, status (passed/failed), files_checked, files_failed, details |
| **Disk** | Physical disk detected by system (Phase 2+) | disk_id, disk_number, size, partition_style (GPT/MBR), model |
| **Partition** | Disk partition (Phase 2+) | partition_id, disk_id, partition_type, file_system, size, used_space, is_system |
| **ImageBackup** | Partition or system image backup (Phase 2+) | image_id, partition_id, destination_path, size, compression_type, block_checksums, created_at, vss_snapshot_id, verification_status |
| **RecoveryMedia** | Bootable recovery USB record (Phase 3+) | media_id, drive_letter, created_at, version, status |

---

## 13. Compatibility Requirements

| Compat ID | Requirement | Priority | Notes |
|---|---|---|---|
| CMP-01 | Windows 7 SP1 | Required | 工控机场景广泛部署 |
| CMP-02 | Windows 8/8.1 | Required | 过渡版本兼容 |
| CMP-03 | Windows 10 | Required | 主流桌面版本 |
| CMP-04 | Windows 11 | Required | 当前桌面版本 |
| CMP-05 | Windows Server 2008 R2 ~ 2025 | Required | 服务器全系列兼容 |
| CMP-03 | NTFS file system | Required | Primary file system |
| CMP-04 | GPT partition style | Required | Modern standard |
| CMP-05 | MBR partition style | Required | Legacy support |
| CMP-06 | UEFI boot mode | Required | Modern boot |
| CMP-07 | BIOS / Legacy boot mode | Required | Legacy boot |
| CMP-08 | Internal SATA / NVMe disks | Required | Common disks |
| CMP-09 | USB external disks | Required | Common backup target |
| CMP-10 | SMB network shares | Required | NAS compatibility |
| CMP-11 | Basic NAS usage | Required | Common home NAS |
| CMP-12 | FAT32 / exFAT external drives (file backup only) | Required | Common USB format |
| CMP-13 | ReFS (read, image backup — future) | Research | Research item |

---

## 14. Technical Risk Areas

| Risk ID | Risk | Impact | Mitigation | Phase |
|---|---|---|---|---|
| RISK-01 | **Windows VSS behavior**: VSS snapshot creation can fail silently, or performance degrades under heavy I/O. | Inconsistent system backup | Always verify snapshot creation; fail backup if VSS fails. Log VSS writer states. | Phase 2 |
| RISK-02 | **Open files and locked files**: User files held open by applications cannot be read. | Missing files in backup | Use VSS for volume-level consistency; fall back to Volume Shadow Copy for individual open files. | Phase 1A |
| RISK-03 | **NTFS permissions**: Restored files may lose ACLs if restored without privilege. | Restored files inaccessible | Preserve ACLs in backup metadata; restore with original permissions when running as admin. | Phase 1A |
| RISK-04 | **Long file paths**: Paths > 260 characters on Windows. | Backup/restore failure | Use \\\\?\\ prefix for file operations. Validate path length before backup. | Phase 1A |
| RISK-05 | **Unicode file names**: Non-ASCII file names may not be handled by simple string operations. | Corrupted backup catalog | Use UTF-8 consistently for catalog and metadata. Test with CJK characters. | Phase 1A |
| RISK-06 | **SMB reliability**: Network interruptions during backup or restore. | Incomplete or corrupted backup | Implement retry logic with exponential backoff; use checksum verification after transfer. | Phase 1B |
| RISK-07 | **External disk disconnect**: USB disk disconnected during backup. | Corrupted backup | Detect drive removal; abort and mark backup incomplete. Validate catalog on resume. | Phase 1A |
| RISK-08 | **Backup chain corruption**: Incremental chain broken by missing base. | All dependent backups unusable | Verify chain integrity before each incremental; warn user if base is missing. | Phase 1A |
| RISK-09 | **Image backup consistency**: Power loss during image backup. | Unusable image | Write checksum during image creation; validate before marking complete. Use atomic completion marker. | Phase 2 |
| RISK-10 | **EFI / BCD boot repair**: BCD configuration is fragile; wrong repair can break boot. | System cannot boot | Backup original BCD before modification. Verify boot configuration after repair. | Phase 3 |
| RISK-11 | **GPT / MBR handling**: Incorrect partition table during restore. | Data loss, unbootable system | Validate partition geometry before restore. Never write partial partition table. | Phase 2 |
| RISK-12 | **WinPE driver availability**: Storage drivers missing from WinPE image. | Disks not visible in recovery environment | Include common storage drivers (Intel RST, NVMe, USB 3.x). Document how to add custom drivers. | Phase 3 |
| RISK-13 | **Restore to different disk size**: Target disk smaller than original. | Cannot restore | Always validate target >= source used space. Warn on size mismatch. | Phase 2 |
| RISK-14 | **Disk cloning bootability**: Cloned disk fails to boot (wrong partition layout, missing EFI). | User cannot boot from clone | Validate GPT partition layout after clone. Verify EFI System Partition presence. | Phase 4 |
| RISK-15 | **BitLocker handling**: BitLocker-encrypted volumes cannot be read without key. | Cannot back up encrypted data | Detect BitLocker status; warn user; require decryption or key backup before image backup. | Future |

---

## 15. Phase Exit Criteria

Each phase must produce a validation report before the next phase begins.

### 15.1 Phase 1A Exit Criteria

| EC ID | Criterion | Verification Method |
|---|---|---|
| EC-1A-01 | Backup and restore 1,000 mixed files successfully (small, medium, large) | Integration test: create job, run backup, restore to alternate path, verify checksums |
| EC-1A-02 | Detect corrupted backup file during verification | Integration test: manually corrupt a file in backup destination, run verify, system reports corruption |
| EC-1A-03 | Restore selected files to alternate path with checksum match | Integration test: restore 3 selected files, each checksum matches original |
| EC-1A-04 | Handle destination unavailable gracefully | Integration test: remove destination drive, run backup, system reports error |
| EC-1A-05 | Handle insufficient destination space gracefully | Integration test: fill destination volume, run backup, system reports error |
| EC-1A-06 | Produce readable operation logs with timestamps and results | Manual inspection: logs show backup start/end, file count, checksum status, errors |
| EC-1A-07 | System detects and handles long file paths (> 260 chars) | Integration test: back up folder with deep path, verify catalog and restore |
| EC-1A-08 | System handles Unicode file names (CJK characters) | Integration test: back up files with Chinese/Japanese names, verify catalog and restore |

### 15.2 Phase 1B Exit Criteria

| EC ID | Criterion | Verification Method |
|---|---|---|
| EC-1B-01 | Scheduled backup executes successfully on configured interval | Integration test: create daily schedule, verify 3 consecutive successful runs |
| EC-1B-02 | Backup history shows multiple versions with timestamps and status | UI test: after 3 runs, history lists 3 entries with correct data |
| EC-1B-03 | Retention policy does not delete the latest valid restore point | Integration test: set retention=2, run 3 backups, verify at least 2 versions exist including latest |
| EC-1B-04 | SMB backup destination failure is handled safely | Integration test: configure SMB path, disconnect share, run backup, system reports error |
| EC-1B-05 | Restore to original location handles file conflicts safely | Integration test: restore file that exists with Overwrite / Keep both / Skip options |
| EC-1B-06 | Common folder quick-select works for all 5 folders (Desktop, Documents, Pictures, Videos, Downloads) | UI test: each folder selected and confirmed in backup catalog |

### 15.3 Phase 2 Exit Criteria

| EC ID | Criterion | Verification Method |
|---|---|---|
| EC-2-01 | Disk and partition layout is detected correctly for GPT and MBR disks | Integration test: enumerate disks on test machine with known layout, verify correctness |
| EC-2-02 | Non-system NTFS partition image backup completes successfully | Integration test: back up a data partition; verify image file exists and has correct size |
| EC-2-03 | Image verification detects corruption | Integration test: corrupt image block, run verify, system reports corruption |
| EC-2-04 | Non-system partition restore succeeds in a lab test | Lab VM test: restore partition to secondary disk, verify file contents match original |
| EC-2-05 | VSS snapshot created and released correctly during backup | Integration test: monitor VSS event log during backup; verify snapshot created and removed |
| EC-2-06 | System partition backup completes with VSS | Integration test: back up C: partition with VSS, verify image consistency |

### 15.4 Phase 3 Exit Criteria

| EC ID | Criterion | Verification Method |
|---|---|---|
| EC-3-01 | Bootable USB is created successfully from the application | UI test: insert USB, create media, verify USB is bootable |
| EC-3-02 | WinPE environment boots successfully on test hardware | Lab test: boot target machine from USB, verify WinPE desktop loads |
| EC-3-03 | Recovery application detects local disks | Lab test: in WinPE, launch recovery app, verify all disks enumerated |
| EC-3-04 | Recovery application detects backup images on local and USB disks | Lab test: verify existing backup images are found and listed |
| EC-3-05 | System image restore succeeds in a controlled lab VM | Lab VM test: restore system image to blank disk, verify Windows boots successfully |
| EC-3-06 | Restored Windows boots successfully after restore | Lab VM test: verify OS boots to login screen without errors |
| EC-3-07 | BCD repair restores boot on a test system with damaged BCD | Lab test: corrupt BCD, boot from USB, run repair, verify system boots |

### 15.5 Phase 4 Exit Criteria

| EC ID | Criterion | Verification Method |
|---|---|---|
| EC-4-01 | Disk clone completes from source disk to target disk | Lab test: clone test HDD to target SSD, verify process completes |
| EC-4-02 | Target disk data matches expected source layout and content | Lab test: compare partition layout and file content between source and target |
| EC-4-03 | Cloned Windows disk boots in a controlled lab test | Lab test: replace test machine disk with cloned disk, verify Windows boots |
| EC-4-04 | Wrong target disk selection is prevented by validation and confirmation | UI test: select system disk as target, system blocks with appropriate warning |
| EC-4-05 | Double confirmation is required before clone starts | UI test: verify two distinct confirmation steps before clone execution |
| EC-4-06 | Cloning to a smaller target disk is rejected | Integration test: select target smaller than source used space, system rejects with error |

---

## 16. Engineering Guardrails

This section defines hard engineering rules that must not be violated throughout the project lifecycle.

| GR ID | Guardrail | Rationale |
|---|---|---|
| GR-01 | Do not introduce cloud backup, cloud sync, cloud storage, SaaS account system, antivirus, ransomware protection, or enterprise centralized management. | The product is explicitly local-only. Any cloud or security feature would violate product positioning and scope. **(See Non-Goals section)** |
| GR-02 | Do not implement bootable recovery media before Phase 1A and Phase 1B are validated and pass exit criteria. | Bootable media depends on validated image backup and restore workflows. Building it before the core loop is proven creates cascading risk. |
| GR-03 | Do not implement disk cloning before image backup and recovery media design are validated. | Cloning shares low-level disk I/O mechanics with image backup. Implementing clone without validated image infrastructure is high risk. |
| GR-04 | Do not claim backup success unless restore validation exists. | A backup that cannot be restored is worthless. Every feature must have a defined, tested restore path before it ships. |
| GR-05 | Every backup feature must have a corresponding restore workflow. | Symmetry between backup and restore is mandatory. If a restore workflow is not designed, the backup feature is not complete. |
| GR-06 | Every destructive operation must include: target validation, dry-run where possible, explicit warning, and user confirmation. | Data loss is irreversible. Multiple layers of protection prevent user error. |
| GR-07 | Low-level Windows behaviors such as VSS, BCD repair, GPT/MBR restore, and WinPE driver loading must be treated as research-backed items, not guessed. | These Windows internals have complex behavior that must be verified through testing and documentation, not assumed. |
| GR-08 | Each phase must produce a validation report before the next phase begins. **(See Phase Exit Criteria section)** | Phase gates prevent compounding integration failures. |
| GR-09 | Automated tests must include restore verification, not only backup creation. | Backup-only tests give false confidence. Restore validation is the true quality metric. |
| GR-10 | User-facing workflows must remain simple for personal users. Do not expose backup chain concepts (full, incremental, differential, chain) in the UI. | The target audience includes non-technical users. Complexity must be hidden behind internal engine logic. |
| GR-11 | All external operations, including file I/O, backup destination access, disk operations, restore operations, and network share access, must include robust error handling, clear operation logs, and user-facing error messages. | Silent failures are unacceptable. Users must always understand what happened and what action to take next. |
| GR-12 | Technology stack, implementation language, coding standards, and repository rules must be defined in the Technical Architecture and development guidelines, not in the PRD. | The PRD defines product requirements. Implementation standards belong in the architecture and engineering guidelines documents. |

---

## 17. Future Roadmap

### Phase 1A — Minimal Viable Backup Loop

**Goal**: Prove the smallest reliable backup/restore loop.

| Feature | Scope |
|---|---|
| Local folder selection | Phase 1A |
| Manual backup to internal or external USB disk | Phase 1A |
| Restore selected files/folders to alternate location | Phase 1A |
| Basic backup catalog with file metadata and per-file checksums | Phase 1A |
| Basic operation logs | Phase 1A |
| Checksum-based backup verification | Phase 1A |
| Clear success/failure result | Phase 1A |

**Exit criteria**: All EC-1A-01 through EC-1A-08 pass. Validation report produced.

### Phase 1B — Usable File Backup

**Goal**: Add scheduling, common folder support, SMB, and retention.

| Feature | Scope |
|---|---|
| Common folder quick-select (Desktop, Documents, Pictures, Videos, Downloads) | Phase 1B |
| Scheduled backup (daily, weekly, custom) | Phase 1B |
| Restore to original location with conflict handling | Phase 1B |
| Backup history UI | Phase 1B |
| Configurable retention policy | Phase 1B |
| SMB / NAS destination support | Phase 1B |
| Improved UI workflow | Phase 1B |

**Exit criteria**: All EC-1B-01 through EC-1B-06 pass. Validation report produced.

### Phase 2 — Partition / System Image Backup

**Goal**: Add partition-level backup with VSS. System partition restore requires Phase 3.

| Feature | Scope |
|---|---|
| Disk and partition discovery (GPT/MBR) | Phase 2 |
| NTFS partition image backup | Phase 2 |
| VSS-based system partition backup | Phase 2 |
| Image compression and block-level checksums | Phase 2 |
| Image verification | Phase 2 |
| Non-system partition restore (within Windows, when safe) | Phase 2 |

**Exit criteria**: All EC-2-01 through EC-2-06 pass. Validation report produced.

### Phase 3 — Bootable Recovery Media

**Goal**: Enable full system recovery when Windows cannot boot.

| Feature | Scope |
|---|---|
| WinPE bootable USB creation | Phase 3 |
| Recovery environment (disk detection, image detection, SMB access) | Phase 3 |
| System image restore from recovery environment | Phase 3 |
| Partition restore from recovery environment | Phase 3 |
| BCD boot repair | Phase 3 |

**Exit criteria**: All EC-3-01 through EC-3-07 pass. Validation report produced.

### Phase 4 — Disk Cloning

**Goal**: Enable HDD-to-SSD and disk replacement scenarios.

| Feature | Scope |
|---|---|
| Source disk selection | Phase 4 |
| Target disk selection with validation | Phase 4 |
| Bit-for-bit / intelligent sector copy | Phase 4 |
| Partition resizing on larger target | Phase 4 |
| Bootability validation | Phase 4 |
| Double confirmation for destructive operation | Phase 4 |

**Exit criteria**: All EC-4-01 through EC-4-06 pass. Validation report produced.

### Phase 5 — Advanced Recovery (Future)

**Goal**: Add advanced features after core reliability is proven.

| Feature | Scope |
|---|---|
| Incremental block-level backup | Future |
| Differential backup | Future |
| Image mounting | Future |
| Advanced retention (GFS) | Future |
| Hardware-independent restore | Future |
| BitLocker support | Future |
| ReFS support | Future |

**Exit criteria**: Each feature passes independent restore validation tests with documented results.

---

## 18. Open Questions

| OQ ID | Question | Impact | Suggested Resolution |
|---|---|---|---|
| OQ-01 | **Product name**: What should the product be called? | Branding, marketing | Discuss with stakeholders after PRD review |
| OQ-02 | **UI framework**: Desktop UI technology — WPF, WinUI 3, or web-based (Electron / local web app)? | Development effort, maintainability | Evaluate architecture before Phase 1A start |
| OQ-03 | **Programming language**: Backend engine — C++ (native performance) or C# (.NET ecosystem) or Rust? | All low-level components | Evaluate based on Windows API compatibility |
| OQ-04 | **Backup storage format**: Simple ZIP-like archive, custom container, or VHDX-based? | Performance, recoverability, complexity | Research options in technical architecture phase |
| OQ-05 | **Compression algorithm**: LZ4 (fast), Zstd (balanced), or LZMA (maximum compression)? | Speed vs size trade-off | Benchmark with real data sets |
| OQ-06 | **Encryption support**: Should backup be encrypted at rest? AES-256? | Privacy, recovery complexity | Decide per user demand; no encryption in Phase 1A |
| OQ-07 | **Local license model**: Free, paid perpetual, or donation-based? | Business model | Discuss with stakeholders |
| OQ-08 | **Windows service architecture**: Backup scheduler as Windows service or scheduled task? | Reliability, user permissions | Research in technical architecture phase |
| OQ-09 | **WinPE building strategy**: Use Microsoft ADK to build custom WinPE image, or bundle pre-built? | Complexity, legal/distribution | Research in Phase 3 planning |
| OQ-10 | **BitLocker support in early versions**: Skip BitLocker in Phase 1A / Phase 2? | User limitation | Confirm: BitLocker volumes require Future phase |
| OQ-11 | **NAS / SMB in MVP**: Should SMB share support be in Phase 1B? | User reach vs complexity | Deferred to Phase 1B as per scope split |
| OQ-12 | **Disk cloning in Phase 4 or later**: Achievable alongside recovery media? | Scope management | Prioritize recovery media over cloning if resources limited |
| OQ-13 | **Backup throttle / low priority I/O**: Throttle backup I/O to avoid impacting user work? | User experience during backup | Implement as configurable option in Phase 1A |

---

## 19. Risk Summary

| Risk Category | Key Risks |
|---|---|
| **Windows platform** | VSS failures, NTFS behavior, long paths, permissions, BitLocker |
| **Storage reliability** | SMB disconnects, USB removal, disk full, bad sectors |
| **Recovery correctness** | Partition geometry mismatch, BCD corruption, EFI missing |
| **Bootable media** | WinPE driver availability, storage controller compatibility |
| **Data integrity** | Per-file checksum coverage, incomplete writes, backup chain corruption |
| **User safety** | Wrong target disk, wrong partition, incomplete restore, destructive operation without confirmation |

---

## Appendix A: Document Revision History

| Version | Date | Author | Changes |
|---|---|---|---|
| v1.0 | 2026-07-01 | Codex | Initial PRD |
| v1.1 | 2026-07-01 | Codex | Split Phase 1 into 1A/1B; added file-level incremental semantics; strengthened verification requirements; clarified system image recovery boundary (V1.5 -> V2); added Engineering Guardrails (section 16); replaced MVP acceptance criteria with Phase Exit Criteria (section 15); updated all scope references |
---
## Completion Report (v1.1 Revision)

| Item | Detail |
|---|---|
| **File updated** | docs/01_Product_Requirements_Document.md |
| **Version** | v2.2（Phase mapping aligned） (previously v1.0) |
| **Phase 1A/1B split** | Phase 1A = minimal backup/restore loop (manual only, alternate location restore, folder selection). Phase 1B = scheduling, common folders, SMB, original-location restore, retention, history UI. SMB / NAS explicitly excluded from Phase 1A. |
| **Incremental semantics added** | New subsection "File-Level Incremental Backup Semantics" in section 8.1. Defines: version-per-run, complete restorable tree, change detection (size + mtime + optional checksum), deleted file handling, rename = delete+add, chain integrity validation. |
| **Verification strengthened** | Sections 8.1.4 rewritten: per-file content checksum (SHA-256), catalog checksum, verification status (unverified/passed/failed/suspect), restore-time checksum validation, explicit user-initiated verification, warning on suspect versions. |
| **System image boundary clarified** | Section 7.3 explicitly states: "System partition restore for a non-bootable Windows system requires WinPE or another offline recovery environment — this belongs to Phase 3, not Phase 2." V1.5 only includes non-system partition restore. |
| **Engineering guardrails added** | New section 16 with 12 guardrails (GR-01 through GR-12) covering scope enforcement, phase sequencing, restore-validation-first principle, destructive operation safety, Windows internals research requirement, test coverage, and code quality. |
| **Phase exit criteria added** | New section 15 with 5 phase-specific exit criteria sets (8 + 6 + 6 + 7 + 6 = 33 criteria total), each with measurable verification methods. |
| **Sections changed** | 1-4 (unchanged), 5 (goals updated for 1A/1B), 6 (unchanged), 7 (fully rewritten with 6 sub-phases), 8 (fully rewritten with incremental semantics + stronger verification), 9 (updated), 10 (workflows updated), 11-14 (updated scope references), 15 (new — Phase Exit Criteria), 16 (new — Engineering Guardrails), 17 (roadmap updated), 18-19 (open questions + risk summary updated), Appendix A (revision history) |


---

## Revision History

| 版本 | 日期 | 变更原因 |
|------|------|---------|
| v1.0 | 2026-07-01 | 初始 PRD |
| v1.1 | 2026-07-04 | 补充功能讨论细节 |
| v2.0 | 2026-07-05 | 产品名统一为 Nüwa Backup |
| v2.1 | 2026-07-05 | 灾备专家审查后修订：明确阶段边界，收敛 MVP 范围 |
| v2.2 | 2026-07-07 | Phase mapping update: added Phase 2.5 (Tauri GUI + Application Layer). Original Phase 2→3→4 renumbered. Historical roadmap preserved for traceability. |

