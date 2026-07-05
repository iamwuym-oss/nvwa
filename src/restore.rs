// ============================================================================
// restore.rs -- Nuwa Backup restore executor
//
// Core flow:
// 1. Read manifest.json from backup directory
// 2. Recreate all directory structures
// 3. Restore files one by one:
//    a. If destination file exists and --overwrite=false → skip
//    b. If destination file exists and --overwrite=true → check if locked
//    c. If locked → skip and report
//    d. Otherwise → atomic write to destination
//    e. Verify SHA-256 after restore
// 4. Output restore summary
// ============================================================================

use crate::checksum;
use crate::errors::NuwaError;
use crate::storage::{self, read_manifest};
use std::fs::OpenOptions;
use std::path::Path;

/// Restore operation result statistics
pub struct RestoreResult {
    /// Number of files successfully restored
    pub restored_count: u64,
    /// Number of files skipped (already exists with --overwrite=false, or file locked)
    pub skipped_count: u64,
    /// Number of files with checksum failures
    pub checksum_failures: u64,
}

/// Execute file-level restore
//
// Parameters:
// - backup_dir: Backup point directory path (contains manifest.json and files/ subdirectory)
// - dest: Restore destination path
// - overwrite: Whether to overwrite existing files
//
// Safety design:
// - Does not overwrite existing files (unless --overwrite specified)
// - Detects if a file is locked by another process
// - Uses atomic writes (.tmp → rename) so interruption does not produce corrupt files
// - Verifies SHA-256 for each restored file
pub fn execute_restore(
    backup_dir: &Path,
    dest: &Path,
    overwrite: bool,
) -> Result<RestoreResult, NuwaError> {
    let manifest = read_manifest(backup_dir)?;
    let compression = manifest.compression.enabled;

    // ===== Step 1: Recreate all directories =====
    // Create directories before restoring files to ensure structure is complete
    for dir_entry in &manifest.directories {
        std::fs::create_dir_all(dest.join(&dir_entry.relative_path))?;
    }

    let mut result = RestoreResult {
        restored_count: 0,
        skipped_count: 0,
        checksum_failures: 0,
    };

    // ===== Step 2: Restore files one by one =====
    for file_entry in &manifest.files {
        let dest_path = dest.join(&file_entry.relative_path);

        // Case 1: File exists and --overwrite not specified → skip
        if dest_path.exists() && !overwrite {
            println!("Skipping (file exists): {}", file_entry.relative_path);
            result.skipped_count += 1;
            continue;
        }

        // Case 2: File exists and --overwrite specified
        // But the file may be locked by another process, check first
        if dest_path.exists() && overwrite {
            match is_file_locked(&dest_path) {
                Ok(true) => {
                    // File is locked, skip and notify user
                    println!(
                        "Skipping (file locked by another process): {}",
                        file_entry.relative_path
                    );
                    println!("  Tip: Close the program using this file and retry");
                    result.skipped_count += 1;
                    continue;
                }
                Ok(false) => {
                    // File is not locked, restore can proceed
                }
                Err(e) => {
                    // Detection failed (e.g. path issue), return error
                    return Err(e);
                }
            }
        }

        // ===== Execute restore (atomic write) =====
        storage::restore_file(backup_dir, file_entry, &dest_path, compression)?;

        // ===== Verify SHA-256 after restore =====
        match checksum::verify_file_checksum(&dest_path, &file_entry.sha256) {
            Ok(true) => {
                result.restored_count += 1;
                println!("Restored: {}", file_entry.relative_path);
            }
            Ok(false) => {
                result.checksum_failures += 1;
                println!("Checksum mismatch: {}", file_entry.relative_path);
            }
            Err(e) => {
                return Err(NuwaError::VerificationFailed {
                    detail: format!(
                        "Post-restore verification failed ({}): {}",
                        file_entry.relative_path, e
                    ),
                });
            }
        }
    }

    // ===== Step 3: Output summary =====
    println!(
        "\nRestore complete: {} restored, {} skipped, {} checksum failures",
        result.restored_count, result.skipped_count, result.checksum_failures
    );

    if result.checksum_failures > 0 {
        return Err(NuwaError::VerificationFailed {
            detail: format!(
                "{} files have checksum mismatches after restore",
                result.checksum_failures
            ),
        });
    }

    Ok(result)
}

/// Check if a target file is locked by another process
//
// Implementation:
// Attempts to open the existing file in write mode (without creating or truncating).
// On Windows, if another process holds a handle without FILE_SHARE_WRITE,
// CreateFileW returns "Permission Denied".
// On Linux/macOS, opening a write-locked file returns "Resource temporarily unavailable".
//
// Returns:
// - Ok(true)  → File is locked, cannot write
// - Ok(false) → File is not locked, safe to write
// - Err(...)  → Error during detection
pub fn is_file_locked(path: &Path) -> Result<bool, NuwaError> {
    if !path.exists() {
        // File does not exist -- definitely not locked
        return Ok(false);
    }

    // Open with write access, but do not create new file or truncate existing content
    // This is a lock check, not a file modification
    match OpenOptions::new()
        .write(true)
        .create(false)
        .truncate(false)
        .open(path)
    {
        Ok(_file) => {
            // Opened successfully → file is not locked
            // _file is automatically closed when it goes out of scope
            Ok(false)
        }
        Err(e) => {
            // Determine based on error type
            match e.kind() {
                // Windows: exclusive lock returns PermissionDenied
                // POSIX: write lock returns WouldBlock
                std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::WouldBlock => Ok(true),
                _ => Err(NuwaError::Io {
                    source: Some(e),
                    path: Some(path.to_path_buf()),
                    detail: format!("Cannot check file status: {}", path.display()),
                    suggestion: "Check if the file is being used by another program".to_string(),
                }),
            }
        }
    }
}
