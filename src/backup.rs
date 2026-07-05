// ============================================================================
// backup.rs -- Backup executor (no walkdir dependency, manual recursive traversal)
// ============================================================================

use crate::errors::NuwaError;
use crate::manifest::{CompressionConfig, DirectoryEntry, Manifest};
use crate::storage::BackupStorage;
use std::fs;
use std::path::{Path, PathBuf};

const SAFETY_MARGIN_RATIO: f64 = 0.10;

// Manual recursive directory traversal (replaces walkdir)
// Design principles:
// - Recursively processes subdirectories
// - Skips symbolic links (reason: linked files may be outside backup scope)
// - Returns flat file list and directory list for subsequent processing
fn walk_dir(dir: &Path, _base: &Path) -> Result<(Vec<PathBuf>, Vec<PathBuf>), NuwaError> {
    let mut files = Vec::new();
    let mut dirs = Vec::new();
    for entry in fs::read_dir(dir).map_err(|e| NuwaError::Io {
        source: Some(e),
        path: Some(dir.to_path_buf()),
        detail: format!("Cannot read directory '{}'", dir.display()),
        suggestion: "Check permissions and path".to_string(),
    })? {
        let entry = entry.map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(dir.to_path_buf()),
            detail: "Failed to read directory entry".to_string(),
            suggestion: "Insufficient permissions".to_string(),
        })?;
        let path = entry.path();
        let ft = entry.file_type().map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(path.clone()),
            detail: "Cannot get file type".to_string(),
            suggestion: "File may have been deleted".to_string(),
        })?;
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            dirs.push(path.clone());
            let (sf, sd) = walk_dir(&path, _base)?;
            files.extend(sf);
            dirs.extend(sd);
        } else if ft.is_file() {
            files.push(path);
        }
    }
    Ok((files, dirs))
}

fn to_relative(path: &Path, base: &Path) -> Result<String, NuwaError> {
    let r = path.strip_prefix(base).map_err(|_| NuwaError::General {
        detail: "Path prefix stripping failed".to_string(),
        suggestion: "Internal error".to_string(),
    })?;
    Ok(r.to_string_lossy().to_string().replace('\\', "/"))
}

fn calc_size(source: &Path) -> Result<u64, NuwaError> {
    let (files, _) = walk_dir(source, source)?;
    let mut total = 0u64;
    for f in &files {
        total += fs::metadata(f)?.len();
    }
    Ok(total)
}

/// Generate backup directory name: YYYYMMDD_HHMMSS_<source_dir_name>
//
// Format notes:
// - Timestamp uses local time so users can identify backup time by name
// - Source directory name is the last component of the path
// - If a collision occurs (two backups in the same second), appends millisecond suffix
pub fn generate_backup_dir_name(source: &Path) -> String {
    let ts = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let name = source
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "root".to_string());
    format!("{}_{}", ts, name)
}

/// Execute full backup
//
// Flow:
// 1. Verify source path exists
// 2. Reject source path = destination path (prevents circular reference)
// 3. Check destination path is writable
// 4. Calculate total data size (for space estimation)
// 5. Create backup storage structure
// 6. Traverse source directory, copy files to backup storage
// 7. Write JSON Manifest
//
// Safety design:
// - Atomic writes: each file is written as .tmp first, then renamed
// - Manifest is also written atomically after completion
// - Backup interruption does not corrupt existing backups
pub fn execute_backup(
    source: &Path,
    dest_root: &Path,
    compress: bool,
) -> Result<String, NuwaError> {
    // ===== Pre-check: source path must exist =====
    // Must execute before any canonicalize or write operations
    if !source.exists() {
        return Err(NuwaError::source_not_found(source));
    }

    // ===== Path containment check (zero side effects) =====
    // Goal: Determine containment without creating the destination directory.
    // Approach: canonicalize_partial for destination -- canonicalize the nearest
    // existing ancestor directory, then append remaining path components.
    // This resolves 8.3 short filenames and unifies \\?\ prefixes
    // without any filesystem side effects.
    fn canonicalize_partial(path: &Path) -> Result<PathBuf, NuwaError> {
        match std::fs::canonicalize(path) {
            Ok(p) => Ok(p),
            Err(_) => {
                let mut components: Vec<&std::ffi::OsStr> = Vec::new();
                let mut current = path;
                loop {
                    match current.file_name() {
                        Some(name) => {
                            components.push(name);
                            current = current.parent().unwrap_or_else(|| std::path::Path::new(""));
                            // Try to canonicalize the remaining ancestor path
                            if let Ok(base) = std::fs::canonicalize(current) {
                                let mut result = base;
                                for comp in components.iter().rev() {
                                    result.push(comp);
                                }
                                return Ok(result);
                            }
                        }
                        None => {
                            return Err(NuwaError::InvalidArgument {
                                detail: format!(
                                    "Cannot resolve path '{}': no existing ancestor directory found",
                                    path.display()
                                ),
                                suggestion: "Please verify the path format".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    let src_canon = fs::canonicalize(source)?;
    let dst_canon = canonicalize_partial(dest_root)?;

    if src_canon == dst_canon {
        return Err(NuwaError::same_source_dest(source, dest_root));
    }

    // ===== Path containment check =====
    // Use canonicalized paths (prefix unified, short names resolved) for Path::starts_with comparison
    if dst_canon.starts_with(&src_canon) {
        return Err(NuwaError::SafetyViolation {
            detail: format!(
                "Destination path is inside the source path. Operation blocked.\n  Source: {}\n  Dest:   {}",
                src_canon.display(),
                dst_canon.display()
            ),
            suggestion: "Choose a destination outside the source directory. Consider using another drive or USB device.".to_string(),
        });
    }
    if src_canon.starts_with(&dst_canon) {
        return Err(NuwaError::SafetyViolation {
            detail: format!(
                "Source path is inside the destination path. Operation blocked.\n  Source: {}\n  Dest:   {}",
                src_canon.display(),
                dst_canon.display()
            ),
            suggestion: "Choose a destination outside the source directory. Consider using another drive or USB device.".to_string(),
        });
    }

    // ===== All safety checks passed, ensure destination root exists =====
    // Execute last to avoid residual directories if safety check fails
    if !dest_root.exists() {
        std::fs::create_dir_all(dest_root).map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(dest_root.to_path_buf()),
            detail: format!(
                "Cannot create backup destination directory '{}'",
                dest_root.display()
            ),
            suggestion: "Check that the destination path is valid and permissions are sufficient"
                .to_string(),
        })?;
    }

    // ===== Calculate total backup data size for space estimation =====
    let total = calc_size(source)?;
    // Reserve 10% safety margin
    let required = (total as f64 * (1.0 + SAFETY_MARGIN_RATIO)).ceil() as u64;

    // ===== Destination disk space check =====
    // Uses Win32 API GetDiskFreeSpaceExW on Windows for accurate check
    // Non-Windows platforms fall back to writability check
    // Returns error on insufficient space -- no partial backup
    crate::diskspace::check_disk_space(dest_root, required)?;

    let dir_name = generate_backup_dir_name(source);
    let store = BackupStorage::new(dest_root.to_path_buf(), dir_name.clone());
    // If directory name conflicts, append millisecond suffix
    let final_name = if store.backup_dir().exists() {
        format!("{}_{}", dir_name, chrono::Utc::now().format("%S%f"))
    } else {
        dir_name
    };
    let store = BackupStorage::new(dest_root.to_path_buf(), final_name.clone());
    store.create_backup_dirs()?;

    // ===== Check if compression feature is available =====
    // --compress flag only works when the compress feature is compiled in
    // If not enabled but user specified --compress, return a clear error
    #[cfg(not(feature = "compress"))]
    if compress {
        return Err(NuwaError::InvalidArgument {
            detail: "Compression feature is not enabled. The current binary was not compiled with the compress feature. The --compress flag cannot be used.".to_string(),
            suggestion: "Use a version compiled with compress support (e.g. cargo build --features compress), or remove the --compress flag.".to_string(),
        });
    }

    // ===== Initialize Manifest =====
    let mut manifest = Manifest::new(
        uuid::Uuid::new_v4().to_string(),
        src_canon.to_string_lossy().to_string(),
        CompressionConfig {
            enabled: compress,
            algorithm: if compress {
                Some("zstd".to_string())
            } else {
                None
            },
        },
    );

    // ===== Traverse source directory and copy all files =====
    let (files, dirs) = walk_dir(source, source)?;
    for d in &dirs {
        manifest.add_directory(DirectoryEntry {
            relative_path: to_relative(d, source)?,
        });
    }
    for f in &files {
        let rel = to_relative(f, source)?;
        let fe = store.copy_file_with_atomic_write(f, &rel, compress)?;
        manifest.add_file(fe);
    }

    // ===== Atomic write Manifest =====
    store.write_manifest(&manifest)?;
    Ok(final_name)
}
