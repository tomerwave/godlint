use godlint_core::{
    config::{FunctionStatementsRule, Severity},
    rules::{FunctionRule, Rule, Violation, function_statements::FunctionStatements},
};

use super::support::function;

fn configuration(max_statements: u32) -> FunctionStatementsRule {
    FunctionStatementsRule {
        severity: Severity::Error,
        max_statements,
    }
}

fn count(path: &str, source: &str) -> u32 {
    function(path, source).1.statement_count().value()
}

#[test]
fn counts_direct_statements() {
    assert_eq!(
        FunctionStatements::ID,
        "maintainability/function-statements"
    );
    assert_eq!(
        count(
            "src/example.rs",
            "fn example() {\n    one();\n    two();\n}"
        ),
        2
    );
}

/// Statements inside a block still belong to the function; otherwise wrapping a body in
/// `if true { … }` would hide any number of statements behind a count of one.
#[test]
fn counts_statements_inside_nested_blocks() {
    assert_eq!(
        count(
            "src/example.rs",
            "fn example() {\n    if a {\n        one();\n        two();\n    }\n}"
        ),
        3
    );
}

/// A comment is documentation, not work, and the size rules already skip it by default.
#[test]
fn ignores_comments() {
    assert_eq!(
        count(
            "src/example.rs",
            "fn example() {\n    // why\n    one();\n    // more\n    two();\n}"
        ),
        2
    );
}

#[test]
fn excludes_statements_owned_by_a_nested_function() {
    assert_eq!(
        count(
            "src/example.rs",
            "fn example() {\n    let f = || {\n        one();\n        two();\n    };\n}"
        ),
        1
    );
}

/// A concise body does exactly one thing, however many operators the expression contains.
#[test]
fn counts_an_expression_body_as_one_statement() {
    assert_eq!(
        count(
            "src/example.ts",
            "const example = (x: number) => x + 1 + 2 + 3;"
        ),
        1
    );
}

#[test]
fn reports_a_function_over_its_limit() {
    let (facts, busy) = function(
        "src/example.rs",
        "fn example() {\n    one();\n    two();\n}",
    );

    assert_eq!(
        FunctionStatements::check(&busy, &facts, &configuration(1)),
        Some(Violation::StatementCount { actual: 2, max: 1 })
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    let (facts, busy) = function(
        "src/example.rs",
        "fn example() {\n    one();\n    two();\n}",
    );

    assert_eq!(
        FunctionStatements::check(&busy, &facts, &configuration(2)),
        None
    );
}
