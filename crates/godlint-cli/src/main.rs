use std::process::ExitCode;

use godlint_core::VERSION;

const USAGE: &str = "Usage: godlint [--help] [--version]\n\nGodlint is a deterministic code-policy engine for polyglot repositories.";

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
