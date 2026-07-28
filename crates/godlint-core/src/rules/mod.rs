use std::{error::Error, fmt, path::PathBuf};

use crate::{
    analyzers::SourceFacts,
    config::{Config, Severity},
    facts::{CommentFact, FunctionFact},
    source::{SourceFile, SourceFileError, SourceRange},
};

pub mod cyclomatic_complexity;
pub mod empty_function;
pub mod file_size;
pub mod function_nesting;
pub mod function_size;
pub mod function_statements;
mod line_count;
pub mod parameter_count;
pub mod return_count;
pub mod todo_requires_reference;

/// What a rule found, kept as data rather than as a prepared sentence.
///
/// Reporters other than the terminal need the numbers, and sorting findings by a
/// rendered English message would make output order depend on wording.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Violation {
    FunctionLines { actual: u32, max: u32 },
    FileLines { actual: u32, max: u32 },
    BlockDepth { actual: u32, max: u32 },
    ParameterCount { actual: u32, max: u32 },
    Complexity { actual: u32, max: u32 },
    ReturnPaths { actual: u32, max: u32 },
    StatementCount { actual: u32, max: u32 },
    EmptyBody,
    MissingReference { marker: String },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub rule_id: &'static str,
    pub violation: Violation,
}

impl Finding {
    /// Renders the operator-facing sentence for this finding.
    pub fn message(&self) -> String {
        self.violation.to_string()
    }
}

#[derive(Debug)]
pub enum RuleError {
    LocatesSource { source: SourceFileError },
}

/// Identity and enablement, shared by every rule regardless of what it inspects.
pub trait Rule {
    const ID: &'static str;

    type Configuration;

    fn severity(configuration: &Self::Configuration) -> Severity;
}

/// A rule that judges one function at a time.
pub trait FunctionRule: Rule {
    fn check(
        function: &FunctionFact,
        facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation>;
}

/// A rule that judges a whole file.
pub trait FileRule: Rule {
    fn check(facts: &SourceFacts, configuration: &Self::Configuration) -> Option<Violation>;
}

/// A rule that judges commentary, possibly reporting more than once per comment.
pub trait CommentRule: Rule {
    fn check(
        comment: &CommentFact,
        configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)>;
}

/// Drives a function rule over every function in every file.
///
/// The severity gate is evaluated once here rather than inside each rule, so `off` costs
/// nothing and no rule can forget to honour it.
pub fn evaluate_function_rule<R: FunctionRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    let severity = R::severity(configuration);

    if severity == Severity::Off {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();

    for source_facts in facts {
        for function in source_facts.functions() {
            let Some(violation) = R::check(function, source_facts, configuration) else {
                continue;
            };

            findings.push(finding(
                source_facts.source(),
                function.range(),
                severity,
                R::ID,
                violation,
            )?);
        }
    }

    Ok(findings)
}

/// Drives a file rule over every scanned file.
pub fn evaluate_file_rule<R: FileRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    let severity = R::severity(configuration);

    if severity == Severity::Off {
        return Ok(Vec::new());
    }

    facts
        .iter()
        .filter_map(|source_facts| {
            R::check(source_facts, configuration).map(|violation| {
                finding(
                    source_facts.source(),
                    source_facts.source().full_range(),
                    severity,
                    R::ID,
                    violation,
                )
            })
        })
        .collect()
}

/// Drives a comment rule over every comment in every file.
pub fn evaluate_comment_rule<R: CommentRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    let severity = R::severity(configuration);

    if severity == Severity::Off {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();

    for source_facts in facts {
        for comment in source_facts.comments() {
            for (range, violation) in R::check(comment, configuration) {
                findings.push(finding(
                    source_facts.source(),
                    range,
                    severity,
                    R::ID,
                    violation,
                )?);
            }
        }
    }

    Ok(findings)
}

fn finding(
    source: &SourceFile,
    range: SourceRange,
    severity: Severity,
    rule_id: &'static str,
    violation: Violation,
) -> Result<Finding, RuleError> {
    let location = source
        .location(range)
        .map_err(|source| RuleError::LocatesSource { source })?;

    Ok(Finding {
        path: source.path().to_path_buf(),
        line: location.start.line,
        column: location.start.column,
        severity,
        rule_id,
        violation,
    })
}

/// Evaluates one configured rule, yielding nothing when the rule is absent.
type Evaluator = fn(&[SourceFacts], &Config) -> Result<Vec<Finding>, RuleError>;

/// The rule registry.
///
/// Adding a rule appends one entry here rather than growing a branch in `evaluate`,
/// whose complexity previously rose with every rule shipped.
const EVALUATORS: &[Evaluator] = &[
    function_size::evaluate,
    function_nesting::evaluate,
    file_size::evaluate,
    empty_function::evaluate,
    todo_requires_reference::evaluate,
    parameter_count::evaluate,
    cyclomatic_complexity::evaluate,
    return_count::evaluate,
    function_statements::evaluate,
];

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    let mut findings = Vec::new();

    for evaluate_rule in EVALUATORS {
        findings.extend(evaluate_rule(facts, config)?);
    }

    findings.sort_by(|left, right| {
        (&left.path, left.line, left.column, left.rule_id).cmp(&(
            &right.path,
            right.line,
            right.column,
            right.rule_id,
        ))
    });

    Ok(findings)
}

/// Runs `evaluate` only when the rule is configured.
pub(crate) fn when_configured<C>(
    configuration: Option<&C>,
    evaluate: impl FnOnce(&C) -> Result<Vec<Finding>, RuleError>,
) -> Result<Vec<Finding>, RuleError> {
    configuration.map_or_else(|| Ok(Vec::new()), evaluate)
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FunctionLines { actual, max } => write!(
                formatter,
                "Function has {actual} effective lines (max {max})."
            ),
            Self::FileLines { actual, max } => {
                write!(formatter, "File has {actual} effective lines (max {max}).")
            }
            Self::BlockDepth { actual, max } => write!(
                formatter,
                "Function nests blocks {actual} levels deep (max {max})."
            ),
            Self::ParameterCount { actual, max } => {
                write!(formatter, "Function has {actual} parameters (max {max}).")
            }
            Self::Complexity { actual, max } => write!(
                formatter,
                "Function has cyclomatic complexity {actual} (max {max})."
            ),
            Self::ReturnPaths { actual, max } => {
                write!(formatter, "Function has {actual} return paths (max {max}).")
            }
            Self::StatementCount { actual, max } => {
                write!(formatter, "Function has {actual} statements (max {max}).")
            }
            Self::EmptyBody => write!(formatter, "Function has an empty body."),
            Self::MissingReference { marker } => {
                write!(formatter, "{marker} comment requires an issue reference.")
            }
        }
    }
}

impl fmt::Display for RuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocatesSource { source } => write!(formatter, "invalid source file: {source}"),
        }
    }
}

impl Error for RuleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LocatesSource { source } => Some(source),
        }
    }
}
