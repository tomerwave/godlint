use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::paths;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    JavaScript,
    Python,
    Rust,
    TypeScript,
}

pub struct Workflow;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dialect {
    JavaScript,
    Python,
    Rust,
    Workflow,
    Repository,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TextFile {
    path: Arc<Path>,
    path_text: Arc<str>,
    text: Arc<str>,
    line_starts: Arc<[usize]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFile {
    text: TextFile,
    language: Language,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceLocation {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourcePosition {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceRange {
    start: usize,
    end: usize,
}

pub(crate) fn range_contains(container: SourceRange, candidate: SourceRange) -> bool {
    container.start() <= candidate.start() && candidate.end() <= container.end()
}

#[derive(Debug)]
pub enum SourceFileError {
    AbsolutePath { path: PathBuf },
    PathOutsideRepository { path: PathBuf },
    UnsupportedLanguage { path: PathBuf },
    InvalidRange { range: SourceRange },
    InvalidUtf8Boundary { offset: usize },
    ReversedRange { start: usize, end: usize },
}

const BYTE_ORDER_MARK: &str = "\u{feff}";

impl Language {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("cjs" | "js" | "jsx" | "mjs") => Some(Self::JavaScript),
            Some("py" | "pyi") => Some(Self::Python),
            Some("rs") => Some(Self::Rust),
            Some("cts" | "mts" | "ts" | "tsx") => Some(Self::TypeScript),
            _ => None,
        }
    }

    pub fn dialect(self) -> Dialect {
        match self {
            Self::JavaScript | Self::TypeScript => Dialect::JavaScript,
            Self::Python => Dialect::Python,
            Self::Rust => Dialect::Rust,
        }
    }
}

impl Workflow {
    const DIRECTORY: &'static str = ".github/workflows";
    const EXTENSIONS: [&'static str; 2] = ["yaml", "yml"];

    pub fn names(path: &Path) -> bool {
        let text = paths::slashed(&path.to_string_lossy()).into_owned();
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default();

        Self::EXTENSIONS.contains(&extension) && names_a_workflow_directory(&text)
    }
}

fn names_a_workflow_directory(text: &str) -> bool {
    let Some(directory) = text.rsplit_once('/').map(|(directory, _)| directory) else {
        return false;
    };

    directory == Workflow::DIRECTORY || directory.ends_with(&format!("/{}", Workflow::DIRECTORY))
}

impl Dialect {
    pub const EVERY: [Self; 5] = [
        Self::JavaScript,
        Self::Python,
        Self::Rust,
        Self::Workflow,
        Self::Repository,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::JavaScript => "JS/TS",
            Self::Python => "Python",
            Self::Rust => "Rust",
            Self::Workflow => "Workflow",
            Self::Repository => "Repository",
        }
    }
}

impl TextFile {
    pub fn new(path: PathBuf, text: String) -> Result<Self, SourceFileError> {
        validate_path(&path)?;

        let text = text
            .strip_prefix(BYTE_ORDER_MARK)
            .map_or(text.as_str(), |stripped| stripped);

        Ok(Self {
            path_text: Arc::from(paths::slashed(&path.to_string_lossy()).into_owned()),
            path: Arc::from(path),
            line_starts: line_starts(text),
            text: Arc::from(text),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn path_text(&self) -> &str {
        self.path_text.as_ref()
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn full_range(&self) -> SourceRange {
        SourceRange {
            start: 0,
            end: self.text.len(),
        }
    }

    pub fn slice(&self, range: SourceRange) -> &str {
        &self.text[range.start..range.end]
    }

    pub fn range(&self, start: usize, end: usize) -> Result<SourceRange, SourceFileError> {
        if start > end {
            return Err(SourceFileError::ReversedRange { start, end });
        }

        self.validate_offset(start)?;
        self.validate_offset(end)?;

        Ok(SourceRange { start, end })
    }

    pub fn line(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|start| *start <= offset)
    }

    pub fn location(&self, range: SourceRange) -> SourceLocation {
        SourceLocation {
            start: self.position(range.start),
            end: self.position(range.end),
        }
    }

    fn validate_offset(&self, offset: usize) -> Result<(), SourceFileError> {
        if offset > self.text.len() {
            return Err(SourceFileError::InvalidRange {
                range: SourceRange {
                    start: offset,
                    end: offset,
                },
            });
        }

        if !self.text.is_char_boundary(offset) {
            return Err(SourceFileError::InvalidUtf8Boundary { offset });
        }

        Ok(())
    }

    fn position(&self, offset: usize) -> SourcePosition {
        let line_index = self.line_starts.partition_point(|start| *start <= offset) - 1;
        let line_start = self.line_starts.get(line_index).copied().unwrap_or(0);
        let column = self.text[line_start..offset].chars().count() + 1;

        SourcePosition {
            line: line_index + 1,
            column,
        }
    }
}

impl SourceFile {
    pub fn new(path: PathBuf, source: String) -> Result<Self, SourceFileError> {
        let language = Language::from_path(&path)
            .ok_or_else(|| SourceFileError::UnsupportedLanguage { path: path.clone() })?;

        Ok(Self {
            text: TextFile::new(path, source)?,
            language,
        })
    }

    pub fn is_interface_stub(&self) -> bool {
        self.text
            .path()
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension == "pyi")
    }

    pub fn text_file(&self) -> &TextFile {
        &self.text
    }

    pub fn path(&self) -> &Path {
        self.text.path()
    }

    pub fn path_text(&self) -> &str {
        self.text.path_text()
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn source(&self) -> &str {
        self.text.text()
    }

    pub fn full_range(&self) -> SourceRange {
        self.text.full_range()
    }

    pub fn range(&self, start: usize, end: usize) -> Result<SourceRange, SourceFileError> {
        self.text.range(start, end)
    }

    pub fn line(&self, offset: usize) -> usize {
        self.text.line(offset)
    }

    pub fn location(&self, range: SourceRange) -> SourceLocation {
        self.text.location(range)
    }
}

impl SourceRange {
    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

fn line_starts(source: &str) -> Arc<[usize]> {
    let mut starts = vec![0];

    starts.extend(
        source
            .bytes()
            .enumerate()
            .filter(|(_, byte)| *byte == b'\n')
            .map(|(index, _)| index + 1),
    );

    Arc::from(starts)
}

fn validate_path(path: &Path) -> Result<(), SourceFileError> {
    if path.is_absolute() {
        return Err(SourceFileError::AbsolutePath {
            path: path.to_path_buf(),
        });
    }

    if paths::climbs(path) {
        return Err(SourceFileError::PathOutsideRepository {
            path: path.to_path_buf(),
        });
    }

    Ok(())
}

impl fmt::Display for SourceFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AbsolutePath { path } => {
                write!(
                    formatter,
                    "source path must be repository-relative: {}",
                    path.display()
                )
            }
            Self::PathOutsideRepository { path } => {
                write!(
                    formatter,
                    "source path escapes the repository: {}",
                    path.display()
                )
            }
            Self::UnsupportedLanguage { path } => {
                write!(formatter, "unsupported source language: {}", path.display())
            }
            Self::InvalidRange { range } => {
                write!(
                    formatter,
                    "source range is outside the file: {}..{}",
                    range.start(),
                    range.end()
                )
            }
            Self::InvalidUtf8Boundary { offset } => {
                write!(
                    formatter,
                    "source offset is not on a UTF-8 boundary: {offset}"
                )
            }
            Self::ReversedRange { start, end } => {
                write!(formatter, "source range start exceeds end: {start}..{end}")
            }
        }
    }
}

impl Error for SourceFileError {}
