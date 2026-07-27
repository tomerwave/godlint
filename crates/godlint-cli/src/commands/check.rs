use std::{
    fs,
    path::{Component, Path, PathBuf},
    process::ExitCode,
};

use godlint_core::{
    config::{Config, Severity},
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
    let current_directory = match std::env::current_dir() {
        Ok(directory) => directory,
        Err(error) => {
            eprintln!("Unable to determine the scan root: {error}");
            return ExitCode::from(2);
        }
    };
    let requested_paths = match requested_paths(paths, &current_directory) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("Invalid scan path: {error}");
            return ExitCode::from(2);
        }
    };
    let root = match config_root(&requested_paths) {
        Ok(root) => root,
        Err(error) => {
            eprintln!("Unable to determine the configuration root: {error}");
            return ExitCode::from(2);
        }
    };
    let paths = match scan_paths(&requested_paths, &root) {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("Invalid scan path: {error}");
            return ExitCode::from(2);
        }
    };
    let config = match Config::load(root.join("godlint.yaml")) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("Configuration is invalid: {error}");
            return ExitCode::from(2);
        }
    };

    match scan(&root, &paths, &config) {
        Ok(report) if report.findings.is_empty() && report.issues.is_empty() => {
            println!("No findings.");
            ExitCode::SUCCESS
        }
        Ok(report) => {
            let exit_code = scan_exit_code(&report);

            for finding in report.findings {
                println!(
                    "{}:{}:{}: {}[{}] Function has {} effective lines (max {}).",
                    finding.path.display(),
                    finding.line,
                    finding.column,
                    severity_name(finding.severity),
                    finding.rule_id,
                    finding.effective_line_count,
                    finding.max_lines
                );
            }

            for issue in report.issues {
                eprintln!("{}: {}", issue.path.display(), issue.message);
            }

            exit_code
        }
        Err(error) => {
            eprintln!("Unable to scan source files: {error}");
            ExitCode::from(2)
        }
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

fn scan_exit_code(report: &godlint_core::scan::ScanReport) -> ExitCode {
    if !report.issues.is_empty() {
        return ExitCode::from(2);
    }

    ExitCode::from(1)
}

fn severity_name(severity: Severity) -> &'static str {
    match severity {
        Severity::Error => "error",
        Severity::Info => "info",
        Severity::Off => "off",
        Severity::Warning => "warning",
    }
}
