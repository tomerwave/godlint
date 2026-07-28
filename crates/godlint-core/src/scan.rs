use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::{
    analyzers::{SourceFacts, analyze},
    discovery::{DiscoveryError, discover},
    source::{SourceFile, SourceFileError},
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
    DiscoversFiles {
        source: DiscoveryError,
    },
    ReadsSource {
        path: PathBuf,
        source: std::io::Error,
    },
    SourcePath {
        path: PathBuf,
    },
    CreatesSource {
        source: SourceFileError,
    },
}

pub fn scan(root: &Path, paths: &[PathBuf]) -> Result<ScanReport, ScanError> {
    let files = discover(paths).map_err(|source| ScanError::DiscoversFiles { source })?;
    let mut facts = Vec::new();
    let mut issues = Vec::new();

    for path in files {
        scan_file(root, &path, &mut facts, &mut issues)?;
    }

    issues.sort_by(|left, right| (&left.path, &left.message).cmp(&(&right.path, &right.message)));

    Ok(ScanReport { facts, issues })
}

fn scan_file(
    root: &Path,
    path: &Path,
    facts: &mut Vec<SourceFacts>,
    issues: &mut Vec<ScanIssue>,
) -> Result<(), ScanError> {
    let relative_path = path
        .strip_prefix(root)
        .map_err(|_| ScanError::SourcePath {
            path: path.to_path_buf(),
        })?
        .to_path_buf();
    let source = fs::read_to_string(path).map_err(|source| ScanError::ReadsSource {
        path: path.to_path_buf(),
        source,
    })?;
    let source = SourceFile::new(relative_path, source)
        .map_err(|source| ScanError::CreatesSource { source })?;
    let source_facts = match analyze(&source) {
        Ok(facts) => facts,
        Err(error) => {
            issues.push(ScanIssue {
                path: source.path().to_path_buf(),
                message: error.to_string(),
            });

            return Ok(());
        }
    };

    facts.push(source_facts);

    Ok(())
}

impl fmt::Display for ScanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DiscoversFiles { source } => {
                write!(formatter, "unable to discover files: {source}")
            }
            Self::ReadsSource { path, source } => {
                write!(formatter, "unable to read {}: {source}", path.display())
            }
            Self::SourcePath { path } => {
                write!(
                    formatter,
                    "source file is outside scan root: {}",
                    path.display()
                )
            }
            Self::CreatesSource { source } => write!(formatter, "invalid source file: {source}"),
        }
    }
}

impl Error for ScanError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DiscoversFiles { source } => Some(source),
            Self::ReadsSource { source, .. } => Some(source),
            Self::CreatesSource { source } => Some(source),
            Self::SourcePath { .. } => None,
        }
    }
}
