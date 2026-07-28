use std::{
    error::Error,
    fmt,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    JavaScript,
    Python,
    Rust,
    TypeScript,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFile {
    path: PathBuf,
    language: Language,
    source: Arc<str>,
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
}

#[derive(Debug)]
pub enum SourceRangeError {
    Reversed { start: usize, end: usize },
}

impl Language {
    pub fn from_path(path: &Path) -> Option<Self> {
        match path.extension().and_then(|extension| extension.to_str()) {
            Some("js" | "jsx") => Some(Self::JavaScript),
            Some("py" | "pyi") => Some(Self::Python),
            Some("rs") => Some(Self::Rust),
            Some("ts" | "tsx") => Some(Self::TypeScript),
            _ => None,
        }
    }
}

impl SourceFile {
    pub fn new(path: PathBuf, source: String) -> Result<Self, SourceFileError> {
        validate_path(&path)?;

        let language = Language::from_path(&path)
            .ok_or_else(|| SourceFileError::UnsupportedLanguage { path: path.clone() })?;

        Ok(Self {
            path,
            language,
            source: Arc::from(source),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn location(&self, range: SourceRange) -> Result<SourceLocation, SourceFileError> {
        Ok(SourceLocation {
            start: self.position(range.start)?,
            end: self.position(range.end)?,
        })
    }

    /// Confirms a range addresses real character boundaries without deriving positions.
    ///
    /// Line and column numbers are derived only at reporting boundaries, so callers that
    /// merely need to validate offsets use this instead of discarding a [`SourceLocation`].
    pub(crate) fn validate_range(&self, range: SourceRange) -> Result<(), SourceFileError> {
        self.validate_offset(range.start)?;
        self.validate_offset(range.end)
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

    fn position(&self, offset: usize) -> Result<SourcePosition, SourceFileError> {
        self.validate_offset(offset)?;

        let prefix = &self.source[..offset];
        let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = prefix
            .rsplit('\n')
            .next()
            .map_or(1, |line| line.chars().count() + 1);

        Ok(SourcePosition { line, column })
    }
}

impl SourceRange {
    pub fn new(start: usize, end: usize) -> Result<Self, SourceRangeError> {
        if start > end {
            return Err(SourceRangeError::Reversed { start, end });
        }

        Ok(Self { start, end })
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn end(&self) -> usize {
        self.end
    }
}

fn validate_path(path: &Path) -> Result<(), SourceFileError> {
    if path.is_absolute() {
        return Err(SourceFileError::AbsolutePath {
            path: path.to_path_buf(),
        });
    }

    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
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
        }
    }
}

impl Error for SourceFileError {}

impl fmt::Display for SourceRangeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reversed { start, end } => {
                write!(formatter, "source range start exceeds end: {start}..{end}")
            }
        }
    }
}

impl Error for SourceRangeError {}
