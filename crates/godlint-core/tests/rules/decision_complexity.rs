use godlint_core::{
    config::{DecisionComplexityRule, Severity},
    rules::{Metric, Rule, Violation, decision_complexity::DecisionComplexity},
};

use super::support::{function, function_limits};

fn configuration(max_complexity: u32) -> DecisionComplexityRule {
    DecisionComplexityRule {
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
        DecisionComplexity::ID,
        "maintainability/decision-complexity"
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
fn counts_a_multiway_branch_once_in_every_language() {
    let cases = [
        (
            "src/example.rs",
            "fn example(x: u32) -> u32 {\n    match x {\n        1 => 1,\n        2 => 2,\n        3 => 3,\n        _ => 0,\n    }\n}",
        ),
        (
            "src/example.ts",
            "function example(x: number): number {\n  switch (x) {\n    case 1: return 1;\n    case 2: return 2;\n    case 3: return 3;\n    default: return 0;\n  }\n}",
        ),
        (
            "src/example.py",
            "def example(x):\n    match x:\n        case 1:\n            return 1\n        case 2:\n            return 2\n        case 3:\n            return 3\n        case _:\n            return 0\n",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(
            complexity(path, source),
            2,
            "{path}: an exhaustive multiway branch is one decision, not one per arm"
        );
    }
}

#[test]
fn arm_count_does_not_change_the_measurement() {
    let two =
        "fn example(x: u32) -> u32 {\n    match x {\n        1 => 1,\n        _ => 0,\n    }\n}";
    let seven = "fn example(x: u32) -> u32 {\n    match x {\n        1 => 1,\n        2 => 2,\n        3 => 3,\n        4 => 4,\n        5 => 5,\n        6 => 6,\n        _ => 0,\n    }\n}";

    assert_eq!(complexity("src/a.rs", two), complexity("src/b.rs", seven));
}

#[test]
fn counts_a_guard_on_an_arm() {
    assert_eq!(
        complexity(
            "src/example.rs",
            "fn example(x: Option<u32>) -> u32 {\n    match x {\n        Some(n) if n > 100 => 1,\n        Some(n) if n > 10 => 2,\n        Some(_) => 3,\n        None => 0,\n    }\n}"
        ),
        4,
        "one for the function, one for the match, one for each guard"
    );
    assert_eq!(
        complexity(
            "src/example.py",
            "def example(x, flag):\n    match x:\n        case 1 if flag:\n            return 1\n        case 2 if not flag:\n            return 2\n        case _:\n            return 0\n"
        ),
        4
    );
}

#[test]
fn counts_branching_inside_an_arm() {
    assert_eq!(
        complexity(
            "src/example.rs",
            "fn example(x: Option<u32>, flag: bool) -> u32 {\n    match x {\n        Some(n) => {\n            if flag {\n                if n > 5 { 1 } else { 2 }\n            } else {\n                3\n            }\n        }\n        None => 0,\n    }\n}"
        ),
        4,
        "the match is one decision; the conditions nested in an arm are their own"
    );
}

#[test]
fn does_not_count_a_comprehension_filter() {
    assert_eq!(
        complexity(
            "src/example.py",
            "def example(values):\n    return [v for v in values if v > 0]\n"
        ),
        1,
        "a comprehension filter is not counted; only a case guard is"
    );
}

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

#[test]
fn a_rust_let_else_adds_one_decision_and_an_ordinary_let_adds_none() {
    let straight = "fn example() { run(); }";
    let ordinary = "fn example() { let value = 1; run(value); }";
    let refutable = "fn example(option: Option<u32>) { let Some(value) = option else { return; }; run(value); }";

    assert_eq!(complexity("src/straight.rs", straight), 1);
    assert_eq!(complexity("src/ordinary.rs", ordinary), 1);
    assert_eq!(complexity("src/refutable.rs", refutable), 2);
}

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
    assert_eq!(
        function_limits::<DecisionComplexity>(
            "src/example.rs",
            "fn example(a: bool, b: bool) {\n    if a {}\n    if b {}\n}",
            &configuration(2),
        ),
        vec![Violation::limit(Metric::Complexity, 3, 2)]
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    assert!(
        function_limits::<DecisionComplexity>(
            "src/example.rs",
            "fn example(a: bool) {\n    if a {}\n}",
            &configuration(2),
        )
        .is_empty()
    );
}
