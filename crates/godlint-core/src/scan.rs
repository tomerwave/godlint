use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::{
    analyzers::{SourceFacts, analyze},
    discovery::{DiscoveryError, Scope, discover},
    source::SourceFile,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanIssue {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanReport {
    pub facts: Vec<SourceFacts>,
    pub issues: Vec<ScanIssue>,
}

#[derive(Debug)]
pub enum ScanError {
    DiscoversFiles { source: DiscoveryError },
    SourcePath { path: PathBuf },
}

pub fn scan(root: &Path, paths: &[PathBuf], excludes: &[String]) -> Result<ScanReport, ScanError> {
    let scope = Scope { root, excludes };
    let files = discover(paths, &scope).map_err(|source| ScanError::DiscoversFiles { source })?;
    let mut report = ScanReport {
        facts: Vec::new(),
        issues: Vec::new(),
    };

    for path in files {
        scan_file(root, &path, &mut report)?;
    }

    report
        .issues
        .sort_by(|left, right| (&left.path, &left.message).cmp(&(&right.path, &right.message)));

    Ok(report)
}

fn scan_file(root: &Path, path: &Path, report: &mut ScanReport) -> Result<(), ScanError> {
    let relative_path = path
        .strip_prefix(root)
        .map_err(|_| ScanError::SourcePath {
            path: path.to_path_buf(),
        })?
        .to_path_buf();

    match read_facts(relative_path.clone(), path) {
        Ok(facts) => report.facts.push(facts),
        Err(message) => report.issues.push(ScanIssue {
            path: relative_path,
            message,
        }),
    }

    Ok(())
}

fn read_facts(relative_path: PathBuf, path: &Path) -> Result<SourceFacts, String> {
    let contents = fs::read_to_string(path).map_err(|error| format!("unable to read: {error}"))?;
    let source = SourceFile::new(relative_path, contents)
        .map_err(|error| format!("invalid source: {error}"))?;

    analyze(&source).map_err(|error| error.to_string())
}

impl fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiscoversFiles { source } => {
                write!(formatter, "unable to discover files: {source}")
            }
            Self::SourcePath { path } => {
                write!(
                    formatter,
                    "source file is outside scan root: {}",
                    path.display()
                )
            }
        }
    }
}

impl Error for ScanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DiscoversFiles { source } => Some(source),
            Self::SourcePath { .. } => None,
        }
    }
}
