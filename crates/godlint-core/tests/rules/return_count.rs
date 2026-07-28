use godlint_core::{
    config::{ReturnCountRule, Severity},
    rules::{FunctionRule, Rule, Violation, return_count::ReturnCount},
};

use super::support::function;

fn configuration(max_returns: u32) -> ReturnCountRule {
    ReturnCountRule {
        severity: Severity::Error,
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

/// Rust yields its last value without writing `return`, so counting only explicit returns
/// would report fewer exits than the identical TypeScript, which must write one.
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

/// `?` leaves the function when the value is an error, so it is an exit path.
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
    let (facts, exits) = function(
        "src/example.ts",
        "function example(a: boolean): number {\n  if (a) {\n    return 1;\n  }\n\n  return 2;\n}",
    );

    assert_eq!(
        ReturnCount::check(&exits, &facts, &configuration(1)),
        Some(Violation::ReturnPaths { actual: 2, max: 1 })
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    let (facts, exits) = function(
        "src/example.ts",
        "function example(a: boolean): number {\n  if (a) {\n    return 1;\n  }\n\n  return 2;\n}",
    );

    assert_eq!(ReturnCount::check(&exits, &facts, &configuration(2)), None);
}
