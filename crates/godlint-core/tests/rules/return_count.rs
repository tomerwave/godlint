use godlint_core::{
    config::{ReturnCountRule, Severity},
    facts::FunctionFact,
    rules::{Rule, return_count::ReturnCount},
};

use super::function_fact_fixture::FunctionFactFixture;

fn function(return_count: u32) -> FunctionFact {
    FunctionFactFixture::new()
        .with_return_count(return_count)
        .build()
}

fn configuration(max_returns: u32) -> ReturnCountRule {
    ReturnCountRule {
        severity: Severity::Error,
        max_returns,
    }
}

#[test]
fn reports_a_function_with_more_returns_than_its_limit() {
    let violation = ReturnCount::evaluate(&function(2), &configuration(1));

    assert_eq!(ReturnCount::ID, "maintainability/return-count");
    assert_eq!(violation.map(|violation| violation.return_count), Some(2));
}

#[test]
fn accepts_a_function_at_its_limit() {
    assert_eq!(ReturnCount::evaluate(&function(1), &configuration(1)), None);
}

#[test]
fn disables_evaluation_when_the_rule_is_off() {
    let configuration = ReturnCountRule {
        severity: Severity::Off,
        max_returns: 0,
    };

    assert_eq!(ReturnCount::evaluate(&function(1), &configuration), None);
}
