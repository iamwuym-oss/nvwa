// ============================================================================
// backup_service.rs -- Backup domain service (Repository-only)
//
// Responsibilities:
//   - List all configured backup jobs with derived status
//   - Execute a backup for a given job through Repository Engine
//   - Record backup results in the history database
//
// This service is the single entry point for all backup operations.
// P-07: Flat-file storage has been removed. All backups use Repository Engine.
// ============================================================================

use std::path::Path;
use std::time::Instant;

use crate::app::error::AppError;
use crate::app::models::backup::{BackupJobStatus, BackupJobView, BackupResult};
use crate::app::services::repo_registry::RepoRegistry;
use crate::config::{Config, JobConfig};
use crate::history::{HistoryDb, OperationRecord};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// List all configured backup jobs with derived status.
pub fn list_jobs() -> Result<Vec<BackupJobView>, AppError> {
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => return Ok(Vec::new()),
    };

    let jobs: Vec<BackupJobView> = config
        .job
        .iter()
        .map(|(name, job_cfg)| build_job_view(name, job_cfg))
        .collect();

    Ok(jobs)
}

/// Get a single job's detailed view by name.
pub fn get_job_detail(name: &str) -> Result<BackupJobView, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let job_cfg = config
        .job
        .get(name)
        .ok_or_else(|| AppError::config(format!("Job '{}' not found", name)))?;

    Ok(build_job_view(name, job_cfg))
}

/// Execute a backup for the given job name.
///
/// P-07: Repository-only. Jobs must have storage_type="repository"
/// and a valid repository_id.
pub fn run_backup(job_name: &str) -> Result<BackupResult, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let job_cfg = config
        .job
        .get(job_name)
        .ok_or_else(|| AppError::config(format!("Job '{}' not found", job_name)))?;

    // Validate source exists
    if !job_cfg.source.exists() {
        return Err(AppError::config(format!(
            "Source path '{}' does not exist",
            job_cfg.source.display()
        )));
    }

    // Repository-only: use Repository Engine for backup
    let repo_id = job_cfg.repository_id.as_deref().ok_or_else(|| {
        AppError::config(format!(
            "Job '{}' has no repository_id. All backups now require Repository storage.",
            job_name
        ))
    })?;

    run_repo_backup(
        job_name,
        &job_cfg.source,
        repo_id,
        &job_cfg.dest,
        job_cfg.compress,
    )
}

/// Run a backup without recording history (for testing/validation).
pub fn run_backup_dry(job_name: &str) -> Result<BackupResult, AppError> {
    let config = Config::load().map_err(AppError::from)?;
    let job_cfg = config
        .job
        .get(job_name)
        .ok_or_else(|| AppError::config(format!("Job '{}' not found", job_name)))?;

    if !job_cfg.source.exists() {
        return Err(AppError::config(format!(
            "Source path '{}' does not exist",
            job_cfg.source.display()
        )));
    }

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

// ---------------------------------------------------------------------------
// Repository backup (Repository Engine only, P-07)
// ---------------------------------------------------------------------------

/// Execute a backup to a Repository engine.
///
/// Lifecycle:
///   1. Resolve repo path from RepoRegistry
///   2. Open repo via open_repo()
///   3. Register job in repo.db
///   4. Create RepositoryBackupWriter, begin transaction
///   5. backup_directory() — walk and write all files
///   6. finalize() — complete all phases and commit
///   7. Record history with accurate stats
fn run_repo_backup(
    job_name: &str,
    source: &Path,
    repo_id: &str,
    dest: &Path,
    compress: bool,
) -> Result<BackupResult, AppError> {
    use crate::repository::{open_repo, RepositoryBackupWriter};

    // 1-2. Resolve and open repository
    let registry = RepoRegistry::load();
    let repo_path = registry.resolve_path(repo_id)?;

    let repo = open_repo(&repo_path)
        .map_err(|e| AppError::internal(format!("Cannot open repository: {}", e)))?;

    let start = Instant::now();

    // 3. Register job in repo.db (idempotent)
    {
        let conn = repo
            .repo_db()
            .map_err(|e| AppError::internal(format!("Cannot open repo database: {}", e)))?;
        let _ = conn.execute(
            "INSERT OR IGNORE INTO backup_jobs (job_id, job_name, source_type, created_at, status) VALUES (?1, ?2, 0, datetime('now'), 'active')",
            rusqlite::params![job_name, job_name],
        );
    }

    // 4-6. Backup pipeline
    let mut writer = RepositoryBackupWriter::new(&repo, job_name, compress)
        .map_err(|e| AppError::internal(format!("Cannot create backup writer: {}", e)))?;

    writer
        .begin()
        .map_err(|e| AppError::internal(format!("Cannot begin backup transaction: {}", e)))?;

    if let Err(e) = writer.backup_directory(source) {
        let _ = writer.fail();
        return Err(AppError::internal(format!("Backup failed: {}", e)));
    }

    let repo_result = writer
        .finalize()
        .map_err(|e| AppError::internal(format!("Backup finalize failed: {}", e)))?;

    let duration_ms = start.elapsed().as_millis() as u64;

    let backup_id = repo_result.point_id;
    let file_count = repo_result.file_count;
    let total_bytes = repo_result.total_raw_bytes;

    // 7. Record history
    let _ = record_backup_in_history(
        &backup_id,
        job_name,
        source,
        dest,
        file_count,
        total_bytes,
        duration_ms,
        "success",
    );

    let timestamp = chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    Ok(BackupResult {
        backup_id,
        timestamp,
        file_count,
        total_bytes,
        duration_ms,
        status: "success".into(),
        error: None,
    })
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Build a BackupJobView from config data + history query.
fn build_job_view(name: &str, cfg: &JobConfig) -> BackupJobView {
    let retention = match &cfg.retention {
        Some(r) => match (r.keep_count, r.keep_days) {
            (Some(c), None) => format!("Keep last {} versions", c),
            (None, Some(d)) => format!("Keep {} days", d),
            (Some(c), Some(d)) => format!("Keep {} versions or {} days", c, d),
            (None, None) => "Keep all".into(),
        },
        None => "Keep all".into(),
    };

    let (last_time, last_status, last_files, last_bytes) =
        query_last_backup_for_job(name, &cfg.dest);

    let status = derive_job_status(cfg, &last_status);

    BackupJobView {
        name: name.into(),
        source: cfg.source.to_string_lossy().into_owned(),
        dest: cfg.dest.to_string_lossy().into_owned(),
        compress: cfg.compress,
        retention,
        status,
        last_backup_time: last_time,
        last_backup_status: last_status,
        last_backup_files: last_files,
        last_backup_bytes: last_bytes,
    }
}

/// Query the history database for the last backup of a specific job.
fn query_last_backup_for_job(
    job_name: &str,
    dest: &Path,
) -> (Option<String>, Option<String>, u64, u64) {
    let db_path = HistoryDb::history_db_path(dest);
    let db = match HistoryDb::open_or_create(&db_path) {
        Ok(db) => db,
        Err(_) => return (None, None, 0, 0),
    };

    let records = match db.query_history(10, Some("backup")) {
        Ok(r) => r,
        Err(_) => return (None, None, 0, 0),
    };

    for r in &records {
        if r.job_name.as_deref() == Some(job_name) {
            return (
                Some(r.timestamp.clone()),
                Some(r.status.clone()),
                r.file_count,
                r.total_bytes,
            );
        }
    }

    (None, None, 0, 0)
}

/// Derive the job's display status from config validity and last backup.
fn derive_job_status(cfg: &JobConfig, last_status: &Option<String>) -> BackupJobStatus {
    if !cfg.source.exists() {
        return BackupJobStatus::Misconfigured;
    }

    if let Some(parent) = cfg.dest.parent() {
        if !parent.exists() {
            return BackupJobStatus::Misconfigured;
        }
    }

    match last_status {
        None => BackupJobStatus::NeverRun,
        Some(s) if s == "success" => BackupJobStatus::Active,
        _ => BackupJobStatus::Error,
    }
}

/// Record a backup operation in the history database.
#[allow(clippy::too_many_arguments)]
fn record_backup_in_history(
    backup_id: &str,
    job_name: &str,
    source: &Path,
    dest: &Path,
    file_count: u64,
    total_bytes: u64,
    duration_ms: u64,
    status: &str,
) -> Result<(), AppError> {
    let db_path = HistoryDb::history_db_path(dest);
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let db = HistoryDb::open_or_create(&db_path)
        .map_err(|e| AppError::history(format!("Cannot open history DB: {}", e)))?;

    let timestamp = chrono::Local::now()
        .format("%Y-%m-%dT%H:%M:%S%.3fZ")
        .to_string();

    db.record_operation(&OperationRecord {
        backup_id: backup_id.into(),
        operation: "backup".into(),
        timestamp,
        source_root: source.to_string_lossy().into_owned(),
        dest_path: dest.to_string_lossy().into_owned(),
        job_name: Some(job_name.into()),
        file_count,
        total_bytes,
        duration_ms,
        exit_code: if status == "success" { 0 } else { 1 },
        status: status.into(),
    })
    .map_err(|e| AppError::history(format!("Failed to record history: {}", e)))?;

    Ok(())
}
