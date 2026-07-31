use std::{error::Error, path::PathBuf};

use godlint_core::{
    analyzers::AnalyzerError,
    facts::FunctionFactError,
    source::{SourceFile, SourceFileError, SourceRange},
};

fn ranges() -> (SourceRange, SourceRange) {
    let file = SourceFile::new(PathBuf::from("example.rs"), "fn example() {}".into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));
    let function = file
        .range(0, 4)
        .unwrap_or_else(|error| panic!("takes a range: {error}"));
    let body = file
        .range(5, 9)
        .unwrap_or_else(|error| panic!("takes a range: {error}"));

    (function, body)
}

fn path() -> PathBuf {
    PathBuf::from("src/example.rs")
}

#[test]
fn an_analyzer_error_names_the_file_it_could_not_read() {
    let (function_range, body_range) = ranges();
    let cases = [
        (
            AnalyzerError::MissingSyntaxTree { path: path() },
            "parser produced no syntax tree for src/example.rs",
        ),
        (
            AnalyzerError::InvalidRange {
                path: path(),
                source: SourceFileError::InvalidUtf8Boundary { offset: 3 },
            },
            "invalid range in src/example.rs: source offset is not on a UTF-8 boundary: 3",
        ),
        (
            AnalyzerError::InvalidFunction {
                path: path(),
                source: FunctionFactError::BodyOutsideFunction {
                    function_range,
                    body_range,
                },
            },
            "invalid function in src/example.rs: function body 5..9 is outside function 0..4",
        ),
    ];

    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
    }
}

#[test]
fn an_analyzer_error_carries_the_error_beneath_it() {
    let carried = AnalyzerError::InvalidRange {
        path: path(),
        source: SourceFileError::InvalidUtf8Boundary { offset: 3 },
    };

    assert!(
        carried.source().is_some(),
        "the cause must survive, or a caller reports the layer instead of the fault"
    );
    assert!(
        AnalyzerError::MissingSyntaxTree { path: path() }
            .source()
            .is_none(),
        "nothing failed beneath a tree the parser simply did not produce"
    );
}
