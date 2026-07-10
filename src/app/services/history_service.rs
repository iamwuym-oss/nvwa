// ============================================================================
// history_service.rs -- History domain service
// ============================================================================

use crate::app::error::AppError;
use crate::app::models::history::{HistoryFilter, HistoryQueryResult, HistoryRecordView};
use crate::config::Config;
use crate::history::HistoryDb;

const DEFAULT_LIMIT: u32 = 50;
const MAX_LIMIT: u32 = 200;

pub fn query_history(filter: HistoryFilter) -> Result<HistoryQueryResult, AppError> {
    match Config::load() {
        Ok(config) => query_history_from_config(&config, filter),
        Err(_) => Ok(HistoryQueryResult {
            total: 0,
            records: Vec::new(),
        }),
    }
}

fn query_history_from_config(
    config: &Config,
    filter: HistoryFilter,
) -> Result<HistoryQueryResult, AppError> {
    let limit = filter.limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT);
    let mut all_records: Vec<HistoryRecordView> = Vec::new();

    for job_cfg in config.job.values() {
        let db_path = HistoryDb::history_db_path(&job_cfg.dest);
        if !db_path.exists() {
            continue;
        }
        let db = match HistoryDb::open_or_create(&db_path) {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Warning: cannot open history DB at {:?}: {}", db_path, e);
                continue;
            }
        };
        let records = match db.query_history(MAX_LIMIT, filter.operation.as_deref()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Warning: query history failed at {:?}: {}", db_path, e);
                continue;
            }
        };
        for rec in &records {
            all_records.push(HistoryRecordView {
                backup_id: rec.backup_id.clone(),
                operation: rec.operation.clone(),
                timestamp: rec.timestamp.clone(),
                source_root: rec.source_root.clone(),
                dest_path: rec.dest_path.clone(),
                job_name: rec.job_name.clone(),
                file_count: rec.file_count,
                total_bytes: rec.total_bytes,
                duration_ms: rec.duration_ms,
                status: rec.status.clone(),
                exit_info: exit_code_to_info(rec.exit_code),
            });
        }
    }

    all_records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    let total = all_records.len() as u32;
    all_records.truncate(limit as usize);

    Ok(HistoryQueryResult {
        total,
        records: all_records,
    })
}

pub fn list_operation_types() -> Vec<String> {
    vec![
        "backup".to_string(),
        "delete_backup_set".to_string(),
        "restore".to_string(),
        "verify".to_string(),
    ]
}

fn exit_code_to_info(code: i32) -> String {
    match code {
        0 => "Completed successfully".to_string(),
        1 => "General failure".to_string(),
        2 => "Invalid arguments".to_string(),
        3 => "I/O error".to_string(),
        4 => "Checksum or verification failure".to_string(),
        5 => "Restore validation failure".to_string(),
        6 => "Safety rule violation".to_string(),
        7 => "Manifest error".to_string(),
        _ => format!("Exit code {}", code),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::history::OperationRecord;
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;

    fn setup_test_history_db(dir: &Path) {
        fs::create_dir_all(dir).unwrap();
        let db_path = HistoryDb::history_db_path(dir);
        let _ = fs::remove_file(&db_path);
        let db = HistoryDb::open_or_create(&db_path).unwrap();
        for i in 0..3 {
            db.record_operation(&OperationRecord {
                backup_id: format!("test-backup-{:04}", i + 1),
                operation: "backup".into(),
                timestamp: format!("2026-07-{:02}T10:00:0{}Z", 5 + i, i),
                source_root: "C:\\Users\\Test\\Documents".into(),
                dest_path: dir.to_string_lossy().into_owned(),
                job_name: Some("Test Job".into()),
                file_count: 100 + i * 10,
                total_bytes: 1048576 + i * 102400,
                duration_ms: 3000 + i * 500,
                exit_code: 0,
                status: "success".into(),
            })
            .unwrap();
        }
        db.record_operation(&OperationRecord {
            backup_id: "test-verify-0001".into(),
            operation: "verify".into(),
            timestamp: "2026-07-08T12:00:00Z".into(),
            source_root: "C:\\Users\\Test\\Documents".into(),
            dest_path: dir.to_string_lossy().into_owned(),
            job_name: Some("Test Job".into()),
            file_count: 130,
            total_bytes: 1352668,
            duration_ms: 2000,
            exit_code: 0,
            status: "success".into(),
        })
        .unwrap();
        db.record_operation(&OperationRecord {
            backup_id: "test-restore-0001".into(),
            operation: "restore".into(),
            timestamp: "2026-07-08T14:00:00Z".into(),
            source_root: "C:\\Users\\Test\\Documents".into(),
            dest_path: dir.to_string_lossy().into_owned(),
            job_name: Some("Test Job".into()),
            file_count: 120,
            total_bytes: 1153433,
            duration_ms: 5000,
            exit_code: 0,
            status: "success".into(),
        })
        .unwrap();
    }

    fn make_test_config(backup_dir: &Path) -> Config {
        let mut job = HashMap::new();
        job.insert(
            "Test Job".to_string(),
            crate::config::JobConfig {
                source: Path::new("C:\\Users\\Test\\Documents").to_path_buf(),
                dest: backup_dir.to_path_buf(),
                compress: false,
                retention: None,
                schedule_id: None,
                storage_type: None,
                repository_id: None,
            },
        );
        Config {
            job,
            schedules: HashMap::new(),
        }
    }

    #[test]
    fn test_query_all_history() {
        let temp_dir = std::env::temp_dir().join("nuwa_test_hist_svc_all");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let backup_dir = temp_dir.join("backups").join("Test");
        setup_test_history_db(&backup_dir);
        let config = make_test_config(&backup_dir);
        let result = query_history_from_config(
            &config,
            HistoryFilter {
                limit: None,
                operation: None,
            },
        )
        .expect("query_history should succeed");
        assert_eq!(result.total, 5);
        assert_eq!(result.records.len(), 5);
        assert_eq!(result.records[0].backup_id, "test-restore-0001");
        assert_eq!(result.records[0].operation, "restore");
        assert_eq!(result.records[0].status, "success");
        assert_eq!(result.records[0].exit_info, "Completed successfully");
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_query_filter_by_operation() {
        let temp_dir = std::env::temp_dir().join("nuwa_test_hist_svc_filter");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let backup_dir = temp_dir.join("backups").join("Test");
        setup_test_history_db(&backup_dir);
        let config = make_test_config(&backup_dir);
        let result = query_history_from_config(
            &config,
            HistoryFilter {
                limit: None,
                operation: Some("backup".into()),
            },
        )
        .expect("query_history should succeed");
        assert_eq!(result.total, 3);
        assert!(result.records.iter().all(|r| r.operation == "backup"));
        let result = query_history_from_config(
            &config,
            HistoryFilter {
                limit: None,
                operation: Some("verify".into()),
            },
        )
        .expect("query_history should succeed");
        assert_eq!(result.total, 1);
        assert_eq!(result.records[0].operation, "verify");
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_query_history_limit() {
        let temp_dir = std::env::temp_dir().join("nuwa_test_hist_svc_limit");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let backup_dir = temp_dir.join("backups").join("Test");
        setup_test_history_db(&backup_dir);
        let config = make_test_config(&backup_dir);
        let result = query_history_from_config(
            &config,
            HistoryFilter {
                limit: Some(2),
                operation: None,
            },
        )
        .expect("query_history should succeed");
        assert_eq!(result.total, 5);
        assert_eq!(result.records.len(), 2);
        assert_eq!(result.records[0].operation, "restore");
        assert_eq!(result.records[1].operation, "verify");
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_query_history_multiple_jobs() {
        let temp_dir = std::env::temp_dir().join("nuwa_test_hist_svc_multi");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        let backup_dir_a = temp_dir.join("backups").join("JobA");
        let backup_dir_b = temp_dir.join("backups").join("JobB");
        setup_test_history_db(&backup_dir_a);
        fs::create_dir_all(&backup_dir_b).unwrap();
        let db_path = HistoryDb::history_db_path(&backup_dir_b);
        let db = HistoryDb::open_or_create(&db_path).unwrap();
        db.record_operation(&OperationRecord {
            backup_id: "jobb-backup-0001".into(),
            operation: "backup".into(),
            timestamp: "2026-07-09T10:00:00Z".into(),
            source_root: "C:\\Users\\Test\\Photos".into(),
            dest_path: backup_dir_b.to_string_lossy().into_owned(),
            job_name: Some("JobB".into()),
            file_count: 500,
            total_bytes: 524288000,
            duration_ms: 15000,
            exit_code: 0,
            status: "success".into(),
        })
        .unwrap();
        db.record_operation(&OperationRecord {
            backup_id: "jobb-verify-0001".into(),
            operation: "verify".into(),
            timestamp: "2026-07-09T12:00:00Z".into(),
            source_root: "C:\\Users\\Test\\Photos".into(),
            dest_path: backup_dir_b.to_string_lossy().into_owned(),
            job_name: Some("JobB".into()),
            file_count: 500,
            total_bytes: 524288000,
            duration_ms: 8000,
            exit_code: 0,
            status: "success".into(),
        })
        .unwrap();
        let mut job = HashMap::new();
        job.insert(
            "JobA".to_string(),
            crate::config::JobConfig {
                source: Path::new("C:\\Users\\Test\\Documents").to_path_buf(),
                dest: backup_dir_a.clone(),
                compress: false,
                retention: None,
                schedule_id: None,
                storage_type: None,
                repository_id: None,
            },
        );
        job.insert(
            "JobB".to_string(),
            crate::config::JobConfig {
                source: Path::new("C:\\Users\\Test\\Photos").to_path_buf(),
                dest: backup_dir_b.clone(),
                compress: false,
                retention: None,
                schedule_id: None,
                storage_type: None,
                repository_id: None,
            },
        );
        let config = Config {
            job,
            schedules: HashMap::new(),
        };
        let result = query_history_from_config(
            &config,
            HistoryFilter {
                limit: None,
                operation: None,
            },
        )
        .expect("query_history should succeed");
        assert_eq!(result.total, 7);
        assert_eq!(result.records[0].backup_id, "jobb-verify-0001");
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_query_history_empty_config() {
        let config = Config {
            job: HashMap::new(),
            schedules: HashMap::new(),
        };
        let result = query_history_from_config(
            &config,
            HistoryFilter {
                limit: None,
                operation: None,
            },
        )
        .expect("query_history should succeed");
        assert_eq!(result.total, 0);
        assert!(result.records.is_empty());
    }

    #[test]
    fn test_list_operation_types() {
        let types = list_operation_types();
        assert_eq!(types.len(), 4);
        assert!(types.contains(&"backup".to_string()));
        assert!(types.contains(&"delete_backup_set".to_string()));
    }

    #[test]
    fn test_exit_code_to_info() {
        assert_eq!(exit_code_to_info(0), "Completed successfully");
        assert_eq!(exit_code_to_info(4), "Checksum or verification failure");
        assert_eq!(exit_code_to_info(99), "Exit code 99");
    }
}
