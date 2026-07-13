use nuwa_backup::cli::Command;
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
        Command::Backup { .. } => {
            eprintln!("Error: Backup is temporarily unavailable during storage engine redesign.");
            eprintln!("  The new engine will support .nwb single-file backup format.");
            ExitCode::GeneralFailure
        }
        Command::Restore { .. } => {
            eprintln!("Error: Restore is temporarily unavailable during storage engine redesign.");
            eprintln!("  The new engine will support .nwb single-file restore.");
            ExitCode::RestoreFailure
        }
        Command::History {
            dest,
            limit,
            operation,
            json_output,
        } => {
            let db_path = nuwa_backup::history::HistoryDb::history_db_path(&dest);
            let records = match nuwa_backup::history::HistoryDb::open_or_create(&db_path) {
                Ok(db) => match db.query_history(limit, operation.as_deref()) {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("{}", e);
                        process::exit(ExitCode::GeneralFailure as i32);
                    }
                },
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(ExitCode::GeneralFailure as i32);
                }
            };
            if json_output {
                for r in &records {
                    println!(
                        "{{\"operation\":\"{}\",\"timestamp\":\"{}\",\"status\":\"{}\",\"source\":\"{}\",\"file_count\":{},\"total_bytes\":{}}}",
                        r.operation, r.timestamp, r.status, r.source_root, r.file_count, r.total_bytes
                    );
                }
            } else {
                nuwa_backup::history::print_history(&records);
            }
            ExitCode::Success
        }
        Command::Schedule { subcommand } => match subcommand {
            nuwa_backup::cli::ScheduleSubcommand::Create { job_name, trigger } => {
                let task_id = format!("NuwaBackup-{}", job_name);
                match nuwa_backup::scheduler::create_task(&task_id, &trigger) {
                    Ok(()) => {
                        println!("[OK] Scheduled task '{}' created.", task_id);
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
    };
    process::exit(exit as i32);
}
