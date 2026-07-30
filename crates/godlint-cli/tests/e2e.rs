#![allow(clippy::expect_used, clippy::unwrap_used)]

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

macro_rules! fixture_tests {
    ($($name:ident => $directory:literal),+ $(,)?) => {
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
    accountable_suppression => "accountable-suppression",
    clean_repository => "clean",
    cognitive_complexity => "cognitive-complexity",
    cognitive_complexity_clean => "cognitive-complexity-clean",
    condition_complexity => "condition-complexity",
    condition_complexity_clean => "condition-complexity-clean",
    decision_complexity => "decision-complexity",
    documented_empty_body => "documented-empty-body",
    else_if_chain => "else-if-chain",
    empty_function => "empty-function",
    empty_error_handler => "empty-error-handler",
    enclosing_scope => "enclosing-scope",
    excluded_path => "excluded-path",
    file_size => "file-size",
    function_nesting => "function-nesting",
    function_size => "function-size",
    function_statements => "function-statements",
    invalid_syntax => "invalid-syntax",
    marker_word_boundary => "marker-word-boundary",
    module_independence => "module-independence",
    module_independence_clean => "module-independence-clean",
    multiway_branch => "multiway-branch",
    no_comments => "no-comments",
    no_comments_strict => "no-comments-strict",
    parameter_count => "parameter-count",
    receiver_parameters => "receiver-parameters",
    return_count => "return-count",
    restricted_call => "restricted-call",
    restricted_call_clean => "restricted-call-clean",
    no_dynamic_execution => "no-dynamic-execution",
    no_dynamic_execution_clean => "no-dynamic-execution-clean",
    direct_environment_read => "direct-environment-read",
    direct_environment_read_clean => "direct-environment-read-clean",
    explicit_timer_delay => "explicit-timer-delay",
    no_insecure_random => "no-insecure-random",
    assertion_required => "assertion-required",
    no_empty_test => "no-empty-test",
    no_focused_test => "no-focused-test",
    no_production_log => "no-production-log",
    no_shell_command => "no-shell-command",
    no_skipped_test => "no-skipped-test",
    no_sleep_in_test => "no-sleep-in-test",
    no_randomness_without_seed => "no-randomness-without-seed",
    no_network_in_unit_test => "no-network-in-unit-test",
    no_weak_hash => "no-weak-hash",
    dependency_boundary => "dependency-boundary",
    filename_case => "filename-case",
    filename_case_clean => "filename-case-clean",
    forbidden_dependency => "forbidden-dependency",
    forbidden_dependency_clean => "forbidden-dependency-clean",
    dependency_boundary_clean => "dependency-boundary-clean",
    restricted_import => "restricted-import",
    restricted_import_clean => "restricted-import-clean",
    no_insecure_random_clean => "no-insecure-random-clean",
    assertion_required_clean => "assertion-required-clean",
    no_empty_test_clean => "no-empty-test-clean",
    no_focused_test_clean => "no-focused-test-clean",
    no_production_log_clean => "no-production-log-clean",
    no_shell_command_clean => "no-shell-command-clean",
    no_skipped_test_clean => "no-skipped-test-clean",
    no_sleep_in_test_clean => "no-sleep-in-test-clean",
    no_randomness_without_seed_clean => "no-randomness-without-seed-clean",
    no_network_in_unit_test_clean => "no-network-in-unit-test-clean",
    no_weak_hash_clean => "no-weak-hash-clean",
    explicit_timer_delay_clean => "explicit-timer-delay-clean",
    empty_error_handler_clean => "empty-error-handler-clean",
    rust_try_operator => "rust-try-operator",
    severity_below_threshold => "severity-below-threshold",
    suppression_applies => "suppression-applies",
    todo_requires_reference => "todo-requires-reference",
    unused_suppression => "unused-suppression",
    unused_suppression_clean => "unused-suppression-clean",
}

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
