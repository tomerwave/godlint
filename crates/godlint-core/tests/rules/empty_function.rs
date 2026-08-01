use godlint_core::{
    config::{EmptyFunctionRule, Severity},
    rules::{
        FunctionRule, Rule, Violation,
        empty_function::{self, EmptyFunction},
    },
};

use super::support::{function, rule_violations};

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

#[test]
fn accepts_every_body_in_an_interface_stub() {
    assert_eq!(
        check("src/example.pyi", "def stub() -> None: ...", &[]),
        None
    );
}

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

#[test]
fn accepts_a_constructor_that_assigns_parameter_properties() {
    for declaration in [
        "public dep: string",
        "private dep: string",
        "protected dep: string",
        "readonly dep: string",
        "private readonly dep: string",
    ] {
        let source = format!("class Service {{\n  constructor({declaration}) {{}}\n}}");
        let (facts, constructor) = super::support::nth_function("src/example.ts", &source, 0);

        assert_eq!(
            EmptyFunction::check(&constructor, &facts, &configuration(&[])),
            None,
            "{declaration} declares and initializes a parameter property"
        );
        assert_eq!(
            constructor.parameter_count().value(),
            1,
            "the property modifier does not change the declared parameter count"
        );
    }
}

#[test]
fn reports_an_empty_constructor_without_a_parameter_property() {
    assert_eq!(
        check(
            "src/example.ts",
            "class Service {\n  constructor(dep: string) {}\n}",
            &[],
        ),
        Some(Violation::EmptyBody)
    );
}

#[test]
fn accepts_typescript_declarations_that_have_no_implementation_body() {
    let body = concat!(
        "version: 1\n",
        "rules:\n",
        "  maintainability/empty-function:\n",
        "    severity: error\n",
    );

    for source in [
        "abstract class Base {\n  abstract run(value: string): void;\n}",
        "interface Runner {\n  run(value: string): void;\n}",
    ] {
        assert!(
            rule_violations(empty_function::evaluate, "src/example.ts", source, body).is_empty(),
            "a declaration without an implementation body is not an empty implementation: {source}"
        );
    }
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
