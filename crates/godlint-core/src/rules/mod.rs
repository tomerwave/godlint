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
pub mod no_comments;
pub mod parameter_count;
pub mod return_count;
pub mod todo_requires_reference;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Metric {
    FunctionLines,
    FileLines,
    BlockDepth,
    ParameterCount,
    Complexity,
    ReturnPaths,
    StatementCount,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Violation {
    Limit {
        metric: Metric,
        actual: u32,
        max: u32,
    },
    EmptyBody,
    MissingReference {
        marker: String,
    },
    CommentNotPermitted,
}

impl Metric {
    fn describe(self, formatter: &mut fmt::Formatter<'_>, actual: u32, max: u32) -> fmt::Result {
        match self {
            Self::FunctionLines => {
                write!(
                    formatter,
                    "Function has {actual} effective lines (max {max})."
                )
            }
            Self::FileLines => write!(formatter, "File has {actual} effective lines (max {max})."),
            Self::BlockDepth => write!(
                formatter,
                "Function nests blocks {actual} levels deep (max {max})."
            ),
            Self::ParameterCount => {
                write!(formatter, "Function has {actual} parameters (max {max}).")
            }
            Self::Complexity => write!(
                formatter,
                "Function has cyclomatic complexity {actual} (max {max})."
            ),
            Self::ReturnPaths => {
                write!(formatter, "Function has {actual} return paths (max {max}).")
            }
            Self::StatementCount => {
                write!(formatter, "Function has {actual} statements (max {max}).")
            }
        }
    }
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

impl Violation {
    pub const fn limit(metric: Metric, actual: u32, max: u32) -> Self {
        Self::Limit {
            metric,
            actual,
            max,
        }
    }
}

impl Finding {
    pub fn message(&self) -> String {
        self.violation.to_string()
    }
}

#[derive(Debug)]
pub enum RuleError {
    LocatesSource { source: SourceFileError },
}

pub trait Rule {
    const ID: &'static str;

    type Configuration;

    fn severity(configuration: &Self::Configuration) -> Severity;
}

pub trait FunctionRule: Rule {
    fn check(
        function: &FunctionFact,
        facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation>;
}

pub trait FileRule: Rule {
    fn check(facts: &SourceFacts, configuration: &Self::Configuration) -> Option<Violation>;
}

pub trait CommentRule: Rule {
    fn check(
        comment: &CommentFact,
        configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)>;
}

pub trait FunctionLimitRule: Rule {
    const METRIC: Metric;

    fn measure(
        function: &FunctionFact,
        facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> u32;

    fn max(configuration: &Self::Configuration) -> u32;
}

pub trait FileLimitRule: Rule {
    const METRIC: Metric;

    fn measure(facts: &SourceFacts, configuration: &Self::Configuration) -> u32;

    fn max(configuration: &Self::Configuration) -> u32;
}

pub fn evaluate_function_rule<R: FunctionRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    evaluate_functions(
        facts,
        R::severity(configuration),
        R::ID,
        |function, source| R::check(function, source, configuration),
    )
}

pub fn evaluate_function_limit_rule<R: FunctionLimitRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    let max = R::max(configuration);

    evaluate_functions(
        facts,
        R::severity(configuration),
        R::ID,
        |function, source| {
            let actual = R::measure(function, source, configuration);

            (actual > max).then_some(Violation::limit(R::METRIC, actual, max))
        },
    )
}

fn evaluate_functions(
    facts: &[SourceFacts],
    severity: Severity,
    rule_id: &'static str,
    check: impl Fn(&FunctionFact, &SourceFacts) -> Option<Violation>,
) -> Result<Vec<Finding>, RuleError> {
    if severity == Severity::Off {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();

    for source_facts in facts {
        for function in source_facts.functions() {
            let Some(violation) = check(function, source_facts) else {
                continue;
            };

            findings.push(finding(
                source_facts.source(),
                function.range(),
                severity,
                rule_id,
                violation,
            )?);
        }
    }

    Ok(findings)
}

pub fn evaluate_file_rule<R: FileRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    evaluate_files(facts, R::severity(configuration), R::ID, |source| {
        R::check(source, configuration)
    })
}

pub fn evaluate_file_limit_rule<R: FileLimitRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    let max = R::max(configuration);

    evaluate_files(facts, R::severity(configuration), R::ID, |source| {
        let actual = R::measure(source, configuration);

        (actual > max).then_some(Violation::limit(R::METRIC, actual, max))
    })
}

fn evaluate_files(
    facts: &[SourceFacts],
    severity: Severity,
    rule_id: &'static str,
    check: impl Fn(&SourceFacts) -> Option<Violation>,
) -> Result<Vec<Finding>, RuleError> {
    if severity == Severity::Off {
        return Ok(Vec::new());
    }

    facts
        .iter()
        .filter_map(|source_facts| {
            check(source_facts).map(|violation| {
                finding(
                    source_facts.source(),
                    source_facts.source().full_range(),
                    severity,
                    rule_id,
                    violation,
                )
            })
        })
        .collect()
}

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

type Evaluator = fn(&[SourceFacts], &Config) -> Result<Vec<Finding>, RuleError>;

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
    no_comments::evaluate,
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

pub(crate) fn when_configured<C>(
    configuration: Option<&C>,
    evaluate: impl FnOnce(&C) -> Result<Vec<Finding>, RuleError>,
) -> Result<Vec<Finding>, RuleError> {
    configuration.map_or_else(|| Ok(Vec::new()), evaluate)
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit {
                metric,
                actual,
                max,
            } => metric.describe(formatter, *actual, *max),
            Self::EmptyBody => write!(formatter, "Function has an empty body."),
            Self::MissingReference { marker } => {
                write!(formatter, "{marker} comment requires an issue reference.")
            }
            Self::CommentNotPermitted => write!(
                formatter,
                "Comment is not permitted; express the intent in the code."
            ),
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
