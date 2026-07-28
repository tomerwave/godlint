use godlint_core::{
    config::{NoCommentsRule, Severity},
    rules::{Rule, no_comments::NoComments},
};

use super::support::comment_violations;

fn configuration(allow_doc_comments: bool) -> NoCommentsRule {
    NoCommentsRule {
        severity: Severity::Error,
        allow_doc_comments,
    }
}

fn violations(path: &str, source: &str, allow_doc_comments: bool) -> usize {
    comment_violations::<NoComments>(path, source, &configuration(allow_doc_comments)).len()
}

#[test]
fn reports_an_inline_comment() {
    assert_eq!(NoComments::ID, "style/no-comments");
    assert_eq!(
        violations("src/example.rs", "// why\nfn example() {}\n", true),
        1
    );
    assert_eq!(
        violations("src/example.py", "# why\ndef example():\n    pass\n", true),
        1
    );
    assert_eq!(
        violations("src/example.ts", "// why\nfunction example() {}\n", true),
        1
    );
}

#[test]
fn reports_a_block_comment() {
    assert_eq!(
        violations("src/example.rs", "/* why */\nfn example() {}\n", true),
        1
    );
}

#[test]
fn reports_every_comment_separately() {
    assert_eq!(
        violations("src/example.rs", "// one\n// two\nfn example() {}\n", true),
        2
    );
}

#[test]
fn permits_documentation_by_default() {
    assert_eq!(
        violations("src/example.rs", "/// Documented.\nfn example() {}\n", true),
        0
    );
    assert_eq!(
        violations(
            "src/example.rs",
            "//! Module documentation.\nfn example() {}\n",
            true
        ),
        0
    );
    assert_eq!(
        violations(
            "src/example.ts",
            "/** Documented. */\nfunction example() {}\n",
            true
        ),
        0
    );
    assert_eq!(
        violations(
            "src/example.py",
            "def example():\n    \"\"\"Documented.\"\"\"\n",
            true
        ),
        0
    );
}

#[test]
fn reports_documentation_when_configured() {
    assert_eq!(
        violations(
            "src/example.rs",
            "/// Documented.\nfn example() {}\n",
            false
        ),
        1
    );
    assert_eq!(
        violations(
            "src/example.py",
            "def example():\n    \"\"\"Documented.\"\"\"\n",
            false
        ),
        1
    );
}

#[test]
fn honours_each_language_documentation_convention() {
    assert_eq!(
        violations("src/a.rs", "/// Documented.\nfn a() {}\n", true),
        0
    );
    assert_eq!(
        violations("src/b.rs", "//! Documented.\nfn b() {}\n", true),
        0
    );
    assert_eq!(
        violations("src/c.ts", "/** Documented. */\nfunction c() {}\n", true),
        0
    );
    assert_eq!(
        violations(
            "src/d.ts",
            "/// <reference types=\"node\" />\nfunction d() {}\n",
            true
        ),
        1
    );
}

#[test]
fn permits_a_shebang() {
    assert_eq!(
        violations(
            "src/example.py",
            "#!/usr/bin/env python3\ndef example():\n    pass\n",
            false
        ),
        0
    );
}

#[test]
fn reports_a_comment_that_merely_resembles_a_shebang() {
    assert_eq!(
        violations(
            "src/example.py",
            "def example():\n    pass\n\n#!not a shebang here\n",
            true
        ),
        1
    );
}

#[test]
fn ignores_a_comment_marker_inside_a_string() {
    assert_eq!(
        violations(
            "src/example.rs",
            "fn example() {\n    let x = \"// not a comment\";\n}\n",
            true
        ),
        0
    );
}

#[test]
fn treats_an_empty_block_comment_as_an_aside() {
    assert_eq!(
        violations("src/example.rs", "/**/\nfn example() {}\n", true),
        1
    );
}

#[test]
fn permits_a_comment_that_is_only_a_suppression_directive() {
    assert_eq!(
        violations(
            "src/example.rs",
            "// godlint-ignore-next-line maintainability/empty-function -- reason\nfn example() {}\n",
            false
        ),
        0
    );
    assert_eq!(
        violations(
            "src/example.rs",
            "/*\ngodlint-ignore-next-line maintainability/empty-function -- reason\n*/\nfn example() {}\n",
            false
        ),
        0
    );
}

#[test]
fn reports_a_comment_that_mixes_prose_with_a_directive() {
    assert_eq!(
        violations(
            "src/example.rs",
            "/*\nThis prose would otherwise be laundered past the rule.\ngodlint-ignore-next-line maintainability/empty-function -- reason\n*/\nfn example() {}\n",
            false
        ),
        1,
        "one directive must not exempt a whole comment"
    );
}
