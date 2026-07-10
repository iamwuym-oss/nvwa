// ============================================================================
// integrity_check.rs — S-13: Cross-component repository integrity check
// ============================================================================
//
// Performs four-level repository integrity verification.
// See Architecture v1.0 §13.1.
//
// Levels:
//   1. Repository metadata integrity  — repo.db schema, required keys
//   2. SQLite integrity              — PRAGMA integrity_check on all DBs
//   3. Restore point FS consistency  — backup-instances/ structural check
//   4. Block reference existence     — block-map db blocks exist in block-store
//
// All checks are READ-ONLY. No data is modified.

use crate::repository::repo_manager::RepoHandle;
use std::fs;
use std::path::PathBuf;

// ======== Check Status Types ========

/// Status of a single check item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CheckStatus {
    Pass,
    Warning,
    Error,
}

/// Result of a single check item
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub status: CheckStatus,
    pub message: String,
}

impl CheckResult {
    pub fn pass(message: impl Into<String>) -> Self {
        CheckResult {
            status: CheckStatus::Pass,
            message: message.into(),
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        CheckResult {
            status: CheckStatus::Warning,
            message: message.into(),
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        CheckResult {
            status: CheckStatus::Error,
            message: message.into(),
        }
    }
}

/// Detailed check item with component name
#[derive(Debug, Clone)]
pub struct CheckDetail {
    pub component: String,
    pub status: CheckStatus,
    pub message: String,
}

/// Summary of all checks
#[derive(Debug, Clone, Default)]
pub struct IntegritySummary {
    pub passed: u32,
    pub warnings: u32,
    pub errors: u32,
}

/// Full integrity report
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub repo_db_integrity: CheckResult,
    pub sqlite_integrity: CheckResult,
    pub restore_point_fs_consistency: CheckResult,
    pub block_reference_check: CheckResult,
    pub summary: IntegritySummary,
    pub details: Vec<CheckDetail>,
}

impl IntegrityReport {
    /// True if no Error-level findings
    pub fn is_healthy(&self) -> bool {
        self.summary.errors == 0
    }
}

/// Run full four-level integrity check on an open repository.
///
/// # Arguments
/// * `handle` — Opened RepoHandle
///
/// # Returns
/// IntegrityReport summarizing all findings.
/// This function never fails — all errors are captured as report entries.
pub fn check_integrity(handle: &RepoHandle) -> IntegrityReport {
    let mut details: Vec<CheckDetail> = Vec::new();

    // Level 1: Repository Metadata
    let repo_db_result = check_repo_db_integrity(handle, &mut details);

    // Level 2: SQLite Integrity
    let sqlite_result = check_sqlite_integrity(handle, &mut details);

    // Level 3: Restore Point FS Consistency
    let fs_result = check_restore_point_fs(handle, &mut details);

    // Level 4: Block Reference Existence
    let block_ref_result = check_block_references(handle, &mut details);

    // Build summary
    let mut summary = IntegritySummary::default();
    for d in &details {
        match d.status {
            CheckStatus::Pass => summary.passed += 1,
            CheckStatus::Warning => summary.warnings += 1,
            CheckStatus::Error => summary.errors += 1,
        }
    }

    IntegrityReport {
        repo_db_integrity: repo_db_result,
        sqlite_integrity: sqlite_result,
        restore_point_fs_consistency: fs_result,
        block_reference_check: block_ref_result,
        summary,
        details,
    }
}

// ======== Level 1: Repository Metadata ========

fn check_repo_db_integrity(handle: &RepoHandle, details: &mut Vec<CheckDetail>) -> CheckResult {
    if !handle.repo_db_path.exists() {
        let r = CheckResult::error("repo.db not found — repository cannot be opened");
        details.push(CheckDetail {
            component: "repo.db".into(),
            status: CheckStatus::Error,
            message: r.message.clone(),
        });
        return r;
    }

    let conn = match handle.repo_db() {
        Ok(c) => c,
        Err(e) => {
            let r = CheckResult::error(format!("Cannot open repo.db: {}", e));
            details.push(CheckDetail {
                component: "repo.db".into(),
                status: CheckStatus::Error,
                message: r.message.clone(),
            });
            return r;
        }
    };

    // Check required tables exist
    let required_tables = ["repository_meta", "backup_jobs", "restore_points"];
    let mut missing_tables: Vec<&str> = Vec::new();
    for table in &required_tables {
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .unwrap_or(false);
        if !exists {
            missing_tables.push(table);
        }
    }

    if !missing_tables.is_empty() {
        let msg = format!("Missing required tables: {}", missing_tables.join(", "));
        details.push(CheckDetail {
            component: "repo.db".into(),
            status: CheckStatus::Error,
            message: msg.clone(),
        });
        return CheckResult::error(msg);
    }

    // Check required metadata keys
    let required_keys = ["version", "repository_id", "block_size"];
    let mut missing_keys: Vec<&str> = Vec::new();
    for key in &required_keys {
        let found: bool = conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM repository_meta WHERE key=?1",
                [key],
                |row| row.get(0),
            )
            .unwrap_or(false);
        if !found {
            missing_keys.push(key);
        }
    }

    if !missing_keys.is_empty() {
        let msg = format!(
            "Missing required metadata keys: {}",
            missing_keys.join(", ")
        );
        details.push(CheckDetail {
            component: "repo.db".into(),
            status: CheckStatus::Error,
            message: msg.clone(),
        });
        return CheckResult::error(msg);
    }

    details.push(CheckDetail {
        component: "repo.db".into(),
        status: CheckStatus::Pass,
        message: "Repository metadata structure valid".into(),
    });
    CheckResult::pass("Repository metadata integrity check passed")
}

// ======== Level 2: SQLite Integrity ========

fn check_sqlite_integrity(handle: &RepoHandle, details: &mut Vec<CheckDetail>) -> CheckResult {
    let conn = match handle.repo_db() {
        Ok(c) => c,
        Err(e) => {
            return CheckResult::error(format!("Cannot open repo.db for SQLite check: {}", e));
        }
    };

    // PRAGMA integrity_check on repo.db
    let integrity_result: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .unwrap_or_else(|_| "ERROR".to_string());

    if integrity_result != "ok" {
        let msg = format!(
            "repo.db SQLite integrity check failed: {}",
            integrity_result
        );
        details.push(CheckDetail {
            component: "repo.db (SQLite)".into(),
            status: CheckStatus::Error,
            message: msg.clone(),
        });
        return CheckResult::error(msg);
    }

    details.push(CheckDetail {
        component: "repo.db (SQLite)".into(),
        status: CheckStatus::Pass,
        message: "SQLite integrity check passed".into(),
    });

    // Check per-instance SQLite DBs
    let instances_dir = &handle.instances_dir;
    if !instances_dir.exists() {
        return CheckResult::warning("No backup-instances directory found (empty repository)");
    }

    let entries = match fs::read_dir(instances_dir) {
        Ok(e) => e,
        Err(_) => {
            return CheckResult::warning("Cannot read backup-instances directory");
        }
    };

    let mut instance_errors = 0u32;
    let mut instance_checked = 0u32;

    for entry in entries.flatten() {
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        // Check block-map.db
        let bm_path = dir_path.join("block-map.db");
        if bm_path.exists() {
            instance_checked += 1;
            if let Ok(inst_conn) = rusqlite::Connection::open(&bm_path) {
                let inst_result: String = inst_conn
                    .query_row("PRAGMA integrity_check", [], |row| row.get(0))
                    .unwrap_or_else(|_| "ERROR".to_string());
                if inst_result != "ok" {
                    instance_errors += 1;
                    let point_name = dir_path.file_name().unwrap_or_default().to_string_lossy();
                    details.push(CheckDetail {
                        component: format!("{}/block-map.db", point_name),
                        status: CheckStatus::Error,
                        message: format!("SQLite integrity check failed: {}", inst_result),
                    });
                }
            }
        }

        // Check catalog.db
        let cat_path = dir_path.join("catalog.db");
        if cat_path.exists() {
            instance_checked += 1;
            if let Ok(inst_conn) = rusqlite::Connection::open(&cat_path) {
                let inst_result: String = inst_conn
                    .query_row("PRAGMA integrity_check", [], |row| row.get(0))
                    .unwrap_or_else(|_| "ERROR".to_string());
                if inst_result != "ok" {
                    instance_errors += 1;
                    let point_name = dir_path.file_name().unwrap_or_default().to_string_lossy();
                    details.push(CheckDetail {
                        component: format!("{}/catalog.db", point_name),
                        status: CheckStatus::Error,
                        message: format!("SQLite integrity check failed: {}", inst_result),
                    });
                }
            }
        }
    }

    if instance_errors > 0 {
        CheckResult::error(format!(
            "{} of {} instance DBs failed SQLite integrity check",
            instance_errors, instance_checked
        ))
    } else if instance_checked > 0 {
        CheckResult::pass(format!(
            "All {} instance DBs passed SQLite integrity check",
            instance_checked
        ))
    } else {
        CheckResult::pass("No instance DBs to check (empty repository)")
    }
}

// ======== Level 3: Restore Point FS Consistency ========

fn check_restore_point_fs(handle: &RepoHandle, details: &mut Vec<CheckDetail>) -> CheckResult {
    let conn = match handle.repo_db() {
        Ok(c) => c,
        Err(e) => {
            return CheckResult::error(format!("Cannot open repo.db for FS check: {}", e));
        }
    };

    let mut stmt = match conn
        .prepare("SELECT point_id, instance_path FROM restore_points WHERE status = 'COMMITTED'")
    {
        Ok(s) => s,
        Err(e) => {
            return CheckResult::error(format!("Cannot query restore_points: {}", e));
        }
    };

    let points: Vec<(String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|r| r.ok())
        .collect();

    if points.is_empty() {
        return CheckResult::pass("No COMMITTED restore points to check (empty repository)");
    }

    let mut missing_dirs = 0u32;
    let mut missing_metadata = 0u32;
    let mut valid = 0u32;

    for (point_id, instance_path) in &points {
        let path = PathBuf::from(instance_path);

        if !path.exists() || !path.is_dir() {
            missing_dirs += 1;
            details.push(CheckDetail {
                component: format!("backup-instances/{}", point_id),
                status: CheckStatus::Error,
                message: format!("Instance directory not found: {}", path.display()),
            });
            continue;
        }

        let meta_path = path.join("backup-metadata.json");
        if !meta_path.exists() {
            missing_metadata += 1;
            details.push(CheckDetail {
                component: format!("{}/backup-metadata.json", point_id),
                status: CheckStatus::Error,
                message: "backup-metadata.json not found".into(),
            });
            continue;
        }

        match fs::read_to_string(&meta_path) {
            Ok(content) => {
                if serde_json::from_str::<serde_json::Value>(&content).is_err() {
                    details.push(CheckDetail {
                        component: format!("{}/backup-metadata.json", point_id),
                        status: CheckStatus::Error,
                        message: "backup-metadata.json is not valid JSON".into(),
                    });
                    continue;
                }
            }
            Err(e) => {
                details.push(CheckDetail {
                    component: format!("{}/backup-metadata.json", point_id),
                    status: CheckStatus::Error,
                    message: format!("Cannot read backup-metadata.json: {}", e),
                });
                continue;
            }
        }

        valid += 1;
    }

    if missing_dirs > 0 || missing_metadata > 0 {
        let msg = format!(
            "{} valid, {} missing directories, {} missing metadata files",
            valid, missing_dirs, missing_metadata
        );
        CheckResult::error(msg)
    } else {
        CheckResult::pass(format!(
            "All {} COMMITTED restore point directories consistent",
            valid
        ))
    }
}

// ======== Level 4: Block Reference Existence ========

fn check_block_references(handle: &RepoHandle, details: &mut Vec<CheckDetail>) -> CheckResult {
    let conn = match handle.repo_db() {
        Ok(c) => c,
        Err(e) => {
            return CheckResult::error(format!(
                "Cannot open repo.db for block reference check: {}",
                e
            ));
        }
    };

    let mut stmt = match conn
        .prepare("SELECT point_id, instance_path FROM restore_points WHERE status = 'COMMITTED'")
    {
        Ok(s) => s,
        Err(e) => {
            return CheckResult::error(format!("Cannot query restore_points: {}", e));
        }
    };

    let points: Vec<(String, String)> = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|r| r.ok())
        .collect();

    if points.is_empty() {
        return CheckResult::pass("No restore points to check (empty repository)");
    }

    let mut total_blocks = 0u64;
    let mut missing_blocks = 0u64;
    let mut points_with_bm = 0u32;
    let mut points_without_bm = 0u32;

    for (point_id, instance_path) in &points {
        let path = PathBuf::from(instance_path);
        let bm_path = path.join("block-map.db");

        if !bm_path.exists() {
            points_without_bm += 1;
            details.push(CheckDetail {
                component: format!("{}/block-map.db", point_id),
                status: CheckStatus::Warning,
                message: "block-map.db not found — cannot verify block references".into(),
            });
            continue;
        }

        let bm_conn = match rusqlite::Connection::open(&bm_path) {
            Ok(c) => c,
            Err(_) => {
                details.push(CheckDetail {
                    component: format!("{}/block-map.db", point_id),
                    status: CheckStatus::Warning,
                    message: "Cannot open block-map.db".into(),
                });
                continue;
            }
        };

        let table_exists: bool = bm_conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='block_mappings'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if !table_exists {
            details.push(CheckDetail {
                component: format!("{}/block-map.db", point_id),
                status: CheckStatus::Warning,
                message: "block_mappings table not found".into(),
            });
            continue;
        }

        points_with_bm += 1;

        let mut bm_stmt = match bm_conn.prepare("SELECT block_id FROM block_mappings") {
            Ok(s) => s,
            Err(_) => continue,
        };

        let block_ids: Vec<String> = bm_stmt
            .query_map([], |row| row.get::<_, String>(0))
            .ok()
            .into_iter()
            .flatten()
            .filter_map(|r| r.ok())
            .collect();

        for block_id in &block_ids {
            total_blocks += 1;

            // Block store layout: block-store/{hash[0:2]}/{hash[2:4]}/{full_hash}.block
            let prefix1 = &block_id[..2];
            let prefix2 = if block_id.len() > 4 {
                &block_id[2..4]
            } else {
                "xx"
            };
            let block_path = handle
                .block_store_dir
                .join(prefix1)
                .join(prefix2)
                .join(format!("{}.block", block_id));

            if !block_path.exists() {
                missing_blocks += 1;
                details.push(CheckDetail {
                    component: format!("block-store/{}/{}/{}.block", prefix1, prefix2, block_id),
                    status: CheckStatus::Error,
                    message: format!(
                        "Block referenced by {}/block-map.db not found in block-store",
                        point_id
                    ),
                });
            }
        }
    }

    if missing_blocks > 0 {
        CheckResult::error(format!(
            "{} of {} blocks missing from block-store ({} points checked, {} without block-map)",
            missing_blocks, total_blocks, points_with_bm, points_without_bm
        ))
    } else if points_with_bm > 0 {
        CheckResult::pass(format!(
            "All {} blocks verified in block-store ({} points checked)",
            total_blocks, points_with_bm
        ))
    } else if points_without_bm > 0 {
        CheckResult::warning(format!(
            "{} points have no block-map — block references not verified",
            points_without_bm
        ))
    } else {
        CheckResult::pass("No block references to check (empty repository)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::repo_manager::{init_repo, DEFAULT_BLOCK_SIZE};
    use tempfile::TempDir;

    #[test]
    fn test_integrity_empty_repo() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        let report = check_integrity(&handle);
        assert!(report.is_healthy(), "Empty repo should be healthy");
        assert_eq!(report.summary.errors, 0);
    }

    #[test]
    fn test_integrity_reports_repo_db_missing() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        fs::remove_file(&handle.repo_db_path).unwrap();

        let report = check_integrity(&handle);
        assert!(
            !report.is_healthy(),
            "Repo without repo.db should not be healthy"
        );
        assert!(report.repo_db_integrity.status == CheckStatus::Error);
    }

    #[test]
    fn test_integrity_logs_details() {
        let tmp = TempDir::new().unwrap();
        let handle = init_repo(tmp.path(), DEFAULT_BLOCK_SIZE).unwrap();

        let report = check_integrity(&handle);
        assert!(
            !report.details.is_empty(),
            "Should have at least one detail entry"
        );
    }
}
