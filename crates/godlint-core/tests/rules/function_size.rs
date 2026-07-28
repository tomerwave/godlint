use godlint_core::{
    config::{LineLimitRule, Severity},
    rules::{FunctionRule, Metric, Rule, Violation, function_size::FunctionSize},
};

use super::support::{function, limit};

fn configuration(max_lines: u32, skip_blank_lines: bool, skip_comments: bool) -> LineLimitRule {
    LineLimitRule {
        severity: Severity::Error,
        max_lines: limit(max_lines),
        skip_blank_lines,
        skip_comments,
    }
}

fn lines(path: &str, source: &str, skip_blank_lines: bool, skip_comments: bool) -> u32 {
    let (facts, subject) = function(path, source);

    match FunctionSize::check(
        &subject,
        &facts,
        &configuration(1, skip_blank_lines, skip_comments),
    ) {
        Some(Violation::Limit {
            metric: Metric::FunctionLines,
            actual,
            ..
        }) => actual,
        _ => 1,
    }
}

#[test]
fn counts_effective_lines() {
    assert_eq!(FunctionSize::ID, "maintainability/function-size");
    assert_eq!(
        lines(
            "src/example.rs",
            "fn example() {\n    run();\n}",
            true,
            true
        ),
        3
    );
}

#[test]
fn skips_blank_lines_when_configured() {
    let source = "fn example() {\n\n    run();\n\n}";

    assert_eq!(lines("src/example.rs", source, true, true), 3);
    assert_eq!(lines("src/example.rs", source, false, true), 5);
}

#[test]
fn skips_comment_only_lines() {
    let cases = [
        (
            "src/example.rs",
            "fn example() {\n    // explanation\n    run();\n    /*\n     * detail\n     */\n}",
        ),
        (
            "src/example.ts",
            "function example() {\n  // explanation\n  run();\n  /* detail */\n}",
        ),
        (
            "src/example.py",
            "def example():\n    # explanation\n    run()",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(
            lines(path, source, true, true),
            if path.ends_with(".py") { 2 } else { 3 },
            "{path}"
        );
    }
}

#[test]
fn skips_a_nested_block_comment() {
    assert_eq!(
        lines(
            "src/example.rs",
            "fn example() {\n    /* /* nested */ */\n    run();\n}",
            true,
            true
        ),
        3
    );
}

#[test]
fn skips_a_python_docstring() {
    assert_eq!(
        lines(
            "src/example.py",
            "def example():\n    \"\"\"\n    Detail.\n    More detail.\n    \"\"\"\n    run()",
            true,
            true
        ),
        2
    );
}

#[test]
fn counts_a_line_whose_comment_marker_is_inside_a_string() {
    assert_eq!(
        lines(
            "src/example.rs",
            "fn example() {\n    let url = \"//example\";\n    run(url);\n}",
            true,
            true
        ),
        4
    );
}

#[test]
fn counts_code_that_begins_where_a_comment_ends() {
    assert_eq!(
        lines(
            "src/example.rs",
            "fn example() {\n    /* detail */}",
            true,
            true
        ),
        2
    );
}

#[test]
fn counts_a_line_holding_both_code_and_a_comment() {
    assert_eq!(
        lines(
            "src/example.rs",
            "fn example() {\n    run(); // explanation\n}",
            true,
            true
        ),
        3
    );
}

#[test]
fn reports_a_function_over_its_limit() {
    let (facts, big) = function("src/example.rs", "fn example() {\n    run();\n}");

    assert_eq!(
        FunctionSize::check(&big, &facts, &configuration(2, true, true)),
        Some(Violation::Limit {
            metric: Metric::FunctionLines,
            actual: 3,
            max: 2
        })
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    let (facts, big) = function("src/example.rs", "fn example() {\n    run();\n}");

    assert_eq!(
        FunctionSize::check(&big, &facts, &configuration(3, true, true)),
        None
    );
}
