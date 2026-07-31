#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use godlint_core::{
    rules::{Absence, Languages, rule_ids, rule_languages},
    source::Dialect,
};

const ANALYSED: &str = "✓";
const NO_SUCH_CONSTRUCT: &str = "—";
const NOT_IMPLEMENTED: &str = "·";

type Matrix = BTreeMap<String, Vec<String>>;

fn documented() -> Matrix {
    let reference = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/rules.md");
    let text = fs::read_to_string(reference).unwrap_or_else(|error| panic!("reads rules: {error}"));
    let rows: Matrix = text
        .lines()
        .skip_while(|line| !is_header(line))
        .skip(2)
        .take_while(|line| line.starts_with('|'))
        .filter_map(documented_row)
        .collect();

    assert!(!rows.is_empty(), "docs/rules.md has no support matrix");

    rows
}

fn is_header(line: &str) -> bool {
    let cells: Vec<&str> = line.split('|').map(str::trim).collect();

    cells.first() == Some(&"")
        && cells.get(1) == Some(&"Rule")
        && Dialect::EVERY
            .iter()
            .enumerate()
            .all(|(column, dialect)| cells.get(column + 2) == Some(&dialect.label()))
}

fn documented_row(line: &str) -> Option<(String, Vec<String>)> {
    let mut cells = line.split('|').map(str::trim).skip(1);
    let rule = cells
        .next()?
        .strip_prefix('`')
        .and_then(|rule| rule.strip_suffix('`'))?;
    let marks = cells
        .take(Dialect::EVERY.len())
        .map(str::to_owned)
        .collect();

    Some((rule.to_owned(), marks))
}

fn declared() -> Matrix {
    rule_ids()
        .map(|rule| {
            let languages = rule_languages(rule).expect("registered rule");
            let marks = Dialect::EVERY
                .iter()
                .map(|dialect| mark(languages.absence(*dialect)).to_owned())
                .collect();

            (rule.to_owned(), marks)
        })
        .collect()
}

fn mark(absence: Option<Absence>) -> &'static str {
    match absence {
        None => ANALYSED,
        Some(Absence::NoSuchConstruct) => NO_SUCH_CONSTRUCT,
        Some(Absence::NotImplemented) => NOT_IMPLEMENTED,
    }
}

fn row(marks: Option<&Vec<String>>) -> String {
    marks.map_or_else(|| "no row".to_owned(), |marks| marks.join(" "))
}

fn drift(documented: &Matrix, declared: &Matrix) -> Vec<String> {
    let rules: BTreeSet<&String> = documented.keys().chain(declared.keys()).collect();

    rules
        .into_iter()
        .filter(|rule| documented.get(*rule) != declared.get(*rule))
        .map(|rule| {
            format!(
                "{rule}: docs/rules.md says {}, Rule::LANGUAGES says {}",
                row(documented.get(rule)),
                row(declared.get(rule))
            )
        })
        .collect()
}

#[test]
fn the_support_matrix_states_what_every_rule_declares() {
    let drifted = drift(&documented(), &declared());

    assert!(
        drifted.is_empty(),
        "the support matrix in docs/rules.md has drifted from Rule::LANGUAGES:\n  {}",
        drifted.join("\n  ")
    );
}

#[test]
fn every_rule_declares_a_dialect_it_analyses() {
    for rule in rule_ids() {
        let languages = rule_languages(rule).expect("registered rule");

        assert!(
            Dialect::EVERY
                .iter()
                .any(|dialect| languages.analyses(*dialect)),
            "{rule} declares no dialect it analyses, so it can never report"
        );
    }
}

#[test]
fn a_declaration_names_only_the_dialects_a_rule_cannot_cover() {
    let limited = Languages::all_but(&[(Dialect::Rust, Absence::NotImplemented)]);

    assert_eq!(
        limited.absence(Dialect::Rust),
        Some(Absence::NotImplemented)
    );
    assert!(!limited.analyses(Dialect::Rust));
    assert!(limited.analyses(Dialect::Python));
    assert!(
        Dialect::EVERY
            .iter()
            .filter(|dialect| **dialect != Dialect::Workflow)
            .all(|dialect| Languages::EVERY_LANGUAGE.analyses(*dialect)),
        "the default declaration must claim every language"
    );
    assert!(
        !Languages::EVERY_LANGUAGE.analyses(Dialect::Workflow),
        "a workflow is not a language, so claiming every language must not claim it"
    );
}

#[test]
fn a_rule_reads_workflows_or_source_and_never_both() {
    for rule in rule_ids() {
        let languages = rule_languages(rule).expect("registered rule");
        let reads_workflows = languages.analyses(Dialect::Workflow);
        let reads_source = Dialect::EVERY
            .iter()
            .filter(|dialect| **dialect != Dialect::Workflow)
            .any(|dialect| languages.analyses(*dialect));

        assert_ne!(
            reads_workflows, reads_source,
            "{rule} must read one subject or the other; a workflow has no functions and \
             source has no jobs"
        );
        assert_eq!(
            reads_workflows,
            rule.starts_with("ci/"),
            "{rule} and the ci/ family must agree about whether it reads workflows"
        );
    }
}
