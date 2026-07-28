use godlint_core::{
    config::Config,
    rules::{Violation, direct_environment_read},
};

use super::support::facts;

fn config(body: &str) -> Config {
    yaml_serde::from_str(body).unwrap_or_else(|error| panic!("reads configuration: {error}"))
}

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    direct_environment_read::evaluate(&[facts(path, source)], &config(configuration))
        .unwrap_or_else(|error| panic!("evaluates environment reads: {error}"))
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

#[test]
fn restricts_direct_environment_reads_in_every_supported_language() {
    let configuration = "version: 1\n";

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
    let configuration = "version: 1\n";

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
