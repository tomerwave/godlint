#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use godlint_core::{
    facts::{
        AccessFact, BlockDepth, CallArgument, CallFact, CallFactDetails, CallTarget,
        CognitiveScore, CommentFact, CommentKind, DecisionPoints, ErrorHandlerFact, FunctionFact,
        FunctionFactDetails, FunctionFactError, ParameterCount, ReturnPaths, StatementCount,
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
    source()
        .range(start, end)
        .unwrap_or_else(|error| panic!("creates source range: {error}"))
}

fn details() -> FunctionFactDetails {
    FunctionFactDetails {
        range: range(0, 28),
        body_range: range(11, 28),
        parameter_count: ParameterCount::new(1),
        decision_points: DecisionPoints::new(2),
        cognitive_score: CognitiveScore::new(3),
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
    assert_eq!(fact.cognitive_score(), CognitiveScore::new(3));
    assert_eq!(fact.return_paths(), ReturnPaths::new(3));
    assert_eq!(fact.statement_count(), StatementCount::new(4));
    assert_eq!(fact.block_depth(), BlockDepth::new(5));
    assert!(!fact.body_is_empty());
    assert!(!fact.is_abstract());
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
    let fact = CommentFact::new(source(), range(0, 11), CommentKind::Line);

    assert_eq!(fact.range(), range(0, 11));
    assert_eq!(fact.kind(), CommentKind::Line);
    assert_eq!(fact.text(), "fn outer() ");
}

#[test]
fn records_a_call() {
    let fact = CallFact::new(
        source(),
        range(17, 22),
        range(17, 24),
        CallFactDetails {
            target: CallTarget {
                path: "inner".to_owned(),
                is_macro: false,
            },
            arguments: vec![
                CallArgument {
                    name: None,
                    literal: Some("1".to_owned()),
                },
                CallArgument {
                    name: Some("flag".to_owned()),
                    literal: Some("true".to_owned()),
                },
            ],
        },
    );

    assert_eq!(fact.range(), range(17, 22));
    assert_eq!(
        fact.extent(),
        range(17, 24),
        "the extent spans the whole call, which is what a shape match compares"
    );
    assert_eq!(
        fact.callee(),
        "inner",
        "the range locates the callee and the analyzer resolves what it names"
    );
    assert!(!fact.is_macro());
    assert_eq!(fact.argument_count(), 2);
    assert_eq!(
        fact.positional_literal(0),
        Some("1"),
        "a positional index counts only the positional arguments"
    );
    assert_eq!(fact.positional_literal(1), None);
    assert_eq!(
        fact.named("flag").and_then(|a| a.literal.as_deref()),
        Some("true")
    );
    assert!(fact.named("missing").is_none());
}

#[test]
fn records_an_access() {
    let fact = AccessFact::new(source(), range(3, 8), "outer".to_owned());

    assert_eq!(fact.range(), range(3, 8));
    assert_eq!(
        fact.target(),
        "outer",
        "the range locates the access and the analyzer resolves what it names"
    );
}

#[test]
fn records_an_error_handler() {
    let fact = ErrorHandlerFact::new(source(), range(0, 28), true);

    assert_eq!(fact.range(), range(0, 28));
    assert!(fact.body_is_empty());
}
