#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use godlint_core::{
    analyzers::{SourceFacts, analyze},
    config::{Config, Severity},
    date::Date,
    rules::evaluate,
    source::SourceFile,
    suppression::{Scope, Suppression, collect, is_directive_only},
};

fn facts(path: &str, source: &str) -> SourceFacts {
    let source = SourceFile::new(PathBuf::from(path), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));

    analyze(&source).unwrap_or_else(|error| panic!("analyzes {path}: {error}"))
}

fn suppressions(path: &str, source: &str) -> Vec<Suppression> {
    collect(std::slice::from_ref(&facts(path, source)))
}

fn only(path: &str, source: &str) -> Suppression {
    let mut found = suppressions(path, source);

    assert_eq!(found.len(), 1, "expected exactly one suppression in {path}");

    found.remove(0)
}

fn config(body: &str) -> Config {
    yaml_serde::from_str(body).unwrap_or_else(|error| panic!("reads configuration: {error}"))
}

fn today() -> Date {
    Date::parse("2026-07-28").unwrap_or_else(|error| panic!("parses date: {error}"))
}

#[test]
fn reads_every_field_of_a_directive() {
    let suppression = only(
        "src/example.rs",
        "fn example() {\n    // godlint-ignore-enclosing maintainability/empty-function \
         owner=tomer expires=2999-12-31 -- awaiting #482\n}\n",
    );

    assert_eq!(suppression.scope(), Scope::Enclosing);
    assert_eq!(suppression.rules(), ["maintainability/empty-function"]);
    assert_eq!(suppression.owner(), Some("tomer"));
    assert_eq!(suppression.expires(), Some("2999-12-31"));
    assert_eq!(suppression.justification(), Some("awaiting #482"));
    assert_eq!(suppression.line(), 2);
    assert!(suppression.resolves());
    assert!(suppression.unknown_options().is_empty());
}

#[test]
fn reads_a_rule_list() {
    let suppression = only(
        "src/example.rs",
        "// godlint-ignore-next-line a/b,c/d -- two rules\nfn example() {}\n",
    );

    assert_eq!(suppression.rules(), ["a/b", "c/d"]);
}

#[test]
fn names_the_directive_of_each_scope() {
    assert_eq!(Scope::NextLine.directive(), "godlint-ignore-next-line");
    assert_eq!(Scope::Enclosing.directive(), "godlint-ignore-enclosing");
}

#[test]
fn finds_a_directive_in_every_comment_syntax() {
    let directive = "godlint-ignore-next-line maintainability/empty-function -- reason";

    for (path, source) in [
        ("src/a.rs", format!("// {directive}\nfn a() {{}}\n")),
        ("src/b.rs", format!("/* {directive} */\nfn b() {{}}\n")),
        ("src/c.rs", format!("/// {directive}\nfn c() {{}}\n")),
        ("src/d.py", format!("# {directive}\ndef d():\n    pass\n")),
        (
            "src/e.py",
            format!("def e():\n    \"\"\"{directive}\"\"\"\n"),
        ),
        ("src/f.ts", format!("// {directive}\nfunction f() {{}}\n")),
        (
            "src/g.ts",
            format!("/** {directive} */\nfunction g() {{}}\n"),
        ),
    ] {
        assert_eq!(
            suppressions(path, &source).len(),
            1,
            "{path} carries no directive"
        );
    }
}

#[test]
fn finds_every_directive_in_a_multiline_comment() {
    let source = "/*\n godlint-ignore-next-line a/b -- one\n godlint-ignore-enclosing c/d -- two\n*/\nfn example() {}\n";
    let found = suppressions("src/example.rs", source);

    assert_eq!(found.len(), 2);
    assert_eq!(found[0].line(), 2);
    assert_eq!(found[1].line(), 3);
}

#[test]
fn a_directive_must_open_its_line() {
    assert!(
        suppressions(
            "src/example.rs",
            "// see godlint-ignore-next-line in the docs\nfn example() {}\n"
        )
        .is_empty(),
        "prose that mentions a directive is not a directive"
    );
}

#[test]
fn a_directive_needs_a_word_boundary() {
    assert!(
        suppressions(
            "src/example.rs",
            "// godlint-ignore-next-liner a/b -- typo\nfn example() {}\n"
        )
        .is_empty()
    );
}

#[test]
fn a_directive_in_a_string_is_not_a_directive() {
    assert!(
        suppressions(
            "src/example.rs",
            "fn example() {\n    let _ = \"godlint-ignore-next-line a/b -- text\";\n}\n"
        )
        .is_empty()
    );
}

#[test]
fn recognises_a_directive_without_arguments() {
    let suppression = only(
        "src/example.rs",
        "// godlint-ignore-next-line\nfn example() {}\n",
    );

    assert!(suppression.rules().is_empty());
    assert_eq!(suppression.justification(), None);
}

#[test]
fn requires_the_separator_to_stand_alone() {
    let suppression = only(
        "src/example.rs",
        "// godlint-ignore-next-line a/b --reason\nfn example() {}\n",
    );

    assert_eq!(suppression.justification(), None);
    assert_eq!(suppression.unknown_options(), ["--reason"]);
}

#[test]
fn reports_directives_in_source_order() {
    let found = suppressions(
        "src/example.rs",
        "// godlint-ignore-next-line a/b -- first\nfn a() {}\n\
         // godlint-ignore-next-line c/d -- second\nfn c() {}\n",
    );

    assert_eq!(found.len(), 2);
    assert!(found[0].line() < found[1].line());
}

#[test]
fn an_enclosing_directive_resolves_to_the_innermost_function() {
    let source = "fn outer() {\n    let inner = || {\n        // godlint-ignore-enclosing a/b -- inner\n    };\n    let _ = inner;\n}\n";
    let suppression = only("src/example.rs", source);

    assert!(suppression.resolves());
    assert!(
        !suppression.covers_line(1),
        "the enclosing closure does not extend to the outer function"
    );
}

#[test]
fn identifies_a_comment_that_is_only_a_directive() {
    assert!(is_directive_only(
        "// godlint-ignore-next-line a/b -- reason"
    ));
    assert!(is_directive_only(
        "# godlint-ignore-enclosing a/b -- reason"
    ));
    assert!(is_directive_only(
        "/*\n godlint-ignore-next-line a/b -- reason\n*/"
    ));
    assert!(!is_directive_only("// an ordinary aside"));
}

#[test]
fn prose_beside_a_directive_is_not_a_directive_comment() {
    assert!(
        !is_directive_only(
            "/*\nThis ordinary comment would otherwise be allowed.\n\
             godlint-ignore-next-line a/b -- reason\n*/"
        ),
        "one directive must not launder arbitrary prose past style/no-comments"
    );
}

#[test]
fn a_next_line_directive_reaches_past_the_end_of_its_own_comment() {
    let source = facts(
        "src/example.rs",
        "/*\ngodlint-ignore-next-line maintainability/empty-function -- reason\n*/\nfn a() {}\n",
    );
    let body = "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n";
    let findings = evaluate(std::slice::from_ref(&source), &config(body), today())
        .unwrap_or_else(|error| panic!("evaluates: {error}"));

    assert!(
        findings.is_empty(),
        "the closing delimiter is not the next line: {findings:?}"
    );
}

#[test]
fn a_justification_excludes_the_comments_closing_delimiter() {
    let suppression = only(
        "src/example.rs",
        "fn example() {\n    /* godlint-ignore-enclosing a/b -- awaiting #485 */\n}\n",
    );

    assert_eq!(suppression.justification(), Some("awaiting #485"));
}

#[test]
fn a_justification_keeps_a_trailing_issue_reference() {
    let suppression = only(
        "src/example.rs",
        "// godlint-ignore-next-line a/b -- see #485\nfn example() {}\n",
    );

    assert_eq!(suppression.justification(), Some("see #485"));
}

#[test]
fn suppresses_only_the_rule_the_directive_names() {
    let body = "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n  \
                maintainability/function-nesting:\n    severity: error\n    max-depth: 0\n";
    let source = facts(
        "src/example.rs",
        "// godlint-ignore-next-line maintainability/empty-function -- named\nfn a() { if true {} }\n",
    );
    let findings = evaluate(std::slice::from_ref(&source), &config(body), today())
        .unwrap_or_else(|error| panic!("evaluates: {error}"));

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "maintainability/function-nesting");
}

#[test]
fn a_directive_for_a_rule_that_is_off_suppresses_nothing_visible() {
    let body = "version: 1\nrules:\n  maintainability/empty-function:\n    severity: off\n";
    let source = facts(
        "src/example.rs",
        "// godlint-ignore-next-line maintainability/empty-function -- named\nfn a() {}\n",
    );
    let findings = evaluate(std::slice::from_ref(&source), &config(body), today())
        .unwrap_or_else(|error| panic!("evaluates: {error}"));

    assert!(findings.is_empty());
}

#[test]
fn a_suppression_does_not_reach_another_file() {
    let body = "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n";
    let sources = [
        facts(
            "src/a.rs",
            "// godlint-ignore-next-line maintainability/empty-function -- here\nfn a() {}\n",
        ),
        facts("src/b.rs", "fn b() {}\n"),
    ];
    let findings = evaluate(&sources, &config(body), today())
        .unwrap_or_else(|error| panic!("evaluates: {error}"));

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].path, PathBuf::from("src/b.rs"));
}

#[test]
fn the_accountability_rule_reports_even_when_every_other_rule_is_silent() {
    let body = "version: 1\nrules:\n  policy/accountable-suppression:\n    severity: error\n";
    let source = facts(
        "src/example.rs",
        "// godlint-ignore-next-line maintainability/empty-function\nfn a() {}\n",
    );
    let findings = evaluate(std::slice::from_ref(&source), &config(body), today())
        .unwrap_or_else(|error| panic!("evaluates: {error}"));

    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "policy/accountable-suppression");
    assert_eq!(findings[0].severity, Severity::Error);
}
