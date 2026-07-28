use std::path::PathBuf;

use godlint_core::{
    analyzers::{AnalyzerError, analyze},
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
        let facts = analyze(&source(path, contents))
            .unwrap_or_else(|error| panic!("extracts functions from {path}: {error}"));

        assert_eq!(facts.functions().len(), 1, "{path}");
        assert_eq!(facts.functions()[0].name(), Some("example"), "{path}");
        assert_eq!(facts.functions()[0].parameter_count(), 0, "{path}");
        assert!(!facts.functions()[0].body_is_empty(), "{path}");
    }
}

#[test]
fn rejects_malformed_source() {
    let result = analyze(&source("example.rs", "fn example( {"));

    assert!(matches!(result, Err(AnalyzerError::InvalidSyntax { .. })));
}

#[test]
fn extracts_javascript_function_expressions() {
    let facts = analyze(&source(
        "example.js",
        "const example = function () {\n  work();\n};",
    ))
    .unwrap_or_else(|error| panic!("extracts function expression: {error}"));

    assert_eq!(facts.functions().len(), 1);
    assert_eq!(facts.functions()[0].name(), None);
}

#[test]
fn extracts_parameter_counts_from_every_supported_language() {
    let cases = [
        (
            "example.rs",
            "fn example(one: u32, two: u32, three: u32) {}",
        ),
        ("example.js", "function example(one, two, three) {}"),
        (
            "example.ts",
            "function example(one: number, two: number, three: number) {}",
        ),
        (
            "example.tsx",
            "function example(one: number, two: number, three: number) {}",
        ),
        ("example.py", "def example(one, two, three):\n    pass"),
        ("example.pyi", "def example(one, two, three):\n    pass"),
    ];

    for (path, contents) in cases {
        let facts = analyze(&source(path, contents))
            .unwrap_or_else(|error| panic!("extracts parameters from {path}: {error}"));

        assert_eq!(facts.functions()[0].parameter_count(), 3, "{path}");
    }
}

#[test]
fn extracts_an_unparenthesized_arrow_parameter() {
    let facts = analyze(&source("example.js", "const example = value => value;"))
        .unwrap_or_else(|error| panic!("extracts arrow function: {error}"));

    assert_eq!(facts.functions()[0].parameter_count(), 1);
}

#[test]
fn ignores_rust_trait_methods_without_bodies() {
    let facts = analyze(&source("example.rs", "trait Hook {\n    fn empty();\n}"))
        .unwrap_or_else(|error| panic!("analyzes trait: {error}"));

    assert!(facts.functions().is_empty());
}

#[test]
fn extracts_comments_from_every_supported_language() {
    let cases = [
        ("example.rs", "// TODO: track #1"),
        ("example.js", "// TODO: track #1"),
        ("example.ts", "// TODO: track #1"),
        ("example.tsx", "// TODO: track #1"),
        ("example.py", "# TODO: track #1"),
        ("example.pyi", "# TODO: track #1"),
    ];

    for (path, contents) in cases {
        let facts = analyze(&source(path, contents))
            .unwrap_or_else(|error| panic!("extracts comments from {path}: {error}"));

        assert_eq!(facts.comments().len(), 1, "{path}");
        assert_eq!(facts.comments()[0].text(), contents, "{path}");
    }
}
