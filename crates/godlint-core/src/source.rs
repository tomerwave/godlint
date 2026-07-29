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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFile {
    path: Arc<Path>,
    path_text: Arc<str>,
    language: Language,
    source: Arc<str>,
    line_starts: Arc<[usize]>,
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
}

impl SourceFile {
    pub fn new(path: PathBuf, source: String) -> Result<Self, SourceFileError> {
        validate_path(&path)?;

        let language = Language::from_path(&path)
            .ok_or_else(|| SourceFileError::UnsupportedLanguage { path: path.clone() })?;
        let source = source
            .strip_prefix(BYTE_ORDER_MARK)
            .map_or(source.as_str(), |stripped| stripped);

        Ok(Self {
            path_text: Arc::from(path.to_string_lossy().into_owned()),
            path: Arc::from(path),
            language,
            line_starts: line_starts(source),
            source: Arc::from(source),
        })
    }

    pub fn is_interface_stub(&self) -> bool {
        self.path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension == "pyi")
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn path_text(&self) -> &str {
        self.path_text.as_ref()
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn full_range(&self) -> SourceRange {
        SourceRange {
            start: 0,
            end: self.source.len(),
        }
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
        if offset > self.source.len() {
            return Err(SourceFileError::InvalidRange {
                range: SourceRange {
                    start: offset,
                    end: offset,
                },
            });
        }

        if !self.source.is_char_boundary(offset) {
            return Err(SourceFileError::InvalidUtf8Boundary { offset });
        }

        Ok(())
    }

    fn position(&self, offset: usize) -> SourcePosition {
        let line_index = self.line_starts.partition_point(|start| *start <= offset) - 1;
        let line_start = self.line_starts.get(line_index).copied().unwrap_or(0);
        let column = self.source[line_start..offset].chars().count() + 1;

        SourcePosition {
            line: line_index + 1,
            column,
        }
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
