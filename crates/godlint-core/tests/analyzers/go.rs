use super::{analyze, source};

#[test]
fn counts_function_parameters_without_a_receiver() {
    let facts = analyze(&source(
        "example.go",
        "func example(value string, other int) {}",
    ))
    .unwrap_or_else(|error| panic!("analyzes Go: {error}"));

    assert_eq!(facts.functions()[0].parameter_count().value(), 2);
}

#[test]
fn extracts_calls_and_imports() {
    let facts = analyze(&source(
        "example.go",
        "import (\n  \"fmt\"\n  alias \"example.com/pkg\"\n)\nos.Exit(1); os.Getenv(\"VALUE\");",
    ))
    .unwrap_or_else(|error| panic!("analyzes Go: {error}"));

    assert_eq!(
        facts
            .calls()
            .iter()
            .map(|call| call.callee())
            .collect::<Vec<_>>(),
        ["os.Exit", "os.Getenv"]
    );
    assert_eq!(
        facts
            .imports()
            .iter()
            .map(|import| import.module())
            .collect::<Vec<_>>(),
        ["fmt", "example.com/pkg"]
    );
}

#[test]
fn extracts_selector_accesses_and_numeric_literals() {
    let facts = analyze(&source(
        "example.go",
        "func example() { value.Field = 42; use(42); use(value.Field) }",
    ))
    .unwrap_or_else(|error| panic!("analyzes Go: {error}"));

    assert_eq!(
        facts
            .accesses()
            .iter()
            .map(|access| access.target())
            .collect::<Vec<_>>(),
        ["value.Field", "value.Field"]
    );
    assert_eq!(
        facts
            .calls()
            .iter()
            .find(|call| call.callee() == "use")
            .and_then(|call| call.positional_literal(0)),
        Some("42")
    );
}

#[test]
fn recognizes_go_test_names_and_skip_calls() {
    let facts = analyze(&source(
        "example_test.go",
        "func TestOne(t *testing.T) { helper() }\nfunc TestSkipped(t *testing.T) { t.Skip(\"later\") }\nfunc BenchmarkOne(b *testing.B) {}\nfunc ExampleOne() {}",
    ))
    .unwrap_or_else(|error| panic!("analyzes Go: {error}"));

    assert_eq!(facts.tests().len(), 4);
    assert_eq!(
        facts.tests()[0].focus(),
        godlint_core::facts::TestFocus::Ordinary
    );
    assert_eq!(
        facts.tests()[1].focus(),
        godlint_core::facts::TestFocus::Skipped
    );
    assert!(facts.tests()[2].name() == Some("BenchmarkOne"));
    assert!(facts.tests()[3].name() == Some("ExampleOne"));
}

#[test]
fn recognizes_go_testing_assertions() {
    let facts = analyze(&source(
        "example_test.go",
        "func TestOne(t *testing.T) { t.Error(\"failed\") }",
    ))
    .unwrap_or_else(|error| panic!("analyzes Go: {error}"));

    assert_eq!(facts.assertions().len(), 1);
}
