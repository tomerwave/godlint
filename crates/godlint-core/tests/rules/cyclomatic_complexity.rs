use godlint_core::{
    config::{CyclomaticComplexityRule, Severity},
    facts::FunctionFact,
    rules::{Rule, cyclomatic_complexity::CyclomaticComplexity},
};

use super::function_fact_fixture::FunctionFactFixture;

fn function(decision_points: u32) -> FunctionFact {
    FunctionFactFixture::new()
        .with_decision_points(decision_points)
        .build()
}

fn configuration(max_complexity: u32) -> CyclomaticComplexityRule {
    CyclomaticComplexityRule {
        severity: Severity::Error,
        max_complexity,
    }
}

#[test]
fn reports_a_function_more_complex_than_its_limit() {
    let violation = CyclomaticComplexity::evaluate(&function(2), &configuration(2));

    assert_eq!(
        CyclomaticComplexity::ID,
        "maintainability/cyclomatic-complexity"
    );
    assert_eq!(violation.map(|violation| violation.complexity), Some(3));
}

#[test]
fn accepts_a_function_at_its_limit() {
    assert_eq!(
        CyclomaticComplexity::evaluate(&function(2), &configuration(3)),
        None
    );
}

#[test]
fn disables_evaluation_when_the_rule_is_off() {
    let configuration = CyclomaticComplexityRule {
        severity: Severity::Off,
        max_complexity: 0,
    };

    assert_eq!(
        CyclomaticComplexity::evaluate(&function(0), &configuration),
        None
    );
}
