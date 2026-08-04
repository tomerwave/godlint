use godlint_core::{
    config::{FunctionStatementsRule, Severity},
    rules::{Metric, Rule, Violation, function_statements::FunctionStatements},
};

use super::support::{function, function_limits};

fn configuration(max_statements: u32) -> FunctionStatementsRule {
    FunctionStatementsRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
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
    assert_eq!(
        function_limits::<FunctionStatements>(
            "src/example.rs",
            "fn example() {\n    one();\n    two();\n}",
            &configuration(1),
        ),
        vec![Violation::limit(Metric::StatementCount, 2, 1)]
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    assert!(
        function_limits::<FunctionStatements>(
            "src/example.rs",
            "fn example() {\n    one();\n    two();\n}",
            &configuration(2),
        )
        .is_empty()
    );
}

#[test]
fn a_block_in_a_parameter_is_not_a_statement_of_the_function() {
    let counted = count(
        "src/parameter.rs",
        "fn example(value: [u8; { let first = 1; let second = 2; first + second }]) {\n    run(value);\n}",
    );

    assert_eq!(counted, 1);
}
