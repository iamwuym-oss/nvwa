//! Format Registry Generator — library API.
//!
//! Parses, validates, and generates Rust source from schema v1 TOML files.
//! Publicly exposed so integration tests can exercise the parser, validators,
//! and generators with deliberately invalid inputs.

use std::fs;
use std::path::Path;

pub mod generator;
pub mod parser;
pub mod schema;

/// Mapping from input TOML to output Rust file, relative to the nwb-format crate root.
///
/// This is the single authoritative mapping used by:
/// - `check_registry()` / `check_registry_entry()` (library check API)
/// - `run_check()` in the CLI (`format-registry-generator check`)
/// - `run_generate()` in the CLI (`format-registry-generator generate`)
pub const REGISTRY_MAPPINGS: &[(&str, &str)] = &[
    ("registry/record_types.toml", "src/registry/record_type.rs"),
    ("registry/feature_bits.toml", "src/registry/feature_bit.rs"),
    ("registry/header_enums.toml", "src/registry/header_enums.rs"),
    ("registry/error_ids.toml", "src/registry/error_id.rs"),
];

// ---------------------------------------------------------------------------
// CheckError — structured error classification for check operations
// ---------------------------------------------------------------------------

/// Classified error returned by [`check_registry_entry`].
///
/// Each variant maps to a distinct CLI exit code:
///
/// | Variant      | Exit code | Meaning                          |
/// |--------------|-----------|----------------------------------|
/// | `Stale`      | 1         | Generated file missing or stale |
/// | `Io`         | 2         | I/O error reading a file        |
/// | `Schema`     | 3         | TOML or schema-level error      |
/// | `Semantic`   | 4         | Semantic validation failure     |
#[derive(Debug)]
pub enum CheckError {
    /// Generated `.rs` file is missing or its content does not match the TOML source.
    ///
    /// `missing`: `true` if the file does not exist, `false` if content differs.
    Stale {
        /// `true` if the generated file does not exist; `false` if content mismatch.
        missing: bool,
        /// Human-readable diagnostic message.
        message: String,
    },
    /// I/O error (file read failure, etc.).
    Io(String),
    /// TOML schema or parse error (invalid syntax, missing required field, etc.).
    Schema(String),
    /// Semantic validation error (duplicate discriminant, duplicate bit, etc.).
    Semantic(String),
}

impl std::fmt::Display for CheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckError::Stale { message, .. } => write!(f, "{message}"),
            CheckError::Io(msg) => write!(f, "{msg}"),
            CheckError::Schema(msg) => write!(f, "{msg}"),
            CheckError::Semantic(msg) => write!(f, "{msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Single-entry check primitive
// ---------------------------------------------------------------------------

/// Check that a single generated Rust file matches its TOML source byte-for-byte.
///
/// This is the **single authoritative file-level check primitive**. Both
/// [`check_registry`] and the CLI `check` command reuse this function.
///
/// # Parameters
///
/// * `toml_path` — Path to the schema v1 TOML source file.
/// * `rs_path`   — Path to the expected generated Rust file.
///
/// # Returns
///
/// * `Ok(())` if the generated file is up to date.
/// * `Err(CheckError)` with a classified error on any failure.
pub fn check_registry_entry(toml_path: &Path, rs_path: &Path) -> Result<(), CheckError> {
    // 1. Check TOML exists
    if !toml_path.exists() {
        return Err(CheckError::Io(format!(
            "Missing TOML file: {}",
            toml_path.display()
        )));
    }

    // 2. Read TOML source
    let toml_str = fs::read_to_string(toml_path)
        .map_err(|e| CheckError::Io(format!("Error reading {}: {e}", toml_path.display())))?;

    // 3. Parse and validate
    let registry = parser::parse_and_validate(&toml_str).map_err(|e| match e {
        parser::ParseError::SemanticValidation(msg) => CheckError::Semantic(msg),
        parser::ParseError::Io(msg) => CheckError::Io(msg),
        other => CheckError::Schema(other.to_string()),
    })?;

    // 4. Generate expected Rust source in memory
    let generated = generator::generate(&registry);

    // 5. Check that the generated file exists
    if !rs_path.exists() {
        return Err(CheckError::Stale {
            missing: true,
            message: format!(
                "Missing generated file: {} (run generate first)",
                rs_path.display()
            ),
        });
    }

    // 6. Read existing generated file
    let current = fs::read_to_string(rs_path)
        .map_err(|e| CheckError::Io(format!("Error reading {}: {e}", rs_path.display())))?;

    // 7. Compare content
    if current != generated {
        return Err(CheckError::Stale {
            missing: false,
            message: format!(
                "STALE: {} does not match generated output",
                rs_path.display()
            ),
        });
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Aggregate check (backward-compatible API)
// ---------------------------------------------------------------------------

/// Check that all generated Rust files match the current TOML sources byte-for-byte.
///
/// `crate_root` is the directory containing `nwb-format/Cargo.toml`.
///
/// Returns `Ok(())` if every generated file is up to date.
/// Returns `Err` with one or more diagnostic messages describing stale,
/// missing, or invalid files.
///
/// # Backward compatibility
///
/// This function maintains the same signature (`pub fn check_registry(crate_root: &Path) -> Result<(), String>`)
/// and the same aggregate semantics. Internally it delegates to [`check_registry_entry`].
pub fn check_registry(crate_root: &Path) -> Result<(), String> {
    let mut diagnostics: Vec<String> = Vec::new();
    let mut any_stale = false;
    let mut any_missing = false;

    for &(toml_rel, rs_rel) in REGISTRY_MAPPINGS {
        let toml_path = crate_root.join(toml_rel);
        let rs_path = crate_root.join(rs_rel);

        match check_registry_entry(&toml_path, &rs_path) {
            Ok(()) => {}
            Err(CheckError::Stale { missing, message }) => {
                diagnostics.push(message);
                if missing {
                    any_missing = true;
                } else {
                    any_stale = true;
                }
            }
            // Non-stale errors: return immediately (preserves current early-return behavior)
            Err(e) => return Err(e.to_string()),
        }
    }

    if any_missing {
        diagnostics.push("Some generated files are missing. Run `generate` to update.".into());
    }
    if any_stale {
        diagnostics.push("Some generated files are stale. Run `generate` to update.".into());
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(diagnostics.join("\n"))
    }
}
