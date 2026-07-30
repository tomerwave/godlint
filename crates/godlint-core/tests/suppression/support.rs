use std::path::PathBuf;

use godlint_core::{
    analyzers::SourceFacts, analyzers::analyze, config::Config, date::Date, rules::evaluate,
    source::SourceFile, suppression::Suppression, suppression::collect,
};

pub(super) fn facts(path: &str, source: &str) -> SourceFacts {
    let source = SourceFile::new(PathBuf::from(path), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));

    analyze(&source).unwrap_or_else(|error| panic!("analyzes {path}: {error}"))
}

pub(super) fn suppressions(path: &str, source: &str) -> Vec<Suppression> {
    collect(std::slice::from_ref(&facts(path, source)))
}

pub(super) fn only(path: &str, source: &str) -> Suppression {
    let mut found = suppressions(path, source);

    assert_eq!(found.len(), 1, "expected exactly one suppression in {path}");

    found.remove(0)
}

pub(super) fn config(body: &str) -> Config {
    yaml_serde::from_str(body).unwrap_or_else(|error| panic!("reads configuration: {error}"))
}

pub(super) fn surviving(path: &str, source: &str, body: &str) -> Vec<(usize, usize)> {
    let facts = facts(path, source);

    evaluate(std::slice::from_ref(&facts), &config(body), today())
        .iter()
        .map(|finding| (finding.line, finding.column))
        .collect()
}

pub(super) const EMPTY_FUNCTION: &str =
    "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n";

pub(super) fn today() -> Date {
    Date::parse("2026-07-28").unwrap_or_else(|error| panic!("parses date: {error}"))
}
