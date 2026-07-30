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
    CognitiveScore,
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
    extent: SourceRange,
    target: CallTarget,
    arguments: Vec<CallArgument>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TestFocus {
    Ordinary,
    Only,
    Skipped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestFactDetails {
    pub name: Option<String>,
    pub marker: String,
    pub focus: TestFocus,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TestFact {
    source: SourceFile,
    range: SourceRange,
    details: TestFactDetails,
}

impl TestFact {
    pub fn new(source: SourceFile, range: SourceRange, details: TestFactDetails) -> Self {
        Self {
            source,
            range,
            details,
        }
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn name(&self) -> Option<&str> {
        self.details.name.as_deref()
    }

    pub fn marker(&self) -> &str {
        &self.details.marker
    }

    pub fn focus(&self) -> TestFocus {
        self.details.focus
    }

    pub fn contains(&self, range: SourceRange) -> bool {
        self.range.start() <= range.start() && range.end() <= self.range.end()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssertionFactDetails {
    pub target: CallTarget,
    pub operands: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssertionFact {
    source: SourceFile,
    range: SourceRange,
    details: AssertionFactDetails,
}

impl AssertionFact {
    pub fn new(source: SourceFile, range: SourceRange, details: AssertionFactDetails) -> Self {
        Self {
            source,
            range,
            details,
        }
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn name(&self) -> &str {
        &self.details.target.path
    }

    pub fn is_macro(&self) -> bool {
        self.details.target.is_macro
    }

    pub fn operands(&self) -> usize {
        self.details.operands
    }

    pub fn text(&self) -> &str {
        &self.source.source()[self.range.start()..self.range.end()]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallArgument {
    pub name: Option<String>,
    pub literal: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallFactDetails {
    pub target: CallTarget,
    pub arguments: Vec<CallArgument>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallTarget {
    pub path: String,
    pub is_macro: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportFact {
    source: SourceFile,
    range: SourceRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessFact {
    source: SourceFile,
    range: SourceRange,
    target: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorHandlerFact {
    source: SourceFile,
    range: SourceRange,
    body_is_empty: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConditionFact {
    source: SourceFile,
    range: SourceRange,
    operator_count: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionFact {
    source: SourceFile,
    name: Option<String>,
    range: SourceRange,
    body_range: SourceRange,
    parameter_count: ParameterCount,
    decision_points: DecisionPoints,
    cognitive_score: CognitiveScore,
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
    pub cognitive_score: CognitiveScore,
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
        extent: SourceRange,
        details: CallFactDetails,
    ) -> Self {
        Self {
            source,
            range,
            extent,
            target: details.target,
            arguments: details.arguments,
        }
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn extent(&self) -> SourceRange {
        self.extent
    }

    pub fn callee(&self) -> &str {
        &self.target.path
    }

    pub fn is_macro(&self) -> bool {
        self.target.is_macro
    }

    pub fn argument_count(&self) -> usize {
        self.arguments.len()
    }

    pub fn arguments(&self) -> &[CallArgument] {
        &self.arguments
    }

    pub fn positional(&self, index: usize) -> Option<&CallArgument> {
        self.arguments
            .iter()
            .filter(|argument| argument.name.is_none())
            .nth(index)
    }

    pub fn positional_literal(&self, index: usize) -> Option<&str> {
        self.positional(index)
            .and_then(|argument| argument.literal.as_deref())
    }

    pub fn named(&self, name: &str) -> Option<&CallArgument> {
        self.arguments
            .iter()
            .find(|argument| argument.name.as_deref() == Some(name))
    }
}

impl ImportFact {
    pub fn new(source: SourceFile, range: SourceRange) -> Self {
        Self { source, range }
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn module(&self) -> &str {
        &self.source.source()[self.range.start()..self.range.end()]
    }
}

impl AccessFact {
    pub fn new(source: SourceFile, range: SourceRange, target: String) -> Self {
        Self {
            source,
            range,
            target,
        }
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn target(&self) -> &str {
        &self.target
    }
}

impl ErrorHandlerFact {
    pub fn new(source: SourceFile, range: SourceRange, body_is_empty: bool) -> Self {
        Self {
            source,
            range,
            body_is_empty,
        }
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn body_is_empty(&self) -> bool {
        self.body_is_empty
    }
}

impl ConditionFact {
    pub fn new(source: SourceFile, range: SourceRange, operator_count: u32) -> Self {
        Self {
            source,
            range,
            operator_count,
        }
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn operator_count(&self) -> u32 {
        self.operator_count
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
            cognitive_score: details.cognitive_score,
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

    pub fn cognitive_score(&self) -> CognitiveScore {
        self.cognitive_score
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
