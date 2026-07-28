use std::process::ExitCode;

pub mod check;
pub mod config;
pub mod suppressions;

struct Command {
    usage: &'static str,
    run: fn(&[String]) -> Option<ExitCode>,
}

const COMMANDS: [Command; 3] = [
    Command {
        usage: check::USAGE,
        run: check::run,
    },
    Command {
        usage: config::USAGE,
        run: config::run,
    },
    Command {
        usage: suppressions::USAGE,
        run: suppressions::run,
    },
];

pub fn run(arguments: &[String]) -> Option<ExitCode> {
    COMMANDS.iter().find_map(|command| (command.run)(arguments))
}

pub fn usage() -> String {
    let commands = COMMANDS
        .iter()
        .map(|command| format!("  godlint {}", command.usage))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Usage:\n{commands}\n  godlint [--help] [--version]\n\nGodlint is a deterministic code-policy engine for polyglot repositories."
    )
}
