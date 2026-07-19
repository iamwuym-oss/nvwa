use std::env;
use std::path::PathBuf;

use traceability_checker::{check_workspace, generate_matrix, CheckError};

const EXIT_SUCCESS: i32 = 0;
const EXIT_USAGE: i32 = 2;
const EXIT_OPERATIONAL: i32 = 3;
const EXIT_SEMANTIC: i32 = 4;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let (command, explicit_root) = match parse_args(&args) {
        Ok(value) => value,
        Err(message) => {
            eprintln!("{message}");
            print_usage();
            std::process::exit(EXIT_USAGE);
        }
    };
    let root = match explicit_root {
        Some(path) => path,
        None => match find_workspace_root() {
            Ok(path) => path,
            Err(message) => {
                eprintln!("{message}");
                std::process::exit(EXIT_OPERATIONAL);
            }
        },
    };

    let result = match command.as_str() {
        "check" => check_workspace(&root).map(|report| {
            eprintln!(
                "Traceability check passed: {} requirements, {} tests, {} mappings ({} P0).",
                report.requirements, report.tests, report.mappings, report.p0_requirements
            );
        }),
        "generate" => generate_matrix(&root).map(|path| {
            eprintln!("Generated {}", path.display());
        }),
        _ => unreachable!("command was validated by parse_args"),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(match error {
            CheckError::Io(_) | CheckError::Schema(_) => EXIT_OPERATIONAL,
            CheckError::Semantic(_) => EXIT_SEMANTIC,
        });
    }

    std::process::exit(EXIT_SUCCESS);
}

fn parse_args(args: &[String]) -> Result<(String, Option<PathBuf>), String> {
    let Some(command) = args.first() else {
        return Err("missing command".to_owned());
    };
    if command != "check" && command != "generate" {
        return Err(format!("unknown command: {command}"));
    }

    let mut root = None;
    let mut index = 1;
    while index < args.len() {
        if args[index] != "--root" {
            return Err(format!("unknown argument: {}", args[index]));
        }
        let Some(value) = args.get(index + 1) else {
            return Err("--root requires a path".to_owned());
        };
        if root.is_some() {
            return Err("--root may only be provided once".to_owned());
        }
        root = Some(PathBuf::from(value));
        index += 2;
    }

    Ok((command.clone(), root))
}

fn find_workspace_root() -> Result<PathBuf, String> {
    let mut candidate = env::current_dir().map_err(|error| error.to_string())?;
    loop {
        if candidate.join("Cargo.toml").is_file()
            && candidate
                .join("docs/storageEngine/Nuwa_NWB_Engineering_Document_Set_v1.0")
                .is_dir()
        {
            return Ok(candidate);
        }
        if !candidate.pop() {
            return Err("could not locate the NWB workspace root; pass --root PATH".to_owned());
        }
    }
}

fn print_usage() {
    eprintln!("Usage: traceability-checker <check|generate> [--root PATH]");
}
