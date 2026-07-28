#![allow(clippy::expect_used, clippy::unwrap_used)]

//! Fixture-driven end-to-end checks.
//!
//! `expected.yaml` records the literal output the CLI must produce. Deriving it from a
//! copy of the production format string would make this a second implementation of the
//! contract rather than an expectation of it, and a consistent change to both would keep
//! the fixtures green while the documented output changed.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ExpectedResult {
    #[serde(rename = "exit-code")]
    exit_code: i32,
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    stderr: String,
}

/// Declares one test per fixture so a failure names the rule and can be run alone.
macro_rules! fixture_tests {
    ($($name:ident => $directory:literal),+ $(,)?) => {
        /// Every declared fixture, used to prove none is left untested.
        const DECLARED: &[&str] = &[$($directory),+];

        $(
            #[test]
            fn $name() {
                assert_fixture($directory);
            }
        )+
    };
}

fixture_tests! {
    clean_repository => "clean",
    cyclomatic_complexity => "cyclomatic-complexity",
    documented_empty_body => "documented-empty-body",
    else_if_chain => "else-if-chain",
    empty_function => "empty-function",
    excluded_path => "excluded-path",
    file_size => "file-size",
    function_nesting => "function-nesting",
    function_size => "function-size",
    function_statements => "function-statements",
    invalid_syntax => "invalid-syntax",
    marker_word_boundary => "marker-word-boundary",
    parameter_count => "parameter-count",
    receiver_parameters => "receiver-parameters",
    return_count => "return-count",
    rust_try_operator => "rust-try-operator",
    severity_below_threshold => "severity-below-threshold",
    todo_requires_reference => "todo-requires-reference",
}

/// Guards against a fixture directory that no test exercises.
#[test]
fn every_fixture_directory_is_declared() {
    let present: BTreeSet<String> = fs::read_dir(fixtures_root())
        .unwrap_or_else(|error| panic!("reads fixtures: {error}"))
        .map(|entry| entry.unwrap_or_else(|error| panic!("reads entry: {error}")))
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    let declared: BTreeSet<String> = DECLARED.iter().map(|name| (*name).to_owned()).collect();

    assert_eq!(present, declared, "fixture directories and tests disagree");
}

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/rules")
}

fn assert_fixture(directory: &str) {
    let fixture = fixtures_root().join(directory);
    let expected = expected_result(&fixture);
    let output = run(&fixture);

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        expected.stdout,
        "{directory}: stdout"
    );
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        expected.stderr,
        "{directory}: stderr"
    );
    assert_eq!(
        output.status.code(),
        Some(expected.exit_code),
        "{directory}: exit code"
    );
}

fn expected_result(fixture: &Path) -> ExpectedResult {
    let path = fixture.join("expected.yaml");
    let source = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reads {}: {error}", path.display()));

    yaml_serde::from_str(&source)
        .unwrap_or_else(|error| panic!("parses {}: {error}", path.display()))
}

fn run(fixture: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_godlint"))
        .current_dir(fixture)
        .args(["check", "."])
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"))
}
