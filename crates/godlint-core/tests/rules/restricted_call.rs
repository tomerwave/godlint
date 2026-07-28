use godlint_core::rules::{Violation, restricted_call};

use super::support::rule_violations;

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(restricted_call::evaluate, path, source, configuration)
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

#[test]
fn a_configured_name_keeps_the_macro_distinction() {
    let source = concat!(
        "fn dbg(value: u32) -> u32 {\n",
        "    value\n",
        "}\n",
        "\n",
        "pub fn run(v: u32) -> u32 {\n",
        "    dbg(v);\n",
        "    dbg!(v)\n",
        "}\n"
    );
    let allows_macro = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-call:\n",
        "    severity: error\n",
        "    calls:\n",
        "      - name: \"dbg!\"\n",
        "        allow-in:\n",
        "          - src/example.rs\n"
    );

    assert!(
        violations("src/example.rs", source, allows_macro).is_empty(),
        "naming the macro with its ! scopes the macro and leaves a function alone"
    );

    let restricts_function = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-call:\n",
        "    severity: error\n",
        "    calls:\n",
        "      - name: dbg\n"
    );

    assert_eq!(
        violations("src/example.rs", source, restricts_function).len(),
        2,
        "naming the function restricts it, and the macro stays restricted by default"
    );
}

#[test]
fn naming_a_built_in_scopes_it_to_the_language_that_defines_it() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-call:\n",
        "    severity: error\n",
        "    calls:\n",
        "      - name: print\n",
        "        allow-in:\n",
        "          - reporting/**\n"
    );

    assert_eq!(
        violations(
            "svc.py",
            "def emit(rows):\n    print(rows)\n",
            configuration
        )
        .len(),
        1,
        "Python print is restricted outside its boundary"
    );
    assert!(
        violations(
            "reporting/ok.py",
            "def emit(rows):\n    print(rows)\n",
            configuration
        )
        .is_empty(),
        "and allowed inside it"
    );
    assert!(
        violations(
            "widget.ts",
            "function print(m: string): string {\n  return m;\n}\nexport const go = () => print(\"x\");\n",
            configuration
        )
        .is_empty(),
        "scoping Python's built-in must not restrict a TypeScript function of that name"
    );
}

#[test]
fn a_name_that_is_not_a_built_in_applies_to_every_language() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  architecture/restricted-call:\n",
        "    severity: error\n",
        "    calls:\n",
        "      - name: loadConfig\n",
        "        allow-in:\n",
        "          - config/**\n"
    );

    for (path, source) in [
        ("svc.ts", "export function b(){ return loadConfig(); }\n"),
        ("svc.py", "def c():\n    return loadConfig()\n"),
        ("svc.rs", "pub fn d() -> u32 { loadConfig() }\n"),
    ] {
        assert_eq!(
            violations(path, source, configuration).len(),
            1,
            "{path}: a callee the project names is restricted wherever it is called"
        );
    }
}
