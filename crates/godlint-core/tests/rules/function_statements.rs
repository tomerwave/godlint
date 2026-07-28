use std::path::PathBuf;

use godlint_core::{
    config::{FunctionStatementsRule, Severity},
    facts::{FunctionFact, FunctionFactDetails},
    rules::{Rule, function_statements::FunctionStatements},
    source::{SourceFile, SourceRange},
};

fn function(statement_count: u32) -> FunctionFact {
    let source = SourceFile::new(PathBuf::from("src/example.rs"), "fn example() {}".into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));
    let range = SourceRange::new(0, source.source().len())
        .unwrap_or_else(|error| panic!("creates source range: {error}"));

    FunctionFact::new(
        source,
        Some("example".into()),
        FunctionFactDetails {
            range,
            body_range: range,
            parameter_count: 0,
            decision_points: 0,
            return_count: 0,
            statement_count,
            body_is_empty: false,
            nesting_depth: 0,
        },
    )
    .unwrap_or_else(|error| panic!("creates function fact: {error}"))
}

fn configuration(max_statements: u32) -> FunctionStatementsRule {
    FunctionStatementsRule {
        severity: Severity::Error,
        max_statements,
    }
}

#[test]
fn reports_a_function_with_more_statements_than_its_limit() {
    let violation = FunctionStatements::evaluate(&function(2), &configuration(1));

    assert_eq!(
        FunctionStatements::ID,
        "maintainability/function-statements"
    );
    assert_eq!(
        violation.map(|violation| violation.statement_count),
        Some(2)
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    assert_eq!(
        FunctionStatements::evaluate(&function(1), &configuration(1)),
        None
    );
}

#[test]
fn disables_evaluation_when_the_rule_is_off() {
    let configuration = FunctionStatementsRule {
        severity: Severity::Off,
        max_statements: 0,
    };

    assert_eq!(
        FunctionStatements::evaluate(&function(1), &configuration),
        None
    );
}
