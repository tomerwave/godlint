use godlint_core::{
    config::{EmptyFunctionRule, Severity},
    rules::{FunctionRule, Rule, Violation, empty_function::EmptyFunction},
};

use super::support::function;

fn configuration(allow_names: &[&str]) -> EmptyFunctionRule {
    EmptyFunctionRule {
        severity: Severity::Error,
        allow_names: allow_names.iter().map(|name| (*name).into()).collect(),
    }
}

fn check(path: &str, source: &str, allow_names: &[&str]) -> Option<Violation> {
    let (facts, subject) = function(path, source);

    EmptyFunction::check(&subject, &facts, &configuration(allow_names))
}

#[test]
fn reports_a_body_with_nothing_in_it() {
    assert_eq!(EmptyFunction::ID, "maintainability/empty-function");
    assert_eq!(
        check("src/example.rs", "fn empty() {}", &[]),
        Some(Violation::EmptyBody)
    );
    assert_eq!(
        check("src/example.ts", "function empty() {}", &[]),
        Some(Violation::EmptyBody)
    );
}

#[test]
fn reports_a_python_placeholder_body() {
    assert_eq!(
        check("src/example.py", "def empty():\n    pass", &[]),
        Some(Violation::EmptyBody)
    );
    assert_eq!(
        check("src/example.py", "def empty():\n    ...", &[]),
        Some(Violation::EmptyBody)
    );
}

/// A comment in the body is the author recording that the emptiness is deliberate, which
/// is the same statement an allow-list entry would make and needs no configuration.
#[test]
fn accepts_a_body_that_documents_its_emptiness() {
    assert_eq!(
        check(
            "src/example.rs",
            "fn empty() {\n    // Nothing to do.\n}",
            &[]
        ),
        None
    );
    assert_eq!(
        check(
            "src/example.ts",
            "function empty() {\n  // Nothing to do.\n}",
            &[]
        ),
        None
    );
    assert_eq!(
        check(
            "src/example.py",
            "def empty():\n    \"\"\"Intentionally does nothing.\"\"\"",
            &[]
        ),
        None
    );
}

/// A stub file is signatures and placeholders by construction, so every body in it would
/// otherwise be reported.
#[test]
fn accepts_every_body_in_an_interface_stub() {
    assert_eq!(
        check("src/example.pyi", "def stub() -> None: ...", &[]),
        None
    );
}

/// An abstract method has no implementation on purpose.
#[test]
fn accepts_an_abstract_declaration() {
    assert_eq!(
        check(
            "src/example.py",
            "class Base:\n    @abstractmethod\n    def run(self) -> None:\n        pass",
            &[]
        ),
        None
    );
}

/// TypeScript parameter properties declare and assign a field from the signature, so the
/// empty constructor body is the idiom rather than an omission.
#[test]
fn accepts_a_constructor_that_assigns_parameter_properties() {
    let (facts, constructor) = super::support::nth_function(
        "src/example.ts",
        "class Service {\n  constructor(private readonly dep: string) {}\n}",
        0,
    );

    assert_eq!(
        EmptyFunction::check(&constructor, &facts, &configuration(&[])),
        None
    );
}

#[test]
fn permits_an_explicitly_allowed_name() {
    assert_eq!(
        check("src/example.ts", "function noop() {}", &["noop"]),
        None
    );
}

#[test]
fn requires_an_exact_name_match() {
    assert_eq!(
        check("src/example.ts", "function noopHandler() {}", &["noop"]),
        Some(Violation::EmptyBody)
    );
}

#[test]
fn ignores_a_function_that_does_work() {
    assert_eq!(
        check("src/example.js", "function active() {\n  work();\n}", &[]),
        None
    );
}
