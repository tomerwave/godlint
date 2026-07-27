use std::{path::Path, process::ExitCode};

use godlint_core::config::Config;

pub const USAGE: &str = "config validate [--config <path>]";

pub fn run(arguments: &[String]) -> Option<ExitCode> {
    match arguments {
        [command, subcommand] if command == "config" && subcommand == "validate" => {
            Some(validate(Path::new("godlint.yaml")))
        }
        [command, subcommand, option, path]
            if command == "config" && subcommand == "validate" && option == "--config" =>
        {
            Some(validate(Path::new(path)))
        }
        _ => None,
    }
}

fn validate(path: &Path) -> ExitCode {
    match Config::load(path) {
        Ok(_) => {
            println!("Configuration is valid: {}", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Configuration is invalid: {error}");
            ExitCode::from(2)
        }
    }
}
