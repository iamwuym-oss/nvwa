// ============================================================================
// cli.rs -- CLI argument parsing (manual, no clap dependency)
// ============================================================================

use crate::errors::NuwaError;
use std::path::PathBuf;

pub enum Command {
    Init {
        /// Optional explicit config file path
        config_path: Option<PathBuf>,
    },
    Backup {
        source: Option<PathBuf>,
        dest: Option<PathBuf>,
        compress: bool,
        /// Optional job name, mutually exclusive with source/dest
        job: Option<String>,

        /// Repository path (P-01, mutually exclusive with --dest/--job)
        #[cfg(feature = "repository")]
        repo: Option<PathBuf>,
        /// JSON output mode
        json_output: bool,
    },
    Restore {
        backup: PathBuf,
        dest: PathBuf,
        overwrite: bool,
        /// JSON output mode
        json_output: bool,
    },
    Verify {
        backup: PathBuf,
        /// JSON output mode
        json_output: bool,
    },
    List {
        dest: PathBuf,
        /// JSON output mode
        json_output: bool,
    },
    History {
        /// Backup destination root directory
        dest: PathBuf,
        /// Maximum number of records to return (default 10)
        limit: u32,
        /// Optional operation type filter (backup / restore / verify)
        operation: Option<String>,
        /// Rebuild history from manifest
        rebuild: bool,
        /// JSON output mode
        json_output: bool,
    },
    Prune {
        /// Backup destination root directory
        dest: PathBuf,
        /// Keep last N valid backup points
        keep_count: Option<u32>,
        /// Keep backup points from last N days
        keep_days: Option<u64>,
        /// Dry-run mode (report only, no deletion)
        dry_run: bool,
        /// JSON output mode
        json_output: bool,
    },
    Schedule {
        /// Subcommand: create, list, delete
        subcommand: ScheduleSubcommand,
    },
    /// Repository management (Phase S)
    #[cfg(feature = "repository")]
    Repo {
        subcommand: crate::repository::cli::commands::RepoSubcommand,
    },
}

#[derive(Debug, Clone)]
pub enum ScheduleSubcommand {
    Create {
        job_name: String,
        trigger: crate::scheduler::ScheduleTrigger,
    },
    List,
    Delete {
        task_id: String,
    },
}

impl Command {
    pub fn from_args() -> Result<Self, NuwaError> {
        let args: Vec<String> = std::env::args().collect();
        if args.len() < 2 {
            return Err(NuwaError::InvalidArgument {
                detail: "Missing subcommand".to_string(),
                suggestion: Self::usage(),
            });
        }

        match args[1].to_lowercase().as_str() {
            "init" => Self::parse_init(&args[2..]),
            "backup" => Self::parse_backup(&args[2..]),
            "restore" => Self::parse_restore(&args[2..]),
            "verify" => Self::parse_verify(&args[2..]),
            "list" => Self::parse_list(&args[2..]),
            "prune" => Self::parse_prune(&args[2..]),
            "schedule" => Self::parse_schedule(&args[2..]),
            #[cfg(feature = "repository")]
            "repo" => Self::parse_repo(&args[2..]),
            "history" => Self::parse_history(&args[2..]),
            "--help" | "-h" => {
                println!("{}", Self::usage_full());
                std::process::exit(0);
            }
            "--version" | "-V" => {
                println!("nuwa-backup v{}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => Err(NuwaError::InvalidArgument {
                detail: format!("Unknown subcommand '{}'", args[1]),
                suggestion: Self::usage(),
            }),
        }
    }

    fn parse_init(args: &[String]) -> Result<Self, NuwaError> {
        let mut config_path = None;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--config" => {
                    i += 1;
                    config_path = Some(PathBuf::from(get_string_arg(args, i, "--config")?));
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "Usage: nuwa init [--config <path>]".to_string(),
                    })
                }
            }
            i += 1;
        }
        Ok(Command::Init { config_path })
    }

    fn parse_backup(args: &[String]) -> Result<Self, NuwaError> {
        let mut source = None;
        let mut dest = None;
        let mut compress = false;
        let mut job = None;
        let mut json_output = false;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--source" => {
                    i += 1;
                    source = Some(PathBuf::from(get_string_arg(args, i, "--source")?));
                }
                "--dest" => {
                    i += 1;
                    dest = Some(PathBuf::from(get_string_arg(args, i, "--dest")?));
                }
                "--compress" => {
                    compress = true;
                }
                "--job" => {
                    i += 1;
                    job = Some(get_string_arg(args, i, "--job")?);
                }
                "--json" => {
                    json_output = true;
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "Usage: nuwa backup --job <name>  or  nuwa backup --source <path> --dest <path> [--compress] [--json]"
                            .to_string(),
                    })
                }
            }
            i += 1;
        }

        let has_job = job.is_some();
        let has_source_dest = source.is_some() || dest.is_some();

        if has_job && has_source_dest {
            return Err(NuwaError::InvalidArgument {
                detail: "--job cannot be used together with --source/--dest".to_string(),
                suggestion: "Choose one mode:\n  nuwa backup --job <name>   (use config file)\n  nuwa backup --source <path> --dest <path>  (legacy mode)".to_string(),
            });
        }

        if !has_job && !has_source_dest {
            match crate::config::try_load_job_with_default_fallback(None) {
                Ok((name, job_cfg)) => {
                    source = Some(job_cfg.source.clone());
                    dest = Some(job_cfg.dest.clone());
                    compress = job_cfg.compress;
                    job = Some(name.to_string());
                }
                Err(_) => {
                    return Err(NuwaError::InvalidArgument {
                        detail: "Missing required arguments".to_string(),
                        suggestion: "Usage:\n  nuwa backup --job <name>   (use config file)\n  nuwa backup --source <path> --dest <path>  (legacy mode)\n\nOr run 'nuwa init' first to create a config file.".to_string(),
                    })
                }
            }
        }

        Ok(Command::Backup {
            source,
            dest,
            compress,
            job,
            #[cfg(feature = "repository")]
            repo: None,
            json_output,
        })
    }

    fn parse_restore(args: &[String]) -> Result<Self, NuwaError> {
        let mut backup = None;
        let mut dest = None;
        let mut overwrite = false;
        let mut json_output = false;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--backup" => {
                    i += 1;
                    backup = Some(PathBuf::from(get_string_arg(args, i, "--backup")?));
                }
                "--dest" => {
                    i += 1;
                    dest = Some(PathBuf::from(get_string_arg(args, i, "--dest")?));
                }
                "--overwrite" => {
                    overwrite = true;
                }
                "--json" => {
                    json_output = true;
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "Usage: nuwa restore --backup <backup_dir> --dest <target_path> [--overwrite] [--json]"
                            .to_string(),
                    })
                }
            }
            i += 1;
        }
        Ok(Command::Restore {
            backup: backup.ok_or_else(|| missing_param("--backup"))?,
            dest: dest.ok_or_else(|| missing_param("--dest"))?,
            overwrite,
            json_output,
        })
    }

    fn parse_verify(args: &[String]) -> Result<Self, NuwaError> {
        let mut backup = None;
        let mut json_output = false;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--backup" => {
                    i += 1;
                    backup = Some(PathBuf::from(get_string_arg(args, i, "--backup")?));
                }
                "--json" => {
                    json_output = true;
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "Usage: nuwa verify --backup <backup_dir> [--json]".to_string(),
                    })
                }
            }
            i += 1;
        }
        Ok(Command::Verify {
            backup: backup.ok_or_else(|| missing_param("--backup"))?,
            json_output,
        })
    }

    fn parse_list(args: &[String]) -> Result<Self, NuwaError> {
        let mut dest = None;
        let mut json_output = false;
        let mut i = 0;
        while i < args.len() {
            match args[i].as_str() {
                "--dest" => {
                    i += 1;
                    dest = Some(PathBuf::from(get_string_arg(args, i, "--dest")?));
                }
                "--json" => {
                    json_output = true;
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "Usage: nuwa list --dest <backup_root> [--json]".to_string(),
                    })
                }
            }
            i += 1;
        }
        Ok(Command::List {
            dest: dest.ok_or_else(|| missing_param("--dest"))?,
            json_output,
        })
    }

    fn parse_history(args: &[String]) -> Result<Self, NuwaError> {
        let mut dest = None;
        let mut limit: u32 = 10;
        let mut operation: Option<String> = None;
        let mut rebuild = false;
        let mut json_output = false;
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "--dest" => {
                    i += 1;
                    dest = Some(PathBuf::from(get_string_arg(args, i, "--dest")?));
                }
                "--limit" => {
                    i += 1;
                    let val = get_string_arg(args, i, "--limit")?;
                    limit = val.parse::<u32>().map_err(|_| NuwaError::InvalidArgument {
                        detail: format!("--limit must be a positive integer, got '{}'", val),
                        suggestion: "Usage: nuwa history --dest <path> --limit N".to_string(),
                    })?;
                }
                "--operation" => {
                    i += 1;
                    let op = get_string_arg(args, i, "--operation")?;
                    let lower = op.to_lowercase();
                    if lower != "backup" && lower != "restore" && lower != "verify" {
                        return Err(NuwaError::InvalidArgument {
                            detail: format!("Unsupported operation type '{}'", op),
                            suggestion: "Supported operations: backup, restore, verify".to_string(),
                        });
                    }
                    operation = Some(lower);
                }
                "--rebuild" => {
                    rebuild = true;
                }
                "--json" => {
                    json_output = true;
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "Usage: nuwa history --dest <path> [--limit N] [--operation backup|restore|verify] [--rebuild] [--json]"
                            .to_string(),
                    })
                }
            }
            i += 1;
        }

        let dest_path = dest.ok_or_else(|| NuwaError::InvalidArgument {
            detail: "Missing --dest argument".to_string(),
            suggestion: "Usage: nuwa history --dest <backup_root>".to_string(),
        })?;

        Ok(Command::History {
            dest: dest_path,
            limit,
            operation,
            rebuild,
            json_output,
        })
    }

    fn parse_prune(args: &[String]) -> Result<Self, NuwaError> {
        let mut dest: Option<PathBuf> = None;
        let mut keep_count: Option<u32> = None;
        let mut keep_days: Option<u64> = None;
        let mut dry_run = false;
        let mut json_output = false;
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "--dest" => {
                    i += 1;
                    dest = Some(PathBuf::from(get_string_arg(args, i, "--dest")?));
                }
                "--keep-count" => {
                    i += 1;
                    let val = get_string_arg(args, i, "--keep-count")?;
                    keep_count = Some(val.parse::<u32>().map_err(|_| {
                        NuwaError::InvalidArgument {
                            detail: format!(
                                "--keep-count must be a positive integer, got '{}'",
                                val
                            ),
                            suggestion: "Usage: nuwa prune --dest <path> --keep-count N"
                                .to_string(),
                        }
                    })?);
                }
                "--keep-days" => {
                    i += 1;
                    let val = get_string_arg(args, i, "--keep-days")?;
                    keep_days = Some(val.parse::<u64>().map_err(|_| {
                        NuwaError::InvalidArgument {
                            detail: format!(
                                "--keep-days must be a positive integer, got '{}'",
                                val
                            ),
                            suggestion: "Usage: nuwa prune --dest <path> --keep-days N"
                                .to_string(),
                        }
                    })?);
                }
                "--dry-run" => {
                    dry_run = true;
                }
                "--json" => {
                    json_output = true;
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "Usage: nuwa prune --dest <path> [--keep-count N] [--keep-days N] [--dry-run] [--json]"
                            .to_string(),
                    })
                }
            }
            i += 1;
        }

        let dest_path = dest.ok_or_else(|| NuwaError::InvalidArgument {
            detail: "Missing --dest argument".to_string(),
            suggestion: "Usage: nuwa prune --dest <backup_root>".to_string(),
        })?;

        if keep_count.is_none() && keep_days.is_none() {
            return Err(NuwaError::InvalidArgument {
                detail: "Missing --keep-count or --keep-days".to_string(),
                suggestion:
                    "Specify at least one retention policy: --keep-count N or --keep-days N"
                        .to_string(),
            });
        }

        Ok(Command::Prune {
            dest: dest_path,
            keep_count,
            keep_days,
            dry_run,
            json_output,
        })
    }

    fn parse_schedule(args: &[String]) -> Result<Self, NuwaError> {
        if args.is_empty() {
            return Err(NuwaError::InvalidArgument {
                detail: "Missing schedule subcommand".to_string(),
                suggestion: "Usage: nuwa schedule create|list|delete".to_string(),
            });
        }

        match args[0].to_lowercase().as_str() {
            "create" => Self::parse_schedule_create(&args[1..]),
            "list" => Ok(Command::Schedule {
                subcommand: ScheduleSubcommand::List,
            }),
            "delete" => Self::parse_schedule_delete(&args[1..]),
            _ => Err(NuwaError::InvalidArgument {
                detail: format!("Unknown schedule subcommand '{}'", args[0]),
                suggestion: "Usage: nuwa schedule create|list|delete".to_string(),
            }),
        }
    }

    fn parse_schedule_create(args: &[String]) -> Result<Self, NuwaError> {
        let mut job_name: Option<String> = None;
        let mut once = false;
        let mut daily = false;
        let mut weekly = false;
        let mut monthly = false;
        let mut on_logon = false;
        let mut at: Option<String> = None;
        let mut days: Option<String> = None;
        let mut day: Option<u32> = None;
        let mut delay: Option<u32> = None;
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "--job" => {
                    i += 1;
                    job_name = Some(get_string_arg(args, i, "--job")?);
                }
                "--once" => {
                    once = true;
                }
                "--daily" => {
                    daily = true;
                }
                "--weekly" => {
                    weekly = true;
                }
                "--monthly" => {
                    monthly = true;
                }
                "--on-logon" => {
                    on_logon = true;
                }
                "--at" => {
                    i += 1;
                    at = Some(get_string_arg(args, i, "--at")?);
                }
                "--days" => {
                    i += 1;
                    days = Some(get_string_arg(args, i, "--days")?);
                }
                "--day" => {
                    i += 1;
                    let val = get_string_arg(args, i, "--day")?;
                    day = Some(val.parse::<u32>().map_err(|_| {
                        NuwaError::InvalidArgument {
                            detail: format!("--day must be a positive integer, got '{}'", val),
                            suggestion: "Example: --monthly --day 1 --at 02:00".to_string(),
                        }
                    })?);
                }
                "--delay" => {
                    i += 1;
                    let val = get_string_arg(args, i, "--delay")?;
                    delay = Some(val.parse::<u32>().map_err(|_| {
                        NuwaError::InvalidArgument {
                            detail: format!("--delay must be a positive integer (seconds), got '{}'", val),
                            suggestion: "Example: --on-logon --delay 300".to_string(),
                        }
                    })?);
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "Usage: nuwa schedule create --job <name> [--once|--daily|--weekly|--monthly|--on-logon] [options]".to_string(),
                    })
                }
            }
            i += 1;
        }

        let jn = job_name.ok_or_else(|| NuwaError::InvalidArgument {
            detail: "Missing --job argument".to_string(),
            suggestion: "Usage: nuwa schedule create --job <name> --daily --at 22:00".to_string(),
        })?;

        // Determine trigger type (must specify exactly one)
        let trigger_count = [once, daily, weekly, monthly, on_logon]
            .iter()
            .filter(|&&x| x)
            .count();
        if trigger_count != 1 {
            return Err(NuwaError::InvalidArgument {
                detail: "Must specify exactly one trigger type: --once, --daily, --weekly, --monthly, or --on-logon".to_string(),
                suggestion: "Example: nuwa schedule create --job default --daily --at 22:00".to_string(),
            });
        }

        let trigger = if once {
            let at_val = at.ok_or_else(|| NuwaError::InvalidArgument {
                detail: "--once requires --at".to_string(),
                suggestion: "Example: --once --at \"2026-08-01 10:00\"".to_string(),
            })?;
            crate::scheduler::ScheduleTrigger::Once { at: at_val }
        } else if daily {
            let at_val = at.ok_or_else(|| NuwaError::InvalidArgument {
                detail: "--daily requires --at".to_string(),
                suggestion: "Example: --daily --at 22:00".to_string(),
            })?;
            crate::scheduler::ScheduleTrigger::Daily { at: at_val }
        } else if weekly {
            let at_val = at.ok_or_else(|| NuwaError::InvalidArgument {
                detail: "--weekly requires --at".to_string(),
                suggestion: "Example: --weekly --days Mon,Fri --at 03:00".to_string(),
            })?;
            let days_str = days.ok_or_else(|| NuwaError::InvalidArgument {
                detail: "--weekly requires --days".to_string(),
                suggestion: "Example: --weekly --days Mon,Fri --at 03:00".to_string(),
            })?;
            let days_vec: Vec<String> = days_str.split(',').map(|s| s.trim().to_string()).collect();
            crate::scheduler::ScheduleTrigger::Weekly {
                days: days_vec,
                at: at_val,
            }
        } else if monthly {
            let at_val = at.ok_or_else(|| NuwaError::InvalidArgument {
                detail: "--monthly requires --at".to_string(),
                suggestion: "Example: --monthly --day 1 --at 02:00".to_string(),
            })?;
            let day_val = day.ok_or_else(|| NuwaError::InvalidArgument {
                detail: "--monthly requires --day".to_string(),
                suggestion: "Example: --monthly --day 1 --at 02:00".to_string(),
            })?;
            crate::scheduler::ScheduleTrigger::Monthly {
                day: day_val,
                at: at_val,
            }
        } else if on_logon {
            let delay_val = delay.unwrap_or(0);
            crate::scheduler::ScheduleTrigger::OnLogon {
                delay_seconds: delay_val,
            }
        } else {
            unreachable!()
        };

        Ok(Command::Schedule {
            subcommand: ScheduleSubcommand::Create {
                job_name: jn,
                trigger,
            },
        })
    }

    fn parse_schedule_delete(args: &[String]) -> Result<Self, NuwaError> {
        let mut task_id: Option<String> = None;
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "--task-id" => {
                    i += 1;
                    task_id = Some(get_string_arg(args, i, "--task-id")?);
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "Usage: nuwa schedule delete --task-id <task_name_or_job_name>"
                            .to_string(),
                    })
                }
            }
            i += 1;
        }

        let tid = task_id.ok_or_else(|| NuwaError::InvalidArgument {
            detail: "Missing --task-id argument".to_string(),
            suggestion: "Usage: nuwa schedule delete --task-id NuwaBackup-default".to_string(),
        })?;

        Ok(Command::Schedule {
            subcommand: ScheduleSubcommand::Delete { task_id: tid },
        })
    }

    /// Parse `nuwa repo <subcommand> [args]`
    #[cfg(feature = "repository")]
    fn parse_repo(args: &[String]) -> Result<Self, NuwaError> {
        let sub = crate::repository::cli::commands::RepoSubcommand::parse(args)?;
        Ok(Command::Repo { subcommand: sub })
    }

    fn usage() -> String {
        "Usage: nuwa <subcommand> [options]\n  Subcommands: init, backup, restore, verify, list, history, prune, schedule, repo\n  nuwa --help for detailed help".to_string()
    }

    fn usage_full() -> String {
        let v = env!("CARGO_PKG_VERSION");
        format!(
            r#"Nuwa Backup v{}
File-level backup and restore tool

Subcommands:
  init      Initialize config file
  backup    Full backup
  restore   Restore
  verify    Verify
  list      List backup points
  history   Operation history
  prune     Prune old backup points by retention policy
  schedule  Manage Windows Task Scheduler scheduled tasks

Global option (all commands):
  --json            Output in JSON format (for scripting/automation)

init options:
  --config <path>   Specify config file path (optional)

backup options (choose one mode):
  Mode 1 - Use config file:
  --job <name>      Run backup using a job defined in config

  Mode 2 - Legacy mode:
  --source <path>   Source path
  --dest <path>     Destination path
  --compress        Enable compression (optional)
  --json            Output in JSON format

restore options:

    Mode 3 - Repository mode (requires --features repository):
    --source <path>   Source path
    --repo <path>     Repository path (required, use 'nuwa repo init' first)
    --compress        Enable compression (optional)
    --json            Output in JSON format

  restore options:
  --backup <path>   Backup directory (required)
  --dest <path>     Restore target (required)
  --overwrite       Overwrite existing files (optional)
  --json            Output in JSON format

verify options:
  --backup <path>   Backup directory (required)
  --json            Output in JSON format

list options:
  --dest <path>     Backup root directory (required)
  --json            Output in JSON format

history options:
  --dest <path>           Backup root directory (required)
  --limit N               Show last N records (optional, default 10)
  --operation <type>      Filter by operation: backup / restore / verify (optional)
  --rebuild               Rebuild history database from manifests (optional)
  --json                  Output in JSON format

prune options:
  --dest <path>           Backup root directory (required)
  --keep-count N          Keep last N valid backup points (optional)
  --keep-days N           Keep backup points from last N days (optional)
  --dry-run               Preview what would be deleted (safe, no deletion)
  --json                  Output in JSON format

schedule subcommands:
  create    Create a scheduled backup task
  list      List all Nuwa Backup scheduled tasks
  delete    Delete a scheduled task

schedule create options:
  --job <name>            Job name from config (required)
  --once                  Run once at specified date/time
  --daily                 Run daily at specified time
  --weekly                Run weekly on specified days
  --monthly               Run monthly on specified day
  --on-logon            Run on user logon
  --at "YYYY-MM-DD HH:MM"   Date and time for --once, or time HH:MM for --daily/--weekly/--monthly
  --days Mon,Fri          Days of week for --weekly (comma-separated, 3-letter abbrev)
  --day N                 Day of month for --monthly (1-31)
  --delay N               Delay in seconds for --on-logon (optional, default 0)

schedule list options:
  (no additional arguments needed)

schedule delete options:
  --task-id <id>          Task name (NuwaBackup-<jobname>) or just job name
"#,
            v
        )
    }
}

fn get_string_arg(args: &[String], idx: usize, param: &str) -> Result<String, NuwaError> {
    if idx >= args.len() {
        Err(NuwaError::InvalidArgument {
            detail: format!("Argument {} missing value", param),
            suggestion: format!("{} requires a value after it", param),
        })
    } else {
        Ok(args[idx].clone())
    }
}

fn missing_param(param: &str) -> NuwaError {
    NuwaError::InvalidArgument {
        detail: format!("Missing {} argument", param),
        suggestion: format!("Use --{} <path> to specify", param),
    }
}
