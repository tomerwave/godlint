use std::{error::Error, fmt};

use crate::source::{SourceFile, SourceFileError, SourceRange};

macro_rules! function_metrics {
    ($($(#[$documentation:meta])* $name:ident),+ $(,)?) => {
        $(
            $(#[$documentation])*
            #[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
            pub struct $name(u32);

            impl $name {
                pub const fn new(value: u32) -> Self {
                    Self(value)
                }

                pub const fn value(self) -> u32 {
                    self.0
                }
            }

            impl fmt::Display for $name {
                fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(formatter, "{}", self.0)
                }
            }
        )+
    };
}

function_metrics! {
    ParameterCount,
    DecisionPoints,
    ReturnPaths,
    StatementCount,
    BlockDepth,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentKind {
    Line,
    Block,
    Doc,
    Docstring,
    Shebang,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CommentFact {
    source: SourceFile,
    range: SourceRange,
    kind: CommentKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionFact {
    source: SourceFile,
    name: Option<String>,
    range: SourceRange,
    body_range: SourceRange,
    parameter_count: ParameterCount,
    decision_points: DecisionPoints,
    return_paths: ReturnPaths,
    statement_count: StatementCount,
    block_depth: BlockDepth,
    body_is_empty: bool,
    is_abstract: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionFactDetails {
    pub range: SourceRange,
    pub body_range: SourceRange,
    pub parameter_count: ParameterCount,
    pub decision_points: DecisionPoints,
    pub return_paths: ReturnPaths,
    pub statement_count: StatementCount,
    pub block_depth: BlockDepth,
    pub body_is_empty: bool,
    pub is_abstract: bool,
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
    pub fn new(
        source: SourceFile,
        range: SourceRange,
        kind: CommentKind,
    ) -> Result<Self, CommentFactError> {
        source
            .validate_range(range)
            .map_err(|source| CommentFactError::InvalidCommentRange { source })?;

        Ok(Self {
            source,
            range,
            kind,
        })
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn kind(&self) -> CommentKind {
        self.kind
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
            return_paths: details.return_paths,
            statement_count: details.statement_count,
            block_depth: details.block_depth,
            body_is_empty: details.body_is_empty,
            is_abstract: details.is_abstract,
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

    pub fn parameter_count(&self) -> ParameterCount {
        self.parameter_count
    }

    pub fn decision_points(&self) -> DecisionPoints {
        self.decision_points
    }

    pub fn return_paths(&self) -> ReturnPaths {
        self.return_paths
    }

    pub fn statement_count(&self) -> StatementCount {
        self.statement_count
    }

    pub fn block_depth(&self) -> BlockDepth {
        self.block_depth
    }

    pub fn body_is_empty(&self) -> bool {
        self.body_is_empty
    }

    pub fn is_abstract(&self) -> bool {
        self.is_abstract
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
