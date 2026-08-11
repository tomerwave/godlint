use std::{fmt, path::PathBuf};

use crate::{
    analyzers::{SourceFacts, workflow::WorkflowFacts},
    config::{Config, Scope, Scoped, Severity},
    date::Date,
    facts::{CommentFact, FunctionFact},
    repository::RepositoryFacts,
    source::{SourceFile, SourceRange, TextFile},
    suppression::{self, Suppression},
};

pub mod accountable_suppression;
pub mod assertion_required;
pub mod bot_conditions;
pub mod branch_naming;
mod catalogue;
pub mod cognitive_complexity;
pub mod condition_complexity;
pub mod decision_complexity;
pub mod dependency_boundary;
mod dialect;
pub mod direct_environment_read;
pub mod empty_error_handler;
pub mod empty_function;
mod enclosing;
pub mod explicit_timer_delay;
pub mod explicit_workflow_permissions;
pub mod file_size;
pub mod filename_case;
pub mod forbidden_dependency;
pub mod frozen_lockfile_install;
pub mod function_nesting;
pub mod function_size;
pub mod function_statements;
pub mod hardcoded_container_credentials;
pub mod languages;
mod line_count;
pub mod lockfile_version_drift;
mod metric;
pub mod module_independence;
mod module_path;
pub mod no_comments;
pub mod no_dynamic_execution;
pub mod no_empty_test;
pub mod no_focused_test;
pub mod no_inline_script;
pub mod no_insecure_random;
pub mod no_internal_import;
pub mod no_monolithic_job;
pub mod no_network_in_unit_test;
pub mod no_production_log;
pub mod no_randomness_without_seed;
pub mod no_shell_command;
pub mod no_silenced_failure;
pub mod no_skipped_test;
pub mod no_sleep_in_test;
pub mod no_test_helper_in_production;
pub mod no_weak_hash;
pub mod no_workflow_comments;
pub mod overprovisioned_secrets;
pub mod parameter_count;
pub mod pin_third_party_actions;
mod reference;
mod scoped;
mod violation;

pub(crate) use reference::when_configured;
pub use reference::{
    AccessRule, ActionRule, CallInTestRule, CallRule, ConditionRule, ErrorHandlerRule, ImportRule,
    TestRule, WorkflowRule, evaluate_access_rule, evaluate_action_rule, evaluate_call_in_test_rule,
    evaluate_call_rule, evaluate_condition_rule, evaluate_error_handler_rule, evaluate_import_rule,
    evaluate_test_rule, evaluate_workflow_rule,
};
mod registry;
pub mod restricted_call;
pub mod restricted_import;
pub mod return_count;
pub mod secrets_inherit;
pub mod stale_action_refs;
pub mod template_injection;
pub mod todo_requires_reference;
pub mod unredacted_secrets;
pub mod untrusted_github_env;
pub mod unused_suppression;
mod workflow_condition;
mod workflow_expression;

pub use languages::{Absence, Languages};
pub use metric::Metric;
pub use registry::{
    closest_rule_id, configured_severity, is_known_rule, is_suppressible_rule, rule_ids,
    rule_languages,
};
pub use violation::Violation;

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

pub struct Evaluation<'a> {
    pub facts: &'a [SourceFacts],
    pub workflows: &'a [WorkflowFacts],
    pub repository: &'a RepositoryFacts,
}

pub trait Rule {
    const ID: &'static str;

    const LANGUAGES: Languages = Languages::EVERY_LANGUAGE;

    type Configuration: Scoped;

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
                .map(move |violation| {
                    (
                        suppression.source().text_file(),
                        suppression.range(),
                        violation,
                    )
                })
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
    reporting: Reporting<'_>,
    reported: impl IntoIterator<Item = (&'a TextFile, SourceRange, Violation)>,
) -> Vec<Finding> {
    if reporting.severity == Severity::Off {
        return Vec::new();
    }

    reported
        .into_iter()
        .filter(|(file, _, _)| dialect::supports(reporting.languages, file))
        .filter(|(file, _, _)| reporting.scope.covers(file.path_text()))
        .map(|(file, range, violation)| finding(file, range, reporting, violation))
        .collect()
}

pub(crate) fn collect_findings<'facts, T: 'facts, I>(
    facts: &'facts [SourceFacts],
    reporting: Reporting<'_>,
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
                    .map(move |(range, violation)| (source.source().text_file(), range, violation))
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
    reporting: Reporting<'_>,
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
    reporting: Reporting<'_>,
    check: impl Fn(&SourceFacts) -> Option<Violation>,
) -> Vec<Finding> {
    report(
        reporting,
        facts.iter().filter_map(move |source_facts| {
            let source = source_facts.source();

            check(source_facts)
                .map(|violation| (source.text_file(), source.full_range(), violation))
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
pub struct Reporting<'a> {
    pub severity: Severity,
    pub rule_id: &'static str,
    languages: Languages,
    scope: Scope<'a>,
}

impl<'a> Reporting<'a> {
    pub fn of<R: Rule>(configuration: &'a R::Configuration) -> Self {
        Self {
            severity: R::severity(configuration),
            rule_id: R::ID,
            languages: R::LANGUAGES,
            scope: Scope::of(configuration),
        }
    }
}

fn finding(
    file: &TextFile,
    range: SourceRange,
    reporting: Reporting<'_>,
    violation: Violation,
) -> Finding {
    let location = file.location(range);
    Finding {
        path: file.path().to_path_buf(),
        range,
        line: location.start.line,
        column: location.start.column,
        severity: reporting.severity.min(violation.cap()),
        rule_id: reporting.rule_id,
        violation,
    }
}

type Evaluator = fn(&[SourceFacts], &Config) -> Vec<Finding>;

type WorkflowEvaluator = fn(&[WorkflowFacts], &Config) -> Vec<Finding>;

type RepositoryEvaluator = fn(&RepositoryFacts, &Config) -> Vec<Finding>;

const WORKFLOW_EVALUATORS: &[WorkflowEvaluator] = &[
    bot_conditions::evaluate,
    overprovisioned_secrets::evaluate,
    hardcoded_container_credentials::evaluate,
    no_workflow_comments::evaluate,
    pin_third_party_actions::evaluate,
    stale_action_refs::evaluate,
    explicit_workflow_permissions::evaluate,
    secrets_inherit::evaluate,
    template_injection::evaluate,
    no_inline_script::evaluate,
    frozen_lockfile_install::evaluate,
    no_monolithic_job::evaluate,
    no_silenced_failure::evaluate,
    unredacted_secrets::evaluate,
    untrusted_github_env::evaluate,
];

const REPOSITORY_EVALUATORS: &[RepositoryEvaluator] =
    &[branch_naming::evaluate, lockfile_version_drift::evaluate];

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
    cognitive_complexity::evaluate,
    return_count::evaluate,
    function_statements::evaluate,
    no_comments::evaluate,
    restricted_call::evaluate,
    no_dynamic_execution::evaluate,
    direct_environment_read::evaluate,
    explicit_timer_delay::evaluate,
    assertion_required::evaluate,
    no_empty_test::evaluate,
    no_focused_test::evaluate,
    no_insecure_random::evaluate,
    no_internal_import::evaluate,
    no_network_in_unit_test::evaluate,
    no_production_log::evaluate,
    no_randomness_without_seed::evaluate,
    no_shell_command::evaluate,
    no_skipped_test::evaluate,
    no_test_helper_in_production::evaluate,
    no_sleep_in_test::evaluate,
    no_weak_hash::evaluate,
    restricted_import::evaluate,
    dependency_boundary::evaluate,
    module_independence::evaluate,
    forbidden_dependency::evaluate,
    filename_case::evaluate,
];

pub fn evaluate(input: Evaluation<'_>, config: &Config, today: Date) -> Vec<Finding> {
    let suppressions = suppression::collect(input.facts);
    let mut findings = Vec::new();

    for evaluate_rule in EVALUATORS {
        findings.extend(evaluate_rule(input.facts, config));
    }

    for evaluate_rule in WORKFLOW_EVALUATORS {
        findings.extend(evaluate_rule(input.workflows, config));
    }

    for evaluate_rule in REPOSITORY_EVALUATORS {
        findings.extend(evaluate_rule(input.repository, config));
    }

    let unused_suppressions = unused_suppression::evaluate(&suppressions, &findings, config);
    let mut findings = suppression::apply(findings, &suppressions);

    findings.extend(accountable_suppression::evaluate(
        &suppressions,
        config,
        today,
    ));

    findings.extend(unused_suppressions);

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
