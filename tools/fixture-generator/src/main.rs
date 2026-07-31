use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use fixture_generator::{generate, verify, DEFAULT_SEED};

fn main() -> ExitCode {
    match run() {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("fixture-generator: {message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<String, String> {
    let mut arguments = env::args().skip(1);
    let command = arguments.next().ok_or_else(usage)?;
    let root = PathBuf::from(arguments.next().ok_or_else(usage)?);
    if arguments.next().is_some() {
        return Err(usage());
    }
    match command.as_str() {
        "generate" => generate(&root, DEFAULT_SEED)
            .map(|manifest| format!("generated fixture {}", manifest.root_sha256))
            .map_err(|error| error.to_string()),
        "verify" => verify(&root)
            .map(|manifest| format!("verified fixture {}", manifest.root_sha256))
            .map_err(|error| error.to_string()),
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: fixture-generator <generate|verify> <directory>".to_owned()
}
