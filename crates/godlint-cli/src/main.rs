use std::{path::Path, process::ExitCode};

use godlint_core::{VERSION, config::Config};

const USAGE: &str = "Usage:\n  godlint config validate [--config <path>]\n  godlint [--help] [--version]\n\nGodlint is a deterministic code-policy engine for polyglot repositories.";

fn main() -> ExitCode {
    match std::env::args().skip(1).collect::<Vec<_>>().as_slice() {
        [] => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        [argument] if matches!(argument.as_str(), "--help" | "-h") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        [argument] if matches!(argument.as_str(), "--version" | "-V") => {
            println!("godlint {VERSION}");
            ExitCode::SUCCESS
        }
        [command, subcommand] if command == "config" && subcommand == "validate" => {
            validate_config(Path::new("godlint.yaml"))
        }
        [command, subcommand, option, path]
            if command == "config" && subcommand == "validate" && option == "--config" =>
        {
            validate_config(Path::new(path))
        }
        [argument] => {
            eprintln!("Unknown argument: {argument}\n\n{USAGE}");
            ExitCode::from(2)
        }
        _ => {
            eprintln!("Expected at most one argument.\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn validate_config(path: &Path) -> ExitCode {
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
