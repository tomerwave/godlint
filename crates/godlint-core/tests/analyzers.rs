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
fn extracts_empty_error_handlers() {
    let cases = [
        ("example.js", "try { work(); } catch { }", true),
        (
            "example.ts",
            "try { work(); } catch (error) { report(error); }",
            false,
        ),
        ("example.py", "try:\n    work()\nexcept:\n    pass", true),
        (
            "example.py",
            "try:\n    work()\nexcept Exception:\n    raise",
            false,
        ),
        ("example.rs", "fn example() { work(); }", false),
    ];

    for (path, contents, empty) in cases {
        let facts = analyze(&source(path, contents))
            .unwrap_or_else(|error| panic!("analyzes {path}: {error}"));

        if path.ends_with(".rs") {
            assert!(facts.error_handlers().is_empty(), "{path}");
        } else {
            assert_eq!(facts.error_handlers().len(), 1, "{path}");
            assert_eq!(facts.error_handlers()[0].body_is_empty(), empty, "{path}");
        }
    }
}

#[test]
fn ignores_rust_trait_methods_without_bodies() {
    let facts = analyze(&source("example.rs", "trait Hook {\n    fn empty();\n}"))
        .unwrap_or_else(|error| panic!("analyzes trait: {error}"));

    assert!(facts.functions().is_empty());
}

#[test]
fn extracts_direct_calls_from_each_language() {
    let cases = [
        (
            "example.ts",
            "eval(value); Function(value); process.exit(1); console.log(value); console.debug(value);",
            vec![
                "eval",
                "Function",
                "process.exit",
                "console.log",
                "console.debug",
            ],
        ),
        (
            "example.py",
            "eval(value)\nexec(value)\nsys.exit(1)\nos._exit(1)\nprint(value)\nos.getenv('VALUE')",
            vec!["eval", "exec", "sys.exit", "os._exit", "print", "os.getenv"],
        ),
        (
            "example.rs",
            "std::process::exit(1);\nstd::env::var(\"VALUE\");\ndbg!(value);",
            vec!["std::process::exit", "std::env::var", "dbg"],
        ),
    ];

    for (path, contents, expected) in cases {
        let facts = analyze(&source(path, contents))
            .unwrap_or_else(|error| panic!("analyzes {path}: {error}"));
        let calls: Vec<&str> = facts.calls().iter().map(|call| call.callee()).collect();

        assert_eq!(calls, expected, "{path}");
    }
}

#[test]
fn extracts_imported_modules_in_every_language() {
    let cases = [
        (
            "example.rs",
            "use std::process::exit;\nuse crate::internal::{a, b};\nextern crate serde;\n",
            vec!["std::process::exit", "crate::internal", "serde"],
        ),
        (
            "example.py",
            "import os.path\nfrom a.b import c\nimport x as y\n",
            vec!["os.path", "a.b", "x"],
        ),
        (
            "example.js",
            "import x from \"a/b\";\nimport \"./side\";\nexport { z } from \"pkg\";\n",
            vec!["a/b", "./side", "pkg"],
        ),
        (
            "example.ts",
            "import type { T } from \"./t\";\nimport x from \"a/b\";\n",
            vec!["./t", "a/b"],
        ),
    ];

    for (path, contents, expected) in cases {
        let facts = analyze(&source(path, contents))
            .unwrap_or_else(|error| panic!("analyzes {path}: {error}"));
        let modules: Vec<&str> = facts
            .imports()
            .iter()
            .map(|import| import.module())
            .collect();

        assert_eq!(modules, expected, "{path}");
    }
}

#[test]
fn reads_no_import_from_a_file_that_has_none() {
    let facts = analyze(&source("example.rs", "fn main() {}\n"))
        .unwrap_or_else(|error| panic!("analyzes: {error}"));

    assert!(facts.imports().is_empty());
}

#[test]
fn counts_direct_call_arguments() {
    let facts = analyze(&source(
        "example.ts",
        "setTimeout(work); setInterval(work, 100);",
    ))
    .unwrap_or_else(|error| panic!("analyzes call arguments: {error}"));
    let counts: Vec<usize> = facts
        .calls()
        .iter()
        .map(|call| call.argument_count())
        .collect();

    assert_eq!(counts, vec![1, 2]);
}

#[test]
fn ignores_indirect_calls() {
    let facts = analyze(&source(
        "example.ts",
        "getHandler()();\nhandlers[name]();\nfactory().run();",
    ))
    .unwrap_or_else(|error| panic!("analyzes indirect calls: {error}"));
    let calls: Vec<&str> = facts.calls().iter().map(|call| call.callee()).collect();

    assert_eq!(calls, vec!["getHandler", "factory"]);
}

#[test]
fn extracts_direct_environment_accesses() {
    let cases = [
        (
            "example.ts",
            "const value = process.env.VALUE;",
            "process.env",
        ),
        ("example.py", "value = os.environ['VALUE']", "os.environ"),
    ];

    for (path, contents, expected) in cases {
        let facts = analyze(&source(path, contents))
            .unwrap_or_else(|error| panic!("analyzes {path}: {error}"));
        let accesses: Vec<&str> = facts
            .accesses()
            .iter()
            .map(|access| access.target())
            .collect();

        assert!(accesses.contains(&expected), "{path}");
    }
}

#[test]
fn records_whether_a_call_was_a_macro_invocation() {
    let facts = analyze(&source(
        "example.rs",
        "fn dbg(v: u32) -> u32 { v }\npub fn run(v: u32) -> u32 {\n    dbg(v);\n    dbg!(v)\n}\n",
    ))
    .unwrap_or_else(|error| panic!("analyzes: {error}"));
    let calls: Vec<(&str, bool)> = facts
        .calls()
        .iter()
        .map(|call| (call.callee(), call.is_macro()))
        .collect();

    assert_eq!(
        calls,
        vec![("dbg", false), ("dbg", true)],
        "a macro and a function that share a name must be distinguishable"
    );
}

#[test]
fn a_call_is_never_a_macro_outside_rust() {
    for (path, contents) in [
        ("example.py", "eval(value)\nos.getenv('V')\n"),
        (
            "example.ts",
            "eval(\"x\");\nconst f = new Function(\"y\");\n",
        ),
    ] {
        let facts = analyze(&source(path, contents))
            .unwrap_or_else(|error| panic!("analyzes {path}: {error}"));

        assert!(
            !facts.calls().is_empty(),
            "{path} should record calls to check"
        );
        assert!(
            facts.calls().iter().all(|call| !call.is_macro()),
            "{path} has no macro form"
        );
    }
}

#[test]
fn a_constructed_call_records_its_constructor() {
    let facts = analyze(&source(
        "example.ts",
        "const f = new Function(\"return 1\");\nconst d = new Date();\n",
    ))
    .unwrap_or_else(|error| panic!("analyzes: {error}"));
    let calls: Vec<&str> = facts.calls().iter().map(|call| call.callee()).collect();

    assert_eq!(
        calls,
        vec!["Function", "Date"],
        "new expressions are calls, so a constructed Function can be reported"
    );
}
