use godlint_core::suppression::Scope;

use super::support::{only, suppressions};

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
