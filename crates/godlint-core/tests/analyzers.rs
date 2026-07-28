use std::path::PathBuf;

use godlint_core::{
    analyzers::{AnalyzerError, extract_functions},
    source::SourceFile,
};

fn source(path: &str, contents: &str) -> SourceFile {
    SourceFile::new(PathBuf::from(path), contents.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"))
}

#[test]
fn extracts_function_facts_from_every_supported_language() {
    let cases = [
        ("example.rs", "fn example() {\n    work();\n}"),
        ("example.js", "function example() {\n  work();\n}"),
        ("example.ts", "function example() {\n  work();\n}"),
        ("example.tsx", "function example() {\n  work();\n}"),
        ("example.py", "def example():\n    work()"),
        ("example.pyi", "def example():\n    work()"),
    ];

    for (path, contents) in cases {
        let functions = extract_functions(&source(path, contents))
            .unwrap_or_else(|error| panic!("extracts functions from {path}: {error}"));

        assert_eq!(functions.len(), 1, "{path}");
        assert_eq!(functions[0].name(), Some("example"), "{path}");
    }
}

#[test]
fn rejects_malformed_source() {
    let result = extract_functions(&source("example.rs", "fn example( {"));

    assert!(matches!(result, Err(AnalyzerError::InvalidSyntax { .. })));
}

#[test]
fn extracts_javascript_function_expressions() {
    let functions = extract_functions(&source(
        "example.js",
        "const example = function () {\n  work();\n};",
    ))
    .unwrap_or_else(|error| panic!("extracts function expression: {error}"));

    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name(), None);
}
