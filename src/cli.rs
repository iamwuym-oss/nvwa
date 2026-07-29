// ============================================================================
// cli.rs -- CLI argument parsing (manual, no clap dependency)
// ============================================================================

use crate::errors::NuwaError;
use std::path::PathBuf;

pub enum Command {
    Init {
        config_path: Option<PathBuf>,
    },
    Backup {
        source: Option<PathBuf>,
        dest: Option<PathBuf>,
        compress: bool,
        job: Option<String>,
        json_output: bool,
    },
    Restore {
        backup: PathBuf,
        dest: PathBuf,
        overwrite: bool,
        json_output: bool,
    },
    History {
        dest: PathBuf,
        limit: u32,
        operation: Option<String>,
        json_output: bool,
    },
    Schedule {
        subcommand: ScheduleSubcommand,
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
            "schedule" => Self::parse_schedule(&args[2..]),
            "history" => Self::parse_history(&args[2..]),
            "--help" | "-h" => {
                println!("{}", Self::usage_full());
                std::process::exit(0);
            }
            "--version" | "-V" => {
                let v = nwb_format::version::FormatVersion::current();
                println!("nuwa-backup v{}", env!("CARGO_PKG_VERSION"));
                println!("NWB format {}", v);
                println!("{}", v.lifecycle_banner());
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
                        suggestion:
                            "Usage: nuwa backup --source <path> --dest <path> [--compress] [--json]"
                                .to_string(),
                    })
                }
            }
            i += 1;
        }
        Ok(Command::Backup {
            source,
            dest,
            compress,
            job,
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
                _ => return Err(NuwaError::InvalidArgument {
                    detail: format!("Unknown argument '{}'", args[i]),
                    suggestion:
                        "Usage: nuwa restore --backup <path> --dest <path> [--overwrite] [--json]"
                            .to_string(),
                }),
            }
            i += 1;
        }

        let backup_path = backup.ok_or_else(|| missing_param("backup"))?;
        let dest_path = dest.ok_or_else(|| missing_param("dest"))?;

        Ok(Command::Restore {
            backup: backup_path,
            dest: dest_path,
            overwrite,
            json_output,
        })
    }

    fn parse_history(args: &[String]) -> Result<Self, NuwaError> {
        let mut dest = None;
        let mut limit = 10u32;
        let mut operation = None;
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
                    limit = val.parse().unwrap_or(10);
                }
                "--operation" => {
                    i += 1;
                    operation = Some(get_string_arg(args, i, "--operation")?);
                }
                "--json" => {
                    json_output = true;
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "Usage: nuwa history --dest <path> [--limit N] [--operation backup|restore|verify] [--json]"
                            .to_string(),
                    })
                }
            }
            i += 1;
        }
        let dest_path = dest.ok_or_else(|| missing_param("dest"))?;
        Ok(Command::History {
            dest: dest_path,
            limit,
            operation,
            json_output,
        })
    }

    fn parse_schedule(args: &[String]) -> Result<Self, NuwaError> {
        if args.is_empty() {
            return Err(NuwaError::InvalidArgument {
                detail: "Missing schedule subcommand".to_string(),
                suggestion: "Usage: nuwa schedule create/list/delete [options]".to_string(),
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
                suggestion: "Usage: nuwa schedule create/list/delete".to_string(),
            }),
        }
    }

    fn parse_schedule_create(args: &[String]) -> Result<Self, NuwaError> {
        let mut jn: Option<String> = None;
        let mut trigger: Option<crate::scheduler::ScheduleTrigger> = None;
        let mut i = 0;

        while i < args.len() {
            match args[i].as_str() {
                "--job" => {
                    i += 1;
                    jn = Some(get_string_arg(args, i, "--job")?);
                }
                "--once" => {
                    i += 1;
                    let at = get_string_arg(args, i, "--at")?;
                    trigger = Some(crate::scheduler::ScheduleTrigger::Once { at });
                }
                "--daily" => {
                    i += 1;
                    let at = get_string_arg(args, i, "--at")?;
                    trigger = Some(crate::scheduler::ScheduleTrigger::Daily { at });
                }
                "--weekly" => {
                    i += 1;
                    let at = get_string_arg(args, i, "--at")?;
                    i += 1;
                    let days_str = get_string_arg(args, i, "--days")?;
                    let days: Vec<String> =
                        days_str.split(',').map(|s| s.trim().to_string()).collect();
                    trigger = Some(crate::scheduler::ScheduleTrigger::Weekly { days, at });
                }
                "--monthly" => {
                    i += 1;
                    let at = get_string_arg(args, i, "--at")?;
                    i += 1;
                    let day_str = get_string_arg(args, i, "--day")?;
                    let day: u32 = day_str.parse().map_err(|_| NuwaError::InvalidArgument {
                        detail: format!("Invalid day: '{}'. Expected 1-31", day_str),
                        suggestion: "Use --day 1 through --day 31".to_string(),
                    })?;
                    if !(1..=31).contains(&day) {
                        return Err(NuwaError::InvalidArgument {
                            detail: format!("Invalid day: '{}'. Day must be between 1 and 31", day),
                            suggestion: "Use --day 1 through --day 31".to_string(),
                        });
                    }
                    trigger = Some(crate::scheduler::ScheduleTrigger::Monthly { day, at });
                }
                "--on-logon" => {
                    let mut delay = 0u32;
                    if i + 2 < args.len() && args[i + 1] == "--delay" {
                        i += 2;
                        delay = get_string_arg(args, i, "--delay")?.parse().unwrap_or(0);
                    }
                    trigger = Some(crate::scheduler::ScheduleTrigger::OnLogon {
                        delay_seconds: delay,
                    });
                }
                _ => {
                    return Err(NuwaError::InvalidArgument {
                        detail: format!("Unknown argument '{}'", args[i]),
                        suggestion: "See --help for schedule create options".to_string(),
                    })
                }
            }
            i += 1;
        }

        let job_name = jn.ok_or_else(|| NuwaError::InvalidArgument {
            detail: "Missing --job argument".to_string(),
            suggestion: "Usage: nuwa schedule create --job <name> --daily --at HH:MM".to_string(),
        })?;
        let tr = trigger.ok_or_else(|| NuwaError::InvalidArgument {
            detail: "Missing trigger type (--once/--daily/--weekly/--monthly/--on-logon)"
                .to_string(),
            suggestion: "Specify one trigger type".to_string(),
        })?;

        Ok(Command::Schedule {
            subcommand: ScheduleSubcommand::Create {
                job_name,
                trigger: tr,
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

    fn usage() -> String {
        "Usage: nuwa <subcommand> [options]\n  Subcommands: init, backup, restore, history, schedule\n  nuwa --help for detailed help".to_string()
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
  history   Operation history
  schedule  Manage Windows Task Scheduler scheduled tasks

Global option (all commands):
  --json            Output in JSON format (for scripting/automation)

init options:
  --config <path>   Specify config file path (optional)

backup options:
  --source <path>   Source path
  --dest <path>     Destination path
  --compress        Enable compression (optional)
  --job <name>      Run backup using a job defined in config
  --json            Output in JSON format

restore options:
  --backup <path>   Backup path (required)
  --dest <path>     Restore target (required)
  --overwrite       Overwrite existing files (optional)
  --json            Output in JSON format

history options:
  --dest <path>           Backup root directory (required)
  --limit N               Show last N records (optional, default 10)
  --operation <type>      Filter by operation: backup / restore / verify (optional)
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
