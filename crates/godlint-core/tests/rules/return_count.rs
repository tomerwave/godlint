use std::path::PathBuf;

use godlint_core::{
    config::{ReturnCountRule, Severity},
    facts::{FunctionFact, FunctionFactDetails},
    rules::{Rule, return_count::ReturnCount},
    source::{SourceFile, SourceRange},
};

fn function(return_count: u32) -> FunctionFact {
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
            return_count,
            statement_count: 0,
            body_is_empty: false,
            nesting_depth: 0,
        },
    )
    .unwrap_or_else(|error| panic!("creates function fact: {error}"))
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
