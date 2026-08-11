use std::path::{Path, PathBuf};

use godlint_core::{
    config::Config,
    paths,
    repository::RepositoryFacts,
    scan::{ScanReport, scan},
    source::TextFile,
};

use crate::report;

#[path = "config.rs"]
mod branch;

const CONFIG_NAME: &str = "godlint.yaml";

pub struct Workspace {
    pub config: Config,
    pub repository: RepositoryFacts,
    root: PathBuf,
    scan_paths: Vec<PathBuf>,
}

impl Workspace {
    pub fn prepare(paths: &[String]) -> Result<Self, String> {
        let located = Located::new(paths)?;
        let path = located.root.join(CONFIG_NAME);
        let config =
            Config::load(&path).map_err(|error| format!("Configuration is invalid: {error}"))?;

        for notice in report::notices(&path, &config) {
            eprintln!("{notice}");
        }

        let excludes = config.excludes();

        Ok(Self {
            config,
            repository: repository(&located.root, &excludes)?,
            root: located.root,
            scan_paths: located.scan_paths,
        })
    }

    pub fn scan(&self) -> Result<ScanReport, String> {
        scan(&self.root, &self.scan_paths, &self.config.excludes())
            .map_err(|error| format!("Unable to scan source files: {error}"))
    }
}

struct Located {
    root: PathBuf,
    scan_paths: Vec<PathBuf>,
}

impl Located {
    fn new(paths: &[String]) -> Result<Self, String> {
        let current_directory = std::env::current_dir()
            .map_err(|error| format!("Unable to determine the scan root: {error}"))?;
        let requested = requested_paths(paths, &current_directory)?;
        let root = config_root(&requested)?;

        Ok(Self {
            scan_paths: scan_paths(&requested, &root)?,
            root,
        })
    }
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

fn config_root(requested: &[PathBuf]) -> Result<PathBuf, String> {
    let path = requested
        .first()
        .ok_or_else(|| "Invalid scan path: no scan paths were provided".to_owned())?;
    let directory = if path.is_file() {
        path.parent().unwrap_or(path)
    } else {
        path.as_path()
    };

    paths::find_upward(directory, CONFIG_NAME).ok_or_else(|| {
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

fn repository(root: &Path, excludes: &[String]) -> Result<RepositoryFacts, String> {
    let branch = branch::resolve(root)
        .map(|name| {
            TextFile::new(PathBuf::from("<branch>"), name)
                .map_err(|error| format!("Unable to capture the repository branch: {error}"))
        })
        .transpose()?;

    let drifts = RepositoryFacts::read_version_drifts(root)
        .into_iter()
        .filter(|fact| !excluded(excludes, fact.file().path_text()))
        .collect();

    Ok(RepositoryFacts::new(branch).with_version_drifts(drifts))
}

fn excluded(excludes: &[String], path: &str) -> bool {
    excludes.iter().any(|pattern| {
        godlint_core::glob::matches(pattern, path)
            || path
                .strip_prefix(pattern)
                .is_some_and(|rest| rest.starts_with('/'))
    })
}
