// ============================================================================
// schedule_service.rs -- Schedule profile CRUD service
//
// Responsibilities:
//   - List all schedule profiles
//   - Get a single schedule profile by ID
//   - Create a new schedule profile
//   - Update an existing schedule profile
//   - Delete a schedule profile (with safety checks for in-use)
//   - Enable / disable a schedule profile
//
// This service operates on the Config / ScheduleProfileConfig model from
// src/config.rs, providing a clean Application Layer contract for the
// Schedule page.
//
// Design principles:
//   - used_by_jobs is computed dynamically from JobConfig (never stored twice)
//   - next_run_at / last_run_at are NOT persisted (derived at query time)
//   - Schedule deletion checks for referencing jobs before proceeding
//   - Backward compatible: missing schedules field defaults to empty map
// ============================================================================

use std::collections::HashMap;

use crate::app::error::AppError;
use crate::app::models::schedule::{
    name_to_id, summarize_trigger, ScheduleDeleteResult, ScheduleProfileRequest,
    ScheduleProfileView, TaskSyncStatus, TriggerType,
};
use crate::config::{Config, ScheduleProfileConfig, ScheduleTriggerConfig};
use crate::scheduler::{self, ScheduleTrigger};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all schedule profiles, sorted by name for stable UI ordering.
///
/// Returns an empty Vec if no config file exists (first-time user state).
pub fn list_schedules() -> Result<Vec<ScheduleProfileView>, AppError> {
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };

    let mut views: Vec<ScheduleProfileView> = config
        .schedules
        .iter()
        .map(|(id, s)| build_schedule_view(id, s, &config.job, None, None, None))
        .collect();

    views.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(views)
}

/// Get a single schedule profile by ID.
pub fn get_schedule(id: &str) -> Result<ScheduleProfileView, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let sched = config
        .schedules
        .get(id)
        .ok_or_else(|| AppError::config(format!("Schedule '{}' not found", id)))?;

    Ok(build_schedule_view(
        id,
        sched,
        &config.job,
        None,
        None,
        None,
    ))
}

/// Create a new schedule profile.
///
/// Validation rules:
///   - Name must not be empty
///   - Name must be unique (case-insensitive)
///   - ID is auto-generated from name via name_to_id()
pub fn create_schedule(request: &ScheduleProfileRequest) -> Result<ScheduleProfileView, AppError> {
    validate_request(request)?;

    let mut config = load_or_create_config()?;

    // Check name uniqueness (case-insensitive)
    check_name_unique(&config, &request.name, None)?;

    // Auto-generate ID from name
    let id = name_to_id(&request.name);
    if id.is_empty() {
        return Err(AppError::config(
            "Schedule name could not generate a valid identifier. Use at least one alphanumeric character.",
        ));
    }

    // ID collision is unlikely with name-based generation, but guard anyway
    if config.schedules.contains_key(&id) {
        return Err(AppError::config(format!(
            "Schedule identifier collision for '{}'. Please use a different name.",
            id
        )));
    }

    let now = chrono_timestamp();
    let sched = ScheduleProfileConfig {
        id: id.clone(),
        name: request.name.clone(),
        description: request.description.clone(),
        enabled: request.enabled,
        trigger: request_to_trigger(&request.trigger_type, &request.trigger_params)?,
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };

    config.schedules.insert(id.clone(), sched);
    save_config(&config)?;

    get_schedule(&id)
}

/// Update an existing schedule profile.
///
/// Rules:
///   - Name changes are FORBIDDEN (delete + recreate instead)
///   - Only description, enabled, and trigger may be modified
pub fn update_schedule(
    id: &str,
    request: &ScheduleProfileRequest,
) -> Result<ScheduleProfileView, AppError> {
    validate_request(request)?;

    let mut config = Config::load().map_err(AppError::from)?;

    if !config.schedules.contains_key(id) {
        return Err(AppError::config(format!("Schedule '{}' not found", id)));
    }

    // Name rename is not supported — enforce strict equality
    let existing_name = &config.schedules[id].name;
    if request.name.trim() != existing_name.trim() {
        return Err(AppError::config(format!(
            "Cannot rename schedule from '{}' to '{}'. Delete and recreate instead.",
            existing_name, request.name
        )));
    }

    let now = chrono_timestamp();
    let sched = ScheduleProfileConfig {
        id: id.to_string(),
        name: existing_name.clone(),
        description: request.description.clone(),
        enabled: request.enabled,
        trigger: request_to_trigger(&request.trigger_type, &request.trigger_params)?,
        created_at: config.schedules[id].created_at.clone(),
        updated_at: Some(now),
    };

    config.schedules.insert(id.to_string(), sched);
    save_config(&config)?;

    // Sync schtasks: if schedule is enabled and has referencing jobs, rebuild their tasks.
    // This ensures trigger changes (e.g. Daily 21:00 -> Daily 22:00) are reflected
    // in Windows Task Scheduler immediately.
    if request.enabled {
        let _ = sync_schedule_trigger_update(id);
    }

    get_schedule(id)
}

/// Delete a schedule profile.
///
/// Safety:
///   - If any backup jobs reference this schedule, returns affected_jobs
///     without deleting. Caller must present the list for user confirmation.
///   - If no jobs reference it, deletes immediately.
pub fn delete_schedule(id: &str, confirmed: bool) -> Result<ScheduleDeleteResult, AppError> {
    let mut config = Config::load().map_err(AppError::from)?;
    let result = apply_delete_schedule(&mut config, id, confirmed)?;

    if result.deleted {
        // Best-effort scheduler cleanup
        for job_name in &result.affected_jobs {
            let _ = crate::scheduler::delete_task(job_name);
        }
        let _ = sync_remove_tasks(id);
        save_config(&config)?;
    }

    Ok(result)
}

/// Internal: apply schedule deletion to an in-memory Config without filesystem or scheduler side effects.
/// Returns the same ScheduleDeleteResult semantics as the public API.
/// Used by delete_schedule() and directly by tests to avoid Config::load() / save_config() dependencies.
fn apply_delete_schedule(
    config: &mut Config,
    id: &str,
    confirmed: bool,
) -> Result<ScheduleDeleteResult, AppError> {
    if !config.schedules.contains_key(id) {
        return Err(AppError::config(format!("Schedule '{}' not found", id)));
    }

    let affected_jobs: Vec<String> = config
        .job
        .iter()
        .filter(|(_, j)| j.schedule_id.as_deref() == Some(id))
        .map(|(name, _)| name.clone())
        .collect();

    if !affected_jobs.is_empty() {
        if !confirmed {
            let sched_name = config
                .schedules
                .get(id)
                .map(|s| s.name.as_str())
                .unwrap_or(id);
            let count = affected_jobs.len();
            let message = format!(
                "Schedule '{}' is used by {} backup job{}. Confirm deletion to auto-clear schedule references.",
                sched_name,
                count,
                if count == 1 { "" } else { "s" },
            );
            return Ok(ScheduleDeleteResult {
                deleted: false,
                affected_jobs,
                message,
            });
        }

        // confirmed=true: clear schedule_id from affected jobs (in-memory only)
        for job_name in &affected_jobs {
            if let Some(job) = config.job.get_mut(job_name) {
                job.schedule_id = None;
            }
        }
    }

    // Delete the schedule profile itself
    config.schedules.remove(id);

    Ok(ScheduleDeleteResult {
        deleted: true,
        affected_jobs,
        message: "Schedule deleted. Referencing jobs were set to manual backup.".to_string(),
    })
}

/// Enable a schedule profile.
///
/// This updates config state. Caller (Tauri command) should also sync
/// Windows scheduled tasks for associated jobs.
pub fn enable_schedule(id: &str) -> Result<ScheduleProfileView, AppError> {
    let mut config = Config::load().map_err(AppError::from)?;

    let sched = config
        .schedules
        .get_mut(id)
        .ok_or_else(|| AppError::config(format!("Schedule '{}' not found", id)))?;

    if sched.enabled {
        return get_schedule(id);
    }

    sched.enabled = true;
    sched.updated_at = Some(chrono_timestamp());
    save_config(&config)?;

    get_schedule(id)
}

/// Disable a schedule profile.
///
/// This updates config state. Caller (Tauri command) should also remove
/// Windows scheduled tasks for associated jobs.
pub fn disable_schedule(id: &str) -> Result<ScheduleProfileView, AppError> {
    let mut config = Config::load().map_err(AppError::from)?;

    let sched = config
        .schedules
        .get_mut(id)
        .ok_or_else(|| AppError::config(format!("Schedule '{}' not found", id)))?;

    if !sched.enabled {
        return get_schedule(id);
    }

    sched.enabled = false;
    sched.updated_at = Some(chrono_timestamp());
    save_config(&config)?;

    get_schedule(id)
}

// ---------------------------------------------------------------------------
// View model builder
// ---------------------------------------------------------------------------

/// Build a ScheduleProfileView from config data and optional runtime info.
fn build_schedule_view(
    id: &str,
    sched: &ScheduleProfileConfig,
    jobs: &HashMap<String, crate::config::JobConfig>,
    _next_run: Option<String>,
    _last_run: Option<String>,
    _last_status: Option<String>,
) -> ScheduleProfileView {
    let (trigger_type, trigger_params) = trigger_to_view_params(&sched.trigger);
    let trigger_summary = summarize_trigger(&trigger_type, &trigger_params);

    // Find all jobs referencing this schedule
    let mut used_by: Vec<String> = jobs
        .iter()
        .filter(|(_, j)| j.schedule_id.as_deref() == Some(id))
        .map(|(name, _)| name.clone())
        .collect();
    used_by.sort();

    let used_by_count = used_by.len() as u32;

    // Task sync status: if enabled and has jobs, should be synced
    // Actual sync check happens in the schtasks integration layer
    let task_sync_status = if used_by.is_empty() {
        TaskSyncStatus::NoJobs
    } else if sched.enabled {
        TaskSyncStatus::Unknown
    } else {
        TaskSyncStatus::NoJobs
    };

    ScheduleProfileView {
        id: id.to_string(),
        name: sched.name.clone(),
        description: sched.description.clone(),
        enabled: sched.enabled,
        trigger_type,
        trigger_summary,
        used_by_count,
        used_by_jobs: used_by,
        next_run_at: None,
        last_run_at: None,
        last_run_status: None,
        task_sync_status,
        created_at: sched.created_at.clone(),
        updated_at: sched.updated_at.clone(),
    }
}

// ---------------------------------------------------------------------------
// Trigger conversion helpers
// ---------------------------------------------------------------------------

/// Convert ScheduleTriggerConfig to (TriggerType, params_string) for UI models.
fn trigger_to_view_params(trigger: &ScheduleTriggerConfig) -> (TriggerType, String) {
    match trigger {
        ScheduleTriggerConfig::Once { at } => (TriggerType::Once, at.clone()),
        ScheduleTriggerConfig::Daily { at } => (TriggerType::Daily, at.clone()),
        ScheduleTriggerConfig::Weekly { days, at } => (
            TriggerType::Weekly,
            format!("{days}@{at}", days = days.join(","), at = at),
        ),
        ScheduleTriggerConfig::Monthly { day, .. } => (TriggerType::Monthly, format!("{}", day)),
        ScheduleTriggerConfig::OnLogon { delay_seconds } => {
            (TriggerType::OnLogon, delay_seconds.to_string())
        }
    }
}

/// Convert a TriggerType + params string into a ScheduleTriggerConfig.
fn request_to_trigger(
    trigger_type: &TriggerType,
    params: &str,
) -> Result<ScheduleTriggerConfig, AppError> {
    match trigger_type {
        TriggerType::Once => Ok(ScheduleTriggerConfig::Once {
            at: params.to_string(),
        }),
        TriggerType::Daily => Ok(ScheduleTriggerConfig::Daily {
            at: params.to_string(),
        }),
        TriggerType::Weekly => {
            // params format: "Mon,Wed,Fri@21:00"
            if let Some((days_str, time)) = params.split_once('@') {
                let days: Vec<String> = days_str.split(',').map(|s| s.trim().to_string()).collect();
                Ok(ScheduleTriggerConfig::Weekly {
                    days,
                    at: time.to_string(),
                })
            } else {
                Err(AppError::config(
                    "Weekly trigger requires format 'Day,Day@HH:MM' (e.g. 'Mon,Wed,Fri@21:00')",
                ))
            }
        }
        TriggerType::Monthly => {
            // params format: "1@21:00"
            if let Some((day_str, time)) = params.split_once('@') {
                let day: u32 = day_str.parse().map_err(|_| {
                    AppError::config(format!("Invalid day '{}' for monthly trigger", day_str))
                })?;
                Ok(ScheduleTriggerConfig::Monthly {
                    day,
                    at: time.to_string(),
                })
            } else {
                Err(AppError::config(
                    "Monthly trigger requires format 'day@HH:MM' (e.g. '1@21:00')",
                ))
            }
        }
        TriggerType::OnLogon => {
            let delay: u32 = params.parse().unwrap_or(0);
            Ok(ScheduleTriggerConfig::OnLogon {
                delay_seconds: delay,
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn chrono_timestamp() -> String {
    // Use local time for user-facing timestamps
    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string()
}

fn validate_request(request: &ScheduleProfileRequest) -> Result<(), AppError> {
    if request.name.trim().is_empty() {
        return Err(AppError::config("Schedule name must not be empty"));
    }
    Ok(())
}

/// Check that no existing schedule has the same name (case-insensitive).
///
/// exclude_id can be set during update to skip the current schedule.
fn check_name_unique(
    config: &Config,
    name: &str,
    exclude_id: Option<&str>,
) -> Result<(), AppError> {
    let name_normalized = name.trim().to_lowercase();
    for (sid, sched) in &config.schedules {
        if let Some(exclude) = exclude_id {
            if sid == exclude {
                continue;
            }
        }
        if sched.name.trim().to_lowercase() == name_normalized {
            return Err(AppError::config(format!(
                "A schedule with name '{}' already exists",
                name
            )));
        }
    }
    Ok(())
}

fn load_or_create_config() -> Result<Config, AppError> {
    match Config::load() {
        Ok(c) => Ok(c),
        Err(_) => Ok(Config {
            job: HashMap::new(),
            schedules: HashMap::new(),
        }),
    }
}

fn save_config(config: &Config) -> Result<(), AppError> {
    config.save().map_err(|e| {
        AppError::config(format!("Failed to save configuration: {}", e)).with_detail(e.to_string())
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
// schtasks synchronization helpers
// ---------------------------------------------------------------------------

/// Convert a config ScheduleTriggerConfig to a scheduler ScheduleTrigger.
pub(crate) fn config_trigger_to_scheduler(trigger: &ScheduleTriggerConfig) -> ScheduleTrigger {
    match trigger {
        ScheduleTriggerConfig::Once { at } => ScheduleTrigger::Once { at: at.clone() },
        ScheduleTriggerConfig::Daily { at } => ScheduleTrigger::Daily { at: at.clone() },
        ScheduleTriggerConfig::Weekly { days, at } => ScheduleTrigger::Weekly {
            days: days.clone(),
            at: at.clone(),
        },
        ScheduleTriggerConfig::Monthly { day, at } => ScheduleTrigger::Monthly {
            day: *day,
            at: at.clone(),
        },
        ScheduleTriggerConfig::OnLogon { delay_seconds } => ScheduleTrigger::OnLogon {
            delay_seconds: *delay_seconds,
        },
    }
}

/// Create Windows scheduled tasks for all enabled jobs referencing this schedule.
///
/// This is called after enable_schedule() to ensure schtasks are in sync.
pub fn sync_create_tasks(id: &str) -> Result<u32, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let sched = config
        .schedules
        .get(id)
        .ok_or_else(|| AppError::config(format!("Schedule '{}' not found", id)))?;

    if !sched.enabled {
        return Ok(0);
    }

    let trigger = config_trigger_to_scheduler(&sched.trigger);
    let mut created_count = 0u32;

    for (job_name, job_cfg) in &config.job {
        if job_cfg.schedule_id.as_deref() == Some(id) {
            match scheduler::create_task(job_name, &trigger) {
                Ok(_) => created_count += 1,
                Err(e) => {
                    eprintln!(
                        "Warning: failed to create scheduled task for '{}': {}",
                        job_name, e
                    );
                }
            }
        }
    }

    Ok(created_count)
}

/// Remove Windows scheduled tasks for all jobs referencing this schedule.
///
/// This is called before delete_schedule() or after disable_schedule().
pub fn sync_remove_tasks(id: &str) -> Result<u32, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let mut removed_count = 0u32;

    for (job_name, job_cfg) in &config.job {
        if job_cfg.schedule_id.as_deref() == Some(id) {
            match scheduler::delete_task(job_name) {
                Ok(_) => removed_count += 1,
                Err(e) => {
                    // Task may not exist yet — that's fine
                    let msg = e.to_string();
                    if !msg.contains("not found") && !msg.contains("does not exist") {
                        eprintln!(
                            "Warning: failed to remove scheduled task for '{}': {}",
                            job_name, e
                        );
                    }
                }
            }
        }
    }

    Ok(removed_count)
}

/// Sync Windows scheduled tasks when a schedule's trigger changes.
///
/// Called after update_schedule() to ensure all referencing jobs' schtasks
/// are updated with the new trigger parameters (e.g. time change from 21:00 to 22:00).
///
/// This is a best-effort sync ? individual task failures are logged but not propagated,
/// because a partial sync failure should not block the schedule update itself.
/// The UI should show task_sync_status to reflect actual sync state.
pub fn sync_schedule_trigger_update(id: &str) -> Result<u32, AppError> {
    let config = Config::load().map_err(AppError::from)?;

    let sched = match config.schedules.get(id) {
        Some(s) => s,
        None => return Ok(0),
    };

    if !sched.enabled {
        return Ok(0);
    }

    let trigger = sched.trigger.to_scheduler_trigger();
    let mut updated_count = 0u32;

    for (job_name, job_cfg) in &config.job {
        if job_cfg.schedule_id.as_deref() == Some(id) {
            match crate::scheduler::create_task(job_name, &trigger) {
                Ok(_) => updated_count += 1,
                Err(e) => {
                    eprintln!(
                        "Warning: failed to update scheduled task for '{}': {}",
                        job_name, e
                    );
                }
            }
        }
    }

    Ok(updated_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::JobConfig;
    use std::path::Path;

    fn make_empty_config() -> Config {
        Config {
            job: HashMap::new(),
            schedules: HashMap::new(),
        }
    }

    fn make_config_with_schedule() -> (Config, String) {
        let mut config = make_empty_config();
        let sched_id = "test-daily";
        config.schedules.insert(
            sched_id.to_string(),
            ScheduleProfileConfig {
                id: sched_id.to_string(),
                name: "Test Daily Schedule".to_string(),
                description: Some("A test schedule".to_string()),
                enabled: true,
                trigger: ScheduleTriggerConfig::Daily {
                    at: "21:00".to_string(),
                },
                created_at: Some("2026-07-09T10:00:00".to_string()),
                updated_at: Some("2026-07-09T10:00:00".to_string()),
            },
        );
        (config, sched_id.to_string())
    }

    fn make_config_with_job_referencing_schedule() -> (Config, String) {
        let (mut config, sched_id) = make_config_with_schedule();
        config.job.insert(
            "TestJob".to_string(),
            JobConfig {
                source: Path::new("C:\\Src").to_path_buf(),
                dest: Path::new("D:\\Dst").to_path_buf(),
                compress: false,
                retention: None,
                schedule_id: Some(sched_id.clone()),
                storage_type: None,
                repository_id: None,
            },
        );
        (config, sched_id)
    }

    #[test]
    fn test_build_view_no_jobs() {
        let (config, sched_id) = make_config_with_schedule();
        let sched = config.schedules.get(&sched_id).unwrap();
        let view = build_schedule_view(&sched_id, sched, &config.job, None, None, None);
        assert_eq!(view.name, "Test Daily Schedule");
        assert_eq!(view.trigger_type, TriggerType::Daily);
        assert_eq!(view.trigger_summary, "Daily at 21:00");
        assert_eq!(view.used_by_count, 0);
        assert!(view.used_by_jobs.is_empty());
        assert_eq!(view.task_sync_status, TaskSyncStatus::NoJobs);
    }

    #[test]
    fn test_build_view_with_jobs() {
        let (config, sched_id) = make_config_with_job_referencing_schedule();
        let sched = config.schedules.get(&sched_id).unwrap();
        let view = build_schedule_view(&sched_id, sched, &config.job, None, None, None);
        assert_eq!(view.used_by_count, 1);
        assert_eq!(view.used_by_jobs, vec!["TestJob"]);
    }

    #[test]
    fn test_delete_schedule_not_found() {
        // Non-existent schedule returns error (no file system).
        let mut config = make_empty_config();
        let result = apply_delete_schedule(&mut config, "nonexistent", false);
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("not found"),
            "Expected 'not found' error, got: {}",
            msg
        );
    }
    #[test]
    fn test_delete_schedule_no_jobs() {
        // Schedule with no referencing jobs: immediate delete.
        let (mut config, sched_id) = make_config_with_schedule();
        let result = apply_delete_schedule(&mut config, &sched_id, false);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.deleted, "expected deleted=true");
        assert!(r.affected_jobs.is_empty(), "expected no affected jobs");
        assert!(
            !config.schedules.contains_key(&sched_id),
            "schedule should be removed from config"
        );
    }
    #[test]
    fn test_delete_schedule_with_jobs_blocked() {
        // Schedule with referencing jobs, confirmed=false.
        let (mut config, sched_id) = make_config_with_job_referencing_schedule();
        let result = apply_delete_schedule(&mut config, &sched_id, false);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(!r.deleted, "expected deleted=false when not confirmed");
        assert!(
            r.affected_jobs.contains(&"TestJob".to_string()),
            "affected_jobs should include TestJob"
        );
        assert!(
            config.schedules.contains_key(&sched_id),
            "schedule should still exist when not confirmed"
        );
        assert!(
            config
                .job
                .get("TestJob")
                .and_then(|j| j.schedule_id.as_deref())
                == Some(&sched_id),
            "TestJob.schedule_id should remain unchanged"
        );
    }
    #[test]
    fn test_delete_schedule_confirmed_flow() {
        // Schedule with referencing jobs, confirmed=true: deletes and clears references.
        let (mut config, sched_id) = make_config_with_job_referencing_schedule();
        let result = apply_delete_schedule(&mut config, &sched_id, true);
        assert!(result.is_ok());
        let r = result.unwrap();
        assert!(r.deleted, "expected deleted=true when confirmed");
        assert!(
            r.affected_jobs.contains(&"TestJob".to_string()),
            "affected_jobs should include TestJob"
        );
        assert!(
            !config.schedules.contains_key(&sched_id),
            "schedule should be removed from config"
        );
        assert!(
            config
                .job
                .get("TestJob")
                .and_then(|j| j.schedule_id.as_deref())
                .is_none(),
            "TestJob.schedule_id should be None after confirmed delete"
        );
    }
    #[test]
    fn test_trigger_to_view_params_daily() {
        let trigger = ScheduleTriggerConfig::Daily {
            at: "22:00".to_string(),
        };
        let (t, p) = trigger_to_view_params(&trigger);
        assert_eq!(t, TriggerType::Daily);
        assert_eq!(p, "22:00");
    }

    #[test]
    fn test_trigger_to_view_params_weekly() {
        let trigger = ScheduleTriggerConfig::Weekly {
            days: vec!["Mon".to_string(), "Wed".to_string(), "Fri".to_string()],
            at: "20:00".to_string(),
        };
        let (t, p) = trigger_to_view_params(&trigger);
        assert_eq!(t, TriggerType::Weekly);
        assert_eq!(p, "Mon,Wed,Fri@20:00");
    }

    #[test]
    fn test_request_to_trigger_daily() {
        let trigger = request_to_trigger(&TriggerType::Daily, "21:00").unwrap();
        match trigger {
            ScheduleTriggerConfig::Daily { at } => assert_eq!(at, "21:00"),
            _ => panic!("Expected Daily"),
        }
    }

    #[test]
    fn test_request_to_trigger_weekly() {
        let trigger = request_to_trigger(&TriggerType::Weekly, "Mon,Fri@18:00").unwrap();
        match trigger {
            ScheduleTriggerConfig::Weekly { days, at } => {
                assert_eq!(days, vec!["Mon", "Fri"]);
                assert_eq!(at, "18:00");
            }
            _ => panic!("Expected Weekly"),
        }
    }

    #[test]
    fn test_request_to_trigger_weekly_invalid() {
        let result = request_to_trigger(&TriggerType::Weekly, "badformat");
        assert!(result.is_err());
    }

    #[test]
    fn test_request_to_trigger_monthly() {
        let trigger = request_to_trigger(&TriggerType::Monthly, "15@03:00").unwrap();
        match trigger {
            ScheduleTriggerConfig::Monthly { day, at } => {
                assert_eq!(day, 15);
                assert_eq!(at, "03:00");
            }
            _ => panic!("Expected Monthly"),
        }
    }

    #[test]
    fn test_request_to_trigger_on_logon() {
        let trigger = request_to_trigger(&TriggerType::OnLogon, "300").unwrap();
        match trigger {
            ScheduleTriggerConfig::OnLogon { delay_seconds } => {
                assert_eq!(delay_seconds, 300);
            }
            _ => panic!("Expected OnLogon"),
        }
    }

    #[test]
    fn test_summarize_trigger() {
        assert_eq!(
            summarize_trigger(&TriggerType::Daily, "21:00"),
            "Daily at 21:00"
        );
        assert_eq!(
            summarize_trigger(&TriggerType::Weekly, "Mon,Wed,Fri@20:00"),
            "Weekly on Mon, Wed, Fri at 20:00"
        );
        assert_eq!(
            summarize_trigger(&TriggerType::OnLogon, "0"),
            "On logon (no delay)"
        );
        assert_eq!(
            summarize_trigger(&TriggerType::OnLogon, "300"),
            "On logon (300s delay)"
        );
    }

    #[test]
    fn test_validate_request() {
        let valid = ScheduleProfileRequest {
            name: "Test".to_string(),
            description: None,
            enabled: true,
            trigger_type: TriggerType::Daily,
            trigger_params: "21:00".to_string(),
        };
        assert!(validate_request(&valid).is_ok());

        let invalid = ScheduleProfileRequest {
            name: "   ".to_string(),
            description: None,
            enabled: true,
            trigger_type: TriggerType::Daily,
            trigger_params: "21:00".to_string(),
        };
        assert!(validate_request(&invalid).is_err());
    }

    #[test]
    fn test_name_to_id_basic() {
        assert_eq!(name_to_id("Daily Evening Backup"), "daily-evening-backup");
        assert_eq!(name_to_id("On Login Backup"), "on-login-backup");
        assert_eq!(name_to_id("  Weekly   Backup  "), "weekly---backup");
    }

    #[test]
    fn test_check_name_unique() {
        let (config, _) = make_config_with_schedule();
        // Same name should fail
        assert!(check_name_unique(&config, "Test Daily Schedule", None).is_err());
        // Different name should pass
        assert!(check_name_unique(&config, "Another Schedule", None).is_ok());
        // Same name with exclude_id should pass
        assert!(check_name_unique(&config, "Test Daily Schedule", Some("test-daily")).is_ok());
        // Case-insensitive match should fail
        assert!(check_name_unique(&config, "test daily schedule", None).is_err());
    }
}
