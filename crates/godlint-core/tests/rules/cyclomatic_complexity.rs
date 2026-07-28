use godlint_core::{
    config::{CyclomaticComplexityRule, Severity},
    rules::{FunctionRule, Rule, Violation, cyclomatic_complexity::CyclomaticComplexity},
};

use super::support::function;

fn configuration(max_complexity: u32) -> CyclomaticComplexityRule {
    CyclomaticComplexityRule {
        severity: Severity::Error,
        max_complexity,
    }
}

fn complexity(path: &str, source: &str) -> u32 {
    function(path, source).1.decision_points().value() + 1
}

#[test]
fn starts_at_one_for_a_straight_line_function() {
    assert_eq!(
        CyclomaticComplexity::ID,
        "maintainability/cyclomatic-complexity"
    );
    assert_eq!(
        complexity("src/example.rs", "fn example() {\n    run();\n}"),
        1
    );
}

#[test]
fn counts_one_branch_per_language() {
    let cases = [
        ("src/example.rs", "fn example(a: bool) {\n    if a {}\n}"),
        (
            "src/example.ts",
            "function example(a: boolean) {\n  if (a) {}\n}",
        ),
        ("src/example.js", "function example(a) {\n  if (a) {}\n}"),
        ("src/example.py", "def example(a):\n    if a:\n        pass"),
    ];

    for (path, source) in cases {
        assert_eq!(complexity(path, source), 2, "{path}");
    }
}

#[test]
fn counts_match_arms() {
    assert_eq!(
        complexity(
            "src/example.rs",
            "fn example(x: u32) -> u32 {\n    match x {\n        1 => 1,\n        2 => 2,\n        _ => 0,\n    }\n}"
        ),
        4
    );
}

/// The try operator either continues or returns, which is exactly a branch. Without it,
/// idiomatic Rust error handling reports complexity 1 however many fallible calls it makes.
#[test]
fn counts_the_rust_try_operator() {
    assert_eq!(
        complexity(
            "src/example.rs",
            "fn example() -> Result<u32, E> {\n    let a = f1()?;\n    let b = f2()?;\n    Ok(a + b)\n}"
        ),
        3
    );
}

/// A closure owns its own branching, so the same code has the same complexity whether the
/// language spells the callable `|x|`, `lambda x:`, or `(x) =>`.
#[test]
fn attributes_closure_branching_to_the_closure() {
    let (_, host) = function(
        "src/example.rs",
        "fn host() {\n    let f = |x: u32| {\n        if x > 1 {}\n        if x > 2 {}\n    };\n}",
    );

    assert_eq!(host.decision_points().value() + 1, 1);
}

#[test]
fn reports_a_function_over_its_limit() {
    let (facts, branchy) = function(
        "src/example.rs",
        "fn example(a: bool, b: bool) {\n    if a {}\n    if b {}\n}",
    );

    assert_eq!(
        CyclomaticComplexity::check(&branchy, &facts, &configuration(2)),
        Some(Violation::Complexity { actual: 3, max: 2 })
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    let (facts, branchy) = function("src/example.rs", "fn example(a: bool) {\n    if a {}\n}");

    assert_eq!(
        CyclomaticComplexity::check(&branchy, &facts, &configuration(2)),
        None
    );
}
