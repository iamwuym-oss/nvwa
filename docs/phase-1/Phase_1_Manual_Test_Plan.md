# Phase 1 — Manual Test Plan

## Overview

This document describes the manual test scenarios for Phase 1 that cannot be
fully automated or are too destructive to run in an automated test suite.

---

## Test 1: 10GB+ Large File Backup and Restore

### Goal
Verify that the tool can handle a single file larger than 10 GB.

### Prerequisites
- A drive with at least 30 GB free space.
- A tool to create a large file (e.g., `fsutil` on Windows).

### Test Environment
- Windows 10/11 or Windows Server
- SSD or HDD with NTFS

### Steps

**Step 1: Create a test directory with a 10 GB file inside**
```cmd
mkdir C:\Temp\NuwaLargeFileTest
fsutil file createnew C:\Temp\NuwaLargeFileTest\largefile.dat 10737418240
```

**Step 2: Compute SHA-256**
```cmd
certutil -hashfile C:\Temp\NuwaLargeFileTest\largefile.dat SHA256
```
Record the hash.

**Step 3: Execute backup (source must be a directory)**
```cmd
nuwa backup --source C:\Temp\NuwaLargeFileTest --dest D:\BackupTest
```

**Step 4: Verify backup**
```cmd
nuwa verify --backup D:\BackupTest\<backup_dir>
```

**Step 5: Delete original directory**
```cmd
rmdir /s /q C:\Temp\NuwaLargeFileTest
```

**Step 6: Restore**
```cmd
nuwa restore --backup D:\BackupTest\<backup_dir> --dest C:\Temp\Restored
```

**Step 7: Verify restored file SHA-256**
```cmd
certutil -hashfile C:\Temp\Restored\largefile.dat SHA256
```

### Expected Result
- Backup completes without out-of-memory errors.
- Verify passes.
- Restore produces an identical file.
- SHA-256 matches the original.

### Actual Result
(To be filled during manual test execution.)

---

## Test 2: Destination Disk Full During Backup

### Goal
Verify that the tool handles a full destination disk gracefully without
leaving partial/corrupt backup points.

### Prerequisites
- A small USB drive or a virtual disk (VHD) that can be filled.
- Alternatively, use a RAM disk with limited size.

### Test Environment
- Windows 10/11
- A target drive with less than 100 MB free space

### Steps

**Step 1: Create a test source with moderately sized files**
```cmd
mkdir C:\Temp\DiskFullTest
fsutil file createnew C:\Temp\DiskFullTest\file1.dat 52428800   (50 MB)
fsutil file createnew C:\Temp\DiskFullTest\file2.dat 52428800   (50 MB)
```

**Step 2: Attempt backup to a nearly-full drive**
```cmd
nuwa backup --source C:\Temp\DiskFullTest --dest E:\ (nearly full drive)
```

**Step 3: Check the error message**

### Expected Result
- Backup fails with a clear I/O error message.
- No partial backup point is left at the destination.
- No `.tmp` residue files remain.

### Actual Result
(To be filled during manual test execution.)

---

## Test 3: Windows Locked File Detection

### Goal
Verify that the tool detects locked files during restore with `--overwrite`
and skips them gracefully.

### Prerequisites
- Windows 10/11

### Test Environment
- A local folder with a text file

### Steps

**Step 1: Create and backup a test file**
```cmd
mkdir C:\Temp\LockTest
echo "important data" > C:\Temp\LockTest\myfile.txt
nuwa backup --source C:\Temp\LockTest --dest D:\BackupTest
```

**Step 2: Lock the file using a separate process**
Open the file with exclusive write lock. This can be done with:
- PowerShell: `[System.IO.File]::Open("C:\Temp\LockTest\myfile.txt", "Open", "Read", "None")`
  Keep this PowerShell window open.
- Or use a tool like `lockfile.exe`.

**Step 3: Restore with --overwrite**
```cmd
nuwa restore --backup D:\BackupTest\<backup_dir> --dest C:\Temp\LockTest --overwrite
```

**Step 4: Verify the output**

### Expected Result
- Restore reports that the locked file was skipped.
- Restore continues without crashing.
- The locked file retains its original content.
- The overall restore command reports success (with skipped count > 0).

### Expected Behavior Notes
- Lock detection is **best-effort**. It covers common Windows file locks
  (sharing violation, permission denied) but does not guarantee detection
  of all possible lock scenarios (e.g., mapped file locks, transactional NTFS locks).
- Skipped files are reported to the user with a suggestion to close the
  locking program and retry.

### Actual Result
(To be filled during manual test execution.)

---

## Test 4: Unicode / Special Character Filename Roundtrip

### Goal
Verify that filenames with Unicode characters (Chinese, Japanese, emoji,
umlauts) and special characters (spaces, dots, parentheses) survive
backup and restore correctly.

This test is already automated (`test_consistency` in `backup_restore_tests.rs`)
but can be run manually for additional confidence.

### Manual Command
```cmd
nuwa backup --source <dir_with_special_filenames> --dest D:\BackupTest
nuwa verify --backup D:\BackupTest\<backup_dir>
del <source_dir> /s
nuwa restore --backup D:\BackupTest\<backup_dir> --dest C:\Restored --overwrite
```

### Expected Result
All filenames, including Unicode and special characters, are preserved
exactly as in the source.