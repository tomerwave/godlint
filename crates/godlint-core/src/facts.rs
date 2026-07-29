use std::{error::Error, fmt};

use crate::source::{SourceFile, SourceRange};

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
pub struct CallFact {
    source: SourceFile,
    range: SourceRange,
    is_macro: bool,
    argument_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessFact {
    source: SourceFile,
    range: SourceRange,
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
    BodyOutsideFunction {
        function_range: SourceRange,
        body_range: SourceRange,
    },
}

impl CommentFact {
    pub fn new(source: SourceFile, range: SourceRange, kind: CommentKind) -> Self {
        Self {
            source,
            range,
            kind,
        }
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

impl CallFact {
    pub fn new(
        source: SourceFile,
        range: SourceRange,
        is_macro: bool,
        argument_count: usize,
    ) -> Self {
        Self {
            source,
            range,
            is_macro,
            argument_count,
        }
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn callee(&self) -> &str {
        &self.source.source()[self.range.start()..self.range.end()]
    }

    pub fn is_macro(&self) -> bool {
        self.is_macro
    }

    pub const fn argument_count(&self) -> usize {
        self.argument_count
    }
}

impl AccessFact {
    pub fn new(source: SourceFile, range: SourceRange) -> Self {
        Self { source, range }
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn target(&self) -> &str {
        &self.source.source()[self.range.start()..self.range.end()]
    }
}

impl FunctionFact {
    pub fn new(
        source: SourceFile,
        name: Option<String>,
        details: FunctionFactDetails,
    ) -> Result<Self, FunctionFactError> {
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

impl Error for FunctionFactError {}
