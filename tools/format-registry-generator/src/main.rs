//! Format Registry Generator -- CLI entry point.
//!
//! Commands:
//!   generate    Read TOML, validate, generate Rust, write files
//!   check       Same logic but in-memory, diff against disk files

#![allow(dead_code)]

use format_registry_generator::{generator, parser, CheckError, REGISTRY_MAPPINGS};

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const EXIT_SUCCESS: i32 = 0;
pub(crate) const EXIT_STALE: i32 = 1;
pub(crate) const EXIT_IO_ERROR: i32 = 2;
pub(crate) const EXIT_SCHEMA_ERROR: i32 = 3;
pub(crate) const EXIT_SEMANTIC_ERROR: i32 = 4;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: format-registry-generator <generate|check> [--manifest-path PATH]");
        std::process::exit(EXIT_IO_ERROR);
    }

    let command = &args[1];
    let manifest_dir = resolve_manifest_dir(&args);

    match command.as_str() {
        "generate" => {
            if let Err(code) = run_generate(&manifest_dir) {
                std::process::exit(code);
            }
        }
        "check" => {
            if let Err(code) = run_check(&manifest_dir) {
                std::process::exit(code);
            }
        }
        other => {
            eprintln!("unknown command: {other} (expected generate or check)");
            std::process::exit(EXIT_IO_ERROR);
        }
    }
}

fn resolve_manifest_dir(args: &[String]) -> PathBuf {
    for i in 1..args.len() {
        if args[i] == "--manifest-path" {
            if let Some(val) = args.get(i + 1) {
                let p = PathBuf::from(val);
                if p.is_absolute() {
                    return p;
                }
                return env::current_dir().unwrap_or_default().join(&p);
            }
        }
    }
    // Default: walk up from cwd to find crates/nwb-format
    let cwd = env::current_dir().unwrap_or_default();
    let candidates = [
        cwd.join("crates/nwb-format"),
        cwd.join("../crates/nwb-format"),
        cwd.join("../../crates/nwb-format"),
    ];
    for c in &candidates {
        if c.join("Cargo.toml").exists() {
            return c.clone();
        }
    }
    // Fallback: assume running from workspace root
    cwd.join("crates/nwb-format")
}

// ---------------------------------------------------------------------------
// Generate command
// ---------------------------------------------------------------------------

fn run_generate(manifest_dir: &Path) -> Result<(), i32> {
    let crate_root = manifest_dir;

    for &(toml_rel, rs_rel) in REGISTRY_MAPPINGS {
        let toml_path = crate_root.join(toml_rel);
        let rs_path = crate_root.join(rs_rel);

        let toml_str = fs::read_to_string(&toml_path).map_err(|e| {
            eprintln!("Error reading {}: {e}", toml_path.display());
            EXIT_IO_ERROR
        })?;

        let registry = parser::parse_and_validate(&toml_str).map_err(|e| {
            eprintln!("Error in {}: {e}", toml_path.display());
            match &e {
                parser::ParseError::SemanticValidation(_) => EXIT_SEMANTIC_ERROR,
                _ => EXIT_SCHEMA_ERROR,
            }
        })?;

        let generated = generator::generate(&registry);

        // Ensure parent directory exists
        if let Some(parent) = rs_path.parent() {
            fs::create_dir_all(parent).ok();
        }

        fs::write(&rs_path, &generated).map_err(|e| {
            eprintln!("Error writing {}: {e}", rs_path.display());
            EXIT_IO_ERROR
        })?;

        eprintln!("Generated {}", rs_path.display());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Check command
// ---------------------------------------------------------------------------

fn run_check(manifest_dir: &Path) -> Result<(), i32> {
    let mut any_stale = false;
    let mut any_missing = false;
    let crate_root = manifest_dir;

    for &(toml_rel, rs_rel) in REGISTRY_MAPPINGS {
        let toml_path = crate_root.join(toml_rel);
        let rs_path = crate_root.join(rs_rel);

        match format_registry_generator::check_registry_entry(&toml_path, &rs_path) {
            Ok(()) => {
                eprintln!("OK: {}", rs_path.display());
            }
            Err(CheckError::Stale { missing, message }) => {
                eprintln!("{message}");
                if missing {
                    any_missing = true;
                } else {
                    any_stale = true;
                }
            }
            Err(CheckError::Io(msg)) => {
                eprintln!("{msg}");
                return Err(EXIT_IO_ERROR);
            }
            Err(CheckError::Schema(msg)) => {
                eprintln!("{msg}");
                return Err(EXIT_SCHEMA_ERROR);
            }
            Err(CheckError::Semantic(msg)) => {
                eprintln!("{msg}");
                return Err(EXIT_SEMANTIC_ERROR);
            }
        }
    }

    if any_missing {
        eprintln!("Some generated files are missing. Run `generate` to update.");
        Err(EXIT_STALE)
    } else if any_stale {
        eprintln!("Some generated files are stale. Run `generate` to update.");
        Err(EXIT_STALE)
    } else {
        eprintln!("All generated files are up to date.");
        Ok(())
    }
}
