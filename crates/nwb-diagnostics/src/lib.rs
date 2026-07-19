//! Structured diagnostics and secret-safe JSON logging for the NWB engine.
//!
//! The logging schema deliberately has no arbitrary text, path, payload, or
//! error-source fields. Dynamic context is restricted to typed identifiers,
//! enums, booleans, and counters. Human-facing text is resolved outside the
//! log record from the stable [`ErrorId`] and event identifiers.

use nwb_format::registry::error_id::ErrorId;
use serde::Serialize;
use std::fmt;
use std::io::{self, Write};

const SCHEMA_VERSION: u16 = 1;
const REDACTED: &str = "[REDACTED]";

/// A secret byte sequence whose normal formatting is always redacted.
///
/// The type intentionally does not implement `Clone`, `Serialize`, `AsRef`, or
/// any logging-field conversion. Callers must explicitly invoke
/// [`Secret::expose_secret`] at the narrow cryptographic boundary that needs
/// the bytes. The owned buffer is overwritten before deallocation.
pub struct Secret {
    bytes: Vec<u8>,
}

impl Secret {
    /// Takes ownership of secret bytes without creating another copy.
    #[must_use]
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Takes ownership of a UTF-8 secret without creating another string copy.
    #[must_use]
    pub fn from_string(value: String) -> Self {
        Self {
            bytes: value.into_bytes(),
        }
    }

    /// Exposes the secret only for the narrow operation that requires it.
    #[must_use]
    pub fn expose_secret(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the byte length without exposing the value.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Reports whether the secret is empty without exposing the value.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(REDACTED)
    }
}

impl fmt::Display for Secret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(REDACTED)
    }
}

impl Drop for Secret {
    fn drop(&mut self) {
        self.bytes.fill(0);
    }
}

/// Severity of a structured diagnostic.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Stable engine stage; no caller-controlled text is accepted.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Cli,
    Core,
    Writer,
    Reader,
    Catalog,
    Provider,
    Restore,
    Verify,
    Salvage,
}

/// Whether retrying the same operation is meaningful.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Retryability {
    Never,
    Transient,
    AfterUserAction,
}

/// Whether the condition can affect recoverability.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryImpact {
    None,
    Degraded,
    BlocksCommit,
    BlocksRestore,
}

/// Stable event kind; messages are resolved outside the log record.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
    OperationStarted,
    OperationProgress,
    OperationCompleted,
    OperationFailed,
    IntegrityDegraded,
    SecurityPolicyViolation,
}

/// A typed 128-bit correlation identifier.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct OperationId([u8; 16]);

impl OperationId {
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// A structured error bound to the generated Error Registry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    error_id: ErrorId,
    severity: Severity,
    stage: Stage,
    retryability: Retryability,
    recovery_impact: RecoveryImpact,
    os_error_code: Option<i32>,
}

impl Diagnostic {
    /// Creates a diagnostic. The reserved `Invalid` identifier is rejected.
    pub fn new(
        error_id: ErrorId,
        severity: Severity,
        stage: Stage,
        retryability: Retryability,
        recovery_impact: RecoveryImpact,
    ) -> Result<Self, DiagnosticBuildError> {
        if error_id == ErrorId::Invalid {
            return Err(DiagnosticBuildError::InvalidErrorId);
        }

        Ok(Self {
            error_id,
            severity,
            stage,
            retryability,
            recovery_impact,
            os_error_code: None,
        })
    }

    /// Adds only the numeric operating-system error code, never its free-text source.
    #[must_use]
    pub const fn with_os_error_code(mut self, code: i32) -> Self {
        self.os_error_code = Some(code);
        self
    }

    #[must_use]
    pub const fn error_id(&self) -> ErrorId {
        self.error_id
    }
}

/// Error returned when a diagnostic violates the registry contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticBuildError {
    InvalidErrorId,
}

impl fmt::Display for DiagnosticBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("the reserved Invalid ErrorId cannot be logged")
    }
}

impl std::error::Error for DiagnosticBuildError {}

/// Numeric progress fields allowed in structured logs.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
pub struct Metrics {
    pub objects_processed: u64,
    pub bytes_processed: u64,
    pub retry_count: u32,
    pub warning_count: u32,
}

/// One canonical structured log event.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LogEvent {
    timestamp_unix_ms: u64,
    level: Severity,
    event: EventKind,
    stage: Stage,
    operation_id: Option<OperationId>,
    diagnostic: Option<Diagnostic>,
    metrics: Metrics,
}

impl LogEvent {
    #[must_use]
    pub const fn new(
        timestamp_unix_ms: u64,
        level: Severity,
        event: EventKind,
        stage: Stage,
    ) -> Self {
        Self {
            timestamp_unix_ms,
            level,
            event,
            stage,
            operation_id: None,
            diagnostic: None,
            metrics: Metrics {
                objects_processed: 0,
                bytes_processed: 0,
                retry_count: 0,
                warning_count: 0,
            },
        }
    }

    #[must_use]
    pub const fn with_operation_id(mut self, operation_id: OperationId) -> Self {
        self.operation_id = Some(operation_id);
        self
    }

    #[must_use]
    pub const fn with_diagnostic(mut self, diagnostic: Diagnostic) -> Self {
        self.diagnostic = Some(diagnostic);
        self
    }

    #[must_use]
    pub const fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }

    /// Encodes exactly one deterministic UTF-8 JSON line.
    pub fn to_json_line(&self) -> Result<Vec<u8>, LogWriteError> {
        let wire = LogEventWire::from(self);
        let mut encoded = serde_json::to_vec(&wire).map_err(|_| LogWriteError::Encoding)?;
        encoded.push(b'\n');
        Ok(encoded)
    }

    /// Writes exactly one structured JSON line.
    pub fn write_json_line(&self, writer: &mut impl Write) -> Result<(), LogWriteError> {
        let encoded = self.to_json_line()?;
        writer
            .write_all(&encoded)
            .map_err(|error| LogWriteError::Io(error.kind()))
    }
}

#[derive(Serialize)]
struct LogEventWire {
    schema_version: u16,
    timestamp_unix_ms: u64,
    level: Severity,
    event: EventKind,
    stage: Stage,
    operation_id: Option<String>,
    diagnostic: Option<DiagnosticWire>,
    metrics: Metrics,
}

impl From<&LogEvent> for LogEventWire {
    fn from(event: &LogEvent) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            timestamp_unix_ms: event.timestamp_unix_ms,
            level: event.level,
            event: event.event,
            stage: event.stage,
            operation_id: event.operation_id.map(|id| id.to_string()),
            diagnostic: event.diagnostic.map(DiagnosticWire::from),
            metrics: event.metrics,
        }
    }
}

#[derive(Serialize)]
struct DiagnosticWire {
    error_id: u16,
    error_name: &'static str,
    severity: Severity,
    stage: Stage,
    retryability: Retryability,
    recovery_impact: RecoveryImpact,
    os_error_code: Option<i32>,
}

impl From<Diagnostic> for DiagnosticWire {
    fn from(diagnostic: Diagnostic) -> Self {
        Self {
            error_id: diagnostic.error_id as u16,
            error_name: error_id_name(diagnostic.error_id),
            severity: diagnostic.severity,
            stage: diagnostic.stage,
            retryability: diagnostic.retryability,
            recovery_impact: diagnostic.recovery_impact,
            os_error_code: diagnostic.os_error_code,
        }
    }
}

const fn error_id_name(error_id: ErrorId) -> &'static str {
    match error_id {
        ErrorId::Invalid => "Invalid",
        ErrorId::ChecksumMismatch => "ChecksumMismatch",
        ErrorId::FormatVersion => "FormatVersion",
        ErrorId::Compression => "Compression",
        ErrorId::SegmentCorrupt => "SegmentCorrupt",
        ErrorId::IoError => "IoError",
        ErrorId::VolumeMissing => "VolumeMissing",
        ErrorId::EncryptionAuth => "EncryptionAuth",
        ErrorId::CatalogCorrupt => "CatalogCorrupt",
    }
}

/// Sanitized write failure. The underlying error text is intentionally discarded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LogWriteError {
    Encoding,
    Io(io::ErrorKind),
}

impl fmt::Display for LogWriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Encoding => formatter.write_str("structured log encoding failed"),
            Self::Io(_) => formatter.write_str("structured log write failed"),
        }
    }
}

impl std::error::Error for LogWriteError {}
