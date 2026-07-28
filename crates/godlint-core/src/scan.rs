use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::{
    analyzers::extract_functions,
    discovery::{DiscoveryError, discover},
    facts::FunctionFact,
    source::{SourceFile, SourceFileError},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisIssue {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisReport {
    pub functions: Vec<FunctionFact>,
    pub issues: Vec<AnalysisIssue>,
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

/// Discovers source files and converts their syntax into language-neutral facts.
///
/// This layer deliberately does not know which rules are configured or what
/// constitutes a finding. Rules consume the returned facts separately.
pub fn analyze(root: &Path, paths: &[PathBuf]) -> Result<AnalysisReport, ScanError> {
    let files = discover(paths).map_err(|source| ScanError::DiscoversFiles { source })?;
    let mut functions = Vec::new();
    let mut issues = Vec::new();

    for path in files {
        analyze_file(root, &path, &mut functions, &mut issues)?;
    }

    issues.sort_by(|left, right| (&left.path, &left.message).cmp(&(&right.path, &right.message)));

    Ok(AnalysisReport { functions, issues })
}

fn analyze_file(
    root: &Path,
    path: &Path,
    functions: &mut Vec<FunctionFact>,
    issues: &mut Vec<AnalysisIssue>,
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
    let file_functions = match extract_functions(&source) {
        Ok(functions) => functions,
        Err(error) => {
            issues.push(AnalysisIssue {
                path: source.path().to_path_buf(),
                message: error.to_string(),
            });

            return Ok(());
        }
    };

    functions.extend(file_functions);

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
