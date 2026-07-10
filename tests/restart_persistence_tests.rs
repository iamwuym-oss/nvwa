// restart_persistence_tests.rs -- Restart Persistence Test
// Simulates app restart by writing data, then re-reading from disk.
use nuwa_backup::app::models::history::HistoryFilter;
use nuwa_backup::app::services::{config_service, history_service, schedule_service};
use nuwa_backup::config::Config;
use nuwa_backup::history::{HistoryDb, OperationRecord};
use std::fs;
use std::path::Path;
use std::sync::Mutex;
static CWD_LOCK: Mutex<()> = Mutex::new(());
fn make_config_toml(dir: &Path) -> String {
    let src = dir.join("source").to_string_lossy().replace("\\", "\\\\");
    let dst = dir.join("dest").to_string_lossy().replace("\\", "\\\\");
    format!(
        "\n[job.MyJob]\nsource = \"{}\"\ndest = \"{}\"\ncompress = true\n\n[schedules.daily_backup]\nid = \"daily_backup\"\nname = \"Daily Backup\"\nenabled = true\n[schedules.daily_backup.trigger]\ntrigger_type = \"Daily\"\nat = \"02:00\"\n",
        src, dst
    )
}
fn create_history_in(dest_dir: &Path) {
    let db_path = HistoryDb::history_db_path(dest_dir);
    if let Some(parent) = db_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    let db = HistoryDb::open_or_create(&db_path).unwrap();
    db.record_operation(&OperationRecord {
        backup_id: "persist-test-bp-001".into(),
        operation: "backup".into(),
        timestamp: "2026-07-10T08:00:00Z".into(),
        source_root: dest_dir.join("source").to_string_lossy().into_owned(),
        dest_path: dest_dir.to_string_lossy().into_owned(),
        job_name: Some("MyJob".into()),
        file_count: 42,
        total_bytes: 1048576,
        duration_ms: 3000,
        exit_code: 0,
        status: "success".into(),
    })
    .unwrap();
}
fn with_dir<F: FnOnce(&Path) -> T, T>(name: &str, f: F) -> T {
    let _lock = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join("nuwa_test").join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let old = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();
    let result = f(&dir);
    std::env::set_current_dir(&old).unwrap();
    let _ = fs::remove_dir_all(&dir);
    result
}
#[test]
fn test_restart_config_persistence() {
    with_dir("p_config", |dir| {
        fs::write(dir.join("nuwa.toml"), make_config_toml(dir)).unwrap();
        fs::create_dir_all(dir.join("source")).unwrap();
        let _jobs = config_service::list_job_configs().expect("list jobs");
        let config = Config::load().expect("Config reload");
        assert!(config.job.contains_key("MyJob"));
        assert!(config.schedules.contains_key("daily_backup"));
        assert!(config.schedules["daily_backup"].enabled);
    });
}
#[test]
fn test_restart_history_persistence() {
    with_dir("p_history", |dir| {
        let dest = dir.join("dest_backup");
        fs::create_dir_all(&dest).unwrap();
        let src_esc = dir.join("source").to_string_lossy().replace("\\", "\\\\");
        let dst_esc = dest.to_string_lossy().replace("\\", "\\\\");
        let config_toml = format!(
            "\n[job.MyJob]\nsource = \"{}\"\ndest = \"{}\"\n",
            src_esc, dst_esc
        );
        fs::write(dir.join("nuwa.toml"), &config_toml).unwrap();
        fs::create_dir_all(dir.join("source")).unwrap();
        create_history_in(&dest);
        let r1 = history_service::query_history(HistoryFilter {
            operation: None,
            limit: Some(10),
        })
        .expect("q1");
        assert!(r1.total > 0);
        let r2 = history_service::query_history(HistoryFilter {
            operation: None,
            limit: Some(10),
        })
        .expect("q2");
        assert_eq!(r2.total, r1.total);
        assert_eq!(r2.records[0].backup_id, "persist-test-bp-001");
    });
}
#[test]
fn test_restart_schedule_persistence() {
    with_dir("p_schedule", |dir| {
        fs::write(dir.join("nuwa.toml"), make_config_toml(dir)).unwrap();
        fs::create_dir_all(dir.join("source")).unwrap();
        let s = schedule_service::list_schedules().expect("list");
        assert_eq!(s.len(), 1);
        assert_eq!(s[0].name, "Daily Backup");
        assert!(s[0].enabled);
    });
}
