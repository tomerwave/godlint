use godlint_core::rules::{Violation, no_dynamic_execution};

use super::support::rule_violations;

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(no_dynamic_execution::evaluate, path, source, configuration)
}

#[test]
fn restricts_dynamic_execution_calls() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/no-dynamic-execution:\n",
        "    severity: error\n"
    );

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
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/no-dynamic-execution:\n",
        "    severity: error\n"
    );

    assert!(violations("src/example.ts", "evaluate(input);", configuration).is_empty());
    assert!(violations("src/example.rs", "fn eval() {}", configuration).is_empty());
}

#[test]
fn can_disable_dynamic_execution_policy() {
    let configuration = "version: 1\nrules:\n  security/no-dynamic-execution:\n    severity: off\n";

    assert!(violations("src/example.py", "eval(input)", configuration).is_empty());
}

#[test]
fn stays_silent_until_a_repository_asks_for_it() {
    assert!(
        violations(
            "src/example.py",
            "def run(expression):\n    return eval(expression)\n",
            "version: 1\n"
        )
        .is_empty(),
        "a rule absent from configuration must do nothing"
    );
}

#[test]
fn rust_has_no_dynamic_execution_form() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/no-dynamic-execution:\n",
        "    severity: error\n"
    );

    assert!(
        violations(
            "src/example.rs",
            "pub fn run() -> u32 {\n    eval(1)\n}\n",
            configuration
        )
        .is_empty(),
        "Rust has no eval, so a function named eval is ordinary code"
    );
}

#[test]
fn a_constructed_function_is_dynamic_execution() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/no-dynamic-execution:\n",
        "    severity: error\n"
    );

    assert_eq!(
        violations(
            "src/example.js",
            "const f = new Function(\"return 1\");\n",
            configuration
        )
        .len(),
        1,
        "new Function is the idiomatic spelling and must be caught like a bare call"
    );
}
