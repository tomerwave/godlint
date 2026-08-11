use std::collections::HashMap;

use crate::{
    analyzers::SourceFacts,
    config::{Config, NoProductionLogRule, Severity},
    rules::{Finding, Languages, Reporting, Rule, Violation, report, when_configured},
    source::{Dialect, SourceRange},
};

pub struct NoDuplicateString;

impl Rule for NoDuplicateString {
    const ID: &'static str = "maintainability/no-duplicate-string";
    const LANGUAGES: Languages =
        Languages::all_but(&[(Dialect::Workflow, crate::rules::Absence::NoSuchConstruct)]);
    type Configuration = NoProductionLogRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_duplicate_string.as_ref(), |rule| {
        let reporting = Reporting::of::<NoDuplicateString>(rule);
        facts
            .iter()
            .flat_map(|facts| evaluate_file(facts, reporting))
            .collect()
    })
}

fn evaluate_file(facts: &SourceFacts, reporting: Reporting<'_>) -> Vec<Finding> {
    let literals = literals(facts.source().source());
    let counts = counts(&literals);
    report(
        reporting,
        literals.into_iter().filter_map(|(range, value)| {
            repeated(&value, &counts).then_some((
                facts.source().text_file(),
                range,
                Violation::DuplicateString { value },
            ))
        }),
    )
}

fn counts(literals: &[(SourceRange, String)]) -> HashMap<String, usize> {
    literals
        .iter()
        .fold(HashMap::new(), |mut counts, (_, value)| {
            *counts.entry(value.clone()).or_default() += 1;
            counts
        })
}

fn repeated(value: &str, counts: &HashMap<String, usize>) -> bool {
    value.len() >= 20 && counts.get(value).copied().unwrap_or_default() > 1
}

fn literals(source: &str) -> Vec<(SourceRange, String)> {
    let mut result = Vec::new();
    let mut index = 0;
    while index < source.len() {
        match literal_at(source, index) {
            Some((range, value, next)) => {
                result.push((range, value));
                index = next;
            }
            None => index += 1,
        }
    }
    result
}

fn literal_at(source: &str, start: usize) -> Option<(SourceRange, String, usize)> {
    let quote = source.as_bytes()[start];
    if !matches!(quote, b'\'' | b'"' | b'`') {
        return None;
    }
    let mut index = start + 1;
    while index < source.len() && source.as_bytes()[index] != quote {
        index += 1 + usize::from(source.as_bytes()[index] == b'\\');
    }
    (index < source.len()).then(|| {
        (
            SourceRange::new(start, index + 1),
            source[start + 1..index].to_owned(),
            index + 1,
        )
    })
}
