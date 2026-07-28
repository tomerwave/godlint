#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use godlint_core::{
    analyzers::{SourceFacts, analyze},
    config::{Config, Severity},
    date::Date,
    facts::CommentKind,
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

fn surviving(path: &str, source: &str, body: &str) -> Vec<(usize, usize)> {
    let facts = facts(path, source);

    evaluate(std::slice::from_ref(&facts), &config(body), today())
        .unwrap_or_else(|error| panic!("evaluates: {error}"))
        .iter()
        .map(|finding| (finding.line, finding.column))
        .collect()
}

const EMPTY_FUNCTION: &str =
    "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n";

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
    let directive =
        "        // godlint-ignore-enclosing maintainability/empty-function -- inner is a stub\n";
    let body = "fn outer() {\n    let inner = || {\nBODY    };\n    let _ = inner;\n}\n";

    assert_eq!(
        surviving("src/a.rs", &body.replace("BODY", ""), EMPTY_FUNCTION),
        vec![(2, 17)],
        "without a directive the closure is reported"
    );
    assert_eq!(
        surviving("src/b.rs", &body.replace("BODY", directive), EMPTY_FUNCTION),
        Vec::new(),
        "a directive inside the closure covers the closure"
    );
}

#[test]
fn an_enclosing_directive_does_not_reach_a_nested_declaration() {
    let directive =
        "    // godlint-ignore-enclosing maintainability/empty-function -- outer is a stub\n";
    let body = "fn outer() {\nBODY    let inner = || {};\n    let _ = inner;\n}\n";
    let without = surviving("src/a.rs", &body.replace("BODY", ""), EMPTY_FUNCTION);
    let with = surviving("src/b.rs", &body.replace("BODY", directive), EMPTY_FUNCTION);

    assert_eq!(without.len(), 1, "the closure is reported: {without:?}");
    assert_eq!(
        with.len(),
        1,
        "a justification for the enclosing function does not describe a closure inside it: \
         {with:?}"
    );
}

#[test]
fn an_enclosing_directive_does_not_reach_a_neighbour_on_its_line() {
    let directive =
        " /* godlint-ignore-enclosing maintainability/empty-function -- a is a no-op */ ";
    let body = "export const a = (): void => {BODY}; export const b = (): void => {};\n";
    let without = surviving("src/a.ts", &body.replace("BODY", ""), EMPTY_FUNCTION);
    let with = surviving("src/b.ts", &body.replace("BODY", directive), EMPTY_FUNCTION);

    assert_eq!(without.len(), 2, "both arrows are reported: {without:?}");
    assert_eq!(
        with.len(),
        1,
        "b shares a line with a but is not the declaration a justified: {with:?}"
    );
}

#[test]
fn an_enclosing_directive_stops_where_its_declaration_ends() {
    let directive = "/* godlint-ignore-enclosing maintainability/empty-function -- a */";
    let body = format!("fn a() {{{directive}}}fn b() {{}}\n");

    assert_eq!(
        surviving("src/example.rs", &body, EMPTY_FUNCTION).len(),
        1,
        "b begins at the byte a ends on, and is a different declaration"
    );
}

#[test]
fn an_enclosing_directive_cannot_reach_a_file_level_finding() {
    let source = concat!(
        "fn a() {\n",
        "    // godlint-ignore-enclosing maintainability/file-size -- reaches the file?\n",
        "    work();\n",
        "}\n"
    );
    let body = concat!(
        "version: 1\n",
        "rules:\n",
        "  maintainability/file-size:\n",
        "    severity: error\n",
        "    max-lines: 1\n"
    );

    assert_eq!(
        surviving("src/example.rs", source, body).len(),
        1,
        "a file-level finding spans the whole file, so no declaration encloses it"
    );
}

#[test]
fn identifies_a_comment_that_is_only_a_directive() {
    assert!(is_directive_only(
        "// godlint-ignore-next-line a/b -- reason",
        CommentKind::Line
    ));
    assert!(is_directive_only(
        "# godlint-ignore-enclosing a/b -- reason",
        CommentKind::Line
    ));
    assert!(is_directive_only(
        "/*\n godlint-ignore-next-line a/b -- reason\n*/",
        CommentKind::Block
    ));
    assert!(is_directive_only(
        "\"\"\"godlint-ignore-enclosing a/b -- reason\"\"\"",
        CommentKind::Docstring
    ));
    assert!(!is_directive_only(
        "// an ordinary aside",
        CommentKind::Line
    ));
}

#[test]
fn prose_beside_a_directive_is_not_a_directive_comment() {
    assert!(
        !is_directive_only(
            "/*\nThis ordinary comment would otherwise be allowed.\n\
             godlint-ignore-next-line a/b -- reason\n*/",
            CommentKind::Block
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

#[test]
fn quoted_prose_is_not_a_directive() {
    for source in [
        "// 'godlint-ignore-next-line a/b -- only prose'\nfn example() {}\n",
        "// \"godlint-ignore-next-line a/b -- example\" shows the syntax\nfn example() {}\n",
        "/* 'godlint-ignore-enclosing a/b -- quoted' */\nfn example() {}\n",
    ] {
        assert!(
            suppressions("src/example.rs", source).is_empty(),
            "a quoted directive is prose, not policy: {source}"
        );
    }
}

#[test]
fn a_multiline_docstring_directive_reaches_past_the_closing_delimiter() {
    let source = facts(
        "src/example.py",
        "\"\"\"\ngodlint-ignore-next-line maintainability/empty-function -- reason\n\"\"\"\ndef empty():\n    pass\n",
    );
    let body = "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n  \
                style/no-comments:\n    severity: error\n    allow-doc-comments: false\n";
    let findings = evaluate(std::slice::from_ref(&source), &config(body), today())
        .unwrap_or_else(|error| panic!("evaluates: {error}"));

    assert!(
        findings.is_empty(),
        "a closing docstring delimiter is furniture, not the target line: {findings:?}"
    );
}

#[test]
fn a_quoted_directive_inside_a_docstring_is_still_prose() {
    assert!(
        suppressions(
            "src/example.py",
            "def three():\n    \"\"\"Prose here.\n\n    \'godlint-ignore-enclosing a/b -- quoted\'\n    \"\"\"\n"
        )
        .is_empty(),
        "only a delimiter that opens the docstring may open a directive"
    );
}

#[test]
fn a_docstring_delimiter_still_opens_a_directive() {
    for source in [
        "def example():\n    \"\"\"godlint-ignore-enclosing a/b -- docstring\"\"\"\n",
        "def example():\n    \'\'\'godlint-ignore-enclosing a/b -- docstring\'\'\'\n",
    ] {
        let suppression = only("src/example.py", source);

        assert_eq!(suppression.rules(), ["a/b"]);
        assert_eq!(suppression.justification(), Some("docstring"));
    }
}

#[test]
fn stacked_directives_all_reach_the_code_below_them() {
    let source = "// godlint-ignore-next-line maintainability/empty-function -- first\n\
                  // godlint-ignore-next-line policy/todo-requires-reference -- second\n\
                  fn example() {}\n";
    let body = "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n";

    assert_eq!(suppressions("src/example.rs", source).len(), 2);
    assert_eq!(
        surviving("src/example.rs", source, body),
        Vec::new(),
        "a directive stacked above another must reach the code, not its neighbour"
    );
}

#[test]
fn stacked_directives_inside_one_comment_reach_the_code_below_them() {
    let source = "/*\ngodlint-ignore-next-line maintainability/empty-function -- first\n\
                  godlint-ignore-next-line policy/todo-requires-reference -- second\n*/\n\
                  fn example() {}\n";
    let body = "version: 1\nrules:\n  maintainability/empty-function:\n    severity: error\n";

    assert_eq!(suppressions("src/example.rs", source).len(), 2);
    assert_eq!(surviving("src/example.rs", source, body), Vec::new());
}

#[test]
fn an_empty_option_value_reads_as_absent() {
    let suppression = only(
        "src/example.rs",
        "// godlint-ignore-next-line a/b owner= expires= -- blank values\nfn example() {}\n",
    );

    assert_eq!(suppression.owner(), None);
    assert_eq!(suppression.expires(), None);
    assert!(suppression.repeated_options().is_empty());
}

#[test]
fn a_repeated_option_keeps_the_first_value_and_is_recorded() {
    let suppression = only(
        "src/example.rs",
        "// godlint-ignore-next-line a/b expires=2020-01-01 expires=2999-01-01 -- renewed\nfn example() {}\n",
    );

    assert_eq!(
        suppression.expires(),
        Some("2020-01-01"),
        "an expiry must not be extended by appending a second one"
    );
    assert_eq!(suppression.repeated_options(), ["expires"]);
}
