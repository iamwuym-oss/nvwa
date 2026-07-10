// ============================================================================
// commands.rs -- Repository CLI commands
// ============================================================================
//
// Phase S S-11: Repository management CLI
//
// Commands:
//   nuwa repo init --dest <path>
//   nuwa repo check [--dest <path>]
//   nuwa repo verify [--full] [--dest <path>]
//   nuwa repo rebuild [--dest <path>]
//   nuwa repo orphans [--dest <path>]
//   nuwa repo list [--dest <path>]

use std::path::{Path, PathBuf};

use crate::errors::NuwaError;
use crate::repository::{
    check_integrity, count_orphan_candidates, init_repo, is_repository, list_orphan_candidates,
    open_repo, rebuild_repo, verify_repo, IntegrityReport, VerifyLevel, VerifyReport,
    DEFAULT_BLOCK_SIZE,
};

/// Repo subcommand variants for the CLI parser
pub enum RepoSubcommand {
    Init { dest: Option<PathBuf> },
    Check { dest: Option<PathBuf> },
    Verify { full: bool, dest: Option<PathBuf> },
    Rebuild { dest: Option<PathBuf> },
    Orphans { dest: Option<PathBuf> },
    List { dest: Option<PathBuf> },
}

const REPO_USAGE: &str = "\
Usage:
  nuwa repo init --dest <path>
  nuwa repo check [--dest <path>]
  nuwa repo verify [--full] [--dest <path>]
  nuwa repo rebuild [--dest <path>]
  nuwa repo orphans [--dest <path>]
  nuwa repo list [--dest <path>]";

impl RepoSubcommand {
    pub fn parse(args: &[String]) -> Result<Self, NuwaError> {
        if args.is_empty() {
            return Err(NuwaError::InvalidArgument {
                detail: "Missing repo subcommand".to_string(),
                suggestion: REPO_USAGE.to_string(),
            });
        }
        let sub = &args[0].to_lowercase();
        let rest = &args[1..];
        match sub.as_str() {
            "init" => {
                let mut dest = None;
                let mut i = 0;
                while i < rest.len() {
                    if rest[i] == "--dest" {
                        i += 1;
                        dest = Some(PathBuf::from(get_arg(rest, i, "--dest")?));
                    } else {
                        return Err(NuwaError::InvalidArgument {
                            detail: format!("Unknown argument '{}'", rest[i]),
                            suggestion: "nuwa repo init --dest <path>".to_string(),
                        });
                    }
                    i += 1;
                }
                Ok(RepoSubcommand::Init { dest })
            }
            "check" => {
                let mut dest = None;
                let mut i = 0;
                while i < rest.len() {
                    if rest[i] == "--dest" {
                        i += 1;
                        dest = Some(PathBuf::from(get_arg(rest, i, "--dest")?));
                    } else {
                        return Err(NuwaError::InvalidArgument {
                            detail: format!("Unknown argument '{}'", rest[i]),
                            suggestion: "nuwa repo check [--dest <path>]".to_string(),
                        });
                    }
                    i += 1;
                }
                Ok(RepoSubcommand::Check { dest })
            }
            "verify" => {
                let mut full = false;
                let mut dest = None;
                let mut i = 0;
                while i < rest.len() {
                    match rest[i].as_str() {
                        "--full" => full = true,
                        "--dest" => {
                            i += 1;
                            dest = Some(PathBuf::from(get_arg(rest, i, "--dest")?));
                        }
                        _ => {
                            return Err(NuwaError::InvalidArgument {
                                detail: format!("Unknown argument '{}'", rest[i]),
                                suggestion: "nuwa repo verify [--full] [--dest <path>]".to_string(),
                            });
                        }
                    }
                    i += 1;
                }
                Ok(RepoSubcommand::Verify { full, dest })
            }
            "rebuild" => {
                let mut dest = None;
                let mut i = 0;
                while i < rest.len() {
                    if rest[i] == "--dest" {
                        i += 1;
                        dest = Some(PathBuf::from(get_arg(rest, i, "--dest")?));
                    } else {
                        return Err(NuwaError::InvalidArgument {
                            detail: format!("Unknown argument '{}'", rest[i]),
                            suggestion: "nuwa repo rebuild [--dest <path>]".to_string(),
                        });
                    }
                    i += 1;
                }
                Ok(RepoSubcommand::Rebuild { dest })
            }
            "orphans" => {
                let mut dest = None;
                let mut i = 0;
                while i < rest.len() {
                    if rest[i] == "--dest" {
                        i += 1;
                        dest = Some(PathBuf::from(get_arg(rest, i, "--dest")?));
                    } else {
                        return Err(NuwaError::InvalidArgument {
                            detail: format!("Unknown argument '{}'", rest[i]),
                            suggestion: "nuwa repo orphans [--dest <path>]".to_string(),
                        });
                    }
                    i += 1;
                }
                Ok(RepoSubcommand::Orphans { dest })
            }
            "list" => {
                let mut dest = None;
                let mut i = 0;
                while i < rest.len() {
                    if rest[i] == "--dest" {
                        i += 1;
                        dest = Some(PathBuf::from(get_arg(rest, i, "--dest")?));
                    } else {
                        return Err(NuwaError::InvalidArgument {
                            detail: format!("Unknown argument '{}'", rest[i]),
                            suggestion: "nuwa repo list [--dest <path>]".to_string(),
                        });
                    }
                    i += 1;
                }
                Ok(RepoSubcommand::List { dest })
            }
            _ => Err(NuwaError::InvalidArgument {
                detail: format!("Unknown repo subcommand '{}'", sub),
                suggestion: REPO_USAGE.to_string(),
            }),
        }
    }
}

fn get_arg(args: &[String], idx: usize, flag: &str) -> Result<String, NuwaError> {
    args.get(idx)
        .cloned()
        .ok_or_else(|| NuwaError::InvalidArgument {
            detail: format!("{} requires a value", flag),
            suggestion: "".to_string(),
        })
}

// ======== Helpers ========

fn resolve_repo_path(dest: Option<&PathBuf>) -> Result<PathBuf, NuwaError> {
    if let Some(p) = dest {
        return Ok(p.clone());
    }
    let cwd = std::env::current_dir().map_err(|e| NuwaError::Io {
        source: Some(e),
        path: None,
        detail: "Cannot get current directory".to_string(),
        suggestion: "Use --dest <path> to specify the repository root.".to_string(),
    })?;
    if is_repository(&cwd) {
        Ok(cwd)
    } else {
        let parent = cwd.parent();
        if let Some(p) = parent {
            if is_repository(p) {
                return Ok(p.to_path_buf());
            }
        }
        Err(NuwaError::InvalidArgument {
            detail: "No repository found. Use --dest <path>.".to_string(),
            suggestion: REPO_USAGE.to_string(),
        })
    }
}

fn report_integrity(report: &IntegrityReport) {
    println!("  Integrity Report:");
    println!("    SQLite:      {:?}", report.sqlite_integrity.status);
    println!("    Repo DB:     {:?}", report.repo_db_integrity.status);
    println!(
        "    FS Check:    {:?}",
        report.restore_point_fs_consistency.status
    );
    println!("    Block Ref:   {:?}", report.block_reference_check.status);
    println!(
        "    Summary:     {} passed, {} warnings, {} errors",
        report.summary.passed, report.summary.warnings, report.summary.errors
    );
}

fn report_verify(report: &VerifyReport) {
    println!("  Verification Report:");
    println!("    Restore points:    {}", report.total_restore_points);
    println!("    Total blocks:      {}", report.total_blocks);
    println!("    Verified blocks:   {}", report.verified_blocks);
    println!("    Failed blocks:     {}", report.failed_blocks);
    if report.data_loss_detected {
        println!("    DATA LOSS DETECTED!");
    }
    if !report.failed_block_ids.is_empty() {
        println!("    Failed block IDs:  {:?}", report.failed_block_ids);
    }
    println!("    Summary: {}", report.summary);
}

// ======== Command Handlers ========

pub fn cmd_repo_init(dest: &Path) -> Result<(), NuwaError> {
    if is_repository(dest) {
        return Err(NuwaError::InvalidArgument {
            detail: format!("Repository already exists at '{}'", dest.display()),
            suggestion: "Use a different path or run 'nuwa repo check'.".to_string(),
        });
    }
    let info = init_repo(dest, DEFAULT_BLOCK_SIZE).map_err(|e| NuwaError::Io {
        source: None,
        path: Some(dest.to_path_buf()),
        detail: format!("Failed to init repository: {}", e),
        suggestion: "Check that the destination path is writable.".to_string(),
    })?;
    println!("Repository initialized successfully.");
    println!("  Path:      {}", dest.display());
    println!("  Block:     {} bytes", info.info.chunk_policy.block_size);
    println!("  Format:    v{}", info.info.format_version);
    println!("  ID:        {}", info.info.repository_id);
    Ok(())
}

pub fn cmd_repo_check(dest: Option<&PathBuf>) -> Result<(), NuwaError> {
    let repo_path = resolve_repo_path(dest)?;
    let handle = open_repo(&repo_path).map_err(|e| NuwaError::Io {
        source: None,
        path: Some(repo_path.clone()),
        detail: format!("Cannot open repository: {}", e),
        suggestion: "Run 'nuwa repo init --dest <path>' first.".to_string(),
    })?;
    println!("Repository: {}", repo_path.display());
    println!("  ID:        {}", handle.info.repository_id);
    println!("  Block:     {} bytes", handle.info.chunk_policy.block_size);
    println!("  Format:    v{}", handle.info.format_version);
    println!();
    let report = check_integrity(&handle);
    report_integrity(&report);
    Ok(())
}

pub fn cmd_repo_verify(full: bool, dest: Option<&PathBuf>) -> Result<(), NuwaError> {
    let repo_path = resolve_repo_path(dest)?;
    let handle = open_repo(&repo_path).map_err(|e| NuwaError::Io {
        source: None,
        path: Some(repo_path.clone()),
        detail: format!("Cannot open repository: {}", e),
        suggestion: "".to_string(),
    })?;
    // Build a LocalFsBlockStore for verification
    use crate::repository::block_store::store::LocalFsBlockStore;
    let block_store = LocalFsBlockStore::new(handle.block_store_dir.clone());

    let level = if full {
        VerifyLevel::Full
    } else {
        VerifyLevel::Sampling(100)
    };
    println!("Verifying (level: {:?})...", level);
    match verify_repo(&handle, &block_store, level) {
        Ok(report) => {
            report_verify(&report);
            if report.data_loss_detected {
                return Err(NuwaError::SafetyViolation {
                    detail: "Data loss detected during verification.".to_string(),
                    suggestion: "Run 'nuwa repo rebuild' to attempt recovery.".to_string(),
                });
            }
            Ok(())
        }
        Err(e) => Err(NuwaError::Io {
            source: None,
            path: Some(repo_path),
            detail: format!("Verification error: {}", e),
            suggestion: "".to_string(),
        }),
    }
}

pub fn cmd_repo_rebuild(dest: Option<&PathBuf>) -> Result<(), NuwaError> {
    let repo_path = resolve_repo_path(dest)?;
    println!("Rebuilding repository metadata...");
    rebuild_repo(&repo_path).map_err(|e| NuwaError::Io {
        source: None,
        path: Some(repo_path.clone()),
        detail: format!("Rebuild failed: {}", e),
        suggestion: "Check if backup-instances/ directory exists.".to_string(),
    })?;
    println!("Rebuild complete.");
    Ok(())
}

pub fn cmd_repo_orphans(dest: Option<&PathBuf>) -> Result<(), NuwaError> {
    let repo_path = resolve_repo_path(dest)?;
    let handle = open_repo(&repo_path).map_err(|e| NuwaError::Io {
        source: None,
        path: Some(repo_path.clone()),
        detail: format!("Cannot open repository: {}", e),
        suggestion: "".to_string(),
    })?;
    let count = count_orphan_candidates(&handle).map_err(|e| NuwaError::Io {
        source: None,
        path: Some(repo_path.clone()),
        detail: format!("Cannot count orphans: {}", e),
        suggestion: "".to_string(),
    })?;
    let summary = list_orphan_candidates(&handle).map_err(|e| NuwaError::Io {
        source: None,
        path: Some(repo_path),
        detail: format!("Cannot list orphans: {}", e),
        suggestion: "".to_string(),
    })?;
    println!("Orphan Candidates:");
    println!("  Total count: {}", count);
    println!("  Deleted points: {}", summary.deleted_points.len());
    println!("  Orphan blocks:  {}", summary.total_orphan_blocks);
    for pid in &summary.deleted_points {
        println!("    - {}", pid);
    }
    Ok(())
}

pub fn cmd_repo_list(dest: Option<&PathBuf>) -> Result<(), NuwaError> {
    let repo_path = resolve_repo_path(dest)?;
    let handle = open_repo(&repo_path).map_err(|e| NuwaError::Io {
        source: None,
        path: Some(repo_path.clone()),
        detail: format!("Cannot open repository: {}", e),
        suggestion: "".to_string(),
    })?;
    println!("Repository: {}", repo_path.display());
    println!("  ID:        {}", handle.info.repository_id);
    println!("  Block:     {} bytes", handle.info.chunk_policy.block_size);
    println!("  Format:    v{}", handle.info.format_version);
    // Scan backup-instances
    let instances = repo_path.join("backup-instances");
    if instances.exists() {
        let entries = match std::fs::read_dir(&instances) {
            Ok(e) => e,
            Err(_) => {
                println!("  Instances: 0");
                return Ok(());
            }
        };
        let dirs: Vec<_> = entries.flatten().filter(|e| e.path().is_dir()).collect();
        println!("  Instances: {}", dirs.len());
        if !dirs.is_empty() {
            println!();
            for entry in &dirs {
                let meta = entry.path().join("backup-metadata.json");
                if meta.exists() {
                    if let Ok(content) = std::fs::read_to_string(&meta) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            let rid = json
                                .get("restore_point_id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?");
                            let created = json
                                .get("created_at")
                                .and_then(|v| v.as_str())
                                .unwrap_or("?");
                            let status = json.get("status").and_then(|v| v.as_str()).unwrap_or("?");
                            println!("    {}  [{}]  {}", rid, status, created);
                            continue;
                        }
                    }
                }
                println!("    {}", entry.path().display());
            }
        }
    } else {
        println!("  Instances: 0");
    }
    Ok(())
}

pub fn handle_repo_command(subcmd: &RepoSubcommand) -> Result<(), NuwaError> {
    match subcmd {
        RepoSubcommand::Init { dest } => {
            let d = dest.as_ref().ok_or_else(|| NuwaError::InvalidArgument {
                detail: "--dest is required for repo init".to_string(),
                suggestion: "nuwa repo init --dest <path>".to_string(),
            })?;
            cmd_repo_init(d)
        }
        RepoSubcommand::Check { dest } => cmd_repo_check(dest.as_ref()),
        RepoSubcommand::Verify { full, dest } => cmd_repo_verify(*full, dest.as_ref()),
        RepoSubcommand::Rebuild { dest } => cmd_repo_rebuild(dest.as_ref()),
        RepoSubcommand::Orphans { dest } => cmd_repo_orphans(dest.as_ref()),
        RepoSubcommand::List { dest } => cmd_repo_list(dest.as_ref()),
    }
}

// ======== Tests ========

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn init_test_repo() -> (TempDir, PathBuf) {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("repo");
        cmd_repo_init(&p).unwrap();
        (tmp, p)
    }

    #[test]
    fn test_init_and_check() {
        let (_tmp, p) = init_test_repo();
        assert!(is_repository(&p));
        assert!(cmd_repo_check(Some(&p)).is_ok());
    }

    #[test]
    fn test_init_twice_fails() {
        let tmp = TempDir::new().unwrap();
        let p = tmp.path().join("repo");
        cmd_repo_init(&p).unwrap();
        assert!(cmd_repo_init(&p).is_err());
    }

    #[test]
    fn test_verify_empty() {
        let (_tmp, p) = init_test_repo();
        assert!(cmd_repo_verify(false, Some(&p)).is_ok());
    }

    #[test]
    fn test_list_empty() {
        let (_tmp, p) = init_test_repo();
        assert!(cmd_repo_list(Some(&p)).is_ok());
    }

    #[test]
    fn test_orphans_empty() {
        let (_tmp, p) = init_test_repo();
        assert!(cmd_repo_orphans(Some(&p)).is_ok());
    }

    #[test]
    fn test_rebuild_empty() {
        let (_tmp, p) = init_test_repo();
        assert!(cmd_repo_rebuild(Some(&p)).is_ok());
    }
}
