use std::{error::Error, fmt};

use crate::source::{SourceFile, SourceFileError, SourceRange};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommentFact {
    source: SourceFile,
    range: SourceRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionFact {
    source: SourceFile,
    name: Option<String>,
    range: SourceRange,
    body_range: SourceRange,
    parameter_count: u32,
    decision_points: u32,
    body_is_empty: bool,
    nesting_depth: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionFactDetails {
    pub range: SourceRange,
    pub body_range: SourceRange,
    pub parameter_count: u32,
    pub decision_points: u32,
    pub body_is_empty: bool,
    pub nesting_depth: u32,
}

#[derive(Debug)]
pub enum FunctionFactError {
    InvalidFunctionRange {
        source: SourceFileError,
    },
    InvalidBodyRange {
        source: SourceFileError,
    },
    BodyOutsideFunction {
        function_range: SourceRange,
        body_range: SourceRange,
    },
}

#[derive(Debug)]
pub enum CommentFactError {
    InvalidCommentRange { source: SourceFileError },
}

impl CommentFact {
    pub fn new(source: SourceFile, range: SourceRange) -> Result<Self, CommentFactError> {
        source
            .validate_range(range)
            .map_err(|source| CommentFactError::InvalidCommentRange { source })?;

        Ok(Self { source, range })
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn text(&self) -> &str {
        &self.source.source()[self.range.start()..self.range.end()]
    }
}

impl FunctionFact {
    pub fn new(
        source: SourceFile,
        name: Option<String>,
        details: FunctionFactDetails,
    ) -> Result<Self, FunctionFactError> {
        source
            .validate_range(details.range)
            .map_err(|source| FunctionFactError::InvalidFunctionRange { source })?;
        source
            .validate_range(details.body_range)
            .map_err(|source| FunctionFactError::InvalidBodyRange { source })?;

        if !range_contains(details.range, details.body_range) {
            return Err(FunctionFactError::BodyOutsideFunction {
                function_range: details.range,
                body_range: details.body_range,
            });
        }

        Ok(Self {
            source,
            name,
            range: details.range,
            body_range: details.body_range,
            parameter_count: details.parameter_count,
            decision_points: details.decision_points,
            body_is_empty: details.body_is_empty,
            nesting_depth: details.nesting_depth,
        })
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn body_range(&self) -> SourceRange {
        self.body_range
    }

    pub fn parameter_count(&self) -> u32 {
        self.parameter_count
    }

    pub fn decision_points(&self) -> u32 {
        self.decision_points
    }

    pub fn body_is_empty(&self) -> bool {
        self.body_is_empty
    }

    pub fn nesting_depth(&self) -> u32 {
        self.nesting_depth
    }
}

fn range_contains(container: SourceRange, candidate: SourceRange) -> bool {
    container.start() <= candidate.start() && candidate.end() <= container.end()
}

impl fmt::Display for FunctionFactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFunctionRange { source } => {
                write!(formatter, "function range is invalid: {source}")
            }
            Self::InvalidBodyRange { source } => {
                write!(formatter, "function body range is invalid: {source}")
            }
            Self::BodyOutsideFunction {
                function_range,
                body_range,
            } => write!(
                formatter,
                "function body {}..{} is outside function {}..{}",
                body_range.start(),
                body_range.end(),
                function_range.start(),
                function_range.end()
            ),
        }
    }
}

impl Error for FunctionFactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidFunctionRange { source } | Self::InvalidBodyRange { source } => {
                Some(source)
            }
            Self::BodyOutsideFunction { .. } => None,
        }
    }
}

impl fmt::Display for CommentFactError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidCommentRange { source } => {
                write!(formatter, "comment range is invalid: {source}")
            }
        }
    }
}

impl Error for CommentFactError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidCommentRange { source } => Some(source),
        }
    }
}
