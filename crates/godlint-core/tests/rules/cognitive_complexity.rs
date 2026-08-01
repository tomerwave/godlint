use godlint_core::{
    config::{CognitiveComplexityRule, Severity},
    rules::{Metric, Rule, Violation, cognitive_complexity::CognitiveComplexity},
};

use super::support::{function, function_limits};

fn configuration(max_score: u32) -> CognitiveComplexityRule {
    CognitiveComplexityRule {
        severity: Severity::Error,
        max_score,
    }
}

fn score(path: &str, source: &str) -> u32 {
    function(path, source).1.cognitive_score().value()
}

#[test]
fn a_straight_line_function_scores_nothing() {
    assert_eq!(
        CognitiveComplexity::ID,
        "maintainability/cognitive-complexity"
    );
    assert_eq!(score("src/example.rs", "fn example() {\n    run();\n}"), 0);
}

#[test]
fn nesting_costs_more_than_the_same_branches_laid_flat() {
    let flat = concat!(
        "function classify(x) {\n",
        "  if (x < 0) return 1;\n",
        "  if (x === 0) return 2;\n",
        "  if (x < 10) return 3;\n",
        "  if (x < 100) return 4;\n",
        "  return 5;\n",
        "}",
    );
    let nested = concat!(
        "function classify(x) {\n",
        "  if (x >= 0) {\n",
        "    if (x !== 0) {\n",
        "      if (x >= 10) {\n",
        "        if (x >= 100) return 1;\n",
        "      }\n",
        "    }\n",
        "  }\n",
        "}",
    );

    assert_eq!(score("src/flat.js", flat), 4, "four branches, none nested");
    assert_eq!(
        score("src/nested.js", nested),
        10,
        "1 + 2 + 3 + 4: each level pays for its own depth, which is the whole point of the metric"
    );
}

#[test]
fn an_else_if_chain_stays_flat_in_every_language() {
    let cases = [
        (
            "src/example.rs",
            "fn g(s: u32) -> u32 { if s >= 90 { 1 } else if s >= 80 { 2 } else if s >= 70 { 3 } else { 4 } }",
        ),
        (
            "src/example.py",
            "def g(s):\n    if s >= 90:\n        return 1\n    elif s >= 80:\n        return 2\n    elif s >= 70:\n        return 3\n    else:\n        return 4",
        ),
        (
            "src/example.js",
            "function g(s) {\n  if (s >= 90) return 1;\n  else if (s >= 80) return 2;\n  else if (s >= 70) return 3;\n  else return 4;\n}",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(
            score(path, source),
            4,
            "{path}: an else-if pays once, because the reader already paid for the if"
        );
    }
}

#[test]
fn an_else_if_costs_one_even_when_its_chain_is_nested() {
    for (path, source) in [
        (
            "src/example.js",
            concat!(
                "function classify(a, b, c) {\n",
                "  if (a) {\n",
                "    if (b) return 1;\n",
                "    else if (c) return 2;\n",
                "  }\n",
                "  return 0;\n",
                "}",
            ),
        ),
        (
            "src/example.rs",
            concat!(
                "fn classify(a: bool, b: bool, c: bool) -> u32 {\n",
                "    if a {\n",
                "        if b { return 1; }\n",
                "        else if c { return 2; }\n",
                "    }\n",
                "    0\n",
                "}",
            ),
        ),
    ] {
        assert_eq!(
            score(path, source),
            4,
            "the outer if costs 1, the nested if costs 2, and its else-if still costs only 1: {path}"
        );
    }
}

#[test]
fn a_genuinely_nested_else_still_pays_for_its_depth() {
    assert_eq!(
        score(
            "src/example.js",
            "function k(a, b) {\n  if (a) { return 1; } else { if (b) { return 2; } }\n  return 0;\n}",
        ),
        4,
        "if 1, else 1, and the if inside the else 2 — unlike an else-if, this one is nested"
    );
}

#[test]
fn a_multiway_branch_costs_one_however_many_arms_it_has() {
    let two =
        "function h(x) {\n  switch (x) {\n    case 1: return 1;\n    default: return 0;\n  }\n}";
    let five = concat!(
        "function h(x) {\n  switch (x) {\n",
        "    case 1: return 1;\n    case 2: return 2;\n    case 3: return 3;\n",
        "    case 4: return 4;\n    default: return 0;\n  }\n}",
    );

    assert_eq!(score("src/a.js", two), 1);
    assert_eq!(
        score("src/b.js", five),
        1,
        "a switch is scanned at a glance, so arm count does not change the score"
    );
}

#[test]
fn a_run_of_one_operator_costs_less_than_mixed_operators() {
    let cases = [
        (
            "src/example.rs",
            "fn f(a: bool, b: bool, c: bool, d: bool) { if a && b && c && d { } }",
            2,
        ),
        (
            "src/example.rs",
            "fn f(a: bool, b: bool, c: bool, d: bool) { if a && b || c && d { } }",
            4,
        ),
        (
            "src/example.py",
            "def f(a, b, c, d):\n    if a and b and c and d:\n        pass",
            2,
        ),
        (
            "src/example.py",
            "def f(a, b, c, d):\n    if a and b or c and d:\n        pass",
            4,
        ),
        (
            "src/example.js",
            "function f(a, b, c, d) { if (a && b && c && d) {} }",
            2,
        ),
        (
            "src/example.js",
            "function f(a, b, c, d) { if (a && b || c && d) {} }",
            4,
        ),
    ];

    for (path, source, expected) in cases {
        assert_eq!(
            score(path, source),
            expected,
            "{path}: one increment per run of like operators, plus one for the if — {source}"
        );
    }
}

#[test]
fn a_loop_nests_its_body_the_same_way_a_branch_does() {
    for (path, source) in [
        (
            "src/example.rs",
            "fn f(items: Vec<u32>) { for item in items { if item > 0 { } } }",
        ),
        (
            "src/example.py",
            "def f(items):\n    for item in items:\n        if item > 0:\n            pass",
        ),
        (
            "src/example.js",
            "function f(items) {\n  for (const item of items) {\n    if (item > 0) {}\n  }\n}",
        ),
    ] {
        assert_eq!(
            score(path, source),
            3,
            "{path}: the loop is 1, the nested branch is 2"
        );
    }
}

#[test]
fn a_closures_complexity_belongs_to_the_closure_rather_than_its_host() {
    let source = concat!(
        "function host() {\n",
        "  const classify = (x) => {\n",
        "    if (x > 1) return 1;\n",
        "    if (x > 2) return 2;\n",
        "    return 0;\n",
        "  };\n",
        "}\n",
    );
    let (_, host) = super::support::nth_function("src/example.js", source, 0);
    let (_, closure) = super::support::nth_function("src/example.js", source, 1);

    assert_eq!(
        host.cognitive_score().value(),
        0,
        "the host does not inherit the nested arrow's branches"
    );
    assert_eq!(
        closure.cognitive_score().value(),
        2,
        "the nested arrow retains its own complexity"
    );
}

#[test]
fn reports_a_function_over_its_limit() {
    assert_eq!(
        function_limits::<CognitiveComplexity>(
            "src/example.js",
            "function f(x) {\n  if (x > 0) {\n    if (x > 1) {}\n  }\n}",
            &configuration(2),
        ),
        vec![Violation::limit(Metric::CognitiveScore, 3, 2)]
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    assert!(
        function_limits::<CognitiveComplexity>(
            "src/example.js",
            "function f(x) {\n  if (x > 0) {\n    if (x > 1) {}\n  }\n}",
            &configuration(3),
        )
        .is_empty()
    );
}
