use std::path::PathBuf;

use godlint_core::{
    analyzers::analyze,
    date::Date,
    rules::{Rule, Violation, evaluate, unused_suppression::UnusedSuppression},
    source::SourceFile,
};

const TODAY: &str = "2026-07-28";

fn findings(
    source: &str,
    empty_function_severity: &str,
    unused_suppression_severity: &str,
) -> Vec<Violation> {
    let source = SourceFile::new(PathBuf::from("src/example.rs"), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));
    let facts = analyze(&source).unwrap_or_else(|error| panic!("analyzes source: {error}"));
    let config = yaml_serde::from_str(&format!(
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: {empty_function_severity}\n  policy/unused-suppression:\n    severity: {unused_suppression_severity}\n"
    ))
    .unwrap_or_else(|error| panic!("reads configuration: {error}"));
    let today = Date::parse(TODAY).unwrap_or_else(|error| panic!("parses {TODAY}: {error}"));

    evaluate(&[facts], &[], &config, today)
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

#[test]
fn reports_a_directive_that_silences_no_enabled_finding() {
    assert_eq!(UnusedSuppression::ID, "policy/unused-suppression");
    assert_eq!(
        findings(
            "// godlint-ignore-next-line maintainability/empty-function -- obsolete\nfn example() {\n    work();\n}\n",
            "error",
            "error",
        ),
        vec![Violation::UnusedSuppression]
    );
}

#[test]
fn accepts_a_directive_that_silences_an_enabled_finding() {
    assert!(
        findings(
            "// godlint-ignore-next-line maintainability/empty-function -- needed\nfn example() {}\n",
            "error",
            "error",
        )
        .is_empty()
    );
}

#[test]
fn reports_a_directive_for_a_rule_that_is_switched_off() {
    assert_eq!(
        findings(
            "// godlint-ignore-next-line maintainability/empty-function -- dormant\nfn example() {\n    work();\n}\n",
            "off",
            "error",
        )
        .len(),
        1,
        "a directive that silences nothing is reported however the rule stopped applying: the \
         alternative is an exemption that springs back to life, un-reviewed, when the rule returns"
    );
}

#[test]
fn reports_a_directive_that_would_spring_back_when_the_rule_returns() {
    assert_eq!(
        findings(
            "// godlint-ignore-next-line maintainability/empty-function -- dormant\nfn example() {}\n",
            "off",
            "error",
        )
        .len(),
        1,
        "this is the case the change is about: the body really is empty, so the directive silences \
         a real finding the moment the rule is switched back on, un-reviewed unless reported now"
    );
}

fn findings_without_the_rule_configured(source: &str) -> Vec<Violation> {
    let source = SourceFile::new(PathBuf::from("src/example.rs"), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));
    let facts = analyze(&source).unwrap_or_else(|error| panic!("analyzes source: {error}"));
    let config = yaml_serde::from_str(
        "version: 1\nrules:\n  policy/unused-suppression:\n    severity: error\n",
    )
    .unwrap_or_else(|error| panic!("reads configuration: {error}"));
    let today = Date::parse(TODAY).unwrap_or_else(|error| panic!("parses {TODAY}: {error}"));

    evaluate(&[facts], &[], &config, today)
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

#[test]
fn reports_a_directive_for_a_rule_the_configuration_never_mentions() {
    assert_eq!(
        findings_without_the_rule_configured(
            "// godlint-ignore-next-line maintainability/empty-function -- copied\nfn example() {}\n"
        )
        .len(),
        1,
        "a rule absent from the configuration never runs, so a directive naming it is as dead as \
         one for a rule set to off — the fourth way a directive can silence nothing"
    );
}
