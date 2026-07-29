use godlint_core::rules::{Violation, empty_error_handler};

use super::support::rule_violations;

const ENABLED: &str =
    "version: 1\nrules:\n  reliability/empty-error-handler:\n    severity: error\n";

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(empty_error_handler::evaluate, path, source, configuration)
}

#[test]
fn reports_empty_handlers_in_javascript_typescript_and_python() {
    assert_eq!(
        violations("src/example.js", "try { work(); } catch { }", ENABLED),
        vec![Violation::EmptyErrorHandler]
    );
    assert_eq!(
        violations(
            "src/example.ts",
            "try { work(); } catch (error) { }",
            ENABLED
        ),
        vec![Violation::EmptyErrorHandler]
    );
    assert_eq!(
        violations(
            "src/example.py",
            "try:\n    work()\nexcept:\n    pass",
            ENABLED
        ),
        vec![Violation::EmptyErrorHandler]
    );
}

#[test]
fn permits_handlers_that_handle_or_reraise_errors() {
    assert!(
        violations(
            "src/example.js",
            "try { work(); } catch { recover(); }",
            ENABLED
        )
        .is_empty()
    );
    assert!(
        violations(
            "src/example.py",
            "try:\n    work()\nexcept:\n    raise",
            ENABLED
        )
        .is_empty()
    );
    assert!(
        violations(
            "src/example.py",
            "try:\n    work()\nexcept:\n    \"documented\"",
            ENABLED
        )
        .is_empty()
    );
}

#[test]
fn does_not_apply_to_rust() {
    assert!(violations("src/example.rs", "fn example() { work(); }", ENABLED).is_empty());
}

#[test]
fn can_disable_the_rule() {
    let configuration =
        "version: 1\nrules:\n  reliability/empty-error-handler:\n    severity: off\n";

    assert!(violations("src/example.js", "try { work(); } catch { }", configuration).is_empty());
}
