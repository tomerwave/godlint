use godlint_core::{
    config::{
        AccountableSuppressionRule, EmptyFunctionRule, LineLimitRule, NoCommentsRule, Severity,
        TodoRequiresReferenceRule,
    },
    date::Date,
    rules::{
        accountable_suppression::AccountableSuppression, empty_function::EmptyFunction,
        evaluate_comment_rule, evaluate_file_limit_rule, evaluate_function_rule,
        evaluate_suppression_rule, file_size::FileSize, no_comments::NoComments,
    },
};

use super::support::{comment_violations, facts, limit, suppressions};

fn line_limit(severity: Severity) -> LineLimitRule {
    LineLimitRule {
        severity,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        max_lines: limit(1),
        skip_blank_lines: true,
        skip_comments: true,
    }
}

#[test]
fn a_function_rule_set_to_off_reports_nothing() {
    let source = facts("src/example.rs", "fn empty() {}");
    let configuration = EmptyFunctionRule {
        severity: Severity::Off,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        allow_names: Vec::new(),
    };

    assert!(
        evaluate_function_rule::<EmptyFunction>(std::slice::from_ref(&source), &configuration)
            .is_empty()
    );
}

#[test]
fn a_file_rule_set_to_off_reports_nothing() {
    let source = facts("src/example.rs", "fn a() {}\nfn b() {}\n");

    assert!(
        evaluate_file_limit_rule::<FileSize>(
            std::slice::from_ref(&source),
            &line_limit(Severity::Off)
        )
        .is_empty()
    );
}

#[test]
fn a_comment_rule_set_to_off_reports_nothing() {
    let source = facts("src/example.rs", "// aside\nfn a() {}\n");
    let configuration = NoCommentsRule {
        severity: Severity::Off,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        allow_doc_comments: false,
    };

    assert!(
        evaluate_comment_rule::<NoComments>(std::slice::from_ref(&source), &configuration)
            .is_empty()
    );
}

#[test]
fn a_suppression_rule_set_to_off_reports_nothing() {
    let suppressions = suppressions(
        "src/example.rs",
        "// godlint-ignore-next-line maintainability/empty-function\nfn a() {}\n",
    );
    let configuration = AccountableSuppressionRule {
        severity: Severity::Off,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        require_owner: true,
        require_expiry: true,
    };
    let today = Date::parse("2026-07-28").unwrap_or_else(|error| panic!("parses date: {error}"));

    assert!(
        evaluate_suppression_rule::<AccountableSuppression>(&suppressions, &configuration, today)
            .is_empty(),
        "a directive with three defects is still silent when the rule is off"
    );
}

#[test]
fn a_reference_prefix_followed_by_no_digit_is_not_a_reference() {
    let configuration = TodoRequiresReferenceRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        markers: vec!["TODO".into()],
        reference_prefixes: vec!["#".into()],
    };
    let reported = comment_violations::<
        godlint_core::rules::todo_requires_reference::TodoRequiresReference,
    >("src/example.rs", "// TODO: see #notanumber", &configuration);

    assert_eq!(reported.len(), 1);
}
