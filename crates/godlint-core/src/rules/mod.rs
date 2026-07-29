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
    let reporting = Reporting::of::<R>(configuration);

    if reporting.severity == Severity::Off {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();

    for suppression in suppressions {
        for violation in R::check(suppression, configuration, today) {
            findings.push(finding(
                suppression.source(),
                suppression.range(),
                reporting,
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
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::functions,
        |function, source| R::check(function, source, configuration),
    )
}

pub fn evaluate_function_limit_rule<R: FunctionLimitRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    let max = R::max(configuration);

    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::functions,
        |function, source| {
            let actual = R::measure(function, source, configuration);

            (actual > max).then_some(Violation::limit(R::METRIC, actual, max))
        },
    )
}

pub(crate) fn collect_findings<'facts, T: 'facts, I>(
    facts: &'facts [SourceFacts],
    reporting: Reporting,
    items: impl Fn(&'facts SourceFacts) -> &'facts [T],
    report: impl Fn(&'facts T, &'facts SourceFacts) -> I,
) -> Result<Vec<Finding>, RuleError>
where
    I: IntoIterator<Item = (SourceRange, Violation)>,
{
    if reporting.severity == Severity::Off {
        return Ok(Vec::new());
    }

    facts
        .iter()
        .flat_map(|source| items(source).iter().map(move |item| (source, item)))
        .flat_map(|(source, item)| {
            report(item, source)
                .into_iter()
                .map(move |reported| (source, reported))
        })
        .map(|(source, (range, violation))| finding(source.source(), range, reporting, violation))
        .collect()
}

pub trait Ranged {
    fn source_range(&self) -> SourceRange;
}

impl Ranged for FunctionFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

pub(crate) fn collect_ranged<'facts, R: Ranged + 'facts>(
    facts: &'facts [SourceFacts],
    reporting: Reporting,
    items: impl Fn(&'facts SourceFacts) -> &'facts [R],
    check: impl Fn(&'facts R, &'facts SourceFacts) -> Option<Violation>,
) -> Result<Vec<Finding>, RuleError> {
    collect_findings(facts, reporting, items, |item, source| {
        check(item, source).map(|violation| (item.source_range(), violation))
    })
}

pub fn evaluate_file_limit_rule<R: FileLimitRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    let max = R::max(configuration);

    evaluate_files(facts, Reporting::of::<R>(configuration), |source| {
        let actual = R::measure(source, configuration);

        (actual > max).then_some(Violation::limit(R::METRIC, actual, max))
    })
}

fn evaluate_files(
    facts: &[SourceFacts],
    reporting: Reporting,
    check: impl Fn(&SourceFacts) -> Option<Violation>,
) -> Result<Vec<Finding>, RuleError> {
    if reporting.severity == Severity::Off {
        return Ok(Vec::new());
    }

    facts
        .iter()
        .filter_map(|source_facts| {
            check(source_facts).map(|violation| {
                finding(
                    source_facts.source(),
                    source_facts.source().full_range(),
                    reporting,
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
    collect_findings(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::comments,
        |comment, _| R::check(comment, configuration),
    )
}

#[derive(Clone, Copy)]
pub struct Reporting {
    pub severity: Severity,
    pub rule_id: &'static str,
}

impl Reporting {
    pub fn of<R: Rule>(configuration: &R::Configuration) -> Self {
        Self {
            severity: R::severity(configuration),
            rule_id: R::ID,
        }
    }
}

fn finding(
    source: &SourceFile,
    range: SourceRange,
    reporting: Reporting,
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
        severity: reporting.severity,
        rule_id: reporting.rule_id,
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
