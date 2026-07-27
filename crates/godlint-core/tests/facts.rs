use std::path::PathBuf;

use godlint_core::{
    facts::{FunctionFact, FunctionFactError},
    source::{SourceFile, SourceRange},
};

fn source() -> SourceFile {
    SourceFile::new(
        PathBuf::from("src/example.rs"),
        "fn outer() {\n    inner();\n}\n".into(),
    )
    .unwrap_or_else(|error| panic!("creates source file: {error}"))
}

fn range(start: usize, end: usize) -> SourceRange {
    SourceRange::new(start, end).unwrap_or_else(|error| panic!("creates source range: {error}"))
}

#[test]
fn records_a_language_neutral_function_fact() {
    let fact = FunctionFact::new(
        source(),
        Some("outer".into()),
        range(0, 28),
        range(11, 28),
        0,
    )
    .unwrap_or_else(|error| panic!("creates function fact: {error}"));

    assert_eq!(fact.source().path(), PathBuf::from("src/example.rs"));
    assert_eq!(fact.name(), Some("outer"));
    assert_eq!(fact.range(), range(0, 28));
    assert_eq!(fact.body_range(), range(11, 28));
    assert_eq!(fact.nesting_depth(), 0);
}

#[test]
fn preserves_nesting_for_nested_functions() {
    let fact = FunctionFact::new(
        source(),
        Some("inner".into()),
        range(17, 24),
        range(17, 24),
        1,
    )
    .unwrap_or_else(|error| panic!("creates nested function fact: {error}"));

    assert_eq!(fact.nesting_depth(), 1);
}

#[test]
fn rejects_a_body_outside_the_function_range() {
    let result = FunctionFact::new(
        source(),
        Some("outer".into()),
        range(0, 10),
        range(11, 28),
        0,
    );

    assert!(matches!(
        result,
        Err(FunctionFactError::BodyOutsideFunction { .. })
    ));
}

#[test]
fn rejects_ranges_that_are_invalid_for_the_source_file() {
    let result = FunctionFact::new(
        source(),
        Some("outer".into()),
        range(0, 29),
        range(11, 28),
        0,
    );

    assert!(matches!(
        result,
        Err(FunctionFactError::InvalidFunctionRange { .. })
    ));
}

#[test]
fn rejects_a_body_range_that_is_invalid_for_the_source_file() {
    let result = FunctionFact::new(
        source(),
        Some("outer".into()),
        range(0, 28),
        range(11, 29),
        0,
    );

    assert!(matches!(
        result,
        Err(FunctionFactError::InvalidBodyRange { .. })
    ));
}
