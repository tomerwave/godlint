use std::{fmt, path::PathBuf};

use crate::{
    analyzers::SourceFacts,
    config::{Config, Severity},
    date::Date,
    facts::{CommentFact, FunctionFact},
    source::{SourceFile, SourceRange},
    suppression::{self, Suppression},
};

pub mod accountable_suppression;
pub mod condition_complexity;
pub mod decision_complexity;
pub mod dependency_boundary;
pub mod direct_environment_read;
pub mod empty_error_handler;
pub mod empty_function;
pub mod explicit_timer_delay;
pub mod file_size;
pub mod filename_case;
pub mod forbidden_dependency;
pub mod function_nesting;
pub mod function_size;
pub mod function_statements;
mod line_count;
mod module_path;
pub mod no_comments;
pub mod no_dynamic_execution;
pub mod no_production_log;
pub mod parameter_count;
mod reference;
mod scoped;
mod violation;

pub use reference::{
    AccessRule, CallRule, ConditionRule, ErrorHandlerRule, ImportRule, evaluate_access_rule,
    evaluate_call_rule, evaluate_condition_rule, evaluate_error_handler_rule, evaluate_import_rule,
};
mod registry;
pub mod restricted_call;
pub mod restricted_import;
pub mod return_count;
pub mod todo_requires_reference;
pub mod unused_suppression;

pub use registry::{configured_severity, is_known_rule, is_suppressible_rule, rule_ids};
pub use violation::Violation;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Metric {
    FunctionLines,
    FileLines,
    BlockDepth,
    ParameterCount,
    Complexity,
    ReturnPaths,
    StatementCount,
    ConditionOperators,
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
            Self::ConditionOperators => {
                write!(
                    formatter,
                    "Condition combines {actual} operators; the limit is {max}."
                )
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

impl Finding {
    pub fn message(&self) -> String {
        self.violation.to_string()
    }
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

pub trait FileRule: Rule {
    fn check(source: &SourceFile, configuration: &Self::Configuration) -> Option<Violation>;
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
) -> Vec<Finding> {
    report(
        Reporting::of::<R>(configuration),
        suppressions.iter().flat_map(move |suppression| {
            R::check(suppression, configuration, today)
                .into_iter()
                .map(move |violation| (suppression.source(), suppression.range(), violation))
        }),
    )
}

pub fn evaluate_function_rule<R: FunctionRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
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
) -> Vec<Finding> {
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

pub(crate) fn report<'a>(
    reporting: Reporting,
    reported: impl IntoIterator<Item = (&'a SourceFile, SourceRange, Violation)>,
) -> Vec<Finding> {
    if reporting.severity == Severity::Off {
        return Vec::new();
    }

    reported
        .into_iter()
        .map(|(source, range, violation)| finding(source, range, reporting, violation))
        .collect()
}

pub(crate) fn collect_findings<'facts, T: 'facts, I>(
    facts: &'facts [SourceFacts],
    reporting: Reporting,
    items: impl Fn(&'facts SourceFacts) -> &'facts [T],
    check: impl Fn(&'facts T, &'facts SourceFacts) -> I,
) -> Vec<Finding>
where
    I: IntoIterator<Item = (SourceRange, Violation)>,
{
    let check = &check;

    report(
        reporting,
        facts.iter().flat_map(move |source| {
            items(source).iter().flat_map(move |item| {
                check(item, source)
                    .into_iter()
                    .map(move |(range, violation)| (source.source(), range, violation))
            })
        }),
    )
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
) -> Vec<Finding> {
    collect_findings(facts, reporting, items, |item, source| {
        check(item, source).map(|violation| (item.source_range(), violation))
    })
}

pub fn evaluate_file_limit_rule<R: FileLimitRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
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
) -> Vec<Finding> {
    report(
        reporting,
        facts.iter().filter_map(move |source_facts| {
            let source = source_facts.source();

            check(source_facts).map(|violation| (source, source.full_range(), violation))
        }),
    )
}

pub fn evaluate_file_rule<R: FileRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    evaluate_files(facts, Reporting::of::<R>(configuration), |source_facts| {
        R::check(source_facts.source(), configuration)
    })
}

pub fn evaluate_comment_rule<R: CommentRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
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
) -> Finding {
    let location = source.location(range);

    Finding {
        path: source.path().to_path_buf(),
        range,
        line: location.start.line,
        column: location.start.column,
        severity: reporting.severity,
        rule_id: reporting.rule_id,
        violation,
    }
}

type Evaluator = fn(&[SourceFacts], &Config) -> Vec<Finding>;

const EVALUATORS: &[Evaluator] = &[
    function_size::evaluate,
    function_nesting::evaluate,
    file_size::evaluate,
    empty_function::evaluate,
    empty_error_handler::evaluate,
    todo_requires_reference::evaluate,
    parameter_count::evaluate,
    decision_complexity::evaluate,
    condition_complexity::evaluate,
    return_count::evaluate,
    function_statements::evaluate,
    no_comments::evaluate,
    restricted_call::evaluate,
    no_dynamic_execution::evaluate,
    direct_environment_read::evaluate,
    explicit_timer_delay::evaluate,
    no_production_log::evaluate,
    restricted_import::evaluate,
    dependency_boundary::evaluate,
    forbidden_dependency::evaluate,
    filename_case::evaluate,
];

pub fn evaluate(facts: &[SourceFacts], config: &Config, today: Date) -> Vec<Finding> {
    let suppressions = suppression::collect(facts);
    let mut findings = Vec::new();

    for evaluate_rule in EVALUATORS {
        findings.extend(evaluate_rule(facts, config));
    }

    let raw_findings = findings;
    let mut findings = suppression::apply(raw_findings.clone(), &suppressions);

    findings.extend(accountable_suppression::evaluate(
        &suppressions,
        config,
        today,
    ));

    findings.extend(unused_suppression::evaluate(
        &suppressions,
        &raw_findings,
        config,
    ));

    findings.sort_by(|left, right| {
        (&left.path, left.line, left.column, left.rule_id).cmp(&(
            &right.path,
            right.line,
            right.column,
            right.rule_id,
        ))
    });

    findings
}

pub(crate) fn when_configured<C>(
    configuration: Option<&C>,
    evaluate: impl FnOnce(&C) -> Vec<Finding>,
) -> Vec<Finding> {
    configuration.map_or_else(Vec::new, evaluate)
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
