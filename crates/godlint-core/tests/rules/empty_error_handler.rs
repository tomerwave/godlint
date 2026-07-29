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
fn reports_a_python_handler_whatever_the_except_clause_names() {
    for source in [
        "try:\n    work()\nexcept ValueError:\n    pass",
        "try:\n    work()\nexcept Exception as error:\n    pass",
        "try:\n    work()\nexcept (ValueError, TypeError):\n    pass",
        "try:\n    work()\nexcept* ValueError:\n    pass",
    ] {
        assert_eq!(
            violations("src/example.py", source, ENABLED),
            vec![Violation::EmptyErrorHandler],
            "a named exception must not hide the empty body in {source:?}"
        );
    }
}

#[test]
fn reports_a_body_that_only_stands_in_for_one() {
    assert_eq!(
        violations(
            "src/example.py",
            "try:\n    work()\nexcept:\n    ...",
            ENABLED
        ),
        vec![Violation::EmptyErrorHandler]
    );
    assert_eq!(
        violations("src/example.js", "try { work(); } catch { ; }", ENABLED),
        vec![Violation::EmptyErrorHandler]
    );
}

#[test]
fn a_comment_does_not_count_as_handling_the_error() {
    assert_eq!(
        violations(
            "src/example.py",
            "try:\n    work()\nexcept Exception:\n    # deliberately ignored\n    pass",
            ENABLED
        ),
        vec![Violation::EmptyErrorHandler]
    );
    assert_eq!(
        violations(
            "src/example.js",
            "try { work(); } catch (error) { /* deliberately ignored */ }",
            ENABLED
        ),
        vec![Violation::EmptyErrorHandler]
    );
    assert_eq!(
        violations(
            "src/example.js",
            "try { work(); } catch (error) {\n  // deliberately ignored\n}",
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
    assert!(
        violations(
            "src/example.py",
            "try:\n    work()\nexcept ValueError as error:\n    report(error)",
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
