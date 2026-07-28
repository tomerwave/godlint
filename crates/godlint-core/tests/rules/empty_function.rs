use std::path::PathBuf;

use godlint_core::{
    analyzers::analyze,
    config::{EmptyFunctionRule, Severity},
    rules::{Rule, empty_function::EmptyFunction},
    source::SourceFile,
};

fn facts(path: &str, source: &str) -> godlint_core::analyzers::SourceFacts {
    let source = SourceFile::new(PathBuf::from(path), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));

    analyze(&source).unwrap_or_else(|error| panic!("analyzes source: {error}"))
}

fn configuration(allow_names: &[&str]) -> EmptyFunctionRule {
    EmptyFunctionRule {
        severity: Severity::Error,
        allow_names: allow_names.iter().map(|name| (*name).into()).collect(),
    }
}

#[test]
fn reports_an_empty_brace_body() {
    let facts = facts("src/example.rs", "fn empty() {\n    // detail\n}");
    let violation = EmptyFunction::evaluate(&facts.functions()[0], &configuration(&[]));

    assert_eq!(EmptyFunction::ID, "maintainability/empty-function");
    assert_eq!(violation, Some(()));
}

#[test]
fn reports_a_python_pass_body() {
    let facts = facts("src/example.py", "def empty():\n    pass");

    assert_eq!(
        EmptyFunction::evaluate(&facts.functions()[0], &configuration(&[])),
        Some(())
    );
}

#[test]
fn permits_an_explicitly_allowed_name() {
    let facts = facts("src/example.ts", "function intentionallyEmpty() {}");

    assert_eq!(
        EmptyFunction::evaluate(
            &facts.functions()[0],
            &configuration(&["intentionallyEmpty"])
        ),
        None
    );
}

#[test]
fn ignores_a_function_that_has_statements() {
    let facts = facts("src/example.js", "function active() {\n  work();\n}");

    assert_eq!(
        EmptyFunction::evaluate(&facts.functions()[0], &configuration(&[])),
        None
    );
}

#[test]
fn disables_evaluation_when_the_rule_is_off() {
    let facts = facts("src/example.rs", "fn empty() {}");
    let configuration = EmptyFunctionRule {
        severity: Severity::Off,
        allow_names: Vec::new(),
    };

    assert_eq!(
        EmptyFunction::evaluate(&facts.functions()[0], &configuration),
        None
    );
}
