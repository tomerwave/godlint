use std::{path::PathBuf, process::ExitCode};

use godlint_core::discovery::discover;

pub const USAGE: &str = "check [paths...]";

pub fn run(arguments: &[String]) -> Option<ExitCode> {
    let [command, paths @ ..] = arguments else {
        return None;
    };

    if command != "check" {
        return None;
    }

    Some(check(paths))
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
