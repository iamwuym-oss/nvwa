# Nüwa Backup — Project Proposal

**Date:** 2026-07-05
**Status:** Implementation in progress (Phase 1 complete / PARTIAL)

---

## 1. Project Origin

This project was initiated to create a local-first, single-machine backup and
disaster recovery tool for Windows workstations and servers. The product is
inspired by the backup and recovery capabilities of tools such as Acronis True
Image, but focuses on a narrower, well-defined scope.

## 2. Target Users

- Personal Windows users
- Power users with important local data
- Freelancers, creators, developers, small offices
- PC repair shops (system migration)
- Windows Server operators (edge nodes, standalone servers)
- Future: Linux and domestic OS users (Kylin, UOS)

## 3. Staged Delivery

| Phase | Focus | Status |
|-------|-------|--------|
| Phase 1 | File-level backup/restore CLI (current) | PARTIAL |
| Phase 2 | Scheduler, retention, SMB, GUI | Not started |
| Phase 3 | NTFS non-system volume image, VSS, block backup | Not started |
| Phase 4 | WinPE recovery media, system restore | Not started |
| Phase 5 | Disk cloning | Not started |
| Phase 6+ | Differential backup, encryption, cross-platform | Not started |

## 4. Core Principles

- Reliability over feature quantity.
- Restore success over backup speed.
- Data integrity over convenience.
- Local-first architecture.
- Verifiable backup images.
- Recoverability in real disaster scenarios.

## 5. Permanent Exclusions

- Cloud backup / sync / storage.
- Mobile phone backup.
- Microsoft 365 backup.
- Antivirus / anti-malware / ransomware protection.
- AI threat detection.
- Enterprise centralized management.
- Multi-device cloud dashboard.
- SaaS account system.