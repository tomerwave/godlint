#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use godlint_core::{
    analyzers::workflow,
    source::{TextFile, Workflow},
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
    bot_conditions => "bot-conditions",
    bot_conditions_clean => "bot-conditions-clean",
    clean_repository => "clean",
    cognitive_complexity => "cognitive-complexity",
    cognitive_complexity_clean => "cognitive-complexity-clean",
    ci_no_comments => "ci-no-comments",
    ci_no_comments_clean => "ci-no-comments-clean",
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
    hardcoded_container_credentials => "hardcoded-container-credentials",
    hardcoded_container_credentials_clean => "hardcoded-container-credentials-clean",
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
    rule_scope => "rule-scope",
    restricted_call => "restricted-call",
    restricted_call_clean => "restricted-call-clean",
    no_dynamic_execution => "no-dynamic-execution",
    no_dynamic_execution_clean => "no-dynamic-execution-clean",
    network_timeout_required => "network-timeout-required",
    network_timeout_required_clean => "network-timeout-required-clean",
    no_control_flow_in_finally => "no-control-flow-in-finally",
    no_control_flow_in_finally_clean => "no-control-flow-in-finally-clean",
    redundant_catch_rethrow => "redundant-catch-rethrow",
    redundant_catch_rethrow_clean => "redundant-catch-rethrow-clean",
    no_committed_secret_file => "no-committed-secret-file",
    no_committed_secret_file_clean => "no-committed-secret-file-clean",
    no_commented_code => "no-commented-code",
    no_commented_code_clean => "no-commented-code-clean",
    no_duplicate_string => "no-duplicate-string",
    no_duplicate_string_clean => "no-duplicate-string-clean",
    direct_environment_read => "direct-environment-read",
    direct_environment_read_clean => "direct-environment-read-clean",
    explicit_timer_delay => "explicit-timer-delay",
    no_insecure_random => "no-insecure-random",
    assertion_required => "assertion-required",
    no_empty_test => "no-empty-test",
    no_focused_test => "no-focused-test",
    no_production_log => "no-production-log",
    no_internal_import => "no-internal-import",
    no_inline_script => "no-inline-script",
    no_inline_script_clean => "no-inline-script-clean",
    lockfile_version_drift => "lockfile-version-drift",
    lockfile_version_drift_clean => "lockfile-version-drift-clean",
    frozen_lockfile_install => "frozen-lockfile-install",
    frozen_lockfile_install_clean => "frozen-lockfile-install-clean",
    no_monolithic_job => "no-monolithic-job",
    no_monolithic_job_clean => "no-monolithic-job-clean",
    no_shell_command => "no-shell-command",
    no_skipped_test => "no-skipped-test",
    no_test_helper_in_production => "no-test-helper-in-production",
    no_sleep_in_test => "no-sleep-in-test",
    no_randomness_without_seed => "no-randomness-without-seed",
    no_network_in_unit_test => "no-network-in-unit-test",
    no_weak_hash => "no-weak-hash",
    dependency_boundary => "dependency-boundary",
    filename_case => "filename-case",
    filename_case_clean => "filename-case-clean",
    branch_naming => "branch-naming",
    branch_naming_clean => "branch-naming-clean",
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
    no_internal_import_clean => "no-internal-import-clean",
    no_shell_command_clean => "no-shell-command-clean",
    no_silenced_failure => "no-silenced-failure",
    no_silenced_failure_clean => "no-silenced-failure-clean",
    no_skipped_test_clean => "no-skipped-test-clean",
    no_test_helper_in_production_clean => "no-test-helper-in-production-clean",
    no_sleep_in_test_clean => "no-sleep-in-test-clean",
    no_randomness_without_seed_clean => "no-randomness-without-seed-clean",
    no_network_in_unit_test_clean => "no-network-in-unit-test-clean",
    no_weak_hash_clean => "no-weak-hash-clean",
    go_support => "go-support",
    explicit_timer_delay_clean => "explicit-timer-delay-clean",
    empty_error_handler_clean => "empty-error-handler-clean",
    explicit_workflow_permissions => "explicit-workflow-permissions",
    explicit_workflow_permissions_clean => "explicit-workflow-permissions-clean",
    secrets_inherit => "secrets-inherit",
    secrets_inherit_clean => "secrets-inherit-clean",
    stale_action_refs => "stale-action-refs",
    stale_action_refs_clean => "stale-action-refs-clean",
    overprovisioned_secrets => "overprovisioned-secrets",
    overprovisioned_secrets_clean => "overprovisioned-secrets-clean",
    unredacted_secrets => "unredacted-secrets",
    unredacted_secrets_clean => "unredacted-secrets-clean",
    untrusted_github_env => "untrusted-github-env",
    untrusted_github_env_clean => "untrusted-github-env-clean",
    pin_third_party_actions => "pin-third-party-actions",
    pin_third_party_actions_clean => "pin-third-party-actions-clean",
    rust_try_operator => "rust-try-operator",
    severity_below_threshold => "severity-below-threshold",
    suppression_applies => "suppression-applies",
    todo_requires_reference => "todo-requires-reference",
    template_injection => "template-injection",
    template_injection_clean => "template-injection-clean",
    unused_suppression => "unused-suppression",
    unused_suppression_clean => "unused-suppression-clean",
}

#[test]
fn every_workflow_fixture_is_yaml_that_github_would_accept() {
    let mut checked = 0;

    for path in workflow_fixtures(&fixtures_root()) {
        let relative = PathBuf::from(".github/workflows").join(
            path.file_name()
                .unwrap_or_else(|| panic!("{} has no name", path.display())),
        );
        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("reads {}: {error}", path.display()));
        let file = TextFile::new(relative, contents)
            .unwrap_or_else(|error| panic!("reads {}: {error}", path.display()));
        let facts = workflow::read(&file)
            .unwrap_or_else(|error| panic!("reads {}: {error}", path.display()));

        assert!(
            facts.unparsed().is_empty(),
            "{}: a fixture the grammar cannot read proves whatever the rule happens to do \
             with the wreckage, which is not the rule's behaviour",
            path.display()
        );
        checked += 1;
    }

    assert!(checked > 0, "no workflow fixtures were found to check");
}

fn workflow_fixtures(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();

    for fixture in fs::read_dir(root).unwrap_or_else(|error| panic!("reads fixtures: {error}")) {
        let directory = fixture
            .unwrap_or_else(|error| panic!("reads entry: {error}"))
            .path()
            .join(".github/workflows");

        if let Ok(entries) = fs::read_dir(&directory) {
            found.extend(
                entries
                    .filter_map(Result::ok)
                    .map(|entry| entry.path())
                    .filter(|path| Workflow::names(path)),
            );
        }
    }

    found
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
    let mut command = Command::new(env!("CARGO_BIN_EXE_godlint"));
    command.current_dir(fixture).args(["check", "."]);
    command.env_remove("GITHUB_HEAD_REF");

    if let Ok(branch) = fs::read_to_string(fixture.join("branch.txt")) {
        command.env("GITHUB_HEAD_REF", branch.trim());
    }

    command
        .output()
        .unwrap_or_else(|error| panic!("runs godlint: {error}"))
}
