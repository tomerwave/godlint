use std::path::PathBuf;

use godlint_core::{
    config::{CyclomaticComplexityRule, Severity},
    facts::{FunctionFact, FunctionFactDetails},
    rules::{Rule, cyclomatic_complexity::CyclomaticComplexity},
    source::{SourceFile, SourceRange},
};

fn function(decision_points: u32) -> FunctionFact {
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
            decision_points,
            return_count: 0,
            statement_count: 0,
            body_is_empty: false,
            nesting_depth: 0,
        },
    )
    .unwrap_or_else(|error| panic!("creates function fact: {error}"))
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
