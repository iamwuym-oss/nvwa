// ============================================================================
// scheduler.rs -- Windows Task Scheduler integration
//
// Responsibilities:
// 1. Create scheduled tasks via schtasks.exe (one-time, daily, weekly, monthly, on-logon)
// 2. List all Nuwa Backup scheduled tasks
// 3. Delete scheduled tasks
//
// Design principles:
// - Uses schtasks.exe via std::process::Command -- no daemon/service required
// - All tasks are created under a NuwaBackup-<jobname> naming convention
// - Task runs: nuwa backup --job <jobname>
// - schtasks.exe is available on all modern Windows systems
// - Not a daemon, not a Windows Service, not a long-running process
// ============================================================================

use crate::errors::NuwaError;

/// Prefix for all Nuwa Backup scheduled task names
pub const TASK_PREFIX: &str = "NuwaBackup-";

/// Schedule trigger types
#[derive(Debug, Clone, PartialEq)]
pub enum ScheduleTrigger {
    /// Run once at a specific date/time
    Once { at: String },
    /// Run daily at a specific time
    Daily { at: String },
    /// Run weekly on specific days at a specific time
    Weekly { days: Vec<String>, at: String },
    /// Run monthly on a specific day at a specific time
    Monthly { day: u32, at: String },
    /// Run on user logon with a delay in seconds
    OnLogon { delay_seconds: u32 },
}

/// A scheduled task managed by Nuwa Backup
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    /// Full task name (NuwaBackup-<jobname>)
    pub task_name: String,
    /// Job name this task is associated with
    pub job_name: String,
    /// Status string from schtasks (e.g., "Ready", "Running", "Disabled")
    pub status: String,
    /// Schedule trigger description (e.g., "Daily at 22:00")
    pub schedule: String,
    /// Next run time
    pub next_run: String,
}

/// Create a scheduled task
pub fn create_task(job_name: &str, trigger: &ScheduleTrigger) -> Result<(), NuwaError> {
    // Validate that the job exists in config
    let config = crate::config::Config::load()?;
    config.find_job(Some(job_name)).map_err(|_| {
        NuwaError::InvalidArgument {
            detail: format!("Job '{}' not found in config", job_name),
            suggestion: "Run 'nuwa init' to create a config file with job definitions, or check the job name".to_string(),
        }
    })?;

    let (_, job_cfg) = config.find_job(Some(job_name)).unwrap();
    let task_name = format!("{}{}", TASK_PREFIX, job_name);

    // Build the task command line
    let exe_path = std::env::current_exe().map_err(|e| NuwaError::General {
        detail: format!("Cannot determine executable path: {}", e),
        suggestion: "Internal error".to_string(),
    })?;

    let compress_flag = if job_cfg.compress { " --compress" } else { "" };
    let task_command = format!(
        "\"{}\" backup --job \"{}\"{}",
        exe_path.to_string_lossy(),
        job_name,
        compress_flag
    );

    let mut cmd = std::process::Command::new("schtasks");
    cmd.arg("/CREATE");
    cmd.arg("/TN").arg(&task_name);
    cmd.arg("/TR").arg(&task_command);
    cmd.arg("/F"); // Force: overwrite existing task

    match trigger {
        ScheduleTrigger::Once { at } => {
            let parts: Vec<&str> = at.split_whitespace().collect();
            if parts.len() != 2 {
                return Err(NuwaError::InvalidArgument {
                    detail: format!(
                        "Invalid --at format '{}' -- expected 'YYYY-MM-DD HH:MM'",
                        at
                    ),
                    suggestion: "Example: --once --at \"2026-08-01 10:00\"".to_string(),
                });
            }
            cmd.arg("/SC").arg("ONCE");
            cmd.arg("/SD").arg(parts[0]);
            cmd.arg("/ST").arg(parts[1]);
        }
        ScheduleTrigger::Daily { at } => {
            validate_time(at)?;
            cmd.arg("/SC").arg("DAILY");
            cmd.arg("/ST").arg(at);
        }
        ScheduleTrigger::Weekly { days, at } => {
            validate_time(at)?;
            cmd.arg("/SC").arg("WEEKLY");
            cmd.arg("/D").arg(days.join(","));
            cmd.arg("/ST").arg(at);
        }
        ScheduleTrigger::Monthly { day, at } => {
            validate_time(at)?;
            if *day < 1 || *day > 31 {
                return Err(NuwaError::InvalidArgument {
                    detail: format!("--day must be between 1 and 31, got {}", day),
                    suggestion: "Example: --monthly --day 1 --at 02:00".to_string(),
                });
            }
            cmd.arg("/SC").arg("MONTHLY");
            cmd.arg("/D").arg(day.to_string());
            cmd.arg("/ST").arg(at);
        }
        ScheduleTrigger::OnLogon { delay_seconds } => {
            cmd.arg("/SC").arg("ONLOGON");
            let hours = delay_seconds / 3600;
            let minutes = (delay_seconds % 3600) / 60;
            let secs = delay_seconds % 60;
            let delay_str = format!("{:02}:{:02}:{:02}", hours, minutes, secs);
            cmd.arg("/DELAY").arg(delay_str);
        }
    }

    let output = cmd.output().map_err(|e| NuwaError::General {
        detail: format!("Failed to execute schtasks.exe: {}", e),
        suggestion: "Ensure schtasks.exe is available and you have permission to create tasks"
            .to_string(),
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NuwaError::General {
            detail: format!("schtasks.exe failed: {}", stderr.trim()),
            suggestion: "Check that the task name does not conflict and you have administrator privileges if required".to_string(),
        });
    }

    Ok(())
}

/// List all Nuwa Backup scheduled tasks
pub fn list_tasks() -> Result<Vec<ScheduledTask>, NuwaError> {
    let output = std::process::Command::new("schtasks")
        .arg("/QUERY")
        .arg("/FO")
        .arg("CSV")
        .output()
        .map_err(|e| NuwaError::General {
            detail: format!("Failed to execute schtasks.exe: {}", e),
            suggestion: "Ensure schtasks.exe is available".to_string(),
        })?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut tasks: Vec<ScheduledTask> = Vec::new();

    for line in stdout.lines().skip(1) {
        if line.trim().is_empty() || line.starts_with("\"TaskName\"") {
            continue;
        }

        let columns = parse_csv_line(line);
        if columns.len() < 3 {
            continue;
        }

        let task_name = columns.first().cloned().unwrap_or_default();
        let next_run = columns.get(1).cloned().unwrap_or_default();
        let status = columns.get(2).cloned().unwrap_or_default();
        // Without /V flag, schtasks /FO CSV uses 3 columns:
        // TaskName, NextRun, Status
        let schedule = status.clone();

        if task_name.starts_with(&format!("\\{}", TASK_PREFIX))
            || task_name.starts_with(TASK_PREFIX)
        {
            let job_name = if let Some(name) = task_name.strip_prefix(&format!("\\{}", TASK_PREFIX))
            {
                name.to_string()
            } else {
                task_name
                    .strip_prefix(TASK_PREFIX)
                    .unwrap_or(&task_name)
                    .to_string()
            };

            tasks.push(ScheduledTask {
                task_name,
                job_name,
                status,
                schedule,
                next_run,
            });
        }
    }

    Ok(tasks)
}

/// Delete a scheduled task by task name (full name or job name)
pub fn delete_task(task_id: &str) -> Result<(), NuwaError> {
    let task_name = if task_id.starts_with(TASK_PREFIX) {
        task_id.to_string()
    } else {
        format!("{}{}", TASK_PREFIX, task_id)
    };

    let output = std::process::Command::new("schtasks")
        .arg("/DELETE")
        .arg("/TN")
        .arg(&task_name)
        .arg("/F")
        .output()
        .map_err(|e| NuwaError::General {
            detail: format!("Failed to execute schtasks.exe: {}", e),
            suggestion: "Ensure schtasks.exe is available".to_string(),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(NuwaError::General {
            detail: format!("Failed to delete task '{}': {}", task_name, stderr.trim()),
            suggestion: "Check that the task exists and you have permission to delete it"
                .to_string(),
        });
    }

    Ok(())
}

/// Print scheduled tasks to console
pub fn print_tasks(tasks: &[ScheduledTask]) {
    if tasks.is_empty() {
        println!("No scheduled tasks found.");
        println!("Use 'nuwa schedule create' to create a scheduled backup.");
        return;
    }

    println!("Nuwa Backup scheduled tasks (total: {}):\n", tasks.len());
    for (i, task) in tasks.iter().enumerate() {
        println!("  {}. {} (job: {})", i + 1, task.task_name, task.job_name);
        println!("     Status:    {}", task.status);
        println!("     Schedule:  {}", task.schedule);
        println!("     Next run:  {}", task.next_run);
        println!();
    }
}

/// Validate a time string in HH:MM format
fn validate_time(at: &str) -> Result<(), NuwaError> {
    let parts: Vec<&str> = at.split(':').collect();
    if parts.len() != 2 {
        return Err(NuwaError::InvalidArgument {
            detail: format!("Invalid time format '{}' -- expected HH:MM (24h)", at),
            suggestion: "Example: --at \"22:00\"".to_string(),
        });
    }
    let hour: u32 = parts[0].parse().map_err(|_| NuwaError::InvalidArgument {
        detail: format!("Invalid hour in '{}'", at),
        suggestion: "Hour must be 00-23".to_string(),
    })?;
    let minute: u32 = parts[1].parse().map_err(|_| NuwaError::InvalidArgument {
        detail: format!("Invalid minute in '{}'", at),
        suggestion: "Minute must be 00-59".to_string(),
    })?;
    if hour > 23 {
        return Err(NuwaError::InvalidArgument {
            detail: format!("Hour {} is out of range (00-23)", hour),
            suggestion: "Use 24-hour format, e.g. 22:00 for 10 PM".to_string(),
        });
    }
    if minute > 59 {
        return Err(NuwaError::InvalidArgument {
            detail: format!("Minute {} is out of range (00-59)", minute),
            suggestion: "Use 00-59 for minutes".to_string(),
        });
    }
    Ok(())
}

/// Parse a single CSV line from schtasks output
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut columns = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
            }
            ',' if !in_quotes => {
                columns.push(current.clone());
                current.clear();
            }
            _ => {
                current.push(ch);
            }
        }
    }
    columns.push(current);
    columns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_time_valid() {
        assert!(validate_time("22:00").is_ok());
        assert!(validate_time("00:00").is_ok());
        assert!(validate_time("23:59").is_ok());
    }

    #[test]
    fn test_validate_time_invalid() {
        assert!(validate_time("24:00").is_err());
        assert!(validate_time("22:60").is_err());
        assert!(validate_time("abc").is_err());
        assert!(validate_time("22:00:00").is_err());
    }

    #[test]
    fn test_parse_csv_line_simple() {
        let line = "\"a\",\"b\",\"c\"";
        let cols = parse_csv_line(line);
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0], "a");
        assert_eq!(cols[1], "b");
        assert_eq!(cols[2], "c");
    }

    #[test]
    fn test_parse_csv_line_with_spaces() {
        let line = "\"Microsoft\",\"NuwaBackup-test\",\"6:00:00 AM\",\"Ready\"";
        let cols = parse_csv_line(line);
        assert_eq!(cols.len(), 4);
        assert_eq!(cols[1], "NuwaBackup-test");
        assert_eq!(cols[3], "Ready");
    }

    #[test]
    fn test_parse_csv_line_empty() {
        let cols = parse_csv_line("");
        assert_eq!(cols.len(), 1);
        assert_eq!(cols[0], "");
    }

    #[test]
    fn test_task_prefix_constant() {
        assert_eq!(TASK_PREFIX, "NuwaBackup-");
    }

    #[test]
    fn test_schedule_trigger_debug() {
        let t1 = ScheduleTrigger::Daily {
            at: "22:00".to_string(),
        };
        assert!(format!("{:?}", t1).contains("Daily"));

        let t2 = ScheduleTrigger::Weekly {
            days: vec!["Mon".to_string(), "Fri".to_string()],
            at: "03:00".to_string(),
        };
        assert!(format!("{:?}", t2).contains("Weekly"));
    }

    #[test]
    fn test_delete_task_name_resolution() {
        // When task_id has prefix, use as-is
        let name1 = if "NuwaBackup-test".starts_with(TASK_PREFIX) {
            "NuwaBackup-test".to_string()
        } else {
            format!("{}{}", TASK_PREFIX, "NuwaBackup-test")
        };
        assert_eq!(name1, "NuwaBackup-test");

        // When task_id is just job name, add prefix
        let name2 = if "mydocs".starts_with(TASK_PREFIX) {
            "mydocs".to_string()
        } else {
            format!("{}{}", TASK_PREFIX, "mydocs")
        };
        assert_eq!(name2, "NuwaBackup-mydocs");
    }
    #[test]
    fn test_nonverbose_csv_three_columns() {
        let line = "\"\\\\NuwaBackup-test\",\"2026/7/6 22:00:00\",\"Ready\"";
        let cols = parse_csv_line(line);
        assert_eq!(cols.len(), 3);
        assert_eq!(cols[0], "\\\\NuwaBackup-test");
        assert_eq!(cols[1], "2026/7/6 22:00:00");
        assert_eq!(cols[2], "Ready");
    }

    #[test]
    fn test_task_name_with_backslash_prefix_matches() {
        let task_name = "\\\\NuwaBackup-mydocs";
        assert!(task_name.starts_with(&format!("\\\\{}", TASK_PREFIX)));
        let job_name = task_name
            .strip_prefix(&format!("\\\\{}", TASK_PREFIX))
            .unwrap_or(task_name)
            .to_string();
        assert_eq!(job_name, "mydocs");
    }

    #[test]
    fn test_on_logon_trigger_representation() {
        let trigger = ScheduleTrigger::OnLogon { delay_seconds: 300 };
        let debug_str = format!("{:?}", trigger);
        assert!(debug_str.contains("OnLogon"));
        assert!(debug_str.contains("300"));
    }

    #[test]
    fn test_create_task_command_format_has_quoted_exe_and_job() {
        let exe = r"C:\Program Files\nuwa backup\nuwa.exe";
        let job = "my backup job";
        let compress = " --compress";
        let cmd = format!("\"{}\" backup --job \"{}\"{}", exe, job, compress);
        assert!(cmd.starts_with('"'));
        assert!(cmd.contains("backup --job"));
        assert!(cmd.contains("\"my backup job\""));
        assert!(cmd.contains("--compress"));
    }

    #[test]
    fn test_create_task_command_no_compress() {
        let exe = r"C:\nuwa\nuwa.exe";
        let job = "testjob";
        let cmd = format!("\"{}\" backup --job \"{}\"{}", exe, job, "");
        assert!(cmd.starts_with('"'));
        assert!(cmd.contains("backup --job \"testjob\""));
        assert!(!cmd.contains("--compress"));
    }
}
