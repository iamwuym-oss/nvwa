// ============================================================================
// config_service.rs -- Backup job configuration CRUD service
//
// Responsibilities:
//   - List all job configurations (for Settings page)
//   - Get a single job configuration by name
//   - Create a new job configuration
//   - Update an existing job configuration
//   - Delete a job configuration
//
// This service operates on the existing Config / JobConfig model from
// src/config.rs, providing a clean Application Layer contract for the
// Settings page. It does NOT introduce a new config storage mechanism.
//
// Configuration boundary:
//   ConfigService -> Config::load() / Config::save() -> config.toml
//
// Runtime boundary (separate):
//   BackupService -> Config::load() -> BackupJobView (for Backup page)
// ============================================================================

use std::collections::BTreeMap;

use crate::app::error::AppError;
use crate::app::models::config_job::{JobConfigRequest, JobConfigView};
use crate::config::{Config, JobConfig, RetentionPolicy};
use crate::scheduler;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all job configurations, sorted by name for stable UI ordering.
///
/// Returns an empty Vec if no config file exists (first-time user state).
pub fn list_job_configs() -> Result<Vec<JobConfigView>, AppError> {
    let config = match Config::load() {
        Ok(c) => c,
        // If no config file exists, this is a first-time user — return empty list
        Err(e) if e.to_string().contains("not found") => {
            return Ok(Vec::new());
        }
        Err(e) => return Err(AppError::from(e)),
    };

    // Sort by name using BTreeMap to ensure deterministic UI order
    let sorted: BTreeMap<&String, &JobConfig> = config.job.iter().collect();
    let views: Vec<JobConfigView> = sorted
        .into_iter()
        .map(|(name, cfg)| job_config_to_view(name, cfg))
        .collect();

    Ok(views)
}

/// Get a single job configuration by name.
pub fn get_job_config(name: &str) -> Result<JobConfigView, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let job_cfg = config
        .job
        .get(name)
        .ok_or_else(|| AppError::config(format!("Job '{}' not found", name)))?;
    Ok(job_config_to_view(name, job_cfg))
}

/// Create a new job configuration.
///
/// Validation rules:
///   - Name must not be empty
///   - Source path must not be empty
///   - Destination path must not be empty
///   - Job name must not already exist (no silent overwrite)
pub fn create_job_config(request: &JobConfigRequest) -> Result<JobConfigView, AppError> {
    validate_request(request)?;

    let mut config = load_or_create_config()?;

    if config.job.contains_key(&request.name) {
        return Err(AppError::config(format!(
            "Job '{}' already exists",
            request.name
        )));
    }

    let job_cfg = request_to_job_config(request);
    let schedule_id = job_cfg.schedule_id.clone();
    let job_name = request.name.clone();
    config.job.insert(job_name.clone(), job_cfg);
    save_config(&config)?;

    // Sync schtasks: create if job has an enabled schedule.
    sync_job_schedule_tasks(&None, &schedule_id, &job_name);

    get_job_config(&request.name)
}

/// Update an existing job configuration.
///
/// Validation rules:
///   - Job name must exist (returns error if not found)
///   - Source and dest paths must not be empty
pub fn update_job_config(
    name: &str,
    request: &JobConfigRequest,
) -> Result<JobConfigView, AppError> {
    validate_request(request)?;

    let mut config = Config::load().map_err(AppError::from)?;

    if !config.job.contains_key(name) {
        return Err(AppError::config(format!("Job '{}' not found", name)));
    }

    // Rename is not supported ? name must match the original
    if name != request.name {
        return Err(AppError::config(format!(
            "Cannot rename job '{}' to '{}'. Rename is not supported. Delete and recreate instead.",
            name, request.name
        )));
    }

    // Capture old schedule_id BEFORE updating config, to detect changes for schtasks sync.
    let old_schedule_id = config.job.get(name).and_then(|j| j.schedule_id.clone());

    let job_cfg = request_to_job_config(request);
    let new_schedule_id = job_cfg.schedule_id.clone();
    config.job.insert(request.name.clone(), job_cfg);
    save_config(&config)?;

    // Sync schtasks: if schedule_id changed (None<->Some or Some<->Some with different IDs),
    // delete old tasks and/or create new ones accordingly.
    sync_job_schedule_tasks(&old_schedule_id, &new_schedule_id, name);

    get_job_config(&request.name)
}

/// Delete a job configuration by name.
///
/// Returns an error if the job does not exist (no silent no-op).
pub fn delete_job_config(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::config("Job name must not be empty"));
    }

    let mut config = Config::load().map_err(AppError::from)?;

    // Capture schedule_id before removing the job, so we can clean up schtasks.
    let schedule_id = config.job.get(name).and_then(|j| j.schedule_id.clone());

    if config.job.remove(name).is_none() {
        return Err(AppError::config(format!("Job '{}' not found", name)));
    }

    save_config(&config)?;

    // Remove associated Windows scheduled task if the job had a schedule.
    remove_job_schedule_tasks(name, &schedule_id);

    Ok(())
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Convert a JobConfig (internal core type) to JobConfigView (API contract type).
fn job_config_to_view(name: &str, cfg: &JobConfig) -> JobConfigView {
    let (keep_count, keep_days) = match &cfg.retention {
        Some(r) => (r.keep_count, r.keep_days),
        None => (None, None),
    };

    JobConfigView {
        name: name.to_string(),
        source: cfg.source.to_string_lossy().into_owned(),
        dest: cfg.dest.to_string_lossy().into_owned(),
        compress: cfg.compress,
        retention_keep_count: keep_count,
        retention_keep_days: keep_days,
        schedule_id: cfg.schedule_id.clone(),
        storage_type: cfg.storage_type.clone(),
        repository_id: cfg.repository_id.clone(),
    }
}

/// Convert a JobConfigRequest (API contract type) to JobConfig (internal core type).
///
/// Converts String paths to PathBuf and flattens retention fields into RetentionPolicy.
fn request_to_job_config(request: &JobConfigRequest) -> JobConfig {
    let retention = match (request.retention_keep_count, request.retention_keep_days) {
        (None, None) => None,
        (c, d) => Some(RetentionPolicy {
            keep_count: c,
            keep_days: d,
        }),
    };

    JobConfig {
        source: request.source.clone().into(),
        dest: request.dest.clone().into(),
        compress: request.compress,
        retention,
        schedule_id: request.schedule_id.clone(),
        storage_type: request.storage_type.clone(),
        repository_id: request.repository_id.clone(),
    }
}

/// Validate a job config request.
fn validate_request(request: &JobConfigRequest) -> Result<(), AppError> {
    if request.name.trim().is_empty() {
        return Err(AppError::config("Job name must not be empty"));
    }
    if request.source.trim().is_empty() {
        return Err(AppError::config("Source path must not be empty"));
    }
    if request.dest.trim().is_empty() {
        return Err(AppError::config("Destination path must not be empty"));
    }
    Ok(())
}

/// Sync Windows scheduled tasks when a job's schedule_id changes.
///
/// This function creates or removes schtasks based on the difference between
/// old and new schedule_id values. It is called after config is saved so that
/// scheduler::create_task() can read the fresh config.
///
/// Lifecycle rules:
///   - None ? Some(schedule): create task if schedule is enabled
///   - Some(A) ? Some(B): delete old task A, create task B
///   - Some(A) ? None: delete old task A
///   - No change: do nothing
fn sync_job_schedule_tasks(
    old_schedule_id: &Option<String>,
    new_schedule_id: &Option<String>,
    job_name: &str,
) {
    // When both are the same (no change), exit early to avoid unnecessary schtasks calls.
    if old_schedule_id == new_schedule_id {
        return;
    }

    // Load config fresh to get schedule trigger details.
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => return, // If config can't be loaded, we can't determine the trigger; skip silently.
    };

    // If there was an old schedule, remove its task first (delete old, then create new).
    if let Some(_old_id) = old_schedule_id {
        let _ = scheduler::delete_task(job_name);
    }

    // If there's a new schedule and it's enabled, create a task.
    if let Some(new_id) = new_schedule_id {
        if let Some(sched) = config.schedules.get(new_id) {
            if sched.enabled {
                let trigger = sched.trigger.to_scheduler_trigger();
                let _ = scheduler::create_task(job_name, &trigger);
            }
        }
    }
}

/// Remove Windows scheduled tasks for a job (used when job is deleted or disabled).
fn remove_job_schedule_tasks(job_name: &str, schedule_id: &Option<String>) {
    if schedule_id.is_some() {
        let _ = scheduler::delete_task(job_name);
    }
}

/// Load existing config or create an empty one.
///
/// Unlike Config::load(), this returns an empty Config if no file exists,
/// allowing first-time creation without requiring `nuwa init` first.
fn load_or_create_config() -> Result<Config, AppError> {
    match Config::load() {
        Ok(c) => Ok(c),
        Err(_) => Ok(Config {
            job: std::collections::HashMap::new(),
            schedules: std::collections::HashMap::new(),
        }),
    }
}

/// Save config to its default path.
///
/// Creates the parent directory if it does not exist.
fn save_config(config: &Config) -> Result<(), AppError> {
    config.save().map_err(|e| {
        AppError::config(format!("Failed to save configuration: {}", e)).with_detail(e.to_string())
    })?;
    Ok(())
}

// ---------------------------------------------------------------------------
