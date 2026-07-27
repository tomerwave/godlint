use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use godlint_core::{VERSION, config::Config, discovery::discover};

const USAGE: &str = "Usage:\n  godlint check [paths...]\n  godlint config validate [--config <path>]\n  godlint [--help] [--version]\n\nGodlint is a deterministic code-policy engine for polyglot repositories.";

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
        [command, paths @ ..] if command == "check" => check(paths),
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

fn check(paths: &[String]) -> ExitCode {
    let paths = if paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        paths.iter().map(PathBuf::from).collect()
    };

    match discover(&paths) {
        Ok(files) if files.is_empty() => {
            println!("No supported source files found.");
            ExitCode::SUCCESS
        }
        Ok(files) => {
            println!("Discovered {} supported source files:", files.len());

            for path in files {
                println!("{}", path.display());
            }

            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("Unable to discover source files: {error}");
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
