#![allow(clippy::expect_used, clippy::unwrap_used)]

use godlint_core::config::{Config, ConfigError};

#[path = "support/temporary.rs"]
mod temporary;

use temporary::TemporaryDirectory;

fn load(contents: &str) -> Result<Config, ConfigError> {
    let directory = TemporaryDirectory::new("config");

    Config::load(directory.write("godlint.yaml", contents))
}

#[test]
fn accepts_the_function_size_rule() {
    let result = load(
        "version: 1\nrules:\n  maintainability/function-size:\n    severity: error\n    max-lines: 30\n    skip-blank-lines: true\n    skip-comments: true\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_function_nesting_rule() {
    let result = load(
        "version: 1\nrules:\n  maintainability/function-nesting:\n    severity: error\n    max-depth: 0\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_file_size_rule() {
    let result = load(
        "version: 1\nrules:\n  maintainability/file-size:\n    severity: warning\n    max-lines: 500\n    skip-blank-lines: true\n    skip-comments: true\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_empty_function_rule() {
    let result = load(
        "version: 1\nrules:\n  maintainability/empty-function:\n    severity: warning\n    allow-names:\n      - intentionallyEmpty\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_unused_suppression_rule() {
    let configuration = "version: 1\nrules:\n  policy/unused-suppression:\n    severity: error\n";

    assert!(load(configuration).is_ok());
}

#[test]
fn accepts_the_todo_reference_rule() {
    let result = load(
        "version: 1\nrules:\n  policy/todo-requires-reference:\n    severity: warning\n    reference-prefixes:\n      - GH-\n      - '#'\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_parameter_count_rule() {
    let result = load(
        "version: 1\nrules:\n  maintainability/parameter-count:\n    severity: warning\n    max-parameters: 6\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_decision_complexity_rule() {
    let result = load(
        "version: 1\nrules:\n  maintainability/decision-complexity:\n    severity: warning\n    max-complexity: 10\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_return_count_rule() {
    let result = load(
        "version: 1\nrules:\n  maintainability/return-count:\n    severity: warning\n    max-returns: 3\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_function_statements_rule() {
    let result = load(
        "version: 1\nrules:\n  maintainability/function-statements:\n    severity: warning\n    max-statements: 30\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_restricted_call_rule() {
    let result = load(
        "version: 1\nrules:\n  architecture/restricted-call:\n    severity: error\n    calls:\n      - name: loadConfig\n        allow-in:\n          - '**/config.*'\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_dynamic_execution_rule() {
    let result =
        load("version: 1\nrules:\n  security/no-dynamic-execution:\n    severity: warning\n");

    assert!(result.is_ok());
}

#[test]
fn accepts_the_direct_environment_read_rule() {
    let result = load(
        "version: 1\nrules:\n  security/direct-environment-read:\n    severity: error\n    allow-in:\n      - 'services/**/settings.*'\n",
    );

    assert!(result.is_ok());
}

#[test]
fn accepts_the_explicit_timer_delay_rule() {
    let result =
        load("version: 1\nrules:\n  reliability/explicit-timer-delay:\n    severity: error\n");

    assert!(result.is_ok());
}

#[test]
fn accepts_template_injection_path_exceptions() {
    let config = load(concat!(
        "version: 1\n",
        "rules:\n",
        "  ci/template-injection:\n",
        "    severity: error\n",
        "    allow-in:\n",
        "      - '.github/workflows/generated.yml'\n",
    ))
    .unwrap_or_else(|error| panic!("loads: {error}"));

    assert_eq!(
        config
            .rules
            .template_injection
            .as_ref()
            .expect("template injection")
            .allow_in,
        vec![".github/workflows/generated.yml"]
    );
}

#[test]
fn accepts_no_silenced_failure() {
    let result = load("version: 1\nrules:\n  ci/no-silenced-failure:\n    severity: error\n");

    assert!(result.is_ok());
}

#[test]
fn accepts_no_monolithic_job_path_exceptions() {
    let config = load(concat!(
        "version: 1\n",
        "rules:\n",
        "  ci/no-monolithic-job:\n",
        "    severity: error\n",
        "    max-steps: 7\n",
        "    allow-in:\n",
        "      - '.github/workflows/release.yml'\n",
    ))
    .unwrap_or_else(|error| panic!("loads: {error}"));
    let rule = config
        .rules
        .no_monolithic_job
        .as_ref()
        .expect("no monolithic job");

    assert_eq!(rule.limit(), 7);
    assert_eq!(rule.allow_in, vec![".github/workflows/release.yml"]);
}

#[test]
fn bot_conditions_defaults_to_common_bot_identities() {
    let config = load("version: 1\nrules:\n  ci/bot-conditions:\n    severity: error\n")
        .unwrap_or_else(|error| panic!("loads: {error}"));

    assert_eq!(
        config
            .rules
            .bot_conditions
            .as_ref()
            .expect("bot conditions")
            .bots,
        vec!["dependabot[bot]", "github-actions[bot]", "renovate[bot]"]
    );
}

#[test]
fn accepts_the_empty_error_handler_rule() {
    let result =
        load("version: 1\nrules:\n  reliability/empty-error-handler:\n    severity: error\n");

    assert!(result.is_ok());
}

#[test]
fn ignores_an_unknown_rule_and_says_which() {
    let config = load("version: 1\nrules:\n  maintainability/unknown: {}\n")
        .unwrap_or_else(|error| panic!("a rule this version lacks must not stop a run: {error}"));
    let unrecognised: Vec<&str> = config.rules.unrecognised().collect();

    assert_eq!(unrecognised, vec!["maintainability/unknown"]);
}

#[test]
fn an_unknown_rule_does_not_silence_the_rules_beside_it() {
    let config = load(concat!(
        "version: 1\n",
        "rules:\n",
        "  maintainability/unknown: {}\n",
        "  maintainability/file-size:\n",
        "    severity: error\n",
        "    max-lines: 10\n",
    ))
    .unwrap_or_else(|error| panic!("loads: {error}"));

    assert!(
        config.rules.file_size.is_some(),
        "a newer key must cost only itself"
    );
}

#[test]
fn a_rule_with_an_unknown_option_is_still_refused() {
    let result = load(concat!(
        "version: 1\n",
        "rules:\n",
        "  maintainability/file-size:\n",
        "    severity: error\n",
        "    max-lines: 10\n",
        "    max-line: 10\n",
    ));

    assert!(
        matches!(result, Err(ConfigError::Parse { .. })),
        "an option on a rule that does exist decides how it behaves, so a name it does not \
         know must not be ignored"
    );
}

#[test]
fn rejects_an_unknown_top_level_field() {
    let result = load("version: 1\nunknown: true\n");

    assert!(matches!(result, Err(ConfigError::Parse { .. })));
}

#[test]
fn rejects_an_unsupported_version() {
    let result = load("version: 2\n");

    assert!(matches!(
        result,
        Err(ConfigError::UnsupportedVersion { version: 2 })
    ));
}

#[test]
fn rejects_a_zero_function_size_limit() {
    let result = load(
        "version: 1\nrules:\n  maintainability/function-size:\n    severity: error\n    max-lines: 0\n    skip-blank-lines: true\n    skip-comments: true\n",
    );

    assert!(matches!(result, Err(ConfigError::Parse { .. })));
}

#[test]
fn rejects_a_zero_file_size_limit() {
    let result = load(
        "version: 1\nrules:\n  maintainability/file-size:\n    severity: error\n    max-lines: 0\n    skip-blank-lines: true\n    skip-comments: true\n",
    );

    assert!(matches!(result, Err(ConfigError::Parse { .. })));
}

#[test]
fn rejects_empty_todo_reference_prefixes() {
    let result = load(
        "version: 1\nrules:\n  policy/todo-requires-reference:\n    severity: error\n    reference-prefixes: []\n",
    );

    assert!(matches!(
        result,
        Err(ConfigError::InvalidTodoReferencePrefixes)
    ));
}

#[test]
fn rejects_blank_todo_reference_prefixes() {
    let result = load(
        "version: 1\nrules:\n  policy/todo-requires-reference:\n    severity: error\n    reference-prefixes:\n      - ' '\n",
    );

    assert!(matches!(
        result,
        Err(ConfigError::InvalidTodoReferencePrefixes)
    ));
}

#[test]
fn rejects_a_blank_restricted_call_name() {
    let result = load(
        "version: 1\nrules:\n  architecture/restricted-call:\n    severity: error\n    calls:\n      - name: ' '\n",
    );

    assert!(matches!(
        result,
        Err(ConfigError::InvalidRestrictedCallName)
    ));
}

#[test]
fn rejects_a_blank_test_helper_name() {
    let result = load(
        "version: 1\nrules:\n  testing/no-test-helper-in-production:\n    severity: error\n    helpers:\n      - ' '\n",
    );

    assert!(matches!(
        result,
        Err(ConfigError::BlankAllowIn {
            rule: "testing/no-test-helper-in-production"
        })
    ));
}

#[test]
fn rejects_a_blank_test_helper_path() {
    let result = load(
        "version: 1\nrules:\n  testing/no-test-helper-in-production:\n    severity: error\n    test-paths:\n      - ' '\n",
    );

    assert!(matches!(
        result,
        Err(ConfigError::BlankAllowIn {
            rule: "testing/no-test-helper-in-production"
        })
    ));
}

#[test]
fn rejects_a_blank_restricted_call_allow_in_path() {
    let result = load(
        "version: 1\nrules:\n  architecture/restricted-call:\n    severity: error\n    calls:\n      - name: loadConfig\n        allow-in:\n          - ' '\n",
    );

    assert!(matches!(
        result,
        Err(ConfigError::BlankAllowIn {
            rule: "architecture/restricted-call"
        })
    ));
}

#[test]
fn rejects_a_blank_direct_environment_read_allow_in_path() {
    let result = load(
        "version: 1\nrules:\n  security/direct-environment-read:\n    severity: error\n    allow-in:\n      - ' '\n",
    );

    assert!(matches!(
        result,
        Err(ConfigError::BlankAllowIn {
            rule: "security/direct-environment-read"
        })
    ));
}

#[test]
fn rejects_a_restricted_call_listed_twice() {
    let result = load(concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-call:\n",
        "    severity: error\n",
        "    calls:\n",
        "      - name: console.log\n",
        "        allow-in:\n",
        "          - a.ts\n",
        "      - name: console.log\n",
        "        allow-in:\n",
        "          - b.ts\n"
    ));

    let Err(error) = result else {
        panic!("two entries for one callee leave its boundary ambiguous");
    };
    let message = error.to_string();

    assert!(
        message.contains("console.log") && message.contains("more than once"),
        "{message}"
    );
}
