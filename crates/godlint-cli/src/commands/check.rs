use std::process::ExitCode;

use godlint_core::{
    config::Severity,
    date::Date,
    rules::{Finding, evaluate},
    scan::ScanReport,
};

use crate::workspace::Workspace;

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
    match run_check(paths) {
        Ok(exit_code) => exit_code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn run_check(paths: &[String]) -> Result<ExitCode, String> {
    let workspace = Workspace::prepare(paths)?;
    let report = workspace.scan()?;
    let today = Date::today().ok_or_else(|| "Unable to determine the current date.".to_owned())?;
    let findings = evaluate(&report.facts, &workspace.config, today);

    Ok(report_outcome(&findings, &report, workspace.config.fail_on))
}

fn report_outcome(findings: &[Finding], report: &ScanReport, fail_on: Severity) -> ExitCode {
    if findings.is_empty() && report.issues.is_empty() {
        println!("No findings.");

        return ExitCode::SUCCESS;
    }

    print_findings(findings);
    print_issues(report);

    if !report.issues.is_empty() {
        return ExitCode::from(2);
    }

    if fails(findings, fail_on) {
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}

fn print_findings(findings: &[Finding]) {
    for finding in findings {
        println!(
            "{}:{}:{}: {}[{}] {}",
            finding.path.display(),
            finding.line,
            finding.column,
            severity_name(finding.severity),
            finding.rule_id,
            finding.message()
        );
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

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Info => "info",
        Severity::Off => "off",
        Severity::Warning => "warning",
    }
}
