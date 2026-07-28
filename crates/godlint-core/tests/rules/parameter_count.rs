use godlint_core::{
    config::{ParameterCountRule, Severity},
    rules::{Metric, Rule, Violation, parameter_count::ParameterCount},
};

use super::support::{function, function_limits};

fn configuration(max_parameters: u32) -> ParameterCountRule {
    ParameterCountRule {
        severity: Severity::Error,
        max_parameters,
    }
}

fn count(path: &str, source: &str) -> u32 {
    function(path, source).1.parameter_count().value()
}

#[test]
fn counts_declared_parameters() {
    assert_eq!(ParameterCount::ID, "maintainability/parameter-count");
    assert_eq!(count("src/example.rs", "fn example(a: u32, b: u32) {}"), 2);
    assert_eq!(count("src/example.py", "def example(a, b):\n    pass"), 2);
    assert_eq!(count("src/example.ts", "function example(a: number) {}"), 1);
}

#[test]
fn counts_a_single_unparenthesized_arrow_parameter() {
    assert_eq!(
        count("src/example.ts", "const example = value => value;"),
        1
    );
}

#[test]
fn excludes_the_method_receiver() {
    let rust = count(
        "src/example.rs",
        "struct S;\nimpl S {\n    fn example(&self, a: u32, b: u32, c: u32) {}\n}",
    );
    let python = count(
        "src/example.py",
        "class S:\n    def example(self, a, b, c):\n        pass",
    );
    let typescript = count(
        "src/example.ts",
        "class S {\n  example(a: number, b: number, c: number) {}\n}",
    );

    assert_eq!(rust, 3);
    assert_eq!(python, 3);
    assert_eq!(typescript, 3);
}

#[test]
fn excludes_a_python_class_receiver() {
    assert_eq!(
        count(
            "src/example.py",
            "class S:\n    @classmethod\n    def example(cls, a):\n        pass"
        ),
        1
    );
}

#[test]
fn reports_a_function_over_its_limit() {
    assert_eq!(
        function_limits::<ParameterCount>(
            "src/example.rs",
            "fn example(a: u32, b: u32, c: u32) {}",
            &configuration(2),
        ),
        vec![Violation::limit(Metric::ParameterCount, 3, 2)]
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    assert!(
        function_limits::<ParameterCount>(
            "src/example.rs",
            "fn example(a: u32, b: u32) {}",
            &configuration(2),
        )
        .is_empty()
    );
}
