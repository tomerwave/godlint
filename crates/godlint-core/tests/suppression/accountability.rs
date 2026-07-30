use std::path::PathBuf;

use godlint_core::{
    config::Severity, facts::CommentKind, rules::evaluate, suppression::is_directive_only,
};

use super::support::{config, facts, only, suppressions, surviving, today};

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
    let findings = evaluate(std::slice::from_ref(&source), &config(body), today());

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
    let findings = evaluate(std::slice::from_ref(&source), &config(body), today());

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
    let findings = evaluate(std::slice::from_ref(&source), &config(body), today());

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
    let findings = evaluate(&sources, &config(body), today());

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
    let findings = evaluate(std::slice::from_ref(&source), &config(body), today());

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
    let findings = evaluate(std::slice::from_ref(&source), &config(body), today());

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
    let source = concat!(
        "// godlint-ignore-next-line maintainability/empty-function -- first\n",
        "// godlint-ignore-next-line policy/todo-requires-reference -- second\n",
        "fn example() {} // TODO unreferenced\n"
    );
    let body = concat!(
        "version: 1\n",
        "rules:\n",
        "  maintainability/empty-function:\n",
        "    severity: error\n",
        "  policy/todo-requires-reference:\n",
        "    severity: error\n"
    );

    assert_eq!(suppressions("src/example.rs", source).len(), 2);
    assert_eq!(
        surviving("src/example.rs", source, body),
        Vec::new(),
        "both directives of a stack must reach the code, not each other"
    );
}

#[test]
fn stacked_directives_inside_one_comment_reach_the_code_below_them() {
    let source = concat!(
        "/*\n",
        "godlint-ignore-next-line maintainability/empty-function -- first\n",
        "godlint-ignore-next-line policy/todo-requires-reference -- second\n",
        "*/\n",
        "fn example() {} // TODO unreferenced\n"
    );
    let body = concat!(
        "version: 1\n",
        "rules:\n",
        "  maintainability/empty-function:\n",
        "    severity: error\n",
        "  policy/todo-requires-reference:\n",
        "    severity: error\n"
    );

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

#[test]
fn a_directive_on_a_docstrings_opening_line_occupies_that_line() {
    let source = concat!(
        "# godlint-ignore-next-line style/no-comments owner=x expires=2099-01-01 -- reason\n",
        "\"\"\"godlint-ignore-next-line style/no-comments owner=x expires=2099-01-01 -- reason\n",
        "more prose\n",
        "\"\"\"\n",
        "value = 1\n",
    );
    let body = concat!(
        "version: 1\n",
        "rules:\n",
        "  style/no-comments:\n",
        "    severity: error\n",
        "    allow-doc-comments: false\n",
    );

    assert_eq!(
        surviving("src/example.py", source, body),
        [(2, 1)],
        "the docstring's opening line holds a directive, so the line-1 directive skips past it \
         rather than spending itself on the docstring; the docstring's own prose is still reported"
    );
}
