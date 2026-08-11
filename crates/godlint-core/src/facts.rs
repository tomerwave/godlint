use std::{error::Error, fmt};

use crate::source::{SourceFile, SourceRange, range_contains};

pub mod workflow;

pub use workflow::{
    ActionFact, BooleanFact, CredentialFact, ExpressionFact, JobFact, Secrets, Setting, StepFact,
};

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
    rethrows_only: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FinallyFact {
    source: SourceFile,
    range: SourceRange,
    has_control_flow: bool,
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
    details: FunctionFactDetails,
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
        let module = &self.source.source()[self.range.start()..self.range.end()];
        if matches!(self.source.language(), crate::source::Language::Go) {
            module.trim_matches('"')
        } else {
            module
        }
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
        Self::with_rethrows(source, range, body_is_empty, false)
    }

    pub fn with_rethrows(
        source: SourceFile,
        range: SourceRange,
        body_is_empty: bool,
        rethrows_only: bool,
    ) -> Self {
        Self {
            source,
            range,
            body_is_empty,
            rethrows_only,
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

    pub fn rethrows_only(&self) -> bool {
        self.rethrows_only
    }
}

impl FinallyFact {
    pub fn new(source: SourceFile, range: SourceRange, has_control_flow: bool) -> Self {
        Self {
            source,
            range,
            has_control_flow,
        }
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }
    pub fn range(&self) -> SourceRange {
        self.range
    }
    pub fn has_control_flow(&self) -> bool {
        self.has_control_flow
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
            details,
        })
    }

    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn range(&self) -> SourceRange {
        self.details.range
    }

    pub fn body_range(&self) -> SourceRange {
        self.details.body_range
    }

    pub fn parameter_count(&self) -> ParameterCount {
        self.details.parameter_count
    }

    pub fn decision_points(&self) -> DecisionPoints {
        self.details.decision_points
    }

    pub fn cognitive_score(&self) -> CognitiveScore {
        self.details.cognitive_score
    }

    pub fn return_paths(&self) -> ReturnPaths {
        self.details.return_paths
    }

    pub fn statement_count(&self) -> StatementCount {
        self.details.statement_count
    }

    pub fn block_depth(&self) -> BlockDepth {
        self.details.block_depth
    }

    pub fn body_is_empty(&self) -> bool {
        self.details.body_is_empty
    }

    pub fn is_abstract(&self) -> bool {
        self.details.is_abstract
    }
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
