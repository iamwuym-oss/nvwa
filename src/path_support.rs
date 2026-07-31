// ============================================================================
// path_support.rs -- UNC / SMB path support for Windows backup destinations
//
// Responsibilities:
// 1. Detect and validate UNC paths (\\server\share\...)
// 2. Classify paths as LocalPath, UncPath, or Unsupported
// 3. Preflight accessibility check for backup/restore destinations
// 4. Validate repository paths for safety
//
// Design principles:
// - Works with Rust standard Path/PathBuf (no external dependencies)
// - Handles raw string paths passed from CLI or config
// - Does not save, manage, or validate SMB credentials
// - Uses current Windows logon session for SMB access only
// - Does not create or delete mapped drives
// - UNC paths are validated syntactically; accessibility depends on network
// ============================================================================

use crate::errors::NuwaError;
use std::path::Path;

/// Represents the classification of a user-supplied path
#[derive(Debug, Clone, PartialEq)]
pub enum PathType {
    /// Local absolute path (e.g., C:\Users\Data, D:\Backup)
    LocalPath,
    /// UNC network path (e.g., \\server\share\folder)
    UncPath,
    /// Unsupported path type (relative, device, etc.)
    Unsupported,
}

/// Detect whether a path string starts with a UNC prefix (double backslash)
///
/// Works on the raw string representation because Path canonicalization
/// can alter or fail on inaccessible UNC paths.
pub fn detect_unc_path(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.len() < 3 {
        return false;
    }
    let bytes = trimmed.as_bytes();
    bytes[0] == b'\\' && bytes[1] == b'\\' && bytes[2] != b'\\'
}

/// Validate a UNC path string for structural correctness
///
/// Acceptable forms:
///   \\server\share
///   \\server\share\folder
///   \\server\share\folder\subfolder
///
/// Rejected forms:
///   \\server                          (no share)
///   \\                                (empty server/share)
///   \\\server\share                   (triple backslash)
pub fn validate_unc_path(raw: &str) -> Result<String, NuwaError> {
    if !detect_unc_path(raw) {
        return Err(NuwaError::InvalidArgument {
            detail: format!("Path is not a valid UNC path: '{}'", raw),
            suggestion: "UNC paths must start with \\\\server\\share".to_string(),
        });
    }

    let trimmed = raw.trim().to_string();
    let parts: Vec<&str> = trimmed[2..].split('\\').collect();

    if parts.is_empty() || parts[0].is_empty() {
        return Err(NuwaError::InvalidArgument {
            detail: "UNC path has empty server name".to_string(),
            suggestion: "A UNC path must have a server name: \\\\server\\share".to_string(),
        });
    }

    if parts.len() < 2 || parts[1].is_empty() {
        return Err(NuwaError::InvalidArgument {
            detail: format!("UNC path '{}' has no share name", raw),
            suggestion: "A UNC path must include a share: \\\\server\\share".to_string(),
        });
    }

    Ok(trimmed)
}

/// Classify a user-supplied path (raw string or Path) into its path type
pub fn classify_path(path: &Path) -> PathType {
    let raw = path.to_string_lossy();
    if detect_unc_path(&raw) {
        PathType::UncPath
    } else if path.is_absolute() {
        PathType::LocalPath
    } else {
        PathType::Unsupported
    }
}

/// Validate a backup or restore repository destination path
///
/// Accepts both local absolute paths and UNC network paths.
/// Rejects relative paths and other unsupported path types.
pub fn validate_repository_path(path: &Path) -> Result<(), NuwaError> {
    let raw = path.to_string_lossy();

    if raw.trim().is_empty() {
        return Err(NuwaError::InvalidArgument {
            detail: "Destination path is empty".to_string(),
            suggestion: "Provide a valid backup destination path".to_string(),
        });
    }

    if detect_unc_path(&raw) {
        validate_unc_path(&raw)?;
        return Ok(());
    }

    if !path.is_absolute() {
        return Err(NuwaError::InvalidArgument {
            detail: format!(
                "Destination path is not absolute: '{}'. UNC and absolute local paths are supported",
                raw
            ),
            suggestion: "Use a full path (e.g., C:\\Backup or \\\\server\\share)".to_string(),
        });
    }

    Ok(())
}

/// Preflight check: verify the destination path is accessible and writable
pub fn preflight_dest_check(path: &Path) -> Result<(), NuwaError> {
    let raw = path.to_string_lossy();

    if path.exists() {
        if !path.is_dir() {
            return Err(NuwaError::Io {
                source: None,
                path: Some(path.to_path_buf()),
                detail: format!("Destination exists but is not a directory: '{}'", raw),
                suggestion: "Choose a directory path, not a file path".to_string(),
            });
        }

        let metadata = std::fs::metadata(path).map_err(|e| {
            let err_msg = e.to_string();
            NuwaError::Io {
                source: Some(e),
                path: Some(path.to_path_buf()),
                detail: format!("Cannot access destination '{}': {}", raw, err_msg),
                suggestion: "Check that the path exists and you have permission to access it"
                    .to_string(),
            }
        })?;
        if metadata.permissions().readonly() {
            return Err(NuwaError::Io {
                source: None,
                path: Some(path.to_path_buf()),
                detail: format!("Destination '{}' is read-only", raw),
                suggestion: "Remove the read-only attribute or choose a different destination"
                    .to_string(),
            });
        }

        return Ok(());
    }

    // Destination does not exist yet -- check parent
    if let Some(parent) = path.parent() {
        if parent.exists() {
            if !parent.is_dir() {
                return Err(NuwaError::Io {
                    source: None,
                    path: Some(parent.to_path_buf()),
                    detail: format!(
                        "Cannot create destination '{}': parent is not a directory",
                        raw
                    ),
                    suggestion: "Check the parent path".to_string(),
                });
            }
            return Ok(());
        }

        return Err(NuwaError::Io {
            source: None,
            path: Some(path.to_path_buf()),
            detail: format!(
                "Destination does not exist: '{}'. Parent directory does not exist either",
                raw
            ),
            suggestion: "Create the full directory path first, or use an existing directory"
                .to_string(),
        });
    }

    Err(NuwaError::Io {
        source: None,
        path: Some(path.to_path_buf()),
        detail: format!(
            "Cannot determine parent directory for destination '{}'",
            raw
        ),
        suggestion: "Check the destination path".to_string(),
    })
}

/// Preflight check for read-only operations (list, verify, history, prune)
pub fn preflight_readonly_check(path: &Path) -> Result<(), NuwaError> {
    let raw = path.to_string_lossy();
    validate_repository_path(path)?;

    if !path.exists() {
        return Err(NuwaError::Io {
            source: None,
            path: Some(path.to_path_buf()),
            detail: format!("Backup repository not found: '{}'", raw),
            suggestion: "Check that the path is correct and the backup repository exists"
                .to_string(),
        });
    }

    if !path.is_dir() {
        return Err(NuwaError::Io {
            source: None,
            path: Some(path.to_path_buf()),
            detail: format!("Backup repository path is not a directory: '{}'", raw),
            suggestion: "Provide a directory path that contains backup points".to_string(),
        });
    }

    path.read_dir().map_err(|e| NuwaError::Io {
        source: Some(e),
        path: Some(path.to_path_buf()),
        detail: format!("Cannot read backup repository: '{}'", raw),
        suggestion: "Check that you have permission to access the directory".to_string(),
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_standard_unc() {
        assert!(detect_unc_path(r"\\server\share"));
        assert!(detect_unc_path(r"\\server\share\folder"));
        assert!(detect_unc_path(r"\\server\share\folder\subfolder"));
        assert!(detect_unc_path(r"\\server\share with spaces\backup repo"));
    }

    #[test]
    fn test_detect_not_unc() {
        assert!(!detect_unc_path(r"C:\Users"));
        assert!(!detect_unc_path(r"D:\Backup"));
        assert!(!detect_unc_path(r"relative\path"));
        assert!(!detect_unc_path(r""));
        assert!(!detect_unc_path(r"\"));
        assert!(!detect_unc_path(r"\\"));
    }

    #[test]
    fn test_detect_triple_backslash() {
        assert!(!detect_unc_path(r"\\\server\share"));
    }

    #[test]
    fn test_validate_unc_standard() {
        assert!(validate_unc_path(r"\\server\share").is_ok());
        assert!(validate_unc_path(r"\\server\share\folder").is_ok());
    }

    #[test]
    fn test_validate_unc_no_share() {
        assert!(validate_unc_path(r"\\server").is_err());
    }

    #[test]
    fn test_validate_unc_empty_local() {
        assert!(validate_unc_path(r"").is_err());
        assert!(validate_unc_path(r"C:\Users").is_err());
    }

    #[test]
    fn test_validate_unc_with_spaces() {
        assert!(validate_unc_path(r"\\server\shared folder\backup repo").is_ok());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_classify_local_absolute() {
        assert_eq!(classify_path(Path::new(r"C:\Users")), PathType::LocalPath);
        assert_eq!(
            classify_path(Path::new(r"D:\Backup\Folder")),
            PathType::LocalPath
        );
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_classify_local_absolute() {
        assert_eq!(classify_path(Path::new("/home/user")), PathType::LocalPath);
        assert_eq!(
            classify_path(Path::new("/var/backups/nuwa")),
            PathType::LocalPath
        );
    }

    #[test]
    fn test_classify_unc() {
        assert_eq!(
            classify_path(Path::new(r"\\server\share")),
            PathType::UncPath
        );
        assert_eq!(
            classify_path(Path::new(r"\\NAS\Backup\Data")),
            PathType::UncPath
        );
    }

    #[test]
    fn test_classify_unsupported() {
        assert_eq!(
            classify_path(Path::new(r"relative\path")),
            PathType::Unsupported
        );
        assert_eq!(classify_path(Path::new(r"")), PathType::Unsupported);
    }
    #[cfg(target_os = "windows")]
    #[test]
    fn test_validate_repo_local_absolute() {
        assert!(validate_repository_path(Path::new(r"C:\Backup")).is_ok());
        assert!(validate_repository_path(Path::new(r"D:\Data\Backup")).is_ok());
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn test_validate_repo_local_absolute() {
        assert!(validate_repository_path(Path::new("/home/user")).is_ok());
        assert!(validate_repository_path(Path::new("/var/backups/nuwa")).is_ok());
    }

    #[test]
    fn test_validate_repo_unc() {
        assert!(validate_repository_path(Path::new(r"\\server\share")).is_ok());
        assert!(validate_repository_path(Path::new(r"\\NAS\Backup\Folder")).is_ok());
    }

    #[test]
    fn test_validate_repo_relative() {
        assert!(validate_repository_path(Path::new(r"relative\path")).is_err());
        assert!(validate_repository_path(Path::new(r"backup")).is_err());
    }

    #[test]
    fn test_validate_repo_empty() {
        assert!(validate_repository_path(Path::new("")).is_err());
    }

    #[test]
    fn test_preflight_nonexistent_deep() {
        let path = Path::new(r"C:\_nuwa_test_should_not_exist\_deep");
        assert!(preflight_dest_check(path).is_err());
    }

    #[test]
    fn test_preflight_dest_not_a_directory() {
        let temp_dir = std::env::temp_dir();
        let file_path = temp_dir.join("_nuwa_path_test_file.txt");
        let _ = std::fs::write(&file_path, "test");
        if file_path.exists() {
            let result = preflight_dest_check(&file_path);
            assert!(result.is_err());
            let _ = std::fs::remove_file(&file_path);
        }
    }

    #[test]
    fn test_unc_pathbuf_preserves_backslashes() {
        let pb = PathBuf::from(r"\\server\share\backup");
        let display = pb.to_string_lossy().to_string();
        assert!(display.starts_with(r"\\"));
        assert!(display.contains("\\server"));
    }

    #[test]
    fn test_unc_path_with_spaces_preserved() {
        let pb = PathBuf::from(r"\\server\shared folder\backup repo");
        assert_eq!(pb.to_string_lossy(), r"\\server\shared folder\backup repo");
    }
}
