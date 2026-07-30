use std::process::ExitCode;

use godlint_core::suppression::{Suppression, collect};

use crate::{report::visible, workspace::Workspace};

pub const USAGE: &str = "suppressions [paths...]";

pub fn run(arguments: &[String]) -> Option<ExitCode> {
    let [command, paths @ ..] = arguments else {
        return None;
    };

    if command != "suppressions" {
        return None;
    }

    Some(match list(paths) {
        Ok(exit_code) => exit_code,
        Err(message) => {
            eprintln!("{}", visible(&message));
            ExitCode::from(2)
        }
    })
}

fn list(paths: &[String]) -> Result<ExitCode, String> {
    let workspace = Workspace::prepare(paths)?;
    let report = workspace.scan()?;
    let suppressions = collect(&report.facts);

    if suppressions.is_empty() {
        println!("No suppressions.");

        return Ok(ExitCode::SUCCESS);
    }

    for suppression in &suppressions {
        println!("{}", visible(&describe(suppression)));
    }

    println!("\n{} suppressions.", suppressions.len());

    Ok(ExitCode::SUCCESS)
}

fn describe(suppression: &Suppression) -> String {
    let mut fields = vec![
        format!(
            "{}:{}",
            suppression.source().path().display(),
            suppression.line()
        ),
        suppression.scope().directive().to_owned(),
        suppression.rules().join(","),
    ];

    if let Some(owner) = suppression.owner() {
        fields.push(format!("owner={owner}"));
    }

    if let Some(expires) = suppression.expires() {
        fields.push(format!("expires={expires}"));
    }

    fields.push(format!(
        "-- {}",
        suppression.justification().unwrap_or("(no justification)")
    ));

    fields.join(" ")
}
