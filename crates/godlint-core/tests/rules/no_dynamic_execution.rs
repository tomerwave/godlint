use godlint_core::{
    config::Config,
    rules::{Violation, no_dynamic_execution},
};

use super::support::facts;

fn config(body: &str) -> Config {
    yaml_serde::from_str(body).unwrap_or_else(|error| panic!("reads configuration: {error}"))
}

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    no_dynamic_execution::evaluate(&[facts(path, source)], &config(configuration))
        .unwrap_or_else(|error| panic!("evaluates dynamic execution: {error}"))
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

#[test]
fn restricts_dynamic_execution_calls() {
    let configuration = "version: 1\n";

    assert_eq!(
        violations("src/example.ts", "eval(input);", configuration).len(),
        1
    );
    assert_eq!(
        violations("src/example.ts", "Function(input);", configuration).len(),
        1
    );
    assert_eq!(
        violations("src/example.py", "exec(input)", configuration).len(),
        1
    );
}

#[test]
fn ignores_unrelated_calls_and_rust_names() {
    let configuration = "version: 1\n";

    assert!(violations("src/example.ts", "evaluate(input);", configuration).is_empty());
    assert!(violations("src/example.rs", "fn eval() {}", configuration).is_empty());
}

#[test]
fn can_disable_dynamic_execution_policy() {
    let configuration = "version: 1\nrules:\n  security/no-dynamic-execution:\n    severity: off\n";

    assert!(violations("src/example.py", "eval(input)", configuration).is_empty());
}
