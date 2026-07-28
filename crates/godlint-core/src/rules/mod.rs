use std::{error::Error, fmt, path::PathBuf};

use crate::{
    analyzers::SourceFacts,
    config::{Config, Severity},
    date::Date,
    facts::{CommentFact, FunctionFact},
    source::{SourceFile, SourceFileError, SourceRange},
    suppression::{self, Suppression},
};

pub mod accountable_suppression;
pub mod decision_complexity;
pub mod direct_environment_read;
pub mod empty_function;
pub mod explicit_timer_delay;
pub mod file_size;
pub mod function_nesting;
pub mod function_size;
pub mod function_statements;
mod line_count;
pub mod no_comments;
pub mod no_dynamic_execution;
pub mod parameter_count;
mod reference;

pub use reference::{AccessRule, CallRule, evaluate_access_rule, evaluate_call_rule};
mod registry;
pub mod restricted_call;
pub mod return_count;
pub mod todo_requires_reference;
pub mod unused_suppression;

pub use registry::{configured_severity, is_known_rule, is_suppressible_rule, rule_ids};

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
    UnaccountableSuppression {
        defect: SuppressionDefect,
    },
    UnusedSuppression,
    RestrictedCall {
        callee: String,
    },
    DynamicExecution {
        callee: String,
    },
    DirectEnvironmentRead {
        target: String,
    },
    TimerWithoutDelay {
        callee: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuppressionDefect {
    NoRules,
    UnknownRule { rule: String },
    NotSuppressible { rule: String },
    UnknownOption { option: String },
    RepeatedOption { option: String },
    MissingJustification,
    MissingOwner,
    MissingExpiry,
    InvalidExpiry { value: String },
    Expired { expires: String },
    Unresolved,
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
                "Function has decision complexity {actual} (max {max})."
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
    pub range: SourceRange,
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

pub trait SuppressionRule: Rule {
    fn check(
        suppression: &Suppression,
        configuration: &Self::Configuration,
        today: Date,
    ) -> Vec<Violation>;
}

pub fn evaluate_suppression_rule<R: SuppressionRule>(
    suppressions: &[Suppression],
    configuration: &R::Configuration,
    today: Date,
) -> Result<Vec<Finding>, RuleError> {
    let severity = R::severity(configuration);

    if severity == Severity::Off {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();

    for suppression in suppressions {
        for violation in R::check(suppression, configuration, today) {
            findings.push(finding(
                suppression.source(),
                suppression.range(),
                severity,
                R::ID,
                violation,
            )?);
        }
    }

    Ok(findings)
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
        range,
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
    decision_complexity::evaluate,
    return_count::evaluate,
    function_statements::evaluate,
    no_comments::evaluate,
    restricted_call::evaluate,
    no_dynamic_execution::evaluate,
    direct_environment_read::evaluate,
    explicit_timer_delay::evaluate,
];

pub fn evaluate(
    facts: &[SourceFacts],
    config: &Config,
    today: Date,
) -> Result<Vec<Finding>, RuleError> {
    let suppressions = suppression::collect(facts);
    let mut findings = Vec::new();

    for evaluate_rule in EVALUATORS {
        findings.extend(evaluate_rule(facts, config)?);
    }

    let raw_findings = findings;
    let mut findings = suppression::apply(raw_findings.clone(), &suppressions);

    findings.extend(accountable_suppression::evaluate(
        &suppressions,
        config,
        today,
    )?);

    findings.extend(unused_suppression::evaluate(
        &suppressions,
        &raw_findings,
        config,
    )?);

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
            Self::UnaccountableSuppression { defect } => defect.fmt(formatter),
            Self::UnusedSuppression => write!(
                formatter,
                "Suppression does not silence an enabled finding; remove it or narrow the rule."
            ),
            Self::RestrictedCall { callee } => {
                write!(formatter, "{callee} is restricted by project policy.")
            }
            Self::DynamicExecution { callee } => write!(
                formatter,
                "{callee} executes dynamically generated code; use an explicit, reviewed boundary instead."
            ),
            Self::DirectEnvironmentRead { target } => write!(
                formatter,
                "{target} reads environment directly; read configuration through a config boundary instead."
            ),
            Self::TimerWithoutDelay { callee } => write!(
                formatter,
                "{callee} needs an explicit delay; pass the intended delay in milliseconds."
            ),
        }
    }
}

impl fmt::Display for SuppressionDefect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoRules => write!(
                formatter,
                "Suppression names no rule; list the rule IDs it applies to."
            ),
            Self::UnknownRule { rule } => {
                write!(formatter, "Suppression names unknown rule {rule}.")
            }
            Self::NotSuppressible { rule } => write!(
                formatter,
                "{rule} cannot be suppressed; it is what holds suppressions to account."
            ),
            Self::UnknownOption { option } => {
                write!(formatter, "Suppression option {option} is not recognised.")
            }
            Self::RepeatedOption { option } => write!(
                formatter,
                "Suppression sets {option} more than once; only one value can be accountable."
            ),
            Self::MissingJustification => write!(
                formatter,
                "Suppression has no justification; state the reason after `--`."
            ),
            Self::MissingOwner => write!(
                formatter,
                "Suppression has no owner; name one with `owner=<name>`."
            ),
            Self::MissingExpiry => write!(
                formatter,
                "Suppression has no expiry; set one with `expires=YYYY-MM-DD`."
            ),
            Self::InvalidExpiry { value } => write!(
                formatter,
                "Suppression expiry {value} must be written YYYY-MM-DD."
            ),
            Self::Expired { expires } => {
                write!(formatter, "Suppression expired on {expires}.")
            }
            Self::Unresolved => write!(
                formatter,
                "Suppression has no enclosing declaration; place it inside the declaration \
                 it applies to."
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
