use std::error::Error;

use godlint_core::{
    config::{
        EmptyFunctionRule, LineLimitRule, NoCommentsRule, Severity, TodoRequiresReferenceRule,
    },
    rules::{
        Finding, RuleError, empty_function::EmptyFunction, evaluate_comment_rule,
        evaluate_file_rule, evaluate_function_rule, file_size::FileSize, no_comments::NoComments,
    },
    source::SourceFileError,
};

use super::support::{comment_violations, facts, limit};

fn error() -> RuleError {
    RuleError::LocatesSource {
        source: SourceFileError::InvalidUtf8Boundary { offset: 7 },
    }
}

fn line_limit(severity: Severity) -> LineLimitRule {
    LineLimitRule {
        severity,
        max_lines: limit(1),
        skip_blank_lines: true,
        skip_comments: true,
    }
}

fn evaluated(findings: Result<Vec<Finding>, RuleError>) -> Vec<Finding> {
    findings.unwrap_or_else(|error| panic!("evaluates: {error}"))
}

#[test]
fn describes_a_source_location_failure() {
    let message = error().to_string();

    assert!(message.contains("invalid source file"), "{message}");
    assert!(message.contains("UTF-8 boundary"), "{message}");
}

#[test]
fn exposes_the_underlying_source_error() {
    let source = error().source().map(ToString::to_string);

    assert_eq!(
        source.as_deref(),
        Some("source offset is not on a UTF-8 boundary: 7")
    );
}

#[test]
fn a_function_rule_set_to_off_reports_nothing() {
    let source = facts("src/example.rs", "fn empty() {}");
    let configuration = EmptyFunctionRule {
        severity: Severity::Off,
        allow_names: Vec::new(),
    };

    assert!(
        evaluated(evaluate_function_rule::<EmptyFunction>(
            std::slice::from_ref(&source),
            &configuration
        ))
        .is_empty()
    );
}

#[test]
fn a_file_rule_set_to_off_reports_nothing() {
    let source = facts("src/example.rs", "fn a() {}\nfn b() {}\n");

    assert!(
        evaluated(evaluate_file_rule::<FileSize>(
            std::slice::from_ref(&source),
            &line_limit(Severity::Off)
        ))
        .is_empty()
    );
}

#[test]
fn a_comment_rule_set_to_off_reports_nothing() {
    let source = facts("src/example.rs", "// aside\nfn a() {}\n");
    let configuration = NoCommentsRule {
        severity: Severity::Off,
        allow_doc_comments: false,
    };

    assert!(
        evaluated(evaluate_comment_rule::<NoComments>(
            std::slice::from_ref(&source),
            &configuration
        ))
        .is_empty()
    );
}

#[test]
fn a_reference_prefix_followed_by_no_digit_is_not_a_reference() {
    let configuration = TodoRequiresReferenceRule {
        severity: Severity::Error,
        markers: vec!["TODO".into()],
        reference_prefixes: vec!["#".into()],
    };
    let reported = comment_violations::<
        godlint_core::rules::todo_requires_reference::TodoRequiresReference,
    >("src/example.rs", "// TODO: see #notanumber", &configuration);

    assert_eq!(reported.len(), 1);
}
