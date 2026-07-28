use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

use godlint_core::{
    config::{Config, Severity},
    paths,
    rules::{Finding, evaluate},
    scan::{ScanReport, scan},
};

pub const USAGE: &str = "check [paths...]";

/// Name of the configuration file that anchors a scan.
const CONFIG_NAME: &str = "godlint.yaml";

/// Directory entry that marks the top of a repository.
///
/// Configuration discovery stops here so a stray `godlint.yaml` in a parent directory
/// cannot silently govern an unrelated repository, or move the reported path root.
const REPOSITORY_MARKER: &str = ".git";

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

/// Everything resolved before any source is read.
struct Prepared {
    root: PathBuf,
    scan_paths: Vec<PathBuf>,
    config: Config,
}

/// Reports findings for `paths`, returning the process exit code.
///
/// Every setup failure is returned as the operator-facing message to print, so the
/// caller owns both the reporting and the failure exit code.
fn run_check(paths: &[String]) -> Result<ExitCode, String> {
    let prepared = prepare(paths)?;
    let report = scan(
        &prepared.root,
        &prepared.scan_paths,
        &prepared.config.excludes(),
    )
    .map_err(|error| format!("Unable to scan source files: {error}"))?;
    let findings = evaluate(&report.facts, &prepared.config)
        .map_err(|error| format!("Unable to evaluate rules: {error}"))?;

    Ok(report_outcome(&findings, &report, prepared.config.fail_on))
}

/// Resolves the scan root, the paths to walk, and the configuration governing them.
fn prepare(paths: &[String]) -> Result<Prepared, String> {
    let current_directory = std::env::current_dir()
        .map_err(|error| format!("Unable to determine the scan root: {error}"))?;
    let requested_paths = requested_paths(paths, &current_directory)?;
    let root = config_root(&requested_paths)?;
    let scan_paths = scan_paths(&requested_paths, &root)?;
    let config = Config::load(root.join(CONFIG_NAME))
        .map_err(|error| format!("Configuration is invalid: {error}"))?;

    Ok(Prepared {
        root,
        scan_paths,
        config,
    })
}

/// Prints the run's output and decides the exit code.
///
/// Severity governs failure: a finding below `fail_on` is reported without failing the
/// command, which is what makes it possible to adopt a rule as a warning first.
fn report_outcome(findings: &[Finding], report: &ScanReport, fail_on: Severity) -> ExitCode {
    if findings.is_empty() && report.issues.is_empty() {
        println!("No findings.");

        return ExitCode::SUCCESS;
    }

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

    for issue in &report.issues {
        eprintln!("{}: {}", issue.path.display(), issue.message);
    }

    if !report.issues.is_empty() {
        return ExitCode::from(2);
    }

    if fails(findings, fail_on) {
        return ExitCode::from(1);
    }

    ExitCode::SUCCESS
}

fn fails(findings: &[Finding], fail_on: Severity) -> bool {
    fail_on != Severity::Off && findings.iter().any(|finding| finding.severity >= fail_on)
}

fn requested_paths(arguments: &[String], current_directory: &Path) -> Result<Vec<PathBuf>, String> {
    if arguments.is_empty() {
        return Ok(vec![current_directory.to_path_buf()]);
    }

    arguments
        .iter()
        .map(|argument| requested_path(current_directory, Path::new(argument)))
        .collect()
}

fn requested_path(current_directory: &Path, path: &Path) -> Result<PathBuf, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        current_directory.join(path)
    };

    paths::normalize(&absolute).ok_or_else(|| {
        format!(
            "Invalid scan path: {} escapes the filesystem root",
            path.display()
        )
    })
}

/// Finds the directory whose configuration governs this run.
fn config_root(requested: &[PathBuf]) -> Result<PathBuf, String> {
    let path = requested
        .first()
        .ok_or_else(|| "Invalid scan path: no scan paths were provided".to_owned())?;
    let directory = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path.as_path()
    };

    paths::find_upward(directory, CONFIG_NAME, REPOSITORY_MARKER).ok_or_else(|| {
        format!(
            "No {CONFIG_NAME} found in {} or any parent directory within the repository.",
            directory.display()
        )
    })
}

fn scan_paths(requested: &[PathBuf], root: &Path) -> Result<Vec<PathBuf>, String> {
    if paths::is_symlink(root) {
        return Err(format!(
            "Invalid scan path: {} is a symbolic link",
            root.display()
        ));
    }

    requested.iter().map(|path| scan_path(root, path)).collect()
}

fn scan_path(root: &Path, path: &Path) -> Result<PathBuf, String> {
    if path.strip_prefix(root).is_err() {
        return Err(format!(
            "Invalid scan path: {} is outside {}",
            path.display(),
            root.display()
        ));
    }

    if paths::contains_symlink(root, path) {
        return Err(format!(
            "Invalid scan path: {} contains a symbolic link",
            path.display()
        ));
    }

    Ok(path.to_path_buf())
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Info => "info",
        Severity::Off => "off",
        Severity::Warning => "warning",
    }
}
