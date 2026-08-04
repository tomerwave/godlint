use godlint_core::{
    config::{ReturnCountRule, Severity},
    rules::{Metric, Rule, Violation, return_count::ReturnCount},
};

use super::support::{function, function_limits};

fn configuration(max_returns: u32) -> ReturnCountRule {
    ReturnCountRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        max_returns,
    }
}

fn paths(path: &str, source: &str) -> u32 {
    function(path, source).1.return_paths().value()
}

#[test]
fn counts_explicit_returns() {
    assert_eq!(ReturnCount::ID, "maintainability/return-count");
    assert_eq!(
        paths(
            "src/example.rs",
            "fn example(a: bool) {\n    if a {\n        return;\n    }\n\n    return;\n}"
        ),
        2
    );
}

#[test]
fn counts_an_implicit_tail_expression() {
    let rust = paths(
        "src/example.rs",
        "fn example(x: u32) -> u32 {\n    if x == 1 {\n        return 10;\n    }\n\n    20\n}",
    );
    let typescript = paths(
        "src/example.ts",
        "function example(x: number): number {\n  if (x === 1) {\n    return 10;\n  }\n\n  return 20;\n}",
    );

    assert_eq!(rust, 2);
    assert_eq!(typescript, 2);
}

#[test]
fn a_concise_arrow_body_is_one_implicit_exit_path() {
    assert_eq!(paths("src/example.ts", "const increment = x => x + 1;"), 1);
}

#[test]
fn a_block_arrow_body_counts_its_explicit_return_only() {
    assert_eq!(
        paths(
            "src/example.ts",
            "const increment = x => { return x + 1; };",
        ),
        1
    );
}

#[test]
fn a_concise_arrow_returning_an_object_is_one_implicit_exit_path() {
    assert_eq!(
        paths("src/example.ts", "const wrap = value => ({ value });",),
        1
    );
}

#[test]
fn a_nested_concise_arrow_keeps_its_exit_path_out_of_its_block_bodied_host() {
    let source = concat!(
        "const outer = value => {\n",
        "  const inner = item => item + 1;\n",
        "  return inner(value);\n",
        "};\n",
    );
    let (_, outer) = super::support::nth_function("src/example.ts", source, 0);
    let (_, inner) = super::support::nth_function("src/example.ts", source, 1);

    assert_eq!(outer.return_paths().value(), 1);
    assert_eq!(inner.return_paths().value(), 1);
}

#[test]
fn counts_the_rust_try_operator() {
    assert_eq!(
        paths(
            "src/example.rs",
            "fn example() -> Result<u32, E> {\n    let a = f1()?;\n    Ok(a)\n}"
        ),
        2
    );
}

#[test]
fn reports_a_function_over_its_limit() {
    assert_eq!(
        function_limits::<ReturnCount>(
            "src/example.ts",
            "function example(a: boolean): number {\n  if (a) {\n    return 1;\n  }\n\n  return 2;\n}",
            &configuration(1),
        ),
        vec![Violation::limit(Metric::ReturnPaths, 2, 1)]
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    assert!(
        function_limits::<ReturnCount>(
            "src/example.ts",
            "function example(a: boolean): number {\n  if (a) {\n    return 1;\n  }\n\n  return 2;\n}",
            &configuration(2),
        )
        .is_empty()
    );
}

#[test]
fn a_python_lambda_returns_its_expression_and_a_def_without_return_does_not() {
    let source = concat!(
        "def silent(value):\n",
        "    print(value)\n",
        "\n",
        "\n",
        "double = lambda value: value * 2\n",
    );

    assert_eq!(
        paths("src/example.py", source),
        0,
        "a def that never returns has no exit path to count"
    );
    assert_eq!(
        super::support::nth_function("src/example.py", source, 1)
            .1
            .return_paths()
            .value(),
        1,
        "a lambda returns its expression, so the implicit exit is a path"
    );
}
