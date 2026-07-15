// ============================================================================
// file_browser_service.rs -- In-app file system browsing service
//
// Replaces OS-native directory dialog with N眉wa's own directory browser.
// Used by Settings (source/destination selection) and Restore (destination).
//
// This service is READ-ONLY. It only lists directories; never creates,
// modifies, or deletes anything on the filesystem.
//
// Future: This same service can be extended for Backup Content Browser
// (T2.5-04D) by adding restore-point-aware listing.
// ============================================================================

use crate::app::error::AppError;
use std::path::Path;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// A filesystem entry returned by the browser
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FsEntry {
    /// Display name (folder name or drive label)
    pub name: String,
    /// Full path
    pub path: String,
    /// Whether this is a directory (always true for current implementation)
    pub is_directory: bool,
}

/// Root-level filesystem entry (drive or special folder)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RootEntry {
    /// Display label (e.g. "Local Disk (C:)" or "D:")
    pub label: String,
    /// Root path (e.g. "C:\" or "D:\")
    pub path: String,
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List available drives/roots on the system.
///
/// On Windows, returns all available drive letters (A:\, B:\, etc.).
/// On other platforms, returns "/".
pub fn list_roots() -> Result<Vec<RootEntry>, AppError> {
    let mut drives = Vec::new();

    #[cfg(target_os = "windows")]
    {
        // Query all logical drives via the Win32 API through std::fs
        for letter in 'A'..='Z' {
            let root = format!("{}:\\", letter);
            let p = Path::new(&root);
            if p.exists() {
                let label = get_drive_label(&root);
                drives.push(RootEntry {
                    label: format!("{} ({})", label, root),
                    path: root,
                });
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        drives.push(RootEntry {
            label: "/ (Root)".into(),
            path: "/".into(),
        });
    }

    Ok(drives)
}

/// List child entries (directories) of a given directory path.
///
/// Returns sorted directory entries. Only directories are returned;
/// individual files are excluded because this is a directory picker.
pub fn list_directory(path: &str) -> Result<Vec<FsEntry>, AppError> {
    let dir_path = Path::new(path);

    if !dir_path.exists() {
        return Err(AppError::internal(format!("Path does not exist: {}", path)));
    }

    if !dir_path.is_dir() {
        return Err(AppError::internal(format!(
            "Path is not a directory: {}",
            path
        )));
    }

    let mut entries = Vec::new();

    let read_dir = dir_path
        .read_dir()
        .map_err(|e| AppError::permission(format!("Cannot read directory '{}': {}", path, e)))?;

    for entry in read_dir {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // skip entries we can't read
        };

        let file_type = match entry.file_type() {
            Ok(ft) => ft,
            Err(_) => continue,
        };

        if file_type.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            let full_path = entry.path().to_string_lossy().to_string();

            // Skip hidden / system directories on Windows
            if is_hidden_or_system(&entry) {
                continue;
            }

            entries.push(FsEntry {
                name,
                path: full_path,
                is_directory: true,
            });
        }
    }

    // Sort by name, case-insensitive
    entries.sort_by_key(|a| a.name.to_lowercase());

    Ok(entries)
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Get a human-readable label for a Windows drive.
#[cfg(target_os = "windows")]
fn get_drive_label(root: &str) -> String {
    // Try to read the volume label; fall back to "Local Disk"
    match std::process::Command::new("cmd")
        .args(["/c", "vol", root])
        .output()
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse "Volume in drive C is LABEL" from vol output
            if let Some(line) = stdout.lines().next() {
                let cleaned = line.trim();
                if let Some(idx) = cleaned.find(" is ") {
                    let label_part = &cleaned[idx + 4..].trim_end_matches('.');
                    if !label_part.is_empty() && !label_part.contains("Volume") {
                        return label_part.to_string();
                    }
                }
            }
            "Local Disk".to_string()
        }
        Err(_) => "Local Disk".to_string(),
    }
}

/// Check whether a directory entry should be hidden from the browser.
///
/// On Windows, hides system-protected directories (e.g. "System Volume Information").
#[cfg(target_os = "windows")]
fn is_hidden_or_system(entry: &std::fs::DirEntry) -> bool {
    {
        use std::os::windows::fs::MetadataExt;
        if let Ok(meta) = entry.metadata() {
            let attrs = meta.file_attributes();
            const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
            const FILE_ATTRIBUTE_SYSTEM: u32 = 0x4;
            if (attrs & FILE_ATTRIBUTE_HIDDEN) != 0 || (attrs & FILE_ATTRIBUTE_SYSTEM) != 0 {
                return true;
            }
        }
    }
    false
}

// Skip attribute constants on non-Windows
#[cfg(not(target_os = "windows"))]
fn is_hidden_or_system(entry: &std::fs::DirEntry) -> bool {
    let _ = entry;
    false
}
