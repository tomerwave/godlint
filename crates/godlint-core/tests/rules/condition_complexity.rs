use godlint_core::rules::{Metric, Violation, condition_complexity};

use super::support::rule_violations;

const ENABLED: &str = "version: 1\nrules:\n  maintainability/condition-complexity:\n    severity: error\n    max-operators: 1\n";

fn violations(path: &str, source: &str) -> Vec<Violation> {
    rule_violations(condition_complexity::evaluate, path, source, ENABLED)
}

#[test]
fn permits_a_condition_at_its_limit() {
    for (path, source) in [
        (
            "src/example.rs",
            "fn example(a: bool, b: bool) {\n    if a && b {}\n}",
        ),
        (
            "src/example.py",
            "def example(a, b):\n    if a and b:\n        pass",
        ),
        (
            "src/example.js",
            "function example(a, b) {\n  if (a && b) {}\n}",
        ),
    ] {
        assert!(violations(path, source).is_empty(), "{path}");
    }
}

#[test]
fn reports_a_condition_over_its_limit() {
    for (path, source, actual) in [
        (
            "src/example.rs",
            "fn example(a: bool, b: bool, c: bool) {\n    if a && b || c {}\n}",
            2,
        ),
        (
            "src/example.py",
            "def example(a, b, c):\n    if a and b or c:\n        pass",
            2,
        ),
        (
            "src/example.js",
            "function example(a, b, c) {\n  if (a && b || c) {}\n}",
            2,
        ),
    ] {
        assert_eq!(
            violations(path, source),
            vec![Violation::limit(Metric::ConditionOperators, actual, 1)],
            "{path}"
        );
    }
}

#[test]
fn does_not_count_nonlogical_binary_operators() {
    for (path, source) in [
        (
            "src/example.rs",
            "fn example(a: u32, b: u32, c: u32) {\n    if a + b > c {}\n}",
        ),
        (
            "src/example.js",
            "function example(a, b, c) {\n  if (a + b > c) {}\n}",
        ),
    ] {
        assert!(
            violations(path, source).is_empty(),
            "arithmetic and comparison operators are not logical condition operators: {path}"
        );
    }
}

#[test]
fn counts_every_operator_flat_rather_than_discounting_repeated_ones() {
    assert_eq!(
        violations(
            "src/example.rs",
            "fn example(a: bool, b: bool, c: bool, d: bool) {\n    if a || b && c || d {}\n}",
        ),
        vec![Violation::limit(Metric::ConditionOperators, 3, 1)],
        "three operators, regardless of whether adjacent ones repeat"
    );
}

#[test]
fn counts_a_ternary_nested_inside_a_condition() {
    assert_eq!(
        violations(
            "src/example.py",
            "def example(a, b, c, cond):\n    if (a if cond else b) and c:\n        pass",
        ),
        vec![Violation::limit(Metric::ConditionOperators, 2, 1)],
        "the nested ternary and the `and` both count"
    );
    assert_eq!(
        violations(
            "src/example.js",
            "function example(a, b, c) {\n  if ((a ? b : c) && a) {}\n}",
        ),
        vec![Violation::limit(Metric::ConditionOperators, 2, 1)]
    );
}

#[test]
fn does_not_report_a_standalone_ternary_outside_a_condition() {
    for (path, source) in [
        ("src/example.py", "x = a if b else c"),
        ("src/example.js", "let x = a ? b : c;"),
    ] {
        assert!(
            violations(path, source).is_empty(),
            "{path}: a bare ternary assignment is not attached to an if/while condition"
        );
    }
}

#[test]
fn scores_a_while_condition_the_same_as_an_if() {
    assert_eq!(
        violations(
            "src/example.rs",
            "fn example(a: bool, b: bool, c: bool) {\n    while a && b && c {}\n}",
        ),
        vec![Violation::limit(Metric::ConditionOperators, 2, 1)]
    );
}

#[test]
fn does_not_count_operators_inside_a_nested_function() {
    assert!(
        violations(
            "src/example.js",
            "function example(items) {\n  if (items.some(x => x && true)) {}\n}",
        )
        .is_empty(),
        "the closure's own condition-like logic is not this condition's operator count"
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  maintainability/condition-complexity:\n    severity: off\n    max-operators: 1\n";

    assert!(
        rule_violations(
            condition_complexity::evaluate,
            "src/example.rs",
            "fn example(a: bool, b: bool, c: bool) {\n    if a && b || c {}\n}",
            configuration,
        )
        .is_empty()
    );
}
