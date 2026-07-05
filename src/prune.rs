// ============================================================================
// prune.rs -- Retention policy / prune module
//
// Responsibilities:
// 1. Implement keep-count and keep-days retention policies
// 2. Execute safe deletion of old backup points
// 3. Support dry-run mode for preview before actual deletion
// 4. Auto-prune after backup when job has retention config
//
// Design principles:
// - manifest.json is the sole authority for backup point validity
// - Damaged manifests are never treated as valid backup points
// - The last valid backup point is never deleted
// - Deletion is directory-level, file by file inside the backup point
// - Dry-run never modifies any data
// ============================================================================

use crate::errors::NuwaError;
use crate::list;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Result of a prune operation
#[derive(Debug, Clone)]
pub struct PruneResult {
    /// How many backup points were deleted
    pub deleted_count: usize,
    /// How many backup points were kept
    pub kept_count: usize,
    /// How many backup points had damaged manifests
    pub damaged_count: usize,
    /// Backup IDs of deleted backup points
    pub deleted_backup_ids: Vec<String>,
    /// Directory names of deleted backup points
    pub deleted_dir_names: Vec<String>,
    /// Backup IDs of kept backup points
    pub kept_backup_ids: Vec<String>,
    /// Warnings encountered during prune
    pub warnings: Vec<String>,
}

/// Execute prune operation
///
/// ## Parameters
/// * `dest_path` -- Backup destination root directory
/// * `keep_count` -- Keep last N valid backup points (None = no count limit)
/// * `keep_days` -- Keep backup points from last N days (None = no days limit)
/// * `dry_run` -- If true, only report what would be deleted without deleting
///
/// ## Retention rule (when both keep_count and keep_days are specified):
/// A backup point is kept if it satisfies EITHER rule (union).
/// This is the safer choice -- more backups are retained than either rule alone.
///
/// ## Safety rules
/// 1. Dry-run never deletes any files
/// 2. The last valid backup point is always kept
/// 3. Deletion happens only within the backup destination directory
/// 4. Damaged manifest backup points cannot be reliably pruned
pub fn execute_prune(
    dest_path: &Path,
    keep_count: Option<u32>,
    keep_days: Option<u64>,
    dry_run: bool,
) -> Result<PruneResult, NuwaError> {
    if !dest_path.exists() {
        return Err(NuwaError::InvalidArgument {
            detail: format!(
                "Backup destination does not exist: '{}'",
                dest_path.display()
            ),
            suggestion: "Please verify the backup destination path".to_string(),
        });
    }

    if !dest_path.is_dir() {
        return Err(NuwaError::InvalidArgument {
            detail: format!(
                "Backup destination is not a directory: '{}'",
                dest_path.display()
            ),
            suggestion: "Please enter a backup root directory path".to_string(),
        });
    }

    let summaries = list::execute_list(dest_path)?;

    let mut valid: Vec<&list::BackupPointSummary> = Vec::new();
    let mut damaged_count: usize = 0;

    for s in summaries.iter() {
        if s.manifest_ok {
            valid.push(s);
        } else {
            damaged_count += 1;
        }
    }

    if valid.is_empty() {
        let mut warnings = Vec::new();
        if damaged_count > 0 {
            warnings.push(format!(
                "{} backup point(s) have damaged manifests and cannot be pruned.",
                damaged_count
            ));
        }
        return Ok(PruneResult {
            deleted_count: 0,
            kept_count: 0,
            damaged_count,
            deleted_backup_ids: Vec::new(),
            deleted_dir_names: Vec::new(),
            kept_backup_ids: Vec::new(),
            warnings,
        });
    }

    valid.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    let now = SystemTime::now();
    let mut keep_set: Vec<bool> = vec![false; valid.len()];

    for (idx, summary) in valid.iter().enumerate() {
        let mut should_keep = false;

        if let Some(kc) = keep_count {
            if (idx as u32) < kc {
                should_keep = true;
            }
        }

        if let Some(kd) = keep_days {
            if let Ok(age_days) = days_since_backup(&summary.created_at, now) {
                if age_days <= kd {
                    should_keep = true;
                }
            }
        }

        if keep_count.is_none() && keep_days.is_none() {
            should_keep = true;
        }

        keep_set[idx] = should_keep;
    }

    // Safety: always keep the newest valid backup point
    if !valid.is_empty() {
        keep_set[0] = true;
    }

    let mut final_keep: Vec<&list::BackupPointSummary> = Vec::new();
    let mut final_delete: Vec<&list::BackupPointSummary> = Vec::new();

    for (idx, summary) in valid.iter().enumerate() {
        if keep_set[idx] {
            final_keep.push(summary);
        } else {
            final_delete.push(summary);
        }
    }

    let mut result = PruneResult {
        deleted_count: 0,
        kept_count: final_keep.len(),
        damaged_count,
        deleted_backup_ids: Vec::new(),
        deleted_dir_names: Vec::new(),
        kept_backup_ids: final_keep.iter().map(|s| s.backup_id.clone()).collect(),
        warnings: Vec::new(),
    };

    if damaged_count > 0 {
        result.warnings.push(format!(
            "{} backup point(s) have damaged manifests and were skipped.",
            damaged_count
        ));
    }

    for summary in &final_delete {
        let backup_path = PathBuf::from(&summary.full_path);
        if dry_run {
            result.deleted_count += 1;
            result.deleted_backup_ids.push(summary.backup_id.clone());
            result.deleted_dir_names.push(summary.dir_name.clone());
        } else {
            match std::fs::remove_dir_all(&backup_path) {
                Ok(_) => {
                    result.deleted_count += 1;
                    result.deleted_backup_ids.push(summary.backup_id.clone());
                    result.deleted_dir_names.push(summary.dir_name.clone());
                }
                Err(e) => {
                    result.warnings.push(format!(
                        "Failed to delete backup point '{}': {}",
                        summary.dir_name, e
                    ));
                }
            }
        }
    }

    Ok(result)
}

fn days_since_backup(created_at: &str, now: SystemTime) -> Result<u64, NuwaError> {
    let dt =
        chrono::DateTime::parse_from_rfc3339(created_at).map_err(|e| NuwaError::ManifestError {
            detail: format!("Cannot parse backup timestamp '{}': {}", created_at, e),
            suggestion: "This backup point has an invalid timestamp format".to_string(),
        })?;
    let backup_time = SystemTime::UNIX_EPOCH + Duration::from_secs(dt.timestamp() as u64);
    let age = now.duration_since(backup_time).unwrap_or(Duration::ZERO);
    Ok(age.as_secs() / 86400)
}

pub fn print_prune_result(result: &PruneResult) {
    if result.deleted_count == 0 && result.damaged_count == 0 {
        println!("No backup points to prune.");
        return;
    }
    println!("Prune summary:");
    if !result.deleted_dir_names.is_empty() {
        println!(
            "  Deleted backup points: {}",
            result.deleted_dir_names.len()
        );
        for name in &result.deleted_dir_names {
            println!("    - {}", name);
        }
    }
    if !result.kept_backup_ids.is_empty() {
        println!("  Kept backup points: {}", result.kept_backup_ids.len());
    }
    if result.damaged_count > 0 {
        println!("  Damaged (skipped): {}", result.damaged_count);
    }
    for warning in &result.warnings {
        println!("  Warning: {}", warning);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{CompressionConfig, DirectoryEntry, FileEntry, Manifest};
    use std::fs;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn unique_temp_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("nuwa_prune_test_{}_{}", std::process::id(), n))
    }

    fn create_backup_point(
        dest: &Path,
        dir_name: &str,
        backup_id: &str,
        created_at: &str,
    ) -> PathBuf {
        let bp_path = dest.join(dir_name);
        fs::create_dir_all(bp_path.join("files")).unwrap();
        let manifest = Manifest {
            schema_version: "1.0".to_string(),
            backup_id: backup_id.to_string(),
            created_at: created_at.to_string(),
            source_root: "C:\\test".to_string(),
            storage_format: "flat-file".to_string(),
            compression: CompressionConfig {
                enabled: false,
                algorithm: None,
            },
            files: vec![FileEntry {
                relative_path: "test.txt".to_string(),
                size_bytes: 100,
                modified_time: "2026-01-01T00:00:00+00:00".to_string(),
                sha256: "abc".to_string(),
                stored_path: "test.txt".to_string(),
            }],
            directories: vec![DirectoryEntry {
                relative_path: String::new(),
            }],
            summary: crate::manifest::BackupSummary {
                file_count: 1,
                directory_count: 1,
                total_bytes: 100,
            },
        };
        let json = serde_json::to_string_pretty(&manifest).unwrap();
        fs::write(bp_path.join("manifest.json"), &json).unwrap();
        fs::write(bp_path.join("files").join("test.txt"), b"hello").unwrap();
        bp_path
    }

    fn create_damaged_backup_point(dest: &Path, dir_name: &str) -> PathBuf {
        let bp_path = dest.join(dir_name);
        fs::create_dir_all(bp_path.join("files")).unwrap();
        fs::write(bp_path.join("manifest.json"), b"corrupted json").unwrap();
        fs::write(bp_path.join("files").join("test.txt"), b"hello").unwrap();
        bp_path
    }

    fn create_backup_points_for_retention(
        dest: &Path,
        start_days_ago: u64,
        count: u32,
        days_interval: u64,
    ) -> Vec<String> {
        let now = chrono::Utc::now();
        let mut ids = Vec::new();
        for i in 0..count {
            let age_seconds = (start_days_ago - (i as u64 * days_interval)) * 86400;
            let bp_time = now - chrono::Duration::seconds(age_seconds as i64);
            let dir_name = format!("backup_{}_{}", bp_time.format("%Y%m%d_%H%M%S"), i);
            let backup_id = format!("uuid-{}", i);
            create_backup_point(dest, &dir_name, &backup_id, &bp_time.to_rfc3339());
            ids.push(backup_id);
        }
        ids
    }

    fn cleanup_temp_dir(dir: &Path) {
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn test_keep_count_retains_correct_number() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        create_backup_points_for_retention(&dir, 10, 5, 2);
        let result = execute_prune(&dir, Some(3), None, false).unwrap();
        assert_eq!(result.deleted_count, 2);
        assert_eq!(result.kept_count, 3);
        // uuid-4 is newest (2d ago), uuid-3 is 2nd newest (4d ago) -- both kept
        // uuid-1 (8d ago) and uuid-0 (10d ago) should be deleted
        assert!(!result.kept_backup_ids.contains(&"uuid-1".to_string()));
        assert!(!result.kept_backup_ids.contains(&"uuid-0".to_string()));
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_keep_count_all_retained() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        create_backup_points_for_retention(&dir, 10, 3, 2);
        let result = execute_prune(&dir, Some(10), None, false).unwrap();
        assert_eq!(result.deleted_count, 0);
        assert_eq!(result.kept_count, 3);
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_keep_days_retains_recent() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        create_backup_points_for_retention(&dir, 30, 5, 7);
        let result = execute_prune(&dir, None, Some(14), false).unwrap();
        assert_eq!(result.kept_count, 2);
        assert_eq!(result.deleted_count, 3);
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_dry_run_does_not_delete() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        create_backup_points_for_retention(&dir, 10, 5, 2);
        let result = execute_prune(&dir, Some(3), None, true).unwrap();
        assert_eq!(result.deleted_count, 2);
        assert_eq!(result.kept_count, 3);
        let entries = fs::read_dir(&dir).unwrap().count();
        assert_eq!(entries, 5); // 5 backup dirs, no .nuwa_history.db yet
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_last_backup_never_deleted() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        create_backup_point(
            &dir,
            "only_backup",
            "uuid-single",
            "2026-01-01T00:00:00+00:00",
        );
        let result = execute_prune(&dir, Some(0), None, false).unwrap();
        assert_eq!(result.deleted_count, 0);
        assert_eq!(result.kept_count, 1);
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_damaged_manifest_not_valid() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        create_backup_point(
            &dir,
            "good_backup",
            "uuid-good",
            "2026-01-01T00:00:00+00:00",
        );
        create_damaged_backup_point(&dir, "damaged_backup");
        let result = execute_prune(&dir, Some(1), None, false).unwrap();
        assert_eq!(result.damaged_count, 1);
        assert_eq!(result.kept_count, 1);
        assert!(dir.join("damaged_backup").exists());
        assert!(dir.join("good_backup").exists());
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_dry_run_no_side_effects() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        create_backup_points_for_retention(&dir, 10, 5, 2);
        create_damaged_backup_point(&dir, "damaged_1");
        let result = execute_prune(&dir, Some(2), None, true).unwrap();
        assert_eq!(result.deleted_count, 3);
        assert_eq!(result.damaged_count, 1);
        let count = fs::read_dir(&dir).unwrap().count();
        assert_eq!(count, 6); // 5 valid + 1 damaged, no .nuwa_history.db yet
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_prune_does_not_delete_dest_root() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        let result = execute_prune(&dir, Some(5), None, false).unwrap();
        assert_eq!(result.deleted_count, 0);
        assert!(dir.exists());
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_prune_with_both_count_and_days() {
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        create_backup_points_for_retention(&dir, 30, 6, 5);
        let result = execute_prune(&dir, Some(4), Some(15), false).unwrap();
        assert_eq!(result.kept_count, 4);
        assert_eq!(result.deleted_count, 2);
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_json_output_structure() {
        use crate::cli_output;
        let dir = unique_temp_dir();
        fs::create_dir_all(&dir).unwrap();
        create_backup_points_for_retention(&dir, 10, 5, 2);
        let result = execute_prune(&dir, Some(3), None, false).unwrap();
        let mut out = cli_output::JsonOutput::success("prune", "Prune completed", 0);
        out.deleted_count = Some(result.deleted_count as u64);
        out.kept_count = Some(result.kept_count as u64);
        out.damaged_count = Some(result.damaged_count as u64);
        let json = serde_json::to_string(&out).unwrap();
        assert!(json.contains("\"prune\""));
        assert!(json.contains("\"deleted_count\""));
        assert!(json.contains("\"kept_count\""));
        assert!(json.contains("\"damaged_count\""));
        cleanup_temp_dir(&dir);
    }

    #[test]
    fn test_prune_does_not_delete_outside_dest() {
        // Create a sentinel file outside the backup dest directory
        let sentinel_dir = unique_temp_dir();
        fs::create_dir_all(&sentinel_dir).unwrap();
        let sentinel_file = sentinel_dir.join("sentinel.txt");
        fs::write(&sentinel_file, b"do not delete").unwrap();

        // Create backup points inside dest
        let dest = unique_temp_dir();
        fs::create_dir_all(&dest).unwrap();
        create_backup_points_for_retention(&dest, 10, 5, 2);

        // Run prune with aggressive retention
        let result = execute_prune(&dest, Some(2), None, false).unwrap();
        assert_eq!(result.deleted_count, 3);
        assert_eq!(result.kept_count, 2);

        // Sentinel file must still exist
        assert!(
            sentinel_file.exists(),
            "Sentinel file outside dest was deleted!"
        );
        assert_eq!(fs::read_to_string(&sentinel_file).unwrap(), "do not delete");

        // Clean up both dirs
        let _ = fs::remove_dir_all(&sentinel_dir);
        cleanup_temp_dir(&dest);
    }
}
