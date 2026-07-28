use std::path::PathBuf;

use godlint_core::{
    facts::{CommentFact, CommentFactError, FunctionFact, FunctionFactDetails, FunctionFactError},
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
        FunctionFactDetails {
            range: range(0, 28),
            body_range: range(11, 28),
            parameter_count: 0,
            decision_points: 0,
            return_count: 0,
            statement_count: 0,
            body_is_empty: false,
            nesting_depth: 0,
        },
    )
    .unwrap_or_else(|error| panic!("creates function fact: {error}"));

    assert_eq!(fact.source().path(), PathBuf::from("src/example.rs"));
    assert_eq!(fact.name(), Some("outer"));
    assert_eq!(fact.range(), range(0, 28));
    assert_eq!(fact.body_range(), range(11, 28));
    assert_eq!(fact.parameter_count(), 0);
    assert_eq!(fact.decision_points(), 0);
    assert_eq!(fact.return_count(), 0);
    assert_eq!(fact.statement_count(), 0);
    assert!(!fact.body_is_empty());
    assert_eq!(fact.nesting_depth(), 0);
}

#[test]
fn preserves_nesting_for_nested_functions() {
    let fact = FunctionFact::new(
        source(),
        Some("inner".into()),
        FunctionFactDetails {
            range: range(17, 24),
            body_range: range(17, 24),
            parameter_count: 0,
            decision_points: 0,
            return_count: 0,
            statement_count: 0,
            body_is_empty: false,
            nesting_depth: 1,
        },
    )
    .unwrap_or_else(|error| panic!("creates nested function fact: {error}"));

    assert_eq!(fact.nesting_depth(), 1);
}

#[test]
fn rejects_a_body_outside_the_function_range() {
    let result = FunctionFact::new(
        source(),
        Some("outer".into()),
        FunctionFactDetails {
            range: range(0, 10),
            body_range: range(11, 28),
            parameter_count: 0,
            decision_points: 0,
            return_count: 0,
            statement_count: 0,
            body_is_empty: false,
            nesting_depth: 0,
        },
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
        FunctionFactDetails {
            range: range(0, 29),
            body_range: range(11, 28),
            parameter_count: 0,
            decision_points: 0,
            return_count: 0,
            statement_count: 0,
            body_is_empty: false,
            nesting_depth: 0,
        },
    );

    assert!(matches!(
        result,
        Err(FunctionFactError::InvalidFunctionRange { .. })
    ));
}

#[test]
fn rejects_a_range_that_splits_a_multi_byte_character() {
    let source = SourceFile::new(
        PathBuf::from("src/example.rs"),
        "fn é() {\n    inner();\n}\n".into(),
    )
    .unwrap_or_else(|error| panic!("creates source file: {error}"));

    let result = FunctionFact::new(
        source,
        Some("é".into()),
        FunctionFactDetails {
            range: range(0, 4),
            body_range: range(0, 4),
            parameter_count: 0,
            decision_points: 0,
            return_count: 0,
            statement_count: 0,
            body_is_empty: false,
            nesting_depth: 0,
        },
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
        FunctionFactDetails {
            range: range(0, 28),
            body_range: range(11, 29),
            parameter_count: 0,
            decision_points: 0,
            return_count: 0,
            statement_count: 0,
            body_is_empty: false,
            nesting_depth: 0,
        },
    );

    assert!(matches!(
        result,
        Err(FunctionFactError::InvalidBodyRange { .. })
    ));
}

#[test]
fn records_a_comment_fact() {
    let source = SourceFile::new(PathBuf::from("src/example.rs"), "// TODO: track #1".into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));
    let range = SourceRange::new(0, source.source().len())
        .unwrap_or_else(|error| panic!("creates source range: {error}"));
    let fact = CommentFact::new(source, range)
        .unwrap_or_else(|error| panic!("creates comment fact: {error}"));

    assert_eq!(fact.text(), "// TODO: track #1");
    assert_eq!(fact.range(), range);
}

#[test]
fn rejects_an_invalid_comment_range() {
    let result = CommentFact::new(source(), range(0, 29));

    assert!(matches!(
        result,
        Err(CommentFactError::InvalidCommentRange { .. })
    ));
}
