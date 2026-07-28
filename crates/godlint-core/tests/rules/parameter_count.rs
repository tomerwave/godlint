use std::path::PathBuf;

use godlint_core::{
    analyzers::analyze,
    config::{ParameterCountRule, Severity},
    rules::{
        Rule,
        parameter_count::{ParameterCount, ParameterCountViolation},
    },
    source::SourceFile,
};

fn function(path: &str, source: &str) -> godlint_core::facts::FunctionFact {
    let source = SourceFile::new(PathBuf::from(path), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));
    let facts = analyze(&source).unwrap_or_else(|error| panic!("analyzes source: {error}"));

    facts.functions()[0].clone()
}

fn configuration(max_parameters: u32) -> ParameterCountRule {
    ParameterCountRule {
        severity: Severity::Error,
        max_parameters,
    }
}

#[test]
fn reports_a_function_that_exceeds_its_limit() {
    let function = function(
        "src/example.rs",
        "fn example(one: u32, two: u32, three: u32) {}",
    );
    let violation = ParameterCount::evaluate(&function, &configuration(2));

    assert_eq!(ParameterCount::ID, "maintainability/parameter-count");
    assert_eq!(
        violation.map(|violation| violation.parameter_count),
        Some(3)
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    let function = function("src/example.py", "def example(one, two):\n    pass");

    assert_eq!(ParameterCount::evaluate(&function, &configuration(2)), None);
}

#[test]
fn counts_a_single_arrow_parameter() {
    let function = function("src/example.ts", "const example = value => value;");

    assert_eq!(
        ParameterCount::evaluate(&function, &configuration(0)),
        Some(ParameterCountViolation { parameter_count: 1 })
    );
}

#[test]
fn disables_evaluation_when_the_rule_is_off() {
    let function = function("src/example.rs", "fn example(one: u32) {}");
    let configuration = ParameterCountRule {
        severity: Severity::Off,
        max_parameters: 0,
    };

    assert_eq!(ParameterCount::evaluate(&function, &configuration), None);
}
