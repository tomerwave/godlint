use godlint_core::{
    config::Config,
    rules::{Violation, restricted_call},
};

use super::support::facts;

fn config(body: &str) -> Config {
    yaml_serde::from_str(body).unwrap_or_else(|error| panic!("reads configuration: {error}"))
}

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    restricted_call::evaluate(&[facts(path, source)], &config(configuration))
        .unwrap_or_else(|error| panic!("evaluates restricted calls: {error}"))
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

#[test]
fn restricts_default_exit_and_debug_calls_in_every_supported_language() {
    let configuration = "version: 1\n";

    assert_eq!(
        violations(
            "src/exit.ts",
            "process.exit(1);\nconsole.log('debug');",
            configuration
        )
        .len(),
        2
    );
    assert_eq!(
        violations("src/exit.py", "sys.exit(1)\nprint('debug')", configuration).len(),
        2
    );
    assert_eq!(
        violations(
            "src/exit.rs",
            "fn main() {\n    std::process::exit(1);\n    dbg!(1);\n}",
            configuration
        )
        .len(),
        2
    );
}

#[test]
fn permits_calls_that_are_not_default_restrictions() {
    let configuration = "version: 1\n";

    assert!(
        violations(
            "src/output.rs",
            "fn main() { println!(\"ok\"); }",
            configuration
        )
        .is_empty()
    );
}

#[test]
fn restricts_configured_calls_outside_their_boundary() {
    let configuration = "version: 1\nrules:\n  architecture/restricted-call:\n    severity: error\n    calls:\n      - name: loadConfig\n        allow-in:\n          - '**/config.*'\n";

    assert_eq!(
        violations("src/service.ts", "loadConfig();", configuration).len(),
        1
    );
    assert!(violations("src/config.ts", "loadConfig();", configuration).is_empty());
}

#[test]
fn can_disable_default_restrictions() {
    let configuration = "version: 1\nrules:\n  architecture/restricted-call:\n    severity: off\n";

    assert!(violations("src/exit.ts", "process.exit(1);", configuration).is_empty());
}
