use std::path::PathBuf;

use godlint_core::{
    config::{FunctionNestingRule, Severity},
    facts::FunctionFact,
    rules::{Rule, function_nesting::FunctionNesting},
    source::{SourceFile, SourceRange},
};

fn function(nesting_depth: u32) -> FunctionFact {
    let source = SourceFile::new(PathBuf::from("src/example.rs"), "fn example() {}".into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));
    let range = SourceRange::new(0, source.source().len())
        .unwrap_or_else(|error| panic!("creates source range: {error}"));

    FunctionFact::new(source, Some("example".into()), range, range, nesting_depth)
        .unwrap_or_else(|error| panic!("creates function fact: {error}"))
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
