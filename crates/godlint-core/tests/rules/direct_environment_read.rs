use godlint_core::rules::{Violation, direct_environment_read};

use super::support::rule_violations;

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(
        direct_environment_read::evaluate,
        path,
        source,
        configuration,
    )
}

#[test]
fn restricts_direct_environment_reads_in_every_supported_language() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/direct-environment-read:\n",
        "    severity: error\n"
    );

    assert_eq!(
        violations("src/service.ts", "process.env.PORT;", configuration).len(),
        1
    );
    assert_eq!(
        violations("src/service.py", "os.getenv('PORT')", configuration).len(),
        1
    );
    assert_eq!(
        violations(
            "src/service.rs",
            "fn port() { let _ = std::env::var(\"PORT\"); }",
            configuration
        )
        .len(),
        1
    );
}

#[test]
fn permits_the_default_config_boundary() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/direct-environment-read:\n",
        "    severity: error\n"
    );

    assert!(violations("src/config.ts", "process.env.PORT;", configuration).is_empty());
    assert!(violations("src/config/runtime.py", "os.environ['PORT']", configuration).is_empty());
}

#[test]
fn permits_a_configured_config_boundary() {
    let configuration = "version: 1\nrules:\n  security/direct-environment-read:\n    severity: error\n    allow-in:\n      - 'src/bootstrap.*'\n";

    assert!(violations("src/bootstrap.ts", "process.env.PORT;", configuration).is_empty());
}

#[test]
fn can_disable_direct_environment_read_policy() {
    let configuration =
        "version: 1\nrules:\n  security/direct-environment-read:\n    severity: off\n";

    assert!(violations("src/service.py", "os.environ['PORT']", configuration).is_empty());
}

#[test]
fn stays_silent_until_a_repository_asks_for_it() {
    let source = concat!(
        "import os\n",
        "\n",
        "def url():\n",
        "    return os.environ[\"URL\"]\n"
    );

    assert!(
        violations("src/example.py", source, "version: 1\n").is_empty(),
        "a rule absent from configuration must do nothing"
    );
}

#[test]
fn javascript_reads_the_environment_through_an_access_not_a_call() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/direct-environment-read:\n",
        "    severity: error\n"
    );

    assert!(
        violations(
            "src/example.ts",
            "export function go(): string {\n  return getenv(\"A\");\n}\n",
            configuration
        )
        .is_empty(),
        "no JavaScript or TypeScript call reads the environment directly"
    );
}
