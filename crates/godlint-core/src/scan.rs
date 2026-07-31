use std::{
    error::Error,
    fmt, fs,
    io::Read,
    path::{Path, PathBuf},
};

use crate::{
    analyzers::{SourceFacts, analyze, workflow},
    discovery::{DiscoveryError, Scope, discover},
    source::{SourceFile, SourceRange, TextFile, Workflow},
};

pub const MAX_SOURCE_BYTES: u64 = 4 * 1024 * 1024;
const UNREADABLE: &str = "unable to discover";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanIssue {
    pub path: PathBuf,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanReport {
    pub facts: Vec<SourceFacts>,
    pub workflows: Vec<workflow::WorkflowFacts>,
    pub issues: Vec<ScanIssue>,
}

#[derive(Debug)]
pub enum ScanError {
    DiscoversFiles { source: DiscoveryError },
    SourcePath { path: PathBuf },
}

pub fn scan(root: &Path, paths: &[PathBuf], excludes: &[String]) -> Result<ScanReport, ScanError> {
    let scope = Scope { root, excludes };
    let discovered =
        discover(paths, &scope).map_err(|source| ScanError::DiscoversFiles { source })?;
    let mut report = ScanReport {
        facts: Vec::new(),
        workflows: Vec::new(),
        issues: discovery_issues(root, discovered.failures),
    };

    for path in discovered.files {
        scan_file(root, &path, &mut report)?;
    }

    report
        .issues
        .sort_by(|left, right| (&left.path, &left.message).cmp(&(&right.path, &right.message)));

    Ok(report)
}

fn discovery_issues(root: &Path, failures: Vec<DiscoveryError>) -> Vec<ScanIssue> {
    failures
        .into_iter()
        .map(|failure| ScanIssue {
            path: relative_to(root, failure.path()),
            message: format!("{UNREADABLE}: {}", failure.reason()),
        })
        .collect()
}

fn relative_to(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

fn scan_file(root: &Path, path: &Path, report: &mut ScanReport) -> Result<(), ScanError> {
    let relative_path = path
        .strip_prefix(root)
        .map_err(|_| ScanError::SourcePath {
            path: path.to_path_buf(),
        })?
        .to_path_buf();

    if Workflow::names(path) {
        match read_workflow(relative_path.clone(), path) {
            Ok(facts) => {
                report
                    .issues
                    .extend(torn_issue(&relative_path, facts.file(), facts.unparsed()));
                report.workflows.push(facts);
            }
            Err(message) => report.issues.push(ScanIssue {
                path: relative_path,
                message,
            }),
        }

        return Ok(());
    }

    match read_facts(relative_path.clone(), path) {
        Ok(facts) => {
            report.issues.extend(unparsed_issue(&relative_path, &facts));
            report.facts.push(facts);
        }
        Err(message) => report.issues.push(ScanIssue {
            path: relative_path,
            message,
        }),
    }

    Ok(())
}

fn unparsed_issue(relative_path: &Path, facts: &SourceFacts) -> Option<ScanIssue> {
    torn_issue(relative_path, facts.source().text_file(), facts.unparsed())
}

fn torn_issue(
    relative_path: &Path,
    file: &TextFile,
    unparsed: &[SourceRange],
) -> Option<ScanIssue> {
    let first = unparsed.first()?;
    let line = file.location(*first).start.line;
    let rest = unparsed.len() - 1;
    let also = match rest {
        0 => String::new(),
        1 => ", and 1 more place".to_owned(),
        more => format!(", and {more} more places"),
    };

    Some(ScanIssue {
        path: relative_path.to_path_buf(),
        message: format!(
            "syntax not recognised at line {line}{also}; everything that did parse was still checked"
        ),
    })
}

fn read_facts(relative_path: PathBuf, path: &Path) -> Result<SourceFacts, String> {
    let contents = read_bounded(path)?;
    let source = SourceFile::new(relative_path, contents)
        .map_err(|error| format!("invalid source: {error}"))?;

    analyze(&source).map_err(|error| error.to_string())
}

fn read_workflow(relative_path: PathBuf, path: &Path) -> Result<workflow::WorkflowFacts, String> {
    let contents = read_bounded(path)?;
    let file = TextFile::new(relative_path, contents)
        .map_err(|error| format!("invalid workflow: {error}"))?;

    workflow::read(&file).map_err(|error| error.to_string())
}

fn read_bounded(path: &Path) -> Result<String, String> {
    let file = fs::File::open(path).map_err(|error| format!("unable to read: {error}"))?;
    let mut bytes = Vec::new();

    file.take(MAX_SOURCE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("unable to read: {error}"))?;

    if bytes.len() as u64 > MAX_SOURCE_BYTES {
        return Err(format!(
            "file is larger than the {MAX_SOURCE_BYTES} byte scan limit"
        ));
    }

    String::from_utf8(bytes).map_err(|_| "unable to read: not valid UTF-8".to_owned())
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
