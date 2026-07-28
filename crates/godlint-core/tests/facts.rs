#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use godlint_core::{
    facts::{
        AccessFact, AccessFactError, BlockDepth, CallFact, CallFactError, CommentFact,
        CommentFactError, CommentKind, DecisionPoints, FunctionFact, FunctionFactDetails,
        FunctionFactError, ParameterCount, ReturnPaths, StatementCount,
    },
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

fn details() -> FunctionFactDetails {
    FunctionFactDetails {
        range: range(0, 28),
        body_range: range(11, 28),
        parameter_count: ParameterCount::new(1),
        decision_points: DecisionPoints::new(2),
        return_paths: ReturnPaths::new(3),
        statement_count: StatementCount::new(4),
        block_depth: BlockDepth::new(5),
        body_is_empty: false,
        is_abstract: false,
    }
}

#[test]
fn records_every_metric_separately() {
    let fact = FunctionFact::new(source(), Some("outer".into()), details())
        .unwrap_or_else(|error| panic!("creates function fact: {error}"));

    assert_eq!(fact.source().path(), PathBuf::from("src/example.rs"));
    assert_eq!(fact.name(), Some("outer"));
    assert_eq!(fact.range(), range(0, 28));
    assert_eq!(fact.body_range(), range(11, 28));
    assert_eq!(fact.parameter_count(), ParameterCount::new(1));
    assert_eq!(fact.decision_points(), DecisionPoints::new(2));
    assert_eq!(fact.return_paths(), ReturnPaths::new(3));
    assert_eq!(fact.statement_count(), StatementCount::new(4));
    assert_eq!(fact.block_depth(), BlockDepth::new(5));
    assert!(!fact.body_is_empty());
    assert!(!fact.is_abstract());
}

#[test]
fn rejects_a_function_range_outside_the_file() {
    let result = FunctionFact::new(
        source(),
        None,
        FunctionFactDetails {
            range: range(0, 999),
            ..details()
        },
    );

    assert!(matches!(
        result,
        Err(FunctionFactError::InvalidFunctionRange { .. })
    ));
}

#[test]
fn rejects_a_body_range_outside_the_file() {
    let result = FunctionFact::new(
        source(),
        None,
        FunctionFactDetails {
            body_range: range(0, 999),
            ..details()
        },
    );

    assert!(matches!(
        result,
        Err(FunctionFactError::InvalidBodyRange { .. })
    ));
}

#[test]
fn rejects_a_body_outside_its_function() {
    let result = FunctionFact::new(
        source(),
        None,
        FunctionFactDetails {
            range: range(11, 20),
            body_range: range(0, 10),
            ..details()
        },
    );

    assert!(matches!(
        result,
        Err(FunctionFactError::BodyOutsideFunction { .. })
    ));
}

#[test]
fn records_a_comment_with_its_kind() {
    let fact = CommentFact::new(source(), range(0, 11), CommentKind::Line)
        .unwrap_or_else(|error| panic!("creates comment fact: {error}"));

    assert_eq!(fact.range(), range(0, 11));
    assert_eq!(fact.kind(), CommentKind::Line);
    assert_eq!(fact.text(), "fn outer() ");
}

#[test]
fn rejects_a_comment_range_outside_the_file() {
    let result = CommentFact::new(source(), range(0, 999), CommentKind::Block);

    assert!(matches!(
        result,
        Err(CommentFactError::InvalidCommentRange { .. })
    ));
}

#[test]
fn records_a_call() {
    let fact = CallFact::new(source(), range(17, 22), false)
        .unwrap_or_else(|error| panic!("creates call fact: {error}"));

    assert_eq!(fact.range(), range(17, 22));
    assert_eq!(
        fact.callee(),
        "inner",
        "a callee is read from the range rather than stored beside it"
    );
    assert!(!fact.is_macro());
}

#[test]
fn rejects_a_call_range_outside_the_file() {
    let result = CallFact::new(source(), range(0, 999), false);

    assert!(matches!(
        result,
        Err(CallFactError::InvalidCallRange { .. })
    ));
}

#[test]
fn records_an_access() {
    let fact = AccessFact::new(source(), range(3, 8))
        .unwrap_or_else(|error| panic!("creates access fact: {error}"));

    assert_eq!(fact.range(), range(3, 8));
    assert_eq!(
        fact.target(),
        "outer",
        "a target is read from the range rather than stored beside it"
    );
}

#[test]
fn rejects_an_access_range_outside_the_file() {
    let result = AccessFact::new(source(), range(0, 999));

    assert!(matches!(
        result,
        Err(AccessFactError::InvalidAccessRange { .. })
    ));
}
