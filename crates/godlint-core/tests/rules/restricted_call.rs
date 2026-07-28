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
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-call:\n",
        "    severity: error\n"
    );

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
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-call:\n",
        "    severity: error\n"
    );

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

#[test]
fn stays_silent_until_a_repository_asks_for_it() {
    assert!(
        violations(
            "src/example.py",
            "def render(rows):\n    print(rows)\n",
            "version: 1\n"
        )
        .is_empty(),
        "a rule absent from configuration must do nothing, whatever its defaults"
    );
}

#[test]
fn an_allow_in_entry_scopes_a_default_restriction() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-call:\n",
        "    severity: error\n",
        "    calls:\n",
        "      - name: console.log\n",
        "        allow-in:\n",
        "          - logger.ts\n"
    );

    assert!(
        violations(
            "logger.ts",
            "export function log(m: string): void {\n  console.log(m);\n}\n",
            configuration
        )
        .is_empty(),
        "naming a built-in restriction must let its allow-in boundary apply"
    );
    assert_eq!(
        violations(
            "service.ts",
            "export function go(): void {\n  console.log(\"x\");\n}\n",
            configuration
        )
        .len(),
        1,
        "the same restriction still applies outside its allow-in boundary"
    );
}

#[test]
fn a_function_is_not_the_macro_that_shares_its_name() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-call:\n",
        "    severity: error\n"
    );
    let source = concat!(
        "fn dbg(value: u32) -> u32 {\n",
        "    value\n",
        "}\n",
        "\n",
        "pub fn run() -> u32 {\n",
        "    dbg(1)\n",
        "}\n"
    );

    assert!(
        violations("src/example.rs", source, configuration).is_empty(),
        "a function named dbg is not the dbg! macro"
    );
    assert_eq!(
        violations(
            "src/macro.rs",
            "pub fn run() -> u32 {\n    dbg!(2)\n}\n",
            configuration
        )
        .len(),
        1,
        "the macro is still restricted"
    );
}
