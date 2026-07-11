// ============================================================================
// main.rs - Nuwa Backup program entry
// ============================================================================

use nuwa_backup::cli::Command;
use nuwa_backup::cli_output;
use nuwa_backup::errors::ExitCode;
use std::process;

fn main() {
    let cmd = match Command::from_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(ExitCode::InvalidArgs as i32);
        }
    };

    let exit = match cmd {
        Command::Init { config_path } => match nuwa_backup::config::init(config_path.as_deref()) {
            Ok(_) => ExitCode::Success,
            Err(e) => {
                eprintln!("{}", e);
                ExitCode::from(&e)
            }
        },
        Command::Backup {
            source,
            dest,
            compress,
            job,
            #[cfg(feature = "repository")]
            repo,
            json_output,
        } => {
            let start = std::time::Instant::now();
            let job_name = job.clone();
            #[cfg(feature = "repository")]
            if let Some(ref repo_path) = repo {
                let exit_code =
                    execute_repo_backup(repo_path, &source.unwrap(), compress, json_output);
                process::exit(exit_code as i32);
            }
            let (resolved_source, resolved_dest, resolved_compress) =
                match resolve_backup_params(source, dest, compress, job.as_deref()) {
                    Ok((ref src, ref dst, ref comp)) => {
                        // Validate destination path
                        if let Err(e) = nuwa_backup::path_support::validate_repository_path(dst) {
                            if json_output {
                                let out =
                                    cli_output::JsonOutput::failure("backup", &e.to_string(), 0);
                                let code = cli_output::print_json_compact(&out);
                                process::exit(code as i32);
                            }
                            eprintln!("{}", e);
                            process::exit(ExitCode::from(&e) as i32);
                        }
                        if let Err(e) = nuwa_backup::path_support::preflight_dest_check(dst) {
                            if json_output {
                                let out =
                                    cli_output::JsonOutput::failure("backup", &e.to_string(), 0);
                                let code = cli_output::print_json_compact(&out);
                                process::exit(code as i32);
                            }
                            eprintln!("{}", e);
                            process::exit(ExitCode::from(&e) as i32);
                        }
                        (src.clone(), dst.clone(), *comp)
                    }
                    Err(e) => {
                        if json_output {
                            let out = cli_output::JsonOutput::failure("backup", &e.to_string(), 0);
                            let code = cli_output::print_json_compact(&out);
                            process::exit(code as i32);
                        }
                        eprintln!("{}", e);
                        process::exit(ExitCode::from(&e) as i32);
                    }
                };
            let result = nuwa_backup::backup::execute_backup(
                &resolved_source,
                &resolved_dest,
                resolved_compress,
            );
            let duration_ms = start.elapsed().as_millis() as u64;
            match result {
                Ok(dir) => {
                    let dir_path = std::path::Path::new(&dir);
                    let mut file_count = 0u64;
                    let mut total_bytes = 0u64;
                    let mut backup_id = String::new();
                    if let Some(parent) = dir_path.parent() {
                        let summaries = nuwa_backup::list::execute_list(parent).ok();
                        let dir_name = dir_path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        let backup_info = summaries
                            .and_then(|s| s.into_iter().find(|bp| bp.dir_name == dir_name));
                        if let Some(info) = backup_info {
                            file_count = info.file_count;
                            total_bytes = info.total_bytes;
                            backup_id = info.backup_id.clone();
                            record_operation_history(
                                &info.backup_id,
                                "backup",
                                &resolved_source.to_string_lossy(),
                                &resolved_dest,
                                job_name.as_deref(),
                                info.file_count,
                                info.total_bytes,
                                duration_ms,
                                0,
                                "success",
                            );
                        }
                    }

                    if json_output {
                        let mut out = cli_output::JsonOutput::success(
                            "backup",
                            "Backup completed",
                            duration_ms,
                        );
                        out.backup_point = Some(dir);
                        out.file_count = Some(file_count);
                        out.total_bytes = Some(total_bytes);
                        out.backup_id = Some(backup_id);
                        let code = cli_output::print_json_compact(&out);
                        process::exit(code as i32);
                    }

                    println!("[OK] Backup completed. Backup point: {}", dir);
                    cli_output::print_backup_summary(&dir, file_count, total_bytes, duration_ms);

                    // Auto-prune if job has retention policy
                    if let Some(ref jn) = job_name {
                        if let Ok(config) = nuwa_backup::config::Config::load() {
                            if let Ok((_, job_cfg)) = config.find_job(Some(jn)) {
                                if let Some(ref retention) = job_cfg.retention {
                                    match nuwa_backup::prune::execute_prune(
                                        &resolved_dest,
                                        retention.keep_count,
                                        retention.keep_days,
                                        false,
                                    ) {
                                        Ok(prune_result) => {
                                            if prune_result.deleted_count > 0 {
                                                println!(
                                                    "  Retention: pruned {} old backup point(s)",
                                                    prune_result.deleted_count
                                                );
                                            }
                                        }
                                        Err(e) => {
                                            eprintln!("  [WARNING] Auto-prune failed: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    ExitCode::Success
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    record_operation_history(
                        &format!("failed-{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")),
                        "backup",
                        &resolved_source.to_string_lossy(),
                        &resolved_dest,
                        job_name.as_deref(),
                        0,
                        0,
                        duration_ms,
                        ExitCode::from(&e) as i32,
                        "failure",
                    );

                    if json_output {
                        let out =
                            cli_output::JsonOutput::failure("backup", &error_msg, duration_ms);
                        let code = cli_output::print_json_compact(&out);
                        process::exit(code as i32);
                    }

                    eprintln!("[ERROR] Backup failed: {}", error_msg);
                    ExitCode::from(&e)
                }
            }
        }
        Command::Restore {
            backup,
            dest,
            overwrite,
            json_output,
        } => {
            let start = std::time::Instant::now();
            // Validate restore destination path
            if let Err(e) = nuwa_backup::path_support::validate_repository_path(&dest) {
                if json_output {
                    let out = cli_output::JsonOutput::failure("restore", &e.to_string(), 0);
                    let code = cli_output::print_json_compact(&out);
                    process::exit(code as i32);
                }
                eprintln!("{}", e);
                process::exit(ExitCode::from(&e) as i32);
            }
            let src_info = nuwa_backup::storage::read_manifest(&backup).ok();
            let result = nuwa_backup::restore::execute_restore(&backup, &dest, overwrite);
            let duration_ms = start.elapsed().as_millis() as u64;
            match result {
                Ok(r) => {
                    let (exit_code, status) = if r.checksum_failures > 0 {
                        (ExitCode::RestoreFailure as i32, "partial")
                    } else {
                        (0, "success")
                    };
                    let backup_id = src_info
                        .as_ref()
                        .map(|m| m.backup_id.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    let source_root = src_info
                        .as_ref()
                        .map(|m| m.source_root.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    record_operation_history(
                        &backup_id,
                        "restore",
                        &source_root,
                        &dest,
                        None,
                        r.restored_count,
                        0,
                        duration_ms,
                        exit_code,
                        status,
                    );

                    if json_output {
                        let mut out = cli_output::JsonOutput::success(
                            "restore",
                            "Restore completed",
                            duration_ms,
                        );
                        out.restored_count = Some(r.restored_count);
                        out.skipped_count = Some(r.skipped_count);
                        out.checksum_failures = Some(r.checksum_failures);
                        out.backup_id = Some(backup_id);
                        let code = cli_output::print_json_compact(&out);
                        process::exit(code as i32);
                    }

                    if r.checksum_failures > 0 {
                        eprintln!("[ERROR] Checksum validation failed");
                    } else {
                        println!("[OK] Restore completed!");
                    }
                    cli_output::print_restore_summary(
                        r.restored_count,
                        r.skipped_count,
                        r.checksum_failures,
                        duration_ms,
                    );
                    if r.checksum_failures > 0 {
                        ExitCode::RestoreFailure
                    } else {
                        ExitCode::Success
                    }
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    let backup_id = src_info
                        .as_ref()
                        .map(|m| m.backup_id.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    let source_root = src_info
                        .as_ref()
                        .map(|m| m.source_root.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    record_operation_history(
                        &backup_id,
                        "restore",
                        &source_root,
                        &dest,
                        None,
                        0,
                        0,
                        duration_ms,
                        ExitCode::from(&e) as i32,
                        "failure",
                    );

                    if json_output {
                        let out =
                            cli_output::JsonOutput::failure("restore", &error_msg, duration_ms);
                        let code = cli_output::print_json_compact(&out);
                        process::exit(code as i32);
                    }

                    eprintln!("[ERROR] Restore failed: {}", error_msg);
                    ExitCode::from(&e)
                }
            }
        }
        Command::Verify {
            backup,
            json_output,
        } => {
            let start = std::time::Instant::now();
            // Validate backup path
            if let Err(e) = nuwa_backup::path_support::validate_repository_path(&backup) {
                if json_output {
                    let out = cli_output::JsonOutput::failure("verify", &e.to_string(), 0);
                    let code = cli_output::print_json_compact(&out);
                    process::exit(code as i32);
                }
                eprintln!("{}", e);
                process::exit(ExitCode::from(&e) as i32);
            }
            let src_info = nuwa_backup::storage::read_manifest(&backup).ok();
            let result = nuwa_backup::verify::execute_verify(&backup);
            let duration_ms = start.elapsed().as_millis() as u64;
            match result {
                Ok(r) => {
                    if json_output {
                        let mut out = cli_output::JsonOutput::success(
                            "verify",
                            "Verification passed",
                            duration_ms,
                        );
                        out.total_bytes = None;
                        out.passed = Some(r.passed);
                        out.failed = Some(r.failed);
                        out.inaccessible = Some(r.inaccessible);
                        out.file_count = Some(r.total_files);
                        let bid = src_info
                            .as_ref()
                            .map(|m| m.backup_id.clone())
                            .unwrap_or_default();
                        out.backup_id = Some(bid);
                        let code = cli_output::print_json_compact(&out);
                        process::exit(code as i32);
                    }
                    cli_output::print_verify_summary(
                        r.total_files,
                        r.passed,
                        r.failed,
                        r.inaccessible,
                    );
                    ExitCode::Success
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    if json_output {
                        let out =
                            cli_output::JsonOutput::failure("verify", &error_msg, duration_ms);
                        let code = cli_output::print_json_compact(&out);
                        process::exit(code as i32);
                    }
                    eprintln!("[ERROR] Verification failed: {}", error_msg);
                    ExitCode::from(&e)
                }
            }
        }
        Command::List { dest, json_output } => {
            // Validate repository path
            if let Err(e) = nuwa_backup::path_support::validate_repository_path(&dest) {
                if json_output {
                    let out = cli_output::JsonOutput::failure("list", &e.to_string(), 0);
                    let code = cli_output::print_json_compact(&out);
                    process::exit(code as i32);
                }
                eprintln!("{}", e);
                process::exit(ExitCode::from(&e) as i32);
            }
            match nuwa_backup::list::execute_list(&dest) {
                Ok(s) => {
                    if json_output {
                        let mut out = cli_output::JsonOutput::success("list", "List completed", 0);
                        let points: Vec<cli_output::BackupPointJson> =
                            s.iter().map(|bp| bp.into()).collect();
                        out.backup_points = Some(points);
                        let code = cli_output::print_json_compact(&out);
                        process::exit(code as i32);
                    }
                    nuwa_backup::list::print_list(&s);
                    ExitCode::Success
                }
                Err(e) => {
                    let error_msg = e.to_string();
                    if json_output {
                        let out = cli_output::JsonOutput::failure("list", &error_msg, 0);
                        let code = cli_output::print_json_compact(&out);
                        process::exit(code as i32);
                    }
                    eprintln!("[ERROR] List failed: {}", error_msg);
                    ExitCode::from(&e)
                }
            }
        }
        Command::History {
            dest,
            limit,
            operation,
            rebuild,
            json_output,
        } => {
            let start = std::time::Instant::now();
            // Validate repository path
            if let Err(e) = nuwa_backup::path_support::validate_repository_path(&dest) {
                if json_output {
                    let out = cli_output::JsonOutput::failure("history", &e.to_string(), 0);
                    let code = cli_output::print_json_compact(&out);
                    process::exit(code as i32);
                }
                eprintln!("{}", e);
                process::exit(ExitCode::from(&e) as i32);
            }
            let db_path = nuwa_backup::history::HistoryDb::history_db_path(&dest);
            if rebuild {
                match nuwa_backup::history::HistoryDb::rebuild_from_manifest(&dest) {
                    Ok(count) => {
                        if json_output {
                            let out = cli_output::JsonOutput::success(
                                "history",
                                &format!("Rebuilt. {} records imported.", count),
                                start.elapsed().as_millis() as u64,
                            );
                            let code = cli_output::print_json_compact(&out);
                            process::exit(code as i32);
                        }
                        println!("[OK] History rebuilt. {} records imported.", count);
                        ExitCode::Success
                    }
                    Err(e) => {
                        if json_output {
                            let out = cli_output::JsonOutput::failure(
                                "history",
                                &e.to_string(),
                                start.elapsed().as_millis() as u64,
                            );
                            let code = cli_output::print_json_compact(&out);
                            process::exit(code as i32);
                        }
                        eprintln!("[ERROR] History rebuild failed: {}", e);
                        ExitCode::from(&e)
                    }
                }
            } else {
                match nuwa_backup::history::HistoryDb::open_or_create(&db_path) {
                    Ok(db) => match db.query_history(limit, operation.as_deref()) {
                        Ok(records) => {
                            if json_output {
                                let mut out = cli_output::JsonOutput::success(
                                    "history",
                                    &format!("{} records", records.len()),
                                    start.elapsed().as_millis() as u64,
                                );
                                let entries: Vec<cli_output::HistoryEntryJson> =
                                    records.iter().map(|r| r.into()).collect();
                                out.records = Some(entries);
                                let code = cli_output::print_json_compact(&out);
                                process::exit(code as i32);
                            }
                            nuwa_backup::history::print_history(&records);
                            ExitCode::Success
                        }
                        Err(e) => {
                            if json_output {
                                let out = cli_output::JsonOutput::failure(
                                    "history",
                                    &e.to_string(),
                                    start.elapsed().as_millis() as u64,
                                );
                                let code = cli_output::print_json_compact(&out);
                                process::exit(code as i32);
                            }
                            eprintln!("[ERROR] History query failed: {}", e);
                            ExitCode::from(&e)
                        }
                    },
                    Err(e) => {
                        if json_output {
                            let out = cli_output::JsonOutput::failure(
                                "history",
                                &e.to_string(),
                                start.elapsed().as_millis() as u64,
                            );
                            let code = cli_output::print_json_compact(&out);
                            process::exit(code as i32);
                        }
                        eprintln!("[ERROR] Cannot open history database: {}", e);
                        ExitCode::from(&e)
                    }
                }
            }
        }
        Command::Prune {
            dest,
            keep_count,
            keep_days,
            dry_run,
            json_output,
        } => {
            let start = std::time::Instant::now();
            // Validate repository path
            if let Err(e) = nuwa_backup::path_support::validate_repository_path(&dest) {
                if json_output {
                    let out = cli_output::JsonOutput::failure("prune", &e.to_string(), 0);
                    let code = cli_output::print_json_compact(&out);
                    process::exit(code as i32);
                }
                eprintln!("{}", e);
                process::exit(ExitCode::from(&e) as i32);
            }
            match nuwa_backup::prune::execute_prune(&dest, keep_count, keep_days, dry_run) {
                Ok(result) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    if json_output {
                        let mut out = if dry_run {
                            cli_output::JsonOutput::success(
                                "prune",
                                "Dry-run completed (no files deleted)",
                                duration_ms,
                            )
                        } else {
                            cli_output::JsonOutput::success("prune", "Prune completed", duration_ms)
                        };
                        out.dry_run = Some(dry_run);
                        out.deleted_count = Some(result.deleted_count as u64);
                        out.kept_count = Some(result.kept_count as u64);
                        out.damaged_count = Some(result.damaged_count as u64);
                        if !result.deleted_backup_ids.is_empty() {
                            out.deleted_backup_ids = Some(result.deleted_backup_ids.clone());
                        }
                        if !result.kept_backup_ids.is_empty() {
                            out.kept_backup_ids = Some(result.kept_backup_ids.clone());
                        }
                        if !result.warnings.is_empty() {
                            out.warnings = Some(result.warnings.clone());
                        }
                        let code = cli_output::print_json_compact(&out);
                        process::exit(code as i32);
                    }
                    nuwa_backup::prune::print_prune_result(&result);
                    ExitCode::Success
                }
                Err(e) => {
                    let duration_ms = start.elapsed().as_millis() as u64;
                    if json_output {
                        let out =
                            cli_output::JsonOutput::failure("prune", &e.to_string(), duration_ms);
                        let code = cli_output::print_json_compact(&out);
                        process::exit(code as i32);
                    }
                    eprintln!("{}", e);
                    ExitCode::from(&e)
                }
            }
        }
        Command::Schedule { subcommand } => match subcommand {
            nuwa_backup::cli::ScheduleSubcommand::Create { job_name, trigger } => {
                match nuwa_backup::scheduler::create_task(&job_name, &trigger) {
                    Ok(()) => {
                        println!("[OK] Scheduled task created for job '{}'.", job_name);
                        println!("  Task name: NuwaBackup-{}", job_name);
                        println!("  Use 'nuwa schedule list' to view all tasks.");
                        ExitCode::Success
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        ExitCode::from(&e)
                    }
                }
            }
            nuwa_backup::cli::ScheduleSubcommand::List => {
                match nuwa_backup::scheduler::list_tasks() {
                    Ok(tasks) => {
                        nuwa_backup::scheduler::print_tasks(&tasks);
                        ExitCode::Success
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        ExitCode::from(&e)
                    }
                }
            }
            nuwa_backup::cli::ScheduleSubcommand::Delete { task_id } => {
                match nuwa_backup::scheduler::delete_task(&task_id) {
                    Ok(()) => {
                        println!("[OK] Task deleted.");
                        ExitCode::Success
                    }
                    Err(e) => {
                        eprintln!("{}", e);
                        ExitCode::from(&e)
                    }
                }
            }
        },
        #[cfg(feature = "repository")]
        Command::Repo { subcommand } => {
            match nuwa_backup::repository::cli::commands::handle_repo_command(&subcommand) {
                Ok(_) => ExitCode::Success,
                Err(e) => {
                    eprintln!("{}", e);
                    ExitCode::from(&e)
                }
            }
        }
    };
    process::exit(exit as i32);
}

#[cfg(feature = "repository")]
fn execute_repo_backup(
    repo_path: &std::path::Path,
    source: &std::path::Path,
    compress: bool,
    json_output: bool,
) -> ExitCode {
    use nuwa_backup::repository::RepositoryBackupWriter;

    let start = std::time::Instant::now();
    let repo = match nuwa_backup::repository::open_repo(repo_path) {
        Ok(h) => h,
        Err(_) => {
            eprintln!(
                "Error: Repository not found at '{}'. Use 'nuwa repo init' first.",
                repo_path.display()
            );
            return ExitCode::InvalidArgs;
        }
    };
    let job_id = "cli-repo-backup";
    {
        let conn = match repo.repo_db() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error: Cannot open repository database: {}", e);
                return ExitCode::GeneralFailure;
            }
        };
        let _ = conn.execute(
            "INSERT OR IGNORE INTO backup_jobs (job_id, job_name, source_type, created_at, status) VALUES (?1, ?2, 0, ?3, 'active')",
            rusqlite::params![job_id, job_id, &chrono::Utc::now().to_rfc3339()],
        );
    }
    let mut writer = match RepositoryBackupWriter::new(&repo, job_id, compress) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error: Cannot create backup writer: {}", e);
            return ExitCode::GeneralFailure;
        }
    };
    if let Err(e) = writer.begin() {
        eprintln!("Error: Cannot begin backup transaction: {}", e);
        let _ = writer.fail();
        return ExitCode::GeneralFailure;
    }
    if let Err(e) = writer.backup_directory(source) {
        eprintln!("Error: Backup failed during file processing: {}", e);
        let _ = writer.fail();
        return ExitCode::GeneralFailure;
    }
    match writer.finalize() {
        Ok(r) => {
            let duration_ms = start.elapsed().as_millis() as u64;
            if json_output {
                let msg = format!(
                    r#"{{"restore_point_id":"{}","file_count":{},"directory_count":{},"block_count":{},"total_raw_bytes":{},"duration_ms":{}}}"#,
                    r.point_id,
                    r.file_count,
                    r.directory_count,
                    r.block_count,
                    r.total_raw_bytes,
                    duration_ms
                );
                let out = cli_output::JsonOutput::success("backup", &msg, duration_ms);
                return cli_output::print_json_compact(&out);
            }
            println!("[OK] Backup completed successfully.");
            println!("  Restore Point ID: {}", r.point_id);
            println!(
                "  Files: {}, Directories: {}",
                r.file_count, r.directory_count
            );
            println!(
                "  Blocks: {}, Total Raw Bytes: {}",
                r.block_count, r.total_raw_bytes
            );
            println!("  Duration: {} ms", duration_ms);
            ExitCode::Success
        }
        Err(e) => {
            eprintln!("Error: Backup finalize failed: {}", e);
            ExitCode::GeneralFailure
        }
    }
}
fn resolve_backup_params(
    source: Option<std::path::PathBuf>,
    dest: Option<std::path::PathBuf>,
    compress: bool,
    job: Option<&str>,
) -> Result<(std::path::PathBuf, std::path::PathBuf, bool), nuwa_backup::errors::NuwaError> {
    if let Some(job_name) = job {
        let config = nuwa_backup::config::Config::load()?;
        let (_, job_cfg) = config.find_job(Some(job_name))?;
        return Ok((
            job_cfg.source.clone(),
            job_cfg.dest.clone(),
            job_cfg.compress,
        ));
    }
    if source.is_none() && dest.is_none() {
        if let Ok(config) = nuwa_backup::config::Config::load() {
            if let Ok((_, job_cfg)) = config.find_job(None) {
                return Ok((
                    job_cfg.source.clone(),
                    job_cfg.dest.clone(),
                    job_cfg.compress,
                ));
            }
        }
    }
    let src = source.ok_or_else(|| nuwa_backup::errors::NuwaError::InvalidArgument {
        detail: "Missing --source argument".to_string(),
        suggestion: "Use --source [path] or --job [name] to load from config".to_string(),
    })?;
    let dst = dest.ok_or_else(|| nuwa_backup::errors::NuwaError::InvalidArgument {
        detail: "Missing --dest argument".to_string(),
        suggestion: "Use --dest [path] or --job [name] to load from config".to_string(),
    })?;
    Ok((src, dst, compress))
}

#[allow(clippy::too_many_arguments)]
fn record_operation_history(
    backup_id: &str,
    operation: &str,
    source_root: &str,
    dest_path: &std::path::Path,
    job_name: Option<&str>,
    file_count: u64,
    total_bytes: u64,
    duration_ms: u64,
    exit_code: i32,
    status: &str,
) {
    let db_path = nuwa_backup::history::HistoryDb::history_db_path(dest_path);
    match nuwa_backup::history::HistoryDb::open_or_create(&db_path) {
        Ok(db) => {
            let record = nuwa_backup::history::OperationRecord {
                backup_id: backup_id.to_string(),
                operation: operation.to_string(),
                timestamp: chrono::Utc::now().to_rfc3339(),
                source_root: source_root.to_string(),
                dest_path: dest_path.to_string_lossy().to_string(),
                job_name: job_name.map(|s| s.to_string()),
                file_count,
                total_bytes,
                duration_ms,
                exit_code,
                status: status.to_string(),
            };
            if let Err(e) = db.record_operation(&record) {
                eprintln!("  [WARNING] Failed to record operation history: {}", e);
            }
        }
        Err(e) => {
            eprintln!("  [WARNING] Cannot open history database: {}", e);
        }
    }
}
