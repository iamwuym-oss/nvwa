// ============================================================================
// path_security.rs — Path validation and safe directory creation (M1)
// ============================================================================
//
// P-02 shared module between Repository Backup Writer and Restore Reader.
// Provides path validation and safe directory creation with per-ancestor
// symlink/junction/reparse-point guards.
//
// Core Design:
// - validate_catalog_relative_path: validates and normalizes a catalog-relative
//   path, rejecting path traversal, absolute paths, UNC, and empty paths.
// - prepare_restore_file_target: validates, creates parent dirs (one-by-one
//   with ancestor guards), returns the full destination path for a file.
// - prepare_restore_directory: validates, creates all dir components including
//   the last one, returns the full destination path for a directory.
//
// Safety: Per-ancestor symlink_metadata() check rejects symlinks, junctions,
// and reparse points at every directory level. NEVER uses create_dir_all.

use crate::repository::error::RepositoryError;
use std::path::{Component, Path, PathBuf};

/// Maximum allowed path depth to prevent infinite loops from corrupt data.
const MAX_PATH_DEPTH: usize = 512;

/// Validate and normalize a catalog-relative path.
///
/// Returns the normalized path string (forward-slash, no trailing slash, no
/// consecutive separators) on success.
///
/// # Rejection Rules
/// - Empty path
/// - Whitespace-only path
/// - Absolute paths: starting with `/`, `\`, or Windows drive prefix (`X:\`)
/// - UNC paths (`\\`)
/// - Windows drive-relative paths (`C:foo`)
/// - Parent directory traversal (`..`)
/// - Current directory (`.`)
/// - Consecutive separators (`//`)
/// - Paths exceeding MAX_PATH_DEPTH components
///
/// # Normalization
/// - Backslashes converted to forward slashes
/// - Trailing slash stripped
/// - Consecutive separators collapsed to single `/`
pub fn validate_catalog_relative_path(path: &str) -> Result<String, RepositoryError> {
    let trimmed = path.trim();

    // Reject empty or whitespace-only
    if trimmed.is_empty() {
        return Err(RepositoryError::General {
            detail: "Catalog path is empty or whitespace-only".into(),
        });
    }

    // Normalize: replace backslashes with forward slashes
    let normalized = trimmed.replace('\\', "/");

    // Reject consecutive separators
    if normalized.contains("//") {
        return Err(RepositoryError::General {
            detail: format!(
                "Catalog path '{}' contains consecutive separators",
                normalized
            ),
        });
    }

    // Reject absolute Unix paths
    if normalized.starts_with('/') {
        return Err(RepositoryError::General {
            detail: format!("Catalog path '{}' is an absolute Unix path", normalized),
        });
    }

    // Reject paths that start with backslash (Windows absolute or UNC)
    if path.starts_with('\\') {
        return Err(RepositoryError::General {
            detail: format!(
                "Catalog path '{}' starts with backslash (absolute or UNC path)",
                path
            ),
        });
    }

    // Strip trailing slash (forward or back)
    let stripped = normalized
        .strip_suffix('/')
        .unwrap_or(&normalized)
        .to_string();

    // Parse components for validation
    let components: Vec<&str> = stripped.split('/').filter(|s| !s.is_empty()).collect();

    // Reject if the only component was the separator (i.e., path was "/")
    if components.is_empty() && !stripped.is_empty() {
        return Err(RepositoryError::General {
            detail: "Catalog path resolves to root".into(),
        });
    }

    // Reject if original was just a separator
    if components.is_empty() {
        return Err(RepositoryError::General {
            detail: "Catalog path is empty after normalization".into(),
        });
    }

    // Check maximum depth
    if components.len() > MAX_PATH_DEPTH {
        return Err(RepositoryError::General {
            detail: "Catalog path exceeds maximum depth".into(),
        });
    }

    // Validate each component
    for component in &components {
        // Reject parent directory traversal
        if *component == ".." {
            return Err(RepositoryError::General {
                detail: format!(
                    "Catalog path '{}' contains parent directory traversal ('..')",
                    stripped
                ),
            });
        }

        // Reject current directory references
        if *component == "." {
            return Err(RepositoryError::General {
                detail: format!(
                    "Catalog path '{}' contains current directory reference ('.')",
                    stripped
                ),
            });
        }

        // Reject empty components (shouldn't happen after normalization, but
        // guard against edge cases)
        if component.is_empty() {
            return Err(RepositoryError::General {
                detail: "Catalog path contains empty component".into(),
            });
        }
    }

    // Check for Windows absolute paths using Path::new().is_absolute()
    // This catches patterns like "C:\" or "C:/"
    let p = Path::new(path);
    if p.is_absolute() {
        return Err(RepositoryError::General {
            detail: format!(
                "Catalog path '{}' is an absolute path (Windows drive prefix)",
                path
            ),
        });
    }

    // Windows drive-relative check (e.g., "C:evil.txt")
    // Path::is_absolute() returns false for these on Windows
    #[cfg(windows)]
    {
        let p = Path::new(path);
        let mut comps = p.components();
        if let Some(Component::Prefix(prefix)) = comps.next() {
            if matches!(prefix.kind(), std::path::Prefix::Disk(_)) {
                return Err(RepositoryError::General {
                    detail: format!("Catalog path '{}' is a Windows drive-relative path", path),
                });
            }
        }
    }

    Ok(stripped)
}

/// Prepare the target path for restoring a file.
///
/// Validates the relative path, then creates all parent directories with
/// per-ancestor symlink/junction/reparse-point guards. Does NOT create the
/// file itself.
///
/// Returns the full destination path for the file.
///
/// # Safety
/// - Each ancestor directory is checked with symlink_metadata() before creation
/// - If an ancestor is a symlink, junction, or reparse point: Err
/// - Directories are created one-by-one (never create_dir_all)
/// - Path traversal is rejected
pub fn prepare_restore_file_target(
    dest_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, RepositoryError> {
    let normalized = validate_catalog_relative_path(relative_path)?;

    let dest_path = dest_root.join(&normalized);

    // Verify the joined path is still under dest_root (defense in depth)
    if !dest_path.starts_with(dest_root) {
        return Err(RepositoryError::General {
            detail: format!(
                "Resolved restore path escapes destination root: {}",
                normalized
            ),
        });
    }

    // Create parent directories only (not the last component)
    if let Some(parent) = dest_path.parent() {
        if parent != dest_root {
            create_parent_dirs(dest_root, parent)?;
        }
    }

    Ok(dest_path)
}

/// Prepare the target path for restoring a directory.
///
/// Validates the relative path, then creates all directory components
/// (including the last one) with per-ancestor symlink/junction/reparse-point
/// guards.
///
/// Returns the full destination path for the directory.
///
/// # Safety
/// - Each ancestor directory is checked with symlink_metadata() before creation
/// - If an ancestor is a symlink, junction, or reparse point: Err
/// - Directories are created one-by-one (never create_dir_all)
/// - Path traversal is rejected
pub fn prepare_restore_directory(
    dest_root: &Path,
    relative_path: &str,
) -> Result<PathBuf, RepositoryError> {
    let normalized = validate_catalog_relative_path(relative_path)?;

    let dest_path = dest_root.join(&normalized);

    // Verify the joined path is still under dest_root (defense in depth)
    if !dest_path.starts_with(dest_root) {
        return Err(RepositoryError::General {
            detail: format!(
                "Resolved restore path escapes destination root: {}",
                normalized
            ),
        });
    }

    // Create all directories including the last component
    if dest_path != dest_root {
        create_parent_dirs(dest_root, &dest_path)?;
    }

    Ok(dest_path)
}

/// Create directories from dest_root up to (and including) target.
///
/// Walks from dest_root toward target, creating each missing directory
/// one-at-a-time. Before creating a directory, checks with symlink_metadata()
/// that no existing ancestor is a symlink, junction, or reparse point.
///
/// # Arguments
/// * `dest_root` — The root destination directory (must exist)
/// * `target` — The full target path (must be under dest_root)
fn create_parent_dirs(dest_root: &Path, target: &Path) -> Result<(), RepositoryError> {
    // Collect ancestors from dest_root to target (exclusive of root, inclusive of target)
    let mut ancestors: Vec<&Path> = Vec::new();
    let mut current = target;
    while current != dest_root {
        ancestors.push(current);
        match current.parent() {
            Some(parent) => current = parent,
            None => {
                return Err(RepositoryError::General {
                    detail: format!("Could not walk up from {}", current.display()),
                });
            }
        }
    }

    // Process in reverse order (from closest to dest_root to furthest)
    for ancestor in ancestors.into_iter().rev() {
        // Check for symlink/junction/reparse point at every level
        if ancestor.exists() {
            let meta =
                std::fs::symlink_metadata(ancestor).map_err(|e| RepositoryError::IoError {
                    path: ancestor.to_path_buf(),
                    detail: format!("Failed to read metadata for {}", ancestor.display()),
                    source: e,
                })?;

            // Reject if it's a symlink, junction, or reparse point
            if meta.file_type().is_symlink() {
                return Err(RepositoryError::General {
                    detail: format!(
                        "Path '{}' is a symlink — refusing to follow",
                        ancestor.display()
                    ),
                });
            }

            // On Windows, check for reparse points (junctions)
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x0400;
                if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
                    return Err(RepositoryError::General {
                        detail: format!(
                            "Path '{}' is a reparse point (junction) — refusing to follow",
                            ancestor.display()
                        ),
                    });
                }
            }

            // If already a directory, continue
            if meta.is_dir() {
                continue;
            }

            // Exists but not a directory — error
            return Err(RepositoryError::General {
                detail: format!(
                    "Path '{}' exists but is not a directory",
                    ancestor.display()
                ),
            });
        }

        // Create the directory
        std::fs::create_dir(ancestor).map_err(|e| RepositoryError::IoError {
            path: ancestor.to_path_buf(),
            detail: format!("Failed to create directory {}", ancestor.display()),
            source: e,
        })?;
    }

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // ---- validate_catalog_relative_path ----

    #[test]
    fn test_valid_normal_path() {
        let result = validate_catalog_relative_path("foo/bar/baz.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "foo/bar/baz.txt");
    }

    #[test]
    fn test_valid_backslash_normalized() {
        let result = validate_catalog_relative_path("foo\\bar\\baz.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "foo/bar/baz.txt");
    }

    #[test]
    fn test_valid_trailing_slash_stripped() {
        let result = validate_catalog_relative_path("foo/bar/");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "foo/bar");
    }

    #[test]
    fn test_empty_path_rejected() {
        let result = validate_catalog_relative_path("");
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_path_rejected() {
        let result = validate_catalog_relative_path("   ");
        assert!(result.is_err());
    }

    #[test]
    fn test_parent_traversal_rejected() {
        let result = validate_catalog_relative_path("../foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_deep_parent_traversal_rejected() {
        let result = validate_catalog_relative_path("foo/../../bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_current_dir_rejected() {
        let result = validate_catalog_relative_path("./foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_just_current_dir_rejected() {
        let result = validate_catalog_relative_path(".");
        assert!(result.is_err());
    }

    #[test]
    fn test_unix_absolute_path_rejected() {
        let result = validate_catalog_relative_path("/etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_consecutive_separators_rejected() {
        let result = validate_catalog_relative_path("foo//bar");
        assert!(result.is_err());
    }

    #[test]
    fn test_windows_absolute_rejected() {
        // On Windows, Path::new("C:\\foo").is_absolute() == true
        let result = validate_catalog_relative_path("C:\\foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_windows_drive_relative_rejected() {
        // "C:foo" is drive-relative on Windows
        let result = validate_catalog_relative_path("C:foo.txt");
        assert!(result.is_err());
    }

    #[test]
    fn test_unc_path_rejected() {
        // Backslash prefix indicates UNC on Windows
        let result = validate_catalog_relative_path("\\\\server\\share");
        assert!(result.is_err());
    }

    #[test]
    fn test_backslash_prefix_rejected() {
        let result = validate_catalog_relative_path("\\foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_unicode_filename_allowed() {
        let result = validate_catalog_relative_path("中文/文件.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_spaces_in_filename_allowed() {
        let result = validate_catalog_relative_path("my folder/my file.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_component() {
        let result = validate_catalog_relative_path("file.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "file.txt");
    }

    #[test]
    fn test_deeply_nested() {
        let result = validate_catalog_relative_path("a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn test_mixed_separators_normalized() {
        let result = validate_catalog_relative_path("a\\b/c\\d/file.txt");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "a/b/c/d/file.txt");
    }

    // ---- prepare_restore_file_target ----

    #[test]
    fn test_file_target_creates_parent_dirs() {
        let dir = std::env::temp_dir().join(format!("nuwa_test_m1_f_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = prepare_restore_file_target(&dir, "parent/child/file.txt");
        assert!(result.is_ok());
        let expected = dir.join("parent/child/file.txt");
        assert_eq!(result.unwrap(), expected);
        // Parent dir should exist
        assert!(dir.join("parent/child").exists());
        // File should NOT exist yet
        assert!(!expected.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_target_flat_path() {
        let dir = std::env::temp_dir().join(format!("nuwa_test_m1_f2_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Single component has no parent dir to create
        let result = prepare_restore_file_target(&dir, "file.txt");
        assert!(result.is_ok());
        let expected = dir.join("file.txt");
        assert_eq!(result.unwrap(), expected);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_target_rejects_traversal() {
        let dir = std::env::temp_dir().join(format!("nuwa_test_m1_f3_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = prepare_restore_file_target(&dir, "../escape.txt");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_target_rejects_symlink_ancestor() {
        let dir = std::env::temp_dir().join(format!("nuwa_test_m1_f4_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Create a real dir and a symlink to it
        let real_dir = dir.join("realdir");
        fs::create_dir(&real_dir).unwrap();
        let link_dir = dir.join("linkdir");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_dir, &link_dir).unwrap();
        #[cfg(windows)]
        // On Windows, directory symlinks require specific privileges
        // Try to create but skip if not supported
        if std::os::windows::fs::symlink_dir(&real_dir, &link_dir).is_err() {
            let _ = fs::remove_dir_all(&dir);
            return; // Skip on systems without symlink permission
        }

        let result = prepare_restore_file_target(&dir, "linkdir/child/file.txt");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    // ---- prepare_restore_directory ----

    #[test]
    fn test_directory_target_creates_all_dirs() {
        let dir = std::env::temp_dir().join(format!("nuwa_test_m1_d_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = prepare_restore_directory(&dir, "a/b/c");
        assert!(result.is_ok());
        let expected = dir.join("a/b/c");
        assert_eq!(result.unwrap(), expected);
        // All dirs should exist
        assert!(expected.exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_directory_target_already_exists() {
        let dir = std::env::temp_dir().join(format!("nuwa_test_m1_d2_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::create_dir(dir.join("existing")).unwrap();

        let result = prepare_restore_directory(&dir, "existing");
        assert!(result.is_ok());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_directory_target_rejects_traversal() {
        let dir = std::env::temp_dir().join(format!("nuwa_test_m1_d3_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let result = prepare_restore_directory(&dir, "../escape");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_directory_target_rejects_symlink_ancestor() {
        let dir = std::env::temp_dir().join(format!("nuwa_test_m1_d4_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let real_dir = dir.join("realdir");
        fs::create_dir(&real_dir).unwrap();
        let link_dir = dir.join("linkdir");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&real_dir, &link_dir).unwrap();
        #[cfg(windows)]
        if std::os::windows::fs::symlink_dir(&real_dir, &link_dir).is_err() {
            let _ = fs::remove_dir_all(&dir);
            return;
        }

        let result = prepare_restore_directory(&dir, "linkdir/child");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(&dir);
    }
}
