use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use godlint_core::config::{Config, ConfigError};

static NEXT_CONFIG_ID: AtomicU64 = AtomicU64::new(0);

fn config_file(contents: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let id = NEXT_CONFIG_ID.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("godlint-config-{timestamp}-{id}.yaml"));

    fs::write(&path, contents).unwrap_or_else(|error| panic!("writes config: {error}"));

    path
}

fn load(contents: &str) -> Result<Config, ConfigError> {
    let path = config_file(contents);
    let result = Config::load(&path);

    fs::remove_file(path).unwrap_or_else(|error| panic!("removes config: {error}"));

    result
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
fn accepts_the_cyclomatic_complexity_rule() {
    let result = load(
        "version: 1\nrules:\n  maintainability/cyclomatic-complexity:\n    severity: warning\n    max-complexity: 10\n",
    );

    assert!(result.is_ok());
}

#[test]
fn rejects_an_unknown_rule() {
    let result = load("version: 1\nrules:\n  maintainability/unknown: {}\n");

    assert!(matches!(result, Err(ConfigError::Parse { .. })));
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

    assert!(matches!(result, Err(ConfigError::InvalidFunctionSizeLimit)));
}

#[test]
fn rejects_a_zero_file_size_limit() {
    let result = load(
        "version: 1\nrules:\n  maintainability/file-size:\n    severity: error\n    max-lines: 0\n    skip-blank-lines: true\n    skip-comments: true\n",
    );

    assert!(matches!(result, Err(ConfigError::InvalidFileSizeLimit)));
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
