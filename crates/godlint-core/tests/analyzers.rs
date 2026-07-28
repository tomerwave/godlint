#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use godlint_core::{
    analyzers::{AnalyzerError, analyze},
    facts::CommentKind,
    source::SourceFile,
};

const SUPPORTED: [&str; 11] = [
    "rs", "js", "jsx", "mjs", "cjs", "ts", "tsx", "mts", "cts", "py", "pyi",
];

fn source(path: &str, contents: &str) -> SourceFile {
    SourceFile::new(PathBuf::from(path), contents.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"))
}

fn declaration(extension: &str) -> &'static str {
    match extension {
        "rs" => "fn example() {\n    work();\n}",
        "py" | "pyi" => "def example():\n    work()",
        _ => "function example() {\n  work();\n}",
    }
}

#[test]
fn extracts_a_function_from_every_supported_extension() {
    for extension in SUPPORTED {
        let path = format!("example.{extension}");
        let facts = analyze(&source(&path, declaration(extension)))
            .unwrap_or_else(|error| panic!("analyzes {path}: {error}"));

        assert_eq!(facts.functions().len(), 1, "{path}");
        assert_eq!(facts.functions()[0].name(), Some("example"), "{path}");
        assert!(!facts.functions()[0].body_is_empty(), "{path}");
    }
}

#[test]
fn extracts_a_comment_from_every_supported_extension() {
    for extension in SUPPORTED {
        let path = format!("example.{extension}");
        let contents = if matches!(extension, "py" | "pyi") {
            "# TODO: track #1"
        } else {
            "// TODO: track #1"
        };
        let facts = analyze(&source(&path, contents))
            .unwrap_or_else(|error| panic!("analyzes {path}: {error}"));

        assert_eq!(facts.comments().len(), 1, "{path}");
        assert_eq!(facts.comments()[0].text(), contents, "{path}");
        assert_eq!(facts.comments()[0].kind(), CommentKind::Line, "{path}");
    }
}

#[test]
fn classifies_a_python_docstring_as_commentary() {
    let facts = analyze(&source(
        "example.py",
        "\"\"\"Module detail.\"\"\"\ndef example():\n    work()",
    ))
    .unwrap_or_else(|error| panic!("analyzes docstring: {error}"));

    assert_eq!(facts.comments().len(), 1);
    assert_eq!(facts.comments()[0].kind(), CommentKind::Docstring);
}

#[test]
fn classifies_documentation_per_language_convention() {
    let rust = analyze(&source("a.rs", "/// Doc.\nfn a() {}\n"))
        .unwrap_or_else(|error| panic!("analyzes rust: {error}"));
    let typescript = analyze(&source("a.ts", "/// Directive.\nfunction a() {}\n"))
        .unwrap_or_else(|error| panic!("analyzes typescript: {error}"));
    let jsdoc = analyze(&source("b.ts", "/** Doc. */\nfunction b() {}\n"))
        .unwrap_or_else(|error| panic!("analyzes jsdoc: {error}"));

    assert_eq!(rust.comments()[0].kind(), CommentKind::Doc);
    assert_eq!(typescript.comments()[0].kind(), CommentKind::Line);
    assert_eq!(jsdoc.comments()[0].kind(), CommentKind::Doc);
}

#[test]
fn classifies_a_shebang_separately_from_commentary() {
    let facts = analyze(&source("a.py", "#!/usr/bin/env python3\n# aside\n"))
        .unwrap_or_else(|error| panic!("analyzes shebang: {error}"));

    assert_eq!(facts.comments()[0].kind(), CommentKind::Shebang);
    assert_eq!(facts.comments()[1].kind(), CommentKind::Line);
}

#[test]
fn recognises_a_docstring_that_follows_a_shebang() {
    let facts = analyze(&source(
        "a.py",
        "#!/usr/bin/env python3\n\"\"\"Module detail.\"\"\"\ndef example():\n    work()",
    ))
    .unwrap_or_else(|error| panic!("analyzes shebang and docstring: {error}"));
    let kinds: Vec<CommentKind> = facts.comments().iter().map(|c| c.kind()).collect();

    assert_eq!(kinds, vec![CommentKind::Shebang, CommentKind::Docstring]);
}

#[test]
fn does_not_treat_every_python_string_as_commentary() {
    let facts = analyze(&source(
        "example.py",
        "def example():\n    x = 1\n    \"loose\"",
    ))
    .unwrap_or_else(|error| panic!("analyzes strings: {error}"));

    assert!(facts.comments().is_empty());
}

#[test]
fn treats_closures_and_lambdas_as_functions() {
    let cases = [
        (
            "example.rs",
            "fn host() {\n    let f = |x: u32| x + 1;\n}",
            2,
        ),
        ("example.py", "def host():\n    f = lambda x: x + 1", 2),
        (
            "example.ts",
            "function host() {\n  const f = (x: number) => x + 1;\n}",
            2,
        ),
    ];

    for (path, contents, expected) in cases {
        let facts = analyze(&source(path, contents))
            .unwrap_or_else(|error| panic!("analyzes {path}: {error}"));

        assert_eq!(facts.functions().len(), expected, "{path}");
    }
}

#[test]
fn reports_comments_in_ascending_source_order() {
    let facts = analyze(&source(
        "a.rs",
        "// one
fn a() {
    /* two */
    run(); // three
}
/* four */
",
    ))
    .unwrap_or_else(|error| panic!("analyzes comments: {error}"));
    let starts: Vec<usize> = facts
        .comments()
        .iter()
        .map(|comment| comment.range().start())
        .collect();
    let mut sorted = starts.clone();

    sorted.sort_unstable();

    assert_eq!(starts.len(), 4);
    assert_eq!(starts, sorted);
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
fn ignores_rust_trait_methods_without_bodies() {
    let facts = analyze(&source("example.rs", "trait Hook {\n    fn empty();\n}"))
        .unwrap_or_else(|error| panic!("analyzes trait: {error}"));

    assert!(facts.functions().is_empty());
}
