// ============================================================================
// list.rs -- Nuwa Backup backup point listing
//
// Design principles:
// 1. Scan all backup point directories in the destination path
// 2. Read manifest.json from each backup point for metadata
// 3. Sort by time descending (newest first)
// 4. Display for each backup point: time, source path, file count, total size
// 5. For backup points with corrupted manifests, still list them and mark as "abnormal"
//    -- Reason: even a corrupted backup point lets users know "this backup existed"
//    -- But clearly prompt users to run verify for checking
// ============================================================================

use crate::errors::NuwaError;
use crate::manifest::Manifest;
use std::path::Path;

/// Summary information for a single backup point
pub struct BackupPointSummary {
    /// Backup directory name
    pub dir_name: String,
    /// Full path to backup directory
    pub full_path: String,
    /// Backup ID (UUID)
    pub backup_id: String,
    /// Backup creation time
    pub created_at: String,
    /// Source path
    pub source_root: String,
    /// Number of files
    pub file_count: u64,
    /// Total size (bytes)
    pub total_bytes: u64,
    /// Whether the manifest is readable
    pub manifest_ok: bool,
}

/// Execute list operation
///
/// ## Parameters
/// * dest_path -- Backup destination root path
///
/// ## Flow
/// 1. Traverse all subdirectories in the destination root path
/// 2. Attempt to read manifest.json from each subdirectory
/// 3. Aggregate all backup point information
///
/// ## Returns
/// * Vec<BackupPointSummary> -- List of all backup point information
pub fn execute_list(dest_path: &Path) -> Result<Vec<BackupPointSummary>, NuwaError> {
    // ===== Check if destination path exists =====
    if !dest_path.exists() {
        return Err(NuwaError::InvalidArgument {
            detail: format!("Destination path does not exist: '{}'", dest_path.display()),
            suggestion:
                "Please verify the backup destination path. Use an absolute path, e.g.: nuwa list --dest D:\\Backup"
                    .to_string(),
        });
    }

    if !dest_path.is_dir() {
        return Err(NuwaError::InvalidArgument {
            detail: format!(
                "Destination path is not a directory: '{}'",
                dest_path.display()
            ),
            suggestion: "Enter the backup root directory path, not a file path".to_string(),
        });
    }

    let mut summaries: Vec<BackupPointSummary> = Vec::new();

    // ===== Traverse all entries in the destination root path =====
    let entries = std::fs::read_dir(dest_path).map_err(|e| NuwaError::Io {
        source: Some(e),
        path: Some(dest_path.to_path_buf()),
        detail: "Cannot read destination directory".to_string(),
        suggestion: "Check directory permissions and path".to_string(),
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| NuwaError::Io {
            source: Some(e),
            path: Some(dest_path.to_path_buf()),
            detail: "Error while traversing directory".to_string(),
            suggestion: "A subdirectory may have insufficient permissions".to_string(),
        })?;

        let path = entry.path();

        // Only process subdirectories (each backup point is a directory)
        if !path.is_dir() {
            continue;
        }

        let dir_name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden directories (starting with .)
        // Reason: avoid mistaking system or temp directories as backup points
        if dir_name.starts_with('.') {
            continue;
        }

        // ===== Attempt to read Manifest =====
        let manifest_path = path.join("manifest.json");
        let summary = if manifest_path.exists() {
            match Manifest::from_file(&manifest_path) {
                Ok(manifest) => BackupPointSummary {
                    dir_name: dir_name.clone(),
                    full_path: path.to_string_lossy().to_string(),
                    backup_id: manifest.backup_id,
                    created_at: manifest.created_at,
                    source_root: manifest.source_root,
                    file_count: manifest.summary.file_count,
                    total_bytes: manifest.summary.total_bytes,
                    manifest_ok: true,
                },
                Err(_) => {
                    // Manifest corrupted -- still list but mark as abnormal
                    BackupPointSummary {
                        dir_name: dir_name.clone(),
                        full_path: path.to_string_lossy().to_string(),
                        backup_id: "unknown".to_string(),
                        created_at: "unknown".to_string(),
                        source_root: "unknown (manifest corrupted)".to_string(),
                        file_count: 0,
                        total_bytes: 0,
                        manifest_ok: false,
                    }
                }
            }
        } else {
            // No manifest.json -- skip non-backup directories
            continue;
        };

        summaries.push(summary);
    }

    // ===== Sort by time descending (newest first) =====
    // Reason: users typically check the most recent backup points first
    // Entries with unknown creation time are placed last
    summaries.sort_by(|a, b| {
        if a.manifest_ok && b.manifest_ok {
            b.created_at.cmp(&a.created_at)
        } else if b.manifest_ok {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });

    Ok(summaries)
}

/// Print backup point list to console
pub fn print_list(summaries: &[BackupPointSummary]) {
    if summaries.is_empty() {
        println!("No backup points found.");
        println!("Run 'nuwa backup --source <path> --dest <path>' to create a backup first.");
        return;
    }

    println!(
        "Backup points in destination (total: {}):\n",
        summaries.len()
    );

    for (i, s) in summaries.iter().enumerate() {
        if !s.manifest_ok {
            println!("  {}. ! [ABNORMAL] {}", i + 1, s.dir_name);
            println!("     Path: {}", s.full_path);
            println!("     Manifest corrupted. Run verify to check.\n");
            continue;
        }

        // Format file size
        let size_str = if s.total_bytes > 1024 * 1024 * 1024 {
            format!(
                "{:.2} GB",
                s.total_bytes as f64 / (1024.0 * 1024.0 * 1024.0)
            )
        } else if s.total_bytes > 1024 * 1024 {
            format!("{:.2} MB", s.total_bytes as f64 / (1024.0 * 1024.0))
        } else if s.total_bytes > 1024 {
            format!("{:.2} KB", s.total_bytes as f64 / 1024.0)
        } else {
            format!("{} B", s.total_bytes)
        };

        println!("  {}. {}", i + 1, s.dir_name);
        println!("     Backup time: {}", s.created_at);
        println!("     Source:      {}", s.source_root);
        println!("     Files: {} | Total size: {}", s.file_count, size_str);
        println!("     Backup ID:   {}", s.backup_id);
        println!();
    }
}
