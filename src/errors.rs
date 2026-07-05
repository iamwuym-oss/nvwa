// ============================================================================
// errors.rs -- Nuwa Backup error types and user-friendly error messages
//
// Design principles:
// 1. All external interaction errors (IO, checksum, args) have clear error types
// 2. Do not print stack traces to normal users (but keep Debug info for developers)
// 3. Each error includes a suggestion for what the user can do next
// 4. Exit codes strictly follow AGENTS.md §13 specification
// ============================================================================

use std::fmt;
use std::path::PathBuf;

/// Nuwa Backup exit codes -- strictly mapped to Phase 1 CLI Contract
/// See AGENTS.md §13 Exit codes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,         // Operation completed successfully
    GeneralFailure = 1,  // General error, cannot be classified elsewhere
    InvalidArgs = 2,     // Invalid arguments
    IoError = 3,         // I/O error (disk full, permission denied, file locked, etc.)
    ChecksumFailure = 4, // Checksum mismatch or verification failure
    RestoreFailure = 5,  // Restore validation failure
    SafetyViolation = 6, // Safety rule violation (e.g. source = destination)
    ManifestError = 7,   // Manifest missing, corrupted, or incompatible
}

/// Unified Nuwa Backup error type
/// Each variant encapsulates:
/// - Specific technical error (optional)
/// - User-facing error description
/// - Action suggestion for the user
#[derive(Debug)]
pub enum NuwaError {
    /// Argument parsing error: user entered invalid command line arguments
    InvalidArgument { detail: String, suggestion: String },
    /// I/O error: file read/write, directory creation, disk space, etc.
    Io {
        source: Option<std::io::Error>,
        path: Option<PathBuf>,
        detail: String,
        suggestion: String,
    },
    /// Checksum error: file content does not match recorded checksum
    ChecksumMismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    /// Safety rule violation: operation may damage user data
    SafetyViolation { detail: String, suggestion: String },
    /// Manifest error: JSON format error, version incompatibility, missing fields
    ManifestError { detail: String, suggestion: String },
    /// Backup/restore verification failure
    VerificationFailed { detail: String },
    /// General error -- fallback type
    General { detail: String, suggestion: String },
}

impl fmt::Display for NuwaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NuwaError::InvalidArgument { detail, suggestion } => {
                write!(f, "Argument error: {}\nSuggestion: {}", detail, suggestion)
            }
            NuwaError::Io {
                detail, suggestion, ..
            } => {
                write!(f, "I/O error: {}\nSuggestion: {}", detail, suggestion)
            }
            NuwaError::ChecksumMismatch {
                path,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Checksum mismatch: {} (expected: {}, actual: {})",
                    path.display(),
                    expected,
                    actual
                )
            }
            NuwaError::SafetyViolation { detail, suggestion } => {
                write!(
                    f,
                    "Safety violation: {}\nSuggestion: {}",
                    detail, suggestion
                )
            }
            NuwaError::ManifestError { detail, suggestion } => {
                write!(f, "Manifest error: {}\nSuggestion: {}", detail, suggestion)
            }
            NuwaError::VerificationFailed { detail } => {
                write!(f, "Verification failed: {}", detail)
            }
            NuwaError::General { detail, suggestion } => {
                write!(f, "Error: {}\nSuggestion: {}", detail, suggestion)
            }
        }
    }
}

impl std::error::Error for NuwaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            NuwaError::Io {
                source: Some(e), ..
            } => Some(e),
            _ => None,
        }
    }
}

/// Map NuwaError to its corresponding exit code
/// The main program uses this to determine the process exit value
impl From<&NuwaError> for ExitCode {
    fn from(err: &NuwaError) -> Self {
        match err {
            NuwaError::InvalidArgument { .. } => ExitCode::InvalidArgs,
            NuwaError::Io { .. } => ExitCode::IoError,
            NuwaError::ChecksumMismatch { .. } => ExitCode::ChecksumFailure,
            NuwaError::SafetyViolation { .. } => ExitCode::SafetyViolation,
            NuwaError::ManifestError { .. } => ExitCode::ManifestError,
            NuwaError::VerificationFailed { .. } => ExitCode::RestoreFailure,
            NuwaError::General { .. } => ExitCode::GeneralFailure,
        }
    }
}

/// Convenience conversion from std::io::Error to NuwaError
/// Automatically extracts path information (if available) and provides general I/O suggestions
impl From<std::io::Error> for NuwaError {
    fn from(err: std::io::Error) -> Self {
        let detail = format!(
            "{} (error code: {})",
            err,
            err.raw_os_error()
                .map_or_else(|| "unknown".to_string(), |c| c.to_string())
        );
        NuwaError::Io {
            source: Some(err),
            path: None,
            detail,
            suggestion:
                "Please check: 1) Target path exists and is writable 2) Disk space is sufficient 3) File is not locked by another process"
                    .to_string(),
        }
    }
}

// ============================================================================
// Factory methods -- convenience functions for creating common error types
// ============================================================================
impl NuwaError {
    /// Create "source path not found" error
    pub fn source_not_found(path: &std::path::Path) -> Self {
        NuwaError::InvalidArgument {
            detail: format!("Source path not found: '{}'. Please verify the path is correct", path.display()),
            suggestion: "Use an absolute path, e.g.: nuwa backup --source C:\\Users\\YourName\\Documents --dest D:\\Backup".to_string(),
        }
    }

    /// Create "source = destination" safety error
    pub fn same_source_dest(source: &std::path::Path, dest: &std::path::Path) -> Self {
        NuwaError::SafetyViolation {
            detail: format!(
                "Source and destination paths are the same. Operation blocked.\n  Source: {}\n  Dest:   {}",
                source.display(),
                dest.display()
            ),
            suggestion: "Choose a different destination path. Consider using another drive or USB device.".to_string(),
        }
    }

    /// Create "disk space insufficient" error
    pub fn disk_space(needed: u64, available: u64, path: &std::path::Path) -> Self {
        let needed_mb = needed / (1024 * 1024);
        let available_mb = available / (1024 * 1024);
        NuwaError::Io {
            source: None,
            path: Some(path.to_path_buf()),
            detail: format!(
                "Insufficient disk space on target. Need ~{} MB, available {} MB.",
                needed_mb, available_mb
            ),
            suggestion:
                "Free up disk space on the target, or use a larger backup destination. A 10% safety margin is mandatory."
                    .to_string(),
        }
    }

    /// Create Manifest parse error
    pub fn manifest_parse(path: &std::path::Path, parse_err: &str) -> Self {
        NuwaError::ManifestError {
            detail: format!("Cannot parse manifest file: '{}'\nParse error: {}", path.display(), parse_err),
            suggestion: "The manifest.json file may be corrupted. Try: 1) Run verify to check backup integrity 2) If corrupted, recreate the backup.".to_string(),
        }
    }

    /// Create Manifest version incompatibility error
    pub fn manifest_version(path: &std::path::Path, version: &str) -> Self {
        NuwaError::ManifestError {
            detail: format!(
                "Manifest version incompatible: '{}' (version: {}). Current program only supports version 1.0",
                path.display(),
                version
            ),
            suggestion: "Use the same program version that created this backup for restore operations.".to_string(),
        }
    }
}
