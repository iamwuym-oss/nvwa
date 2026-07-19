//! CLI exit-code integration tests for format-registry-generator.
//!
//! These tests spawn the compiled binary as a child process and verify that
//! each known exit code is produced by the correct real-world condition.
//!
//! Exit-code contract:
//!   0 = SUCCESS   (check reports all files up to date)
//!   1 = STALE     (generated file content differs from current TOML)
//!   2 = IO_ERROR  (missing required input file or I/O failure)
//!   3 = SCHEMA_ERROR (invalid or missing schema-level fields)
//!   4 = SEMANTIC_ERROR (valid schema but semantic rule violation)

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

// ---------------------------------------------------------------------------
// Minimal temp-directory helper (no external crate)
// ---------------------------------------------------------------------------

static DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let count = DIR_COUNTER.fetch_add(1, Ordering::SeqCst);
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = format!("nwb-test-cli-{unique}-{count}");
        let path = std::env::temp_dir().join(&dir);
        std::fs::create_dir_all(&path)
            .unwrap_or_else(|e| panic!("cannot create test dir {path:?}: {e}"));
        TestDir { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).ok();
    }
}

// ---------------------------------------------------------------------------
// Generator binary path resolution
// ---------------------------------------------------------------------------

fn generator_exe() -> PathBuf {
    // Try CARGO_BIN_EXE first (set by Cargo for integration tests)
    if let Ok(p) = std::env::var("CARGO_BIN_EXE_format-registry-generator") {
        return PathBuf::from(p);
    }
    // Fallback: locate next to the test binary
    let mut path = std::env::current_exe().expect("current test binary path");
    path.pop(); // deps/   or   debug/
    if path.ends_with("deps") {
        path.pop(); // debug/
    }
    let exe_name = if cfg!(windows) {
        "format-registry-generator.exe"
    } else {
        "format-registry-generator"
    };
    path.push(exe_name);
    if !path.exists() {
        panic!(
            "generator binary not found at {}. Build the binary with `cargo build -p format-registry-generator` first.",
            path.display()
        );
    }
    path
}

// ---------------------------------------------------------------------------
// Test data -- minimal valid TOML for each registry file
// ---------------------------------------------------------------------------

const VALID_RECORD_TYPES: &str = r#"
[meta]
schema_version = 1
kind = "discriminant"
enum_name = "RecordType"

[entries.TEST_REC]
rust_name = "TestRec"
value = 0x0001
description = "Test record type"
"#;

const VALID_FEATURE_BITS: &str = r#"
[meta]
schema_version = 1
kind = "bit"

[entries.TEST_BIT]
rust_name = "TestBit"
bit = 0
description = "Test feature bit"
"#;

const VALID_HEADER_ENUMS: &str = r#"
[meta]
schema_version = 1
kind = "enum_set"

[enums.TestKind]
rust_name = "TestKind"
repr = "u8"

[enums.TestKind.entries.A]
rust_name = "A"
value = 1
description = "Variant A"
"#;

const VALID_ERROR_IDS: &str = r#"
[meta]
schema_version = 1
kind = "discriminant"
enum_name = "ErrorId"

[entries.TEST_ERR]
rust_name = "TestErr"
value = 0x0001
description = "Test error id"
"#;

const DUPLICATE_RECORD_TYPES: &str = r#"
[meta]
schema_version = 1
kind = "discriminant"
enum_name = "RecordType"

[entries.FIRST]
rust_name = "First"
value = 0x0001
description = "First record type"

[entries.SECOND]
rust_name = "Second"
value = 0x0001
description = "Duplicate record type value"
"#;

const DUPLICATE_ERROR_IDS: &str = r#"
[meta]
schema_version = 1
kind = "discriminant"
enum_name = "ErrorId"

[entries.FIRST]
rust_name = "First"
value = 0x0001
description = "First error id"

[entries.SECOND]
rust_name = "Second"
value = 0x0001
description = "Duplicate error id value"
"#;

const DUPLICATE_FEATURE_BITS: &str = r#"
[meta]
schema_version = 1
kind = "bit"

[entries.FIRST]
rust_name = "FIRST"
bit = 1
description = "First feature bit"

[entries.SECOND]
rust_name = "SECOND"
bit = 1
description = "Duplicate feature bit"
"#;

const OUT_OF_RANGE_FEATURE_BIT: &str = r#"
[meta]
schema_version = 1
kind = "bit"

[entries.TOO_HIGH]
rust_name = "TOO_HIGH"
bit = 64
description = "Feature bit outside the 64-bit mask"
"#;

const DUPLICATE_HEADER_ENUM_VALUES: &str = r#"
[meta]
schema_version = 1
kind = "enum_set"

[enums.TestKind]
rust_name = "TestKind"
repr = "u8"

[enums.TestKind.entries.A]
rust_name = "A"
value = 1
description = "First variant"

[enums.TestKind.entries.B]
rust_name = "B"
value = 1
description = "Duplicate enum value"
"#;

/// Write all four valid registry TOML files into `base/registry/` and
/// ensure `base/src/registry/` exists for generated output.
fn write_valid_tomls(base: &Path) {
    std::fs::create_dir_all(base.join("registry")).unwrap();
    std::fs::create_dir_all(base.join("src/registry")).unwrap();

    std::fs::write(base.join("registry/record_types.toml"), VALID_RECORD_TYPES).unwrap();
    std::fs::write(base.join("registry/feature_bits.toml"), VALID_FEATURE_BITS).unwrap();
    std::fs::write(base.join("registry/header_enums.toml"), VALID_HEADER_ENUMS).unwrap();
    std::fs::write(base.join("registry/error_ids.toml"), VALID_ERROR_IDS).unwrap();
}

fn assert_semantic_exit_4(command: &str, bad_file: &str, bad_toml: &str, diagnostic: &str) {
    let exe = generator_exe();
    let dir = TestDir::new();
    write_valid_tomls(dir.path());
    std::fs::write(dir.path().join("registry").join(bad_file), bad_toml).unwrap();

    let output = Command::new(&exe)
        .arg(command)
        .arg("--manifest-path")
        .arg(dir.path())
        .output()
        .unwrap_or_else(|e| panic!("failed to spawn generator {command}: {e}"));

    let code = output.status.code();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        code,
        Some(4),
        "{command} must classify {bad_file} as SEMANTIC_ERROR; stderr: {stderr}"
    );
    assert!(
        stderr.contains(diagnostic),
        "{command} diagnostic for {bad_file} must contain {diagnostic:?}; stderr: {stderr}"
    );
}

// =========================================================================
// Test: exit code 0 -- check passes
// =========================================================================

#[test]
fn cli_exit_0_success() {
    let exe = generator_exe();
    let dir = TestDir::new();
    let manifest = dir.path().to_path_buf();

    // Write valid TOML files first
    write_valid_tomls(&manifest);

    // 1. Generate output files
    let gen_output = Command::new(&exe)
        .arg("generate")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .expect("failed to spawn generator generate");

    assert!(
        gen_output.status.success(),
        "generate should exit 0, got: {:?}\\nstderr: {}",
        gen_output.status.code(),
        String::from_utf8_lossy(&gen_output.stderr)
    );

    // 2. Check that generated files are up to date
    let check_output = Command::new(&exe)
        .arg("check")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .expect("failed to spawn generator check");

    let code = check_output.status.code();
    let stderr = String::from_utf8_lossy(&check_output.stderr);
    assert_eq!(
        code,
        Some(0),
        "check should exit 0, got: {:?}\\nstderr: {stderr}",
        code,
    );
    assert!(
        stderr.contains("All generated files are up to date"),
        "stderr should report all up to date, got: {stderr}"
    );
}

// =========================================================================
// Test: exit code 1 -- STALE
// =========================================================================

#[test]
fn cli_exit_1_stale() {
    let exe = generator_exe();
    let dir = TestDir::new();
    let manifest = dir.path().to_path_buf();

    write_valid_tomls(&manifest);

    // Generate first so that .rs files exist
    let gen = Command::new(&exe)
        .arg("generate")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .expect("generate");

    assert!(
        gen.status.success(),
        "generate: {:?}\\nstderr: {}",
        gen.status.code(),
        String::from_utf8_lossy(&gen.stderr)
    );

    // Tamper with one generated file
    let rs_path = manifest.join("src/registry/record_type.rs");
    std::fs::write(&rs_path, "// tampered\\n").unwrap();

    // Now check -- should detect staleness
    let check = Command::new(&exe)
        .arg("check")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .expect("check");

    let code = check.status.code();
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert_eq!(
        code,
        Some(1),
        "stale should exit 1, got: {:?}\\nstderr: {stderr}",
        code
    );
    assert!(
        stderr.contains("STALE") || stderr.contains("stale"),
        "stderr should mention STALE, got: {stderr}"
    );
}

// =========================================================================
// Test: exit code 2 -- IO_ERROR (missing required file)
// =========================================================================

#[test]
fn cli_exit_2_io_error() {
    let exe = generator_exe();
    let dir = TestDir::new();
    // Point to an existing directory that has NO TOML files at all
    let manifest = dir.path().to_path_buf();
    // Create registry/ (empty, no TOML files)
    std::fs::create_dir_all(manifest.join("registry")).unwrap();

    let check = Command::new(&exe)
        .arg("check")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .expect("check");

    let code = check.status.code();
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert_eq!(
        code,
        Some(2),
        "missing TOML should exit 2 (IO_ERROR), got: {:?}\\nstderr: {stderr}",
        code
    );
    assert!(
        stderr.contains("Missing TOML"),
        "stderr should say 'Missing TOML', got: {stderr}"
    );
}

// =========================================================================
// Test: exit code 3 -- SCHEMA_ERROR (e.g. missing schema_version)
// =========================================================================

#[test]
fn cli_exit_3_schema_error() {
    let exe = generator_exe();
    let dir = TestDir::new();
    let manifest = dir.path().to_path_buf();

    // Write all 4 TOML files, but record_types.toml has no schema_version
    std::fs::create_dir_all(manifest.join("registry")).unwrap();
    std::fs::create_dir_all(manifest.join("src/registry")).unwrap();

    // Bad TOML: missing schema_version
    let bad_toml = r#"
[meta]
kind = "discriminant"
enum_name = "RecordType"

[entries.FOO]
rust_name = "Foo"
value = 0x0001
description = "foo"
"#;
    std::fs::write(manifest.join("registry/record_types.toml"), bad_toml).unwrap();
    // Other TOML files are valid
    std::fs::write(
        manifest.join("registry/feature_bits.toml"),
        VALID_FEATURE_BITS,
    )
    .unwrap();
    std::fs::write(
        manifest.join("registry/header_enums.toml"),
        VALID_HEADER_ENUMS,
    )
    .unwrap();
    std::fs::write(manifest.join("registry/error_ids.toml"), VALID_ERROR_IDS).unwrap();

    let check = Command::new(&exe)
        .arg("check")
        .arg("--manifest-path")
        .arg(&manifest)
        .output()
        .expect("check");

    let code = check.status.code();
    let stderr = String::from_utf8_lossy(&check.stderr);
    assert_eq!(
        code,
        Some(3),
        "schema error should exit 3, got: {:?}\\nstderr: {stderr}",
        code
    );
    assert!(
        stderr.contains("schema_version") || stderr.contains("unsupported"),
        "stderr should mention schema_version, got: {stderr}"
    );
}

// =========================================================================
// Test: exit code 4 -- SEMANTIC_ERROR for every registry kind
// =========================================================================

#[test]
fn cli_exit_4_semantic_errors_match_for_check_and_generate() {
    let cases = [
        (
            "record_types.toml",
            DUPLICATE_RECORD_TYPES,
            "duplicate value",
        ),
        ("error_ids.toml", DUPLICATE_ERROR_IDS, "duplicate value"),
        (
            "feature_bits.toml",
            DUPLICATE_FEATURE_BITS,
            "duplicate value",
        ),
        (
            "feature_bits.toml",
            OUT_OF_RANGE_FEATURE_BIT,
            "bit 64 out of range",
        ),
        (
            "header_enums.toml",
            DUPLICATE_HEADER_ENUM_VALUES,
            "duplicate value",
        ),
    ];

    for command in ["check", "generate"] {
        for (bad_file, bad_toml, diagnostic) in cases {
            assert_semantic_exit_4(command, bad_file, bad_toml, diagnostic);
        }
    }
}

// =========================================================================
// Negative: invalid command string should also produce IO_ERROR (exit 2)
// =========================================================================

#[test]
fn cli_exit_2_invalid_command() {
    let exe = generator_exe();
    let output = Command::new(&exe)
        .arg("bogus")
        .output()
        .expect("generator bogus command");

    let code = output.status.code();
    assert_eq!(
        code,
        Some(2),
        "bogus command should exit 2 (IO_ERROR), got: {:?}",
        code
    );
}
