use std::process::ExitCode;

use godlint_core::VERSION;

mod commands;

fn main() -> ExitCode {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();

    match arguments.as_slice() {
        [] => {
            println!("{}", commands::usage());
            ExitCode::SUCCESS
        }
        [argument] if matches!(argument.as_str(), "--help" | "-h") => {
            println!("{}", commands::usage());
            ExitCode::SUCCESS
        }
        [argument] if matches!(argument.as_str(), "--version" | "-V") => {
            println!("godlint {VERSION}");
            ExitCode::SUCCESS
        }
        _ => commands::run(&arguments).unwrap_or_else(|| unknown_command(&arguments)),
    }
}

fn unknown_command(arguments: &[String]) -> ExitCode {
    eprintln!(
        "Unknown command or arguments: {}\n\n{}",
        arguments.join(" "),
        commands::usage()
    );

    ExitCode::from(2)
}
