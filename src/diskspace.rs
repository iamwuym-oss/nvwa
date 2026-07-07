// ============================================================================
// diskspace.rs - Target disk free space check
//
// Implementation:
// On Windows, uses kernel32!GetDiskFreeSpaceExW API to query free space on the
// target path's drive. This is a native Windows API, no extra crate needed.
//
// On non-Windows platforms, falls back to writability check (same as before).
//
// Safety design:
// - Checks before backup that free space >= needed space × (1 + safety margin)
// - Safety margin = 10%, prevents edge-case space exhaustion
// - Returns error with specific numbers on insufficient space, no partial backup
// ============================================================================

use crate::errors::NuwaError;
use std::path::Path;

#[cfg(target_os = "windows")]
mod platform {
    use super::*;
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    // Win32 API: GetDiskFreeSpaceExW
    // https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getdiskfreespaceexw
    #[link(name = "kernel32")]
    extern "system" {
        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut u64,
            lpTotalNumberOfBytes: *mut u64,
            lpTotalNumberOfFreeBytes: *mut u64,
        ) -> i32;
    }

    /// Get free disk space (in bytes) for the drive containing the specified path
    ///
    /// Returns (free_bytes, total_bytes)
    pub fn free_space(path: &Path) -> Result<(u64, u64), NuwaError> {
        // Convert path to Windows wide character (UTF-16) format
        let wide: Vec<u16> = OsStr::new(&path)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut free_bytes: u64 = 0;
        let mut total_bytes: u64 = 0;

        // Safe call to Win32 API
        // Note: GetDiskFreeSpaceExW returns 0 for failure, non-zero for success
        let ret = unsafe {
            GetDiskFreeSpaceExW(wide.as_ptr(), &mut free_bytes, &mut total_bytes, null_mut())
        };

        if ret == 0 {
            // API call failed (e.g. invalid path or inaccessible)
            Err(NuwaError::Io {
                source: None,
                path: Some(path.to_path_buf()),
                detail: "Cannot query target disk space (GetDiskFreeSpaceExW call failed)"
                    .to_string(),
                suggestion: "Please confirm the target path is a valid disk path".to_string(),
            })
        } else {
            Ok((free_bytes, total_bytes))
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::*;

    /// Non-Windows platform: space check unavailable, fall back to writability check
    pub fn free_space(path: &Path) -> Result<(u64, u64), NuwaError> {
        Err(NuwaError::General {
            detail: format!(
                "Space check is not available on non-Windows platforms: {}",
                path.display()
            ),
            suggestion: "A basic writability check will be performed instead".to_string(),
        })
    }
}

/// Get free and total disk space for the drive containing the specified path
///
/// This is a READ-ONLY query interface used by the Application Layer
/// (dashboard_service) to display storage usage. It does not check against
/// any required backup size; callers use check_disk_space for that.
///
/// Returns (free_bytes, total_bytes) where:
/// - free_bytes: free space available to the caller
/// - total_bytes: total capacity of the underlying volume
pub fn free_space(path: &Path) -> Result<(u64, u64), NuwaError> {
    // Ensure the path exists so we can query its drive
    if !path.exists() {
        std::fs::create_dir_all(path).map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(path.to_path_buf()),
            detail: "Cannot create directory for space query".to_string(),
            suggestion: "Check that the path is valid and permissions are sufficient".to_string(),
        })?;
    }
    platform::free_space(path)
}
/// Check if the target path has sufficient space for the backup data
///
/// ## Parameters
/// * dest_root - Backup destination root path
/// *
/// * `needed_bytes` - Total bytes needed for backup data (including safety margin)
///
/// ## Returns
/// * Ok(()) - Sufficient space
/// * Err(NuwaError::Io { detail: "disk_space" }) - Insufficient space
/// * Err(...) - Cannot check (falls back to writability check)
///
/// ## Safety margin
/// Caller should already include the 10% safety margin in
/// Caller should already include the 10% safety margin in `needed_bytes`.
pub fn check_disk_space(dest_root: &Path, needed_bytes: u64) -> Result<(), NuwaError> {
    // Step 1: Ensure the destination path exists (create if it does not)
    if !dest_root.exists() {
        std::fs::create_dir_all(dest_root).map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(dest_root.to_path_buf()),
            detail: "Cannot create destination directory".to_string(),
            suggestion: "Check that the path is valid and permissions are sufficient".to_string(),
        })?;
    }

    // Step 2: Attempt to get free disk space
    match platform::free_space(dest_root) {
        Ok((free_bytes, _total_bytes)) => {
            // Check if available space is sufficient
            if free_bytes < needed_bytes {
                return Err(NuwaError::disk_space(needed_bytes, free_bytes, dest_root));
            }

            // Space is sufficient, print friendly message
            let free_mb = free_bytes / (1024 * 1024);
            let need_mb = needed_bytes / (1024 * 1024);
            if free_mb > 1024 {
                println!(
                    "Destination disk space OK: {:.1} GB free, need {} MB",
                    free_bytes as f64 / (1024.0 * 1024.0 * 1024.0),
                    need_mb
                );
            } else {
                println!(
                    "Destination disk space: {} MB free, need {} MB",
                    free_mb, need_mb
                );
            }
            Ok(())
        }
        Err(_) => {
            // Step 3: When API is unavailable, fall back to writability check
            // Note: This is a PARTIAL implementation, only used when Win32 API is unavailable
            let test_file = dest_root.join(".nuwa_space_check.tmp");
            match std::fs::write(&test_file, b"ok") {
                Ok(_) => {
                    let _ = std::fs::remove_file(&test_file);
                    println!("Warning: cannot check disk free space (only verified path is writable). Ensure sufficient space.");
                    println!(
                        "  Estimated space needed: ~{} MB",
                        needed_bytes / (1024 * 1024)
                    );
                    Ok(())
                }
                Err(e) => Err(NuwaError::Io {
                    source: Some(e),
                    path: Some(dest_root.to_path_buf()),
                    detail: "Destination path is not writable".to_string(),
                    suggestion: "Check permissions and disk space".to_string(),
                }),
            }
        }
    }
}
