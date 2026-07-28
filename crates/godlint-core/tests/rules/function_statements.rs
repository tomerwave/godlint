use godlint_core::{
    config::{FunctionStatementsRule, Severity},
    facts::FunctionFact,
    rules::{Rule, function_statements::FunctionStatements},
};

use super::function_fact_fixture::FunctionFactFixture;

fn function(statement_count: u32) -> FunctionFact {
    FunctionFactFixture::new()
        .with_statement_count(statement_count)
        .build()
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
