use std::process::ExitCode;

use godlint_core::{
    config::Severity,
    date::Date,
    rules::{Finding, evaluate},
    scan::ScanReport,
};

use crate::{
    report::{self, FORMATS, Format},
    workspace::Workspace,
};

pub const USAGE: &str = "check [--format <format>] [paths...]";

pub fn run(arguments: &[String]) -> Option<ExitCode> {
    let [command, paths @ ..] = arguments else {
        return None;
    };

    if command != "check" {
        return None;
    }

    Some(check(paths))
}

fn check(arguments: &[String]) -> ExitCode {
    match run_check(arguments) {
        Ok(exit_code) => exit_code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn run_check(arguments: &[String]) -> Result<ExitCode, String> {
    let (format, paths) = requested(arguments)?;
    let workspace = Workspace::prepare(&paths)?;
    let report = workspace.scan()?;
    let today = Date::today().ok_or_else(|| "Unable to determine the current date.".to_owned())?;
    let findings = evaluate(&report.facts, &workspace.config, today);

    Ok(report_outcome(
        format,
        &findings,
        &report,
        workspace.config.fail_on,
    ))
}

fn requested(arguments: &[String]) -> Result<(Format, Vec<String>), String> {
    let mut format = Format::Terminal;
    let mut paths = Vec::new();
    let mut remaining = arguments.iter();

    while let Some(argument) = remaining.next() {
        match named_format(argument, &mut remaining)? {
            Some(requested) => format = requested,
            None => paths.push(argument.clone()),
        }
    }

    Ok((format, paths))
}

fn named_format<'a>(
    argument: &str,
    remaining: &mut impl Iterator<Item = &'a String>,
) -> Result<Option<Format>, String> {
    let name = match argument.strip_prefix("--format") {
        Some("") => remaining.next().map(String::as_str),
        Some(rest) => rest.strip_prefix('='),
        None => return Ok(None),
    };

    match name.map(Format::parse) {
        Some(Some(format)) => Ok(Some(format)),
        Some(None) => Err(format!(
            "Unknown format: {}\n\nAvailable formats are {FORMATS}.",
            name.unwrap_or_default()
        )),
        None => Err(format!("--format needs a format. Available are {FORMATS}.")),
    }
}

fn report_outcome(
    format: Format,
    findings: &[Finding],
    report: &ScanReport,
    fail_on: Severity,
) -> ExitCode {
    print_report(format, findings, report);
    print_issues(report);

    if !report.issues.is_empty() {
        return ExitCode::from(2);
    }

    if fails(findings, fail_on) {
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}

fn print_report(format: Format, findings: &[Finding], report: &ScanReport) {
    let body = report::render(format, findings, report);

    if !body.is_empty() {
        println!("{body}");
    } else if !format.is_machine_readable() && report.issues.is_empty() {
        println!("No findings.");
    }
}

fn print_issues(report: &ScanReport) {
    for issue in &report.issues {
        eprintln!("{}: {}", issue.path.display(), issue.message);
    }
}

fn fails(findings: &[Finding], fail_on: Severity) -> bool {
    fail_on != Severity::Off && findings.iter().any(|finding| finding.severity >= fail_on)
}
