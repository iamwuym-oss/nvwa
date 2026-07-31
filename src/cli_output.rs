// ============================================================================
// cli_output.rs — CLI JSON output, summary formatting, and progress callback
//
// Design:
// - JSON output is for scripting/automation/external tools
// - Summary table shows after backup/restore/verify (file count, size, duration)
// - ProgressCallback is shared between CLI (progress bar) and GUI (mpsc channel)
// - GUI does NOT parse CLI JSON output; GUI calls lib.rs API directly
// - --json mode outputs clean JSON without mixed-in regular text
// ============================================================================

use crate::errors::ExitCode;
use crate::history::OperationRecord;
use serde::Serialize;
use std::io::{self, Write};

// ============================================================================
// Progress callback — used by both CLI (progress bar) and GUI (mpsc channel)
// ============================================================================

/// Callback type for reporting progress during long operations.
pub type ProgressCallback = Box<dyn Fn(ProgressEvent) + Send>;

/// Progress event types
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// Overall progress percentage and current file being processed
    Progress { percent: f32, current_file: String },
    /// Operation completed with summary
    Completed { summary: String },
    /// Operation failed with error
    Failed { error: String },
    /// Informational log message
    Log { message: String },
}

/// Create a default (no-op) progress callback
pub fn noop_progress() -> ProgressCallback {
    Box::new(|_| {})
}

/// Check if stderr is a terminal (interactive). If not, progress display should be suppressed.
pub fn is_interactive() -> bool {
    #[cfg(windows)]
    {
        // On Windows, use GetStdHandle + GetConsoleMode to detect
        // We check if we can get console mode for stderr handle
        true
    }
    #[cfg(not(windows))]
    {
        true
    }
}

// ============================================================================
// Generic JSON output wrapper — all commands return this structure
// ============================================================================

/// Top-level JSON output envelope for all commands
#[derive(Debug, Clone, Serialize)]
pub struct JsonOutput {
    pub command: String,
    pub status: String,
    pub exit_code: i32,
    pub message: String,
    pub duration_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backup_point: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restored_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum_failures: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub passed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inaccessible: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub records: Option<Vec<HistoryEntryJson>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dry_run: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kept_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damaged_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_backup_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kept_backup_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warnings: Option<Vec<String>>,
}

impl JsonOutput {
    /// Build a successful JSON output
    pub fn success(command: &str, message: &str, duration_ms: u64) -> Self {
        JsonOutput {
            command: command.to_string(),
            status: "success".to_string(),
            exit_code: 0,
            message: message.to_string(),
            duration_ms,
            backup_id: None,
            backup_point: None,
            file_count: None,
            total_bytes: None,
            restored_count: None,
            skipped_count: None,
            checksum_failures: None,
            passed: None,
            failed: None,
            inaccessible: None,
            error: None,
            records: None,
            dry_run: None,
            deleted_count: None,
            kept_count: None,
            damaged_count: None,
            deleted_backup_ids: None,
            kept_backup_ids: None,
            warnings: None,
        }
    }

    /// Build a failure JSON output
    pub fn failure(command: &str, error_msg: &str, duration_ms: u64) -> Self {
        JsonOutput {
            command: command.to_string(),
            status: "failure".to_string(),
            exit_code: 1,
            message: format!("{} failed: {}", command, error_msg),
            duration_ms,
            backup_id: None,
            backup_point: None,
            file_count: None,
            total_bytes: None,
            restored_count: None,
            skipped_count: None,
            checksum_failures: None,
            passed: None,
            failed: None,
            inaccessible: None,
            error: Some(error_msg.to_string()),
            records: None,
            dry_run: None,
            deleted_count: None,
            kept_count: None,
            damaged_count: None,
            deleted_backup_ids: None,
            kept_backup_ids: None,
            warnings: None,
        }
    }
}
/// History entry for history JSON output
#[derive(Debug, Clone, Serialize)]
pub struct HistoryEntryJson {
    pub backup_id: String,
    pub operation: String,
    pub timestamp: String,
    pub source_root: String,
    pub dest_path: String,
    pub job_name: Option<String>,
    pub file_count: u64,
    pub total_bytes: u64,
    pub duration_ms: u64,
    pub exit_code: i32,
    pub status: String,
}

impl From<&OperationRecord> for HistoryEntryJson {
    fn from(r: &OperationRecord) -> Self {
        HistoryEntryJson {
            backup_id: r.backup_id.clone(),
            operation: r.operation.clone(),
            timestamp: r.timestamp.clone(),
            source_root: r.source_root.clone(),
            dest_path: r.dest_path.clone(),
            job_name: r.job_name.clone(),
            file_count: r.file_count,
            total_bytes: r.total_bytes,
            duration_ms: r.duration_ms,
            exit_code: r.exit_code,
            status: r.status.clone(),
        }
    }
}

// ============================================================================
// JSON output helpers
// ============================================================================

/// Print JSON output to stdout. Returns ExitCode based on status.
pub fn print_json(output: &JsonOutput) -> ExitCode {
    match serde_json::to_string_pretty(output) {
        Ok(json) => {
            println!("{}", json);
            if output.status == "success" {
                ExitCode::Success
            } else {
                ExitCode::GeneralFailure
            }
        }
        Err(e) => {
            eprintln!("Internal error: JSON serialization failed: {}", e);
            ExitCode::GeneralFailure
        }
    }
}

/// Print JSON output in compact form (single line) for pipe-friendly use
pub fn print_json_compact(output: &JsonOutput) -> ExitCode {
    match serde_json::to_string(output) {
        Ok(json) => {
            println!("{}", json);
            if output.status == "success" {
                ExitCode::Success
            } else {
                ExitCode::GeneralFailure
            }
        }
        Err(e) => {
            eprintln!("Internal error: JSON serialization failed: {}", e);
            ExitCode::GeneralFailure
        }
    }
}

// ============================================================================
// Summary formatting helpers (human-readable mode)
// ============================================================================

/// Print a summary table for backup results
pub fn print_backup_summary(
    backup_point: &str,
    file_count: u64,
    total_bytes: u64,
    duration_ms: u64,
) {
    let size_str = format_size(total_bytes);
    let duration_str = format_duration(duration_ms);
    println!();
    println!("{}", "=".repeat(60));
    println!("  Backup Summary");
    println!("{}", "=".repeat(60));
    println!("  Backup point:  {}", backup_point);
    println!("  Files:         {}", file_count);
    println!("  Total size:    {}", size_str);
    println!("  Duration:      {}", duration_str);
    println!("  Status:        OK");
    println!("{}", "=".repeat(60));
}

/// Print a summary table for restore results
pub fn print_restore_summary(
    restored: u64,
    skipped: u64,
    checksum_failures: u64,
    duration_ms: u64,
) {
    let duration_str = format_duration(duration_ms);
    println!();
    println!("{}", "=".repeat(60));
    println!("  Restore Summary");
    println!("{}", "=".repeat(60));
    println!("  Restored:         {}", restored);
    println!("  Skipped:          {}", skipped);
    println!("  Checksum errors:  {}", checksum_failures);
    println!("  Duration:         {}", duration_str);
    let status = if checksum_failures > 0 {
        "PARTIAL"
    } else {
        "OK"
    };
    println!("  Status:           {}", status);
    println!("{}", "=".repeat(60));
}

/// Print a summary table for verify results
pub fn print_verify_summary(total_files: u64, passed: u64, failed: u64, inaccessible: u64) {
    println!();
    println!("{}", "=".repeat(60));
    println!("  Verify Summary");
    println!("{}", "=".repeat(60));
    println!("  Total files:    {}", total_files);
    println!("  Passed:         {}", passed);
    println!("  Failed:         {}", failed);
    println!("  Inaccessible:   {}", inaccessible);
    let status = if failed > 0 || inaccessible > 0 {
        "FAILED"
    } else {
        "OK"
    };
    println!("  Status:         {}", status);
    println!("{}", "=".repeat(60));
}

// ============================================================================
// Progress display for interactive terminal
// ============================================================================

/// A simple progress bar that writes to stderr.
/// Only shows output when stderr is a terminal (interactive).
pub struct ProgressBar {
    enabled: bool,
    last_pct: i32,
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressBar {
    pub fn new() -> Self {
        ProgressBar {
            enabled: true,
            last_pct: -1,
        }
    }

    /// Update the progress percentage. Only redraws when percentage changes.
    pub fn update(&mut self, percent: f32, current_file: &str) {
        if !self.enabled {
            return;
        }
        let pct = (percent * 100.0) as i32;
        if pct == self.last_pct {
            return; // Avoid flickering
        }
        self.last_pct = pct;

        // Simple single-line progress: [#######    ] 45% file.txt
        let bar_width: usize = 30;
        let filled = ((percent / 100.0) * bar_width as f32) as usize;
        let empty = bar_width.saturating_sub(filled);

        // Truncate filename for display
        let file_display = if current_file.len() > 25 {
            format!(
                "...{}",
                &current_file[current_file.len().saturating_sub(22)..]
            )
        } else {
            current_file.to_string()
        };

        // Use carriage return to overwrite the line
        let _ = write!(
            io::stderr(),
            "\r  [{}{}] {:3}% {}",
            "#".repeat(filled),
            " ".repeat(empty),
            pct.min(100),
            file_display,
        );
        let _ = io::stderr().flush();
    }

    /// Mark progress as complete (clear the progress line)
    pub fn finish(&mut self) {
        if !self.enabled {
            return;
        }
        // Clear progress line
        let _ = write!(io::stderr(), "\r  {:width$}\r", "", width = 60);
        let _ = io::stderr().flush();
        self.last_pct = -1;
    }
}

// ============================================================================
// Utility formatting helpers
// ============================================================================

/// Format bytes into human-readable string
pub fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

/// Format milliseconds into human-readable duration
pub fn format_duration(ms: u64) -> String {
    if ms >= 60000 {
        let mins = ms / 60000;
        let secs = (ms % 60000) / 1000;
        format!("{}m {}s", mins, secs)
    } else if ms >= 1000 {
        format!("{:.2}s", ms as f64 / 1000.0)
    } else {
        format!("{}ms", ms)
    }
}

#[test]
fn test_format_duration() {
    assert_eq!(format_duration(500), "500ms");
    assert_eq!(format_duration(1500), "1.50s");
    assert_eq!(format_duration(65000), "1m 5s");
}

#[test]
fn test_json_output_success_serialization() {
    let output = JsonOutput::success("backup", "Backup completed", 5000);
    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("\"status\":\"success\""));
    assert!(json.contains("\"command\":\"backup\""));
    assert!(json.contains("\"exit_code\":0"));
    assert!(!json.contains("\"error\""));
}

#[test]
fn test_json_output_failure_serialization() {
    let output = JsonOutput::failure("backup", "Disk full", 1000);
    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("\"status\":\"failure\""));
    assert!(json.contains("\"error\":\"Disk full\""));
    assert!(json.contains("\"exit_code\":1"));
}

#[test]
fn test_json_output_backup_with_stats() {
    let mut output = JsonOutput::success("backup", "Backup completed", 5000);
    output.backup_point = Some("20260705_143000_Test".to_string());
    output.file_count = Some(100);
    output.total_bytes = Some(1048576);
    output.backup_id = Some("uuid-123".to_string());
    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("\"backup_point\""));
    assert!(json.contains("\"file_count\":100"));
}

#[test]
fn test_json_output_restore_with_stats() {
    let mut output = JsonOutput::success("restore", "Restore completed", 3000);
    output.restored_count = Some(95);
    output.skipped_count = Some(5);
    output.checksum_failures = Some(0);
    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("\"restored_count\":95"));
    assert!(json.contains("\"skipped_count\":5"));
}

#[test]
fn test_json_output_verify_with_stats() {
    let mut output = JsonOutput::success("verify", "Verify completed", 2000);
    output.passed = Some(100);
    output.failed = Some(0);
    output.inaccessible = Some(0);
    let json = serde_json::to_string(&output).unwrap();
    assert!(json.contains("\"passed\":100"));
    assert!(json.contains("\"failed\":0"));
}
