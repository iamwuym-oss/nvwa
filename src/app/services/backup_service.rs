// ============================================================================
// backup_service.rs -- Backup domain service (storage engine redesign)
//
// The backup service is temporarily unavailable as the storage engine
// is being redesigned to use .nwb single-file format.
// ============================================================================

use crate::app::error::AppError;
use crate::app::models::backup::{BackupJobStatus, BackupJobView, BackupResult};
use crate::config::Config;

pub fn list_jobs() -> Result<Vec<BackupJobView>, AppError> {
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };
    let jobs: Vec<BackupJobView> = config
        .job
        .iter()
        .map(|(name, job_cfg)| {
            let status = if !job_cfg.source.exists() {
                BackupJobStatus::Misconfigured
            } else {
                BackupJobStatus::NeverRun
            };
            BackupJobView {
                name: name.into(),
                source: job_cfg.source.to_string_lossy().into_owned(),
                dest: job_cfg.dest.to_string_lossy().into_owned(),
                compress: job_cfg.compress,
                retention: match &job_cfg.retention {
                    Some(r) => match (r.keep_count, r.keep_days) {
                        (Some(c), None) => format!("Keep last {} versions", c),
                        (None, Some(d)) => format!("Keep {} days", d),
                        (Some(c), Some(d)) => format!("Keep {} versions or {} days", c, d),
                        (None, None) => "Keep all".into(),
                    },
                    None => "Keep all".into(),
                },
                status,
                last_backup_time: None,
                last_backup_status: None,
                last_backup_files: 0,
                last_backup_bytes: 0,
            }
        })
        .collect();
    Ok(jobs)
}

pub fn get_job_detail(name: &str) -> Result<BackupJobView, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let job_cfg = config
        .job
        .get(name)
        .ok_or_else(|| AppError::config(format!("Job '{}' not found", name)))?;
    let status = if !job_cfg.source.exists() {
        BackupJobStatus::Misconfigured
    } else {
        BackupJobStatus::NeverRun
    };
    Ok(BackupJobView {
        name: name.into(),
        source: job_cfg.source.to_string_lossy().into_owned(),
        dest: job_cfg.dest.to_string_lossy().into_owned(),
        compress: job_cfg.compress,
        retention: "Keep all".into(),
        status,
        last_backup_time: None,
        last_backup_status: None,
        last_backup_files: 0,
        last_backup_bytes: 0,
    })
}

pub fn run_backup(_job_name: &str) -> Result<BackupResult, AppError> {
    Err(AppError::internal(
        "Backup is temporarily unavailable during storage engine redesign. \
         The new .nwb single-file engine is being built.",
    ))
}

pub fn run_backup_dry(job_name: &str) -> Result<BackupResult, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let _job_cfg = config
        .job
        .get(job_name)
        .ok_or_else(|| AppError::config(format!("Job '{}' not found", job_name)))?;
    Ok(BackupResult {
        backup_id: "dry-run".into(),
        timestamp: chrono::Local::now()
            .format("%Y-%m-%dT%H:%M:%S%.3fZ")
            .to_string(),
        file_count: 0,
        total_bytes: 0,
        duration_ms: 0,
        status: "dry_run".into(),
        error: None,
    })
}
