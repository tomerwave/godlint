use std::path::PathBuf;

use godlint_core::{
    config::{FunctionSizeRule, Severity},
    facts::FunctionFact,
    rules::{Rule, function_size::FunctionSize},
    source::{SourceFile, SourceRange},
};

fn function(path: &str, source: &str) -> FunctionFact {
    let source = SourceFile::new(PathBuf::from(path), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));
    let range = SourceRange::new(0, source.source().len())
        .unwrap_or_else(|error| panic!("creates source range: {error}"));

    FunctionFact::new(source, Some("example".into()), range, range, 0)
        .unwrap_or_else(|error| panic!("creates function fact: {error}"))
}

fn configuration(max_lines: u32, skip_blank_lines: bool, skip_comments: bool) -> FunctionSizeRule {
    FunctionSizeRule {
        severity: Severity::Error,
        max_lines,
        skip_blank_lines,
        skip_comments,
    }
}

#[test]
fn reports_a_function_that_exceeds_its_limit() {
    let function = function("src/example.rs", "fn example() {\n    run();\n}");
    let violation = FunctionSize::evaluate(&function, &configuration(2, true, true));

    assert_eq!(FunctionSize::ID, "maintainability/function-size");
    assert_eq!(
        violation.map(|violation| violation.effective_line_count),
        Some(3)
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    let function = function("src/example.rs", "fn example() {\n    run();\n}");

    assert_eq!(
        FunctionSize::evaluate(&function, &configuration(3, true, true)),
        None
    );
}

#[test]
fn applies_blank_line_configuration() {
    let function = function("src/example.rs", "fn example() {\n\n    run();\n\n}");

    assert_eq!(
        FunctionSize::evaluate(&function, &configuration(3, true, true)),
        None
    );
    assert_eq!(
        FunctionSize::evaluate(&function, &configuration(3, false, true))
            .map(|violation| violation.effective_line_count),
        Some(5)
    );
}

#[test]
fn skips_rust_comment_only_lines() {
    let function = function(
        "src/example.rs",
        "fn example() {\n    // explanation\n    run();\n    /*\n     * detail\n     */\n}",
    );

    assert_eq!(
        FunctionSize::evaluate(&function, &configuration(3, true, true)),
        None
    );
}

#[test]
fn skips_typescript_comment_only_lines() {
    let function = function(
        "src/example.ts",
        "function example() {\n  // explanation\n  run();\n  /* detail */\n}",
    );

    assert_eq!(
        FunctionSize::evaluate(&function, &configuration(3, true, true)),
        None
    );
}

#[test]
fn skips_python_comment_only_lines() {
    let function = function(
        "src/example.py",
        "def example():\n    # explanation\n    run()",
    );

    assert_eq!(
        FunctionSize::evaluate(&function, &configuration(2, true, true)),
        None
    );
}

#[test]
fn counts_lines_with_code_and_inline_comments() {
    let function = function(
        "src/example.rs",
        "fn example() {\n    run(); // explanation\n}",
    );

    assert_eq!(
        FunctionSize::evaluate(&function, &configuration(2, true, true))
            .map(|violation| violation.effective_line_count),
        Some(3)
    );
}

#[test]
fn disables_evaluation_when_the_rule_is_off() {
    let function = function("src/example.rs", "fn example() {\n    run();\n}");
    let configuration = FunctionSizeRule {
        severity: Severity::Off,
        max_lines: 1,
        skip_blank_lines: true,
        skip_comments: true,
    };

    assert_eq!(FunctionSize::evaluate(&function, &configuration), None);
}
