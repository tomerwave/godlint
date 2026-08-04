use godlint_core::{
    config::{Severity, TodoRequiresReferenceRule},
    rules::{Rule, Violation, todo_requires_reference::TodoRequiresReference},
};

use super::support::{comment_violations, facts};

fn configuration(markers: &[&str], prefixes: &[&str]) -> TodoRequiresReferenceRule {
    TodoRequiresReferenceRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        markers: markers.iter().map(|marker| (*marker).into()).collect(),
        reference_prefixes: prefixes.iter().map(|prefix| (*prefix).into()).collect(),
    }
}

fn violations(path: &str, source: &str, prefixes: &[&str]) -> Vec<Violation> {
    comment_violations::<TodoRequiresReference>(
        path,
        source,
        &configuration(&["TODO", "FIXME"], prefixes),
    )
}

fn markers_reported(path: &str, source: &str, prefixes: &[&str]) -> Vec<String> {
    violations(path, source, prefixes)
        .into_iter()
        .map(|violation| match violation {
            Violation::MissingReference { marker } => marker,
            other => panic!("unexpected violation: {other:?}"),
        })
        .collect()
}

#[test]
fn reports_a_marker_without_a_reference() {
    assert_eq!(TodoRequiresReference::ID, "policy/todo-requires-reference");
    assert_eq!(
        markers_reported("src/example.rs", "// TODO: implement this", &["#"]),
        vec!["TODO".to_owned()]
    );
}

#[test]
fn accepts_a_referenced_marker() {
    assert!(markers_reported("src/example.rs", "// TODO: implement this #123", &["#"]).is_empty());
}

#[test]
fn reports_every_configured_marker() {
    assert_eq!(
        markers_reported("src/example.rs", "// FIXME: broken", &["#"]),
        vec!["FIXME".to_owned()]
    );
}

#[test]
fn requires_a_whole_word_marker() {
    assert!(
        markers_reported(
            "src/example.rs",
            "// AUTODOWNLOAD_ENABLED controls the cache\n// The METODOLOGY note is fine\n// TODOS remain",
            &["#"]
        )
        .is_empty()
    );
}

#[test]
fn rejects_a_reference_that_is_not_an_issue_number() {
    assert_eq!(
        markers_reported(
            "src/example.rs",
            "// TODO: use the accent colour #3366ff",
            &["#"]
        ),
        vec!["TODO".to_owned()]
    );
}

#[test]
fn requires_a_prefix_to_start_a_word() {
    assert_eq!(
        markers_reported("src/example.rs", "// TODO: see NOTJIRA-42", &["JIRA-"]),
        vec!["TODO".to_owned()]
    );
}

#[test]
fn accepts_a_custom_prefix() {
    assert!(markers_reported("src/example.py", "# TODO: implement GH-123", &["GH-"]).is_empty());
}

#[test]
fn rejects_a_reference_using_an_unconfigured_prefix() {
    assert_eq!(
        markers_reported("src/example.py", "# TODO: implement #123", &["GH-"]),
        vec!["TODO".to_owned()]
    );
}

#[test]
fn honours_more_than_one_prefix() {
    assert!(markers_reported("src/example.rs", "// TODO: see GH-7", &["#", "GH-"]).is_empty());
    assert!(markers_reported("src/example.rs", "// TODO: see #7", &["#", "GH-"]).is_empty());
}

#[test]
fn scopes_a_reference_to_the_marker_that_precedes_it() {
    assert_eq!(
        markers_reported(
            "src/example.ts",
            "/*\n * TODO: first thing #12\n * TODO: second thing\n */",
            &["#"]
        ),
        vec!["TODO".to_owned()]
    );
}

#[test]
fn reads_a_python_docstring() {
    assert_eq!(
        markers_reported(
            "src/example.py",
            "def example():\n    \"\"\"TODO: implement this.\"\"\"",
            &["#"]
        ),
        vec!["TODO".to_owned()]
    );
}

#[test]
fn ignores_a_marker_in_a_shebang() {
    assert!(
        markers_reported(
            "src/example.py",
            "#!/usr/bin/env python3 TODO\ndef f():\n    pass\n",
            &["#"]
        )
        .is_empty()
    );
}

#[test]
fn ignores_a_marker_inside_a_string_literal() {
    let facts = facts("src/example.js", "const message = 'TODO: implement this';");

    assert!(facts.comments().is_empty());
}
