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
            dest: _dest,
            compress,
            job,
            #[cfg(feature = "repository")]
            repo,
            json_output,
        } => {
            let _job_name = job.clone();
            #[cfg(feature = "repository")]
            {
                let repo_path = match repo {
                    Some(ref p) => p.as_path(),
                    None => {
                        eprintln!("Error: --repo <path> is required for backup.");
                        process::exit(ExitCode::InvalidArgs as i32);
                    }
                };
                let src_path = match source {
                    Some(ref s) => s.clone(),
                    None => {
                        if let Some(ref jn) = job {
                            if let Ok(config) = nuwa_backup::config::Config::load() {
                                if let Ok((_, job_cfg)) = config.find_job(Some(jn)) {
                                    job_cfg.source.clone()
                                } else {
                                    eprintln!("Error: Job not found in config.");
                                    process::exit(ExitCode::InvalidArgs as i32);
                                }
                            } else {
                                eprintln!("Error: --source <path> or --job <name> is required.");
                                process::exit(ExitCode::InvalidArgs as i32);
                            }
                        } else {
                            eprintln!("Error: --source <path> or --job <name> is required.");
                            process::exit(ExitCode::InvalidArgs as i32);
                        }
                    }
                };
                let exit_code = execute_repo_backup(repo_path, &src_path, compress, json_output);
                process::exit(exit_code as i32);
            }
            #[cfg(not(feature = "repository"))]
            {
                eprintln!("Error: --repo is required. Build with --features repository.");
                process::exit(ExitCode::InvalidArgs as i32);
            }
        }
        Command::Restore {
            backup,
            dest,
            overwrite,
            json_output,
        } => {
            let start = std::time::Instant::now();
            #[cfg(feature = "repository")]
            {
                let repo = match nuwa_backup::repository::open_repo(&backup) {
                    Ok(h) => h,
                    Err(e) => {
                        let msg = format!("Cannot open repository: {}", e);
                        if json_output {
                            let out = cli_output::JsonOutput::failure("restore", msg.as_str(), 0);
                            let code = cli_output::print_json_compact(&out);
                            process::exit(code as i32);
                        }
                        eprintln!("{}", msg);
                        process::exit(ExitCode::GeneralFailure as i32);
                    }
                };
                let point_id = match find_latest_committed_restore_point(&repo) {
                    Ok(Some(id)) => id,
                    Ok(None) => {
                        let msg = "No COMMITTED restore points found in repository.";
                        if json_output {
                            let out = cli_output::JsonOutput::failure("restore", msg, 0);
                            let code = cli_output::print_json_compact(&out);
                            process::exit(code as i32);
                        }
                        eprintln!("{}", msg);
                        process::exit(ExitCode::RestoreFailure as i32);
                    }
                    Err(e) => {
                        let msg1 = format!("Failed to query restore points: {}", e);
                        if json_output {
                            let out = cli_output::JsonOutput::failure("restore", msg1.as_str(), 0);
                            let code = cli_output::print_json_compact(&out);
                            process::exit(code as i32);
                        }
                        eprintln!("{}", msg1);
                        process::exit(ExitCode::GeneralFailure as i32);
                    }
                };
                let reader =
                    match nuwa_backup::repository::file_restore_reader::FileRestoreReader::open(
                        &repo, &point_id,
                    ) {
                        Ok(r) => r,
                        Err(e) => {
                            let msg2 = format!("Cannot open restore reader: {}", e);
                            if json_output {
                                let out =
                                    cli_output::JsonOutput::failure("restore", msg2.as_str(), 0);
                                let code = cli_output::print_json_compact(&out);
                                process::exit(code as i32);
                            }
                            eprintln!("{}", msg2);
                            process::exit(ExitCode::GeneralFailure as i32);
                        }
                    };
                match reader.restore_all(&dest, overwrite) {
                    Ok(outcome) => {
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let (summary_r, status_r) = match &outcome {
                            nuwa_backup::repository::file_restore_reader::RestoreOutcome::Complete(s) => (s, "complete"),
                            nuwa_backup::repository::file_restore_reader::RestoreOutcome::Partial(s) => (s, "partial"),
                        };
                        if json_output {
                            let msg3 = format!(
                                r#"{{"restore_point_id":"{}","files_restored":{},"directories_restored":{},"total_bytes_restored":{},"status":"{}"}}"#,
                                summary_r.point_id,
                                summary_r.files_restored,
                                summary_r.directories_restored,
                                summary_r.total_bytes_restored,
                                status_r
                            );
                            let out =
                                cli_output::JsonOutput::success("restore", &msg3, duration_ms);
                            let code = cli_output::print_json_compact(&out);
                            process::exit(code as i32);
                        }
                        println!("[OK] Restore completed.");
                        println!("  Restore Point: {}", summary_r.point_id);
                        println!("  Files restored: {}", summary_r.files_restored);
                        println!("  Directories restored: {}", summary_r.directories_restored);
                        println!("  Total bytes: {}", summary_r.total_bytes_restored);
                        if summary_r.failed_files.is_empty() {
                            ExitCode::Success
                        } else {
                            println!("  Failed files: {}", summary_r.failed_files.len());
                            ExitCode::RestoreFailure
                        }
                    }
                    Err(e) => {
                        let duration_ms2 = start.elapsed().as_millis() as u64;
                        let msg4 = format!("Restore failed: {}", e);
                        if json_output {
                            let out = cli_output::JsonOutput::failure(
                                "restore",
                                msg4.as_str(),
                                duration_ms2,
                            );
                            let code = cli_output::print_json_compact(&out);
                            process::exit(code as i32);
                        }
                        eprintln!("[ERROR] {}", msg4);
                        ExitCode::GeneralFailure
                    }
                }
            }
            #[cfg(not(feature = "repository"))]
            {
                eprintln!(
                    "Error: Restore requires repository feature. Build with --features repository."
                );
                process::exit(ExitCode::GeneralFailure as i32);
            }
        }
        Command::History {
            dest,
            limit,
            operation,
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
#[cfg(feature = "repository")]
fn find_latest_committed_restore_point(
    repo: &nuwa_backup::repository::RepoHandle,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let conn = repo.repo_db()?;
    let result = conn.query_row(
        "SELECT point_id FROM restore_points WHERE status = 'COMMITTED' ORDER BY created_at DESC LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    );
    match result {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.into()),
    }
}
