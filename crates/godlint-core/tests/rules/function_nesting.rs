use godlint_core::{
    config::{FunctionNestingRule, Severity},
    facts::FunctionFact,
    rules::{Rule, function_nesting::FunctionNesting},
};

use super::function_fact_fixture::FunctionFactFixture;

fn function(nesting_depth: u32) -> FunctionFact {
    FunctionFactFixture::new()
        .with_nesting_depth(nesting_depth)
        .build()
}

fn configuration(max_depth: u32) -> FunctionNestingRule {
    FunctionNestingRule {
        severity: Severity::Error,
        max_depth,
    }
}

#[test]
fn reports_a_function_deeper_than_its_limit() {
    let violation = FunctionNesting::evaluate(&function(1), &configuration(0));

    assert_eq!(FunctionNesting::ID, "maintainability/function-nesting");
    assert_eq!(violation.map(|violation| violation.nesting_depth), Some(1));
}

#[test]
fn accepts_a_function_at_its_limit() {
    assert_eq!(
        FunctionNesting::evaluate(&function(1), &configuration(1)),
        None
    );
}

#[test]
fn disables_evaluation_when_the_rule_is_off() {
    let configuration = FunctionNestingRule {
        severity: Severity::Off,
        max_depth: 0,
    };

    assert_eq!(
        FunctionNesting::evaluate(&function(1), &configuration),
        None
    );
}
