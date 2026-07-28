use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::ExitCode,
};

use godlint_core::{
    config::{Config, Severity},
    rules::evaluate,
    scan::scan,
};

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

/// Reports findings for `paths`, returning the process exit code.
///
/// Every setup failure is returned as the operator-facing message to print, so the
/// caller owns both the reporting and the failure exit code.
fn run_check(paths: &[String]) -> Result<ExitCode, String> {
    let current_directory = std::env::current_dir()
        .map_err(|error| format!("Unable to determine the scan root: {error}"))?;
    let requested_paths = requested_paths(paths, &current_directory)
        .map_err(|error| format!("Invalid scan path: {error}"))?;
    let root = config_root(&requested_paths)
        .map_err(|error| format!("Unable to determine the configuration root: {error}"))?;
    let paths = scan_paths(&requested_paths, &root)
        .map_err(|error| format!("Invalid scan path: {error}"))?;
    let config = Config::load(root.join("godlint.yaml"))
        .map_err(|error| format!("Configuration is invalid: {error}"))?;
    let report =
        scan(&root, &paths).map_err(|error| format!("Unable to scan source files: {error}"))?;
    let findings = evaluate(&report.facts, &config)
        .map_err(|error| format!("Unable to evaluate rules: {error}"))?;

    if findings.is_empty() && report.issues.is_empty() {
        println!("No findings.");
        return Ok(ExitCode::SUCCESS);
    }

    for finding in findings {
        println!(
            "{}:{}:{}: {}[{}] {}",
            finding.path.display(),
            finding.line,
            finding.column,
            severity_name(finding.severity),
            finding.rule_id,
            finding.message
        );
    }

    for issue in &report.issues {
        eprintln!("{}: {}", issue.path.display(), issue.message);
    }

    if report.issues.is_empty() {
        Ok(ExitCode::from(1))
    } else {
        Ok(ExitCode::from(2))
    }
}

fn requested_paths(arguments: &[String], current_directory: &Path) -> Result<Vec<PathBuf>, String> {
    if arguments.is_empty() {
        return Ok(vec![current_directory.to_path_buf()]);
    }

    arguments
        .iter()
        .map(PathBuf::from)
        .map(|path| requested_path(current_directory, path))
        .collect()
}

fn requested_path(current_directory: &Path, path: PathBuf) -> Result<PathBuf, String> {
    let path = if path.is_absolute() {
        path
    } else {
        current_directory.join(path)
    };

    normalize_path(path)
}

fn normalize_path(path: PathBuf) -> Result<PathBuf, String> {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(component) => normalized.push(component),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(format!("{} escapes the filesystem root", path.display()));
                }
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
        }
    }

    Ok(normalized)
}

fn config_root(paths: &[PathBuf]) -> Result<PathBuf, String> {
    let path = paths
        .first()
        .ok_or_else(|| "no scan paths were provided".to_owned())?;
    let directory = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path.as_path()
    };

    Ok(directory
        .ancestors()
        .find(|directory| directory.join("godlint.yaml").is_file())
        .unwrap_or(directory)
        .to_path_buf())
}

fn scan_paths(paths: &[PathBuf], root: &Path) -> Result<Vec<PathBuf>, String> {
    if fs::symlink_metadata(root)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(format!("{} is a symbolic link", root.display()));
    }

    paths
        .iter()
        .cloned()
        .map(|path| scan_path(root, path))
        .collect()
}

fn scan_path(root: &Path, path: PathBuf) -> Result<PathBuf, String> {
    let path = if path.is_absolute() {
        path
    } else {
        root.join(path)
    };
    let relative = path
        .strip_prefix(root)
        .map_err(|_| format!("{} is outside {}", path.display(), root.display()))?;

    if relative
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(format!("{} escapes {}", path.display(), root.display()));
    }

    let mut current = root.to_path_buf();

    for component in relative.components() {
        if let Component::Normal(name) = component {
            current.push(name);

            if fs::symlink_metadata(&current)
                .map(|metadata| metadata.file_type().is_symlink())
                .unwrap_or(false)
            {
                return Err(format!("{} contains a symbolic link", path.display()));
            }
        }
    }

    Ok(path)
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Info => "info",
        Severity::Off => "off",
        Severity::Warning => "warning",
    }
}
