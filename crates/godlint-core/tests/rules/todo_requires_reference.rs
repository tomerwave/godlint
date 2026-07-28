use std::path::PathBuf;

use godlint_core::{
    analyzers::analyze,
    config::{Severity, TodoRequiresReferenceRule},
    rules::{Rule, todo_requires_reference::TodoRequiresReference},
    source::SourceFile,
};

fn comments(path: &str, source: &str) -> godlint_core::analyzers::SourceFacts {
    let source = SourceFile::new(PathBuf::from(path), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));

    analyze(&source).unwrap_or_else(|error| panic!("analyzes source: {error}"))
}

fn configuration(prefixes: &[&str]) -> TodoRequiresReferenceRule {
    TodoRequiresReferenceRule {
        severity: Severity::Error,
        reference_prefixes: prefixes.iter().map(|prefix| (*prefix).into()).collect(),
    }
}

#[test]
fn reports_a_todo_without_a_reference() {
    let facts = comments("src/example.rs", "// TODO: implement this");
    let violation = TodoRequiresReference::evaluate(&facts.comments()[0], &configuration(&["#"]));

    assert_eq!(TodoRequiresReference::ID, "policy/todo-requires-reference");
    assert_eq!(violation, Some(()));
}

#[test]
fn accepts_a_default_issue_reference() {
    let facts = comments("src/example.ts", "// TODO: implement this #123");

    assert_eq!(
        TodoRequiresReference::evaluate(&facts.comments()[0], &configuration(&["#"])),
        None
    );
}

#[test]
fn accepts_a_block_comment_reference() {
    let facts = comments("src/example.ts", "/* TODO: implement this #123 */");

    assert_eq!(
        TodoRequiresReference::evaluate(&facts.comments()[0], &configuration(&["#"])),
        None
    );
}

#[test]
fn accepts_a_custom_issue_reference() {
    let facts = comments("src/example.py", "# TODO: implement this GH-123");

    assert_eq!(
        TodoRequiresReference::evaluate(&facts.comments()[0], &configuration(&["GH-"])),
        None
    );
}

#[test]
fn ignores_a_todo_in_a_string() {
    let facts = comments("src/example.js", "const message = 'TODO: implement this';");

    assert!(facts.comments().is_empty());
}

#[test]
fn disables_evaluation_when_the_rule_is_off() {
    let facts = comments("src/example.rs", "// TODO: implement this");
    let configuration = TodoRequiresReferenceRule {
        severity: Severity::Off,
        reference_prefixes: vec!["#".into()],
    };

    assert_eq!(
        TodoRequiresReference::evaluate(&facts.comments()[0], &configuration),
        None
    );
}
