use std::{error::Error, fmt};

use crate::source::{SourceFile, SourceFileError, SourceRange};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionFact {
    source: SourceFile,
    name: Option<String>,
    range: SourceRange,
    body_range: SourceRange,
    nesting_depth: u32,
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

impl FunctionFact {
    pub fn new(
        source: SourceFile,
        name: Option<String>,
        range: SourceRange,
        body_range: SourceRange,
        nesting_depth: u32,
    ) -> Result<Self, FunctionFactError> {
        source
            .location(range)
            .map_err(|source| FunctionFactError::InvalidFunctionRange { source })?;
        source
            .location(body_range)
            .map_err(|source| FunctionFactError::InvalidBodyRange { source })?;

        if !range_contains(range, body_range) {
            return Err(FunctionFactError::BodyOutsideFunction {
                function_range: range,
                body_range,
            });
        }

        Ok(Self {
            source,
            name,
            range,
            body_range,
            nesting_depth,
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
