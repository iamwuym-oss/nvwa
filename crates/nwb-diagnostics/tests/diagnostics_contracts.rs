use nwb_diagnostics::{
    Diagnostic, DiagnosticBuildError, EventKind, LogEvent, Metrics, OperationId, RecoveryImpact,
    Retryability, Secret, Severity, Stage,
};
use nwb_format::registry::error_id::ErrorId;
use serde_json::Value;
use std::io::{self, Write};

const SECRET_CANARY: &str = "NUWA_SECRET_CANARY_PASSWORD_7f35a6";

fn sample_diagnostic() -> Diagnostic {
    Diagnostic::new(
        ErrorId::ChecksumMismatch,
        Severity::Error,
        Stage::Reader,
        Retryability::Never,
        RecoveryImpact::BlocksRestore,
    )
    .expect("a non-reserved ErrorId must build")
    .with_os_error_code(23)
}

fn sample_event() -> LogEvent {
    LogEvent::from_diagnostic(
        1_721_234_567_890,
        EventKind::OperationFailed,
        sample_diagnostic(),
    )
    .with_operation_id(OperationId::from_bytes([0xAB; 16]))
    .with_metrics(Metrics {
        objects_processed: 4,
        bytes_processed: 8192,
        retry_count: 1,
        warning_count: 2,
    })
}

// TST-ERR-001: structured errors use the generated Error Registry identity.
#[test]
fn structured_error_uses_registry_identity() {
    let line = sample_event().to_json_line().expect("event must encode");
    let value: Value = serde_json::from_slice(&line).expect("JSON line must parse");
    let diagnostic = &value["diagnostic"];

    assert_eq!(diagnostic["error_id"], 1);
    assert_eq!(diagnostic["error_name"], "ChecksumMismatch");
    assert_eq!(diagnostic["retryability"], "never");
    assert_eq!(diagnostic["recovery_impact"], "blocks_restore");
    assert_eq!(diagnostic["os_error_code"], 23);

    for (error_id, expected_name) in [
        (ErrorId::ChecksumMismatch, "ChecksumMismatch"),
        (ErrorId::FormatVersion, "FormatVersion"),
        (ErrorId::Compression, "Compression"),
        (ErrorId::SegmentCorrupt, "SegmentCorrupt"),
        (ErrorId::IoError, "IoError"),
        (ErrorId::VolumeMissing, "VolumeMissing"),
        (ErrorId::EncryptionAuth, "EncryptionAuth"),
        (ErrorId::CatalogCorrupt, "CatalogCorrupt"),
    ] {
        let diagnostic = Diagnostic::new(
            error_id,
            Severity::Error,
            Stage::Core,
            Retryability::Never,
            RecoveryImpact::None,
        )
        .expect("a non-reserved ErrorId must build");
        let line = LogEvent::from_diagnostic(
            1_721_234_567_890,
            EventKind::OperationFailed,
            diagnostic,
        )
        .to_json_line()
        .expect("event must encode");
        let value: Value = serde_json::from_slice(&line).expect("JSON line must parse");

        assert_eq!(value["diagnostic"]["error_name"], expected_name);
        assert_eq!(error_id.to_string(), expected_name);
    }
}

// TST-ERR-002: the reserved zero ErrorId cannot become a valid diagnostic.
#[test]
fn invalid_error_id_is_rejected() {
    let result = Diagnostic::new(
        ErrorId::Invalid,
        Severity::Error,
        Stage::Core,
        Retryability::Never,
        RecoveryImpact::BlocksCommit,
    );

    assert_eq!(result, Err(DiagnosticBuildError::InvalidErrorId));
}

// TST-ERR-003: the canonical JSON line is deterministic and newline-delimited.
#[test]
fn structured_log_is_deterministic_json_line() {
    let event = sample_event();
    let first = event.to_json_line().expect("first encoding must succeed");
    let second = event.to_json_line().expect("second encoding must succeed");

    assert_eq!(first, second);
    assert_eq!(first.last(), Some(&b'\n'));
    assert_eq!(first.iter().filter(|byte| **byte == b'\n').count(), 1);
    assert!(serde_json::from_slice::<Value>(&first).is_ok());
}

// TST-ERR-004: password/key canaries are redacted from normal formatting.
#[test]
fn secret_canary_is_redacted_from_debug_and_display() {
    let secret = Secret::from_string(SECRET_CANARY.to_owned());

    assert_eq!(secret.len(), SECRET_CANARY.len());
    assert_eq!(secret.expose_secret(), SECRET_CANARY.as_bytes());
    assert_eq!(format!("{secret}"), "[REDACTED]");
    assert_eq!(format!("{secret:?}"), "[REDACTED]");
    assert!(!format!("{secret}{secret:?}").contains(SECRET_CANARY));
}

struct CanaryFailingWriter;

impl Write for CanaryFailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other(SECRET_CANARY))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

// TST-ERR-005: log I/O failures discard secret-bearing source text.
#[test]
fn secret_canary_writer_error_is_not_exposed() {
    let error = sample_event()
        .write_json_line(&mut CanaryFailingWriter)
        .expect_err("the failing writer must return an error");

    assert_eq!(error.error_kind(), Some(io::ErrorKind::Other));
    let diagnostic = error
        .diagnostic()
        .expect("an I/O failure must include structured context");
    assert_eq!(diagnostic.error_id(), ErrorId::IoError);
    assert_eq!(diagnostic.severity(), Severity::Error);
    assert_eq!(diagnostic.stage(), Stage::Writer);
    assert_eq!(diagnostic.retryability(), Retryability::Transient);
    assert_eq!(diagnostic.recovery_impact(), RecoveryImpact::Degraded);
    assert_eq!(diagnostic.os_error_code(), None);
    assert!(!error.to_string().contains(SECRET_CANARY));
    assert!(!format!("{error:?}").contains(SECRET_CANARY));
}

// TST-ERR-006: the wire schema contains no free-text, path, or payload field.
#[test]
fn structured_log_schema_has_no_free_text_fields() {
    let line = sample_event().to_json_line().expect("event must encode");
    let value: Value = serde_json::from_slice(&line).expect("JSON line must parse");
    let root = value.as_object().expect("root must be an object");
    let diagnostic = root
        .get("diagnostic")
        .expect("diagnostic field must exist")
        .as_object()
        .expect("diagnostic must be an object");

    let root_keys: Vec<_> = root.keys().map(String::as_str).collect();
    assert_eq!(
        root_keys,
        [
            "diagnostic",
            "event",
            "level",
            "metrics",
            "operation_id",
            "schema_version",
            "stage",
            "timestamp_unix_ms",
        ]
    );

    let diagnostic_keys: Vec<_> = diagnostic.keys().map(String::as_str).collect();
    assert_eq!(
        diagnostic_keys,
        [
            "error_id",
            "error_name",
            "os_error_code",
            "recovery_impact",
            "retryability",
            "severity",
            "stage",
        ]
    );

    let encoded = String::from_utf8(line)
        .expect("JSON must be UTF-8")
        .to_ascii_lowercase();
    for forbidden in [
        SECRET_CANARY,
        "password",
        "secret",
        "key_material",
        "path",
        "payload",
        "message",
        "detail",
        "source_error",
    ] {
        assert!(!encoded.contains(&forbidden.to_ascii_lowercase()));
    }
}

// TST-ERR-007: diagnostic events have one severity and stage source.
#[test]
fn diagnostic_event_derives_severity_and_stage() {
    let diagnostic = Diagnostic::new(
        ErrorId::IoError,
        Severity::Critical,
        Stage::Catalog,
        Retryability::AfterUserAction,
        RecoveryImpact::BlocksCommit,
    )
    .expect("a non-reserved ErrorId must build");
    let line = LogEvent::from_diagnostic(
        1_721_234_567_890,
        EventKind::OperationFailed,
        diagnostic,
    )
    .to_json_line()
    .expect("event must encode");
    let value: Value = serde_json::from_slice(&line).expect("JSON line must parse");

    assert_eq!(value["level"], value["diagnostic"]["severity"]);
    assert_eq!(value["stage"], value["diagnostic"]["stage"]);
    assert_eq!(value["level"], "critical");
    assert_eq!(value["stage"], "catalog");
}
