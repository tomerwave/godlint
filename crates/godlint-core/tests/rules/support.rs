use std::{num::NonZeroU32, path::PathBuf};

use godlint_core::source::SourceFile;
use godlint_core::{
    analyzers::{SourceFacts, analyze},
    config::Config,
    facts::FunctionFact,
    rules::{
        CommentRule, FileLimitRule, Finding, FunctionLimitRule, RuleError, Violation,
        evaluate_file_limit_rule, evaluate_function_limit_rule,
    },
    suppression::{Suppression, collect},
};

pub(super) fn config(body: &str) -> Config {
    yaml_serde::from_str(body).unwrap_or_else(|error| panic!("reads configuration: {error}"))
}

pub(super) fn rule_violations(
    evaluate: fn(&[SourceFacts], &Config) -> Result<Vec<Finding>, RuleError>,
    path: &str,
    source: &str,
    configuration: &str,
) -> Vec<Violation> {
    evaluate(&[facts(path, source)], &config(configuration))
        .unwrap_or_else(|error| panic!("evaluates {path}: {error}"))
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

pub(super) fn facts(path: &str, source: &str) -> SourceFacts {
    let source = SourceFile::new(PathBuf::from(path), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));

    analyze(&source).unwrap_or_else(|error| panic!("analyzes {path}: {error}"))
}

pub(super) fn suppressions(path: &str, source: &str) -> Vec<Suppression> {
    collect(std::slice::from_ref(&facts(path, source)))
}

pub(super) fn nth_function(path: &str, source: &str, index: usize) -> (SourceFacts, FunctionFact) {
    let facts = facts(path, source);
    let function = facts
        .functions()
        .get(index)
        .unwrap_or_else(|| panic!("{path} has no function at index {index}"))
        .clone();

    (facts, function)
}

pub(super) fn function(path: &str, source: &str) -> (SourceFacts, FunctionFact) {
    nth_function(path, source, 0)
}

pub(super) fn limit(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap_or_else(|| panic!("{value} is not a valid limit"))
}

pub(super) fn comment_violations<R: CommentRule>(
    path: &str,
    source: &str,
    configuration: &R::Configuration,
) -> Vec<Violation> {
    facts(path, source)
        .comments()
        .iter()
        .flat_map(|comment| R::check(comment, configuration))
        .map(|(_, violation)| violation)
        .collect()
}

pub(super) fn function_limits<R: FunctionLimitRule>(
    path: &str,
    source: &str,
    configuration: &R::Configuration,
) -> Vec<Violation> {
    let facts = facts(path, source);

    evaluate_function_limit_rule::<R>(std::slice::from_ref(&facts), configuration)
        .unwrap_or_else(|error| panic!("evaluates {}: {error}", R::ID))
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

pub(super) fn file_limits<R: FileLimitRule>(
    path: &str,
    source: &str,
    configuration: &R::Configuration,
) -> Vec<Violation> {
    let facts = facts(path, source);

    evaluate_file_limit_rule::<R>(std::slice::from_ref(&facts), configuration)
        .unwrap_or_else(|error| panic!("evaluates {}: {error}", R::ID))
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}
