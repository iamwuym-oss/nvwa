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
use std::path::{Component, Path};

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

    // Pre-validate all manifest entries for path traversal (safety hardening)
    for dir_entry in &manifest.directories {
        validate_restore_path(dest, &dir_entry.relative_path)?;
    }
    for file_entry in &manifest.files {
        validate_restore_path(dest, &file_entry.relative_path)?;
    }

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

/// Validate that a manifest relative_path does not attempt path traversal.
///
/// Rules:
/// - relative_path must not be empty or whitespace-only
/// - relative_path must not be an absolute path
/// - relative_path must not contain parent directory components (..)
/// - (Windows only) relative_path must not be a drive-relative path (e.g. "C:evil.txt")
/// - The caller is responsible for ensuring the combined path stays within dest
pub fn validate_restore_path(_dest: &Path, relative_path: &str) -> Result<(), NuwaError> {
    fn has_parent_traversal(p: &str) -> bool {
        for component in Path::new(p).components() {
            if component == Component::ParentDir {
                return true;
            }
        }
        false
    }

    if relative_path.trim().is_empty() {
        return Err(NuwaError::SafetyViolation {
            detail: "Restore entry has an empty path".into(),
            suggestion:
                "The backup manifest contains an empty path entry. This backup may be corrupted."
                    .into(),
        });
    }

    if Path::new(relative_path).is_absolute() {
        return Err(NuwaError::SafetyViolation {
                detail: format!("Restore entry '{}' is an absolute path. Relative paths are required.", relative_path),
                suggestion: "This backup manifest may be corrupted or tampered with. Do not restore from untrusted sources.".into(),
            });
    }

    // Windows drive-relative path check (e.g., "C:evil.txt")
    // On Windows, Path::new("C:evil.txt").is_absolute() returns false,
    // but such paths could resolve in unexpected ways depending on the current
    // working directory of the target drive.
    #[cfg(windows)]
    if is_drive_relative(relative_path) {
        return Err(NuwaError::SafetyViolation {
            detail: format!(
                "Restore entry '{}' is a Windows drive-relative path. Drive-relative paths are not allowed.",
                relative_path
            ),
            suggestion: "This backup manifest may be tampered with. Do not restore from untrusted sources."
                .into(),
        });
    }

    /// Returns true if the path is a Windows drive-relative path (e.g., "C:evil.txt").
    /// These have a drive letter prefix but no root directory separator after the colon.
    #[cfg(windows)]
    fn is_drive_relative(p: &str) -> bool {
        let mut components = Path::new(p).components();
        match components.next() {
            Some(Component::Prefix(prefix)) => {
                matches!(prefix.kind(), std::path::Prefix::Disk(_))
                    && !matches!(components.next(), Some(Component::RootDir))
            }
            _ => false,
        }
    }

    if has_parent_traversal(relative_path) {
        return Err(NuwaError::SafetyViolation {
            detail: format!(
                "Restore entry '{}' contains parent directory traversal ('..').",
                relative_path
            ),
            suggestion:
                "This backup manifest may be tampered with. Do not restore from untrusted sources."
                    .into(),
        });
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn check_validation(relative_path: &str) -> Result<(), NuwaError> {
        let dir =
            std::env::temp_dir().join(format!("nuwa_test_restore_val_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let result = validate_restore_path(&dir, relative_path);
        let _ = fs::remove_dir_all(&dir);
        result
    }

    #[test]
    fn test_valid_normal_path() {
        assert!(check_validation("Documents/file.txt").is_ok());
    }

    #[test]
    fn test_valid_nested_path() {
        assert!(check_validation("a/b/c/d/file.txt").is_ok());
    }

    #[test]
    fn test_empty_path_rejected() {
        let err = check_validation("").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[test]
    fn test_whitespace_path_rejected() {
        let err = check_validation("   ").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[test]
    fn test_parent_traversal_rejected() {
        let err = check_validation("../evil.exe").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[test]
    fn test_parent_traversal_nested_rejected() {
        let err = check_validation("subdir/../../evil.exe").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[test]
    fn test_parent_traversal_backslash_rejected() {
        let err = check_validation("..\\evil.exe").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[test]
    fn test_absolute_path_rejected() {
        let path = if cfg!(windows) {
            "C:\\Windows\\System32"
        } else {
            "/etc/passwd"
        };
        let err = check_validation(path).unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[test]
    fn test_deep_parent_traversal_rejected() {
        let err = check_validation("../../../etc/passwd").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[test]
    fn test_path_with_spaces_allowed() {
        assert!(check_validation("My Documents/report.txt").is_ok());
    }

    #[test]
    fn test_unicode_filename_allowed() {
        assert!(check_validation("文档/报告.txt").is_ok());
    }

    #[test]
    fn test_nonexistent_dest_still_rejects_parent_traversal() {
        let dir = std::env::temp_dir().join("nuwa_test_restore_nonexistent");
        let _ = fs::remove_dir_all(&dir);
        assert!(validate_restore_path(&dir, "normal/file.txt").is_ok());
        assert!(validate_restore_path(&dir, "../evil.exe").is_err());
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_drive_absolute_path_rejected() {
        let err = check_validation("C:\\Windows\\System32\\evil.dll").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_drive_relative_path_rejected() {
        let err = check_validation("C:evil.txt").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_drive_relative_nested_rejected() {
        let err = check_validation("C:folder\\evil.txt").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_unc_path_rejected() {
        let err = check_validation("\\\\server\\share\\evil.txt").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_extended_path_rejected() {
        let err = check_validation("\\\\?\\C:\\evil.txt").unwrap_err();
        assert!(matches!(err, NuwaError::SafetyViolation { .. }));
    }
}
