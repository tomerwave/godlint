use std::collections::BTreeMap;

use crate::{
    analyzers::SourceFacts,
    config::{Config, NoDuplicateAssertionRule, Severity},
    facts::TestFact,
    rules::{
        Finding, Reporting, Rule, Violation, collect_findings, enclosing::encloses_a_test,
        when_configured,
    },
    source::SourceRange,
};

pub struct NoDuplicateAssertion;

impl Rule for NoDuplicateAssertion {
    const ID: &'static str = "testing/no-duplicate-assertion";

    type Configuration = NoDuplicateAssertionRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_duplicate_assertion.as_ref(), |rule| {
        collect_findings(
            facts,
            Reporting::of::<NoDuplicateAssertion>(rule),
            SourceFacts::tests,
            repeats,
        )
    })
}

fn repeats(test: &TestFact, facts: &SourceFacts) -> Vec<(SourceRange, Violation)> {
    if encloses_a_test(facts, test) {
        return Vec::new();
    }

    let mut previous: BTreeMap<String, usize> = BTreeMap::new();
    let mut repeated = Vec::new();

    for (range, written) in spans_of(facts, test) {
        let ran = previous.insert(written.clone(), range.end());

        if let Some(end) = ran
            && !acted_between(facts, end, range.start())
        {
            repeated.push((range, Violation::DuplicateAssertion { assertion: written }));
        }
    }

    repeated
}

fn spans_of(facts: &SourceFacts, test: &TestFact) -> Vec<(SourceRange, String)> {
    let mut spans: Vec<(SourceRange, String)> = Vec::new();

    for assertion in facts
        .assertions()
        .iter()
        .filter(|assertion| test.contains(assertion.range()))
    {
        if spans.iter().any(|(range, _)| *range == assertion.range()) {
            continue;
        }

        spans.push((assertion.range(), collapsed(assertion.text())));
    }

    spans
}

fn acted_between(facts: &SourceFacts, end: usize, start: usize) -> bool {
    facts
        .calls()
        .iter()
        .any(|call| call.range().start() >= end && call.range().end() <= start)
}

fn collapsed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<&str>>().join(" ")
}
