use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::{
    analyzers::Analyzer,
    config::{Config, Severity},
    discovery::{DiscoveryError, discover},
    rules::{Rule, function_size::FunctionSize},
    source::{SourceFile, SourceFileError},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub rule_id: &'static str,
    pub effective_line_count: usize,
    pub max_lines: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanIssue {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanReport {
    pub findings: Vec<Finding>,
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

pub fn scan(root: &Path, paths: &[PathBuf], config: &Config) -> Result<ScanReport, ScanError> {
    let files = discover(paths).map_err(|source| ScanError::DiscoversFiles { source })?;
    let mut findings = Vec::new();
    let mut issues = Vec::new();

    for path in files {
        scan_file(root, &path, config, &mut findings, &mut issues)?;
    }

    findings.sort_by(|left, right| {
        (
            &left.path,
            left.line,
            left.column,
            left.rule_id,
            left.effective_line_count,
        )
            .cmp(&(
                &right.path,
                right.line,
                right.column,
                right.rule_id,
                right.effective_line_count,
            ))
    });

    issues.sort_by(|left, right| (&left.path, &left.message).cmp(&(&right.path, &right.message)));

    Ok(ScanReport { findings, issues })
}

fn scan_file(
    root: &Path,
    path: &Path,
    config: &Config,
    findings: &mut Vec<Finding>,
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
    let functions = match Analyzer::extract_functions(&source) {
        Ok(functions) => functions,
        Err(error) => {
            issues.push(ScanIssue {
                path: source.path().to_path_buf(),
                message: error.to_string(),
            });

            return Ok(());
        }
    };

    for function in functions {
        add_function_size_finding(&function, config, findings)?;
    }

    Ok(())
}

fn add_function_size_finding(
    function: &crate::facts::FunctionFact,
    config: &Config,
    findings: &mut Vec<Finding>,
) -> Result<(), ScanError> {
    let Some(configuration) = &config.rules.function_size else {
        return Ok(());
    };
    let Some(violation) = FunctionSize::evaluate(function, configuration) else {
        return Ok(());
    };
    let location = function
        .source()
        .location(function.range())
        .map_err(|source| ScanError::CreatesSource { source })?;

    findings.push(Finding {
        path: function.source().path().to_path_buf(),
        line: location.start.line,
        column: location.start.column,
        severity: configuration.severity,
        rule_id: FunctionSize::ID,
        effective_line_count: violation.effective_line_count,
        max_lines: configuration.max_lines,
    });

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
