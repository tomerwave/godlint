use godlint_core::analyzers::analyze;

use super::source;

fn arguments_of(path: &str, contents: &str) -> Vec<(Option<String>, Option<String>)> {
    let facts =
        analyze(&source(path, contents)).unwrap_or_else(|error| panic!("analyzes {path}: {error}"));

    facts
        .calls()
        .first()
        .unwrap_or_else(|| panic!("no call in {path}"))
        .arguments()
        .iter()
        .map(|argument| (argument.name.clone(), argument.literal.clone()))
        .collect()
}

fn literal(value: &str) -> (Option<String>, Option<String>) {
    (None, Some(value.to_owned()))
}

fn opaque() -> (Option<String>, Option<String>) {
    (None, None)
}

#[test]
fn reads_a_string_literal_argument_in_each_language() {
    assert_eq!(
        arguments_of("a.js", "createHash(\"md5\");"),
        vec![literal("md5")]
    );
    assert_eq!(
        arguments_of("a.ts", "createHash('md5');"),
        vec![literal("md5")]
    );
    assert_eq!(arguments_of("a.py", "new('md5')"), vec![literal("md5")]);
    assert_eq!(
        arguments_of("a.rs", "fn a() { hash(\"md5\"); }"),
        vec![literal("md5")]
    );
}

#[test]
fn reads_an_empty_string_as_a_literal_rather_than_as_unknown() {
    assert_eq!(arguments_of("a.js", "hash(\"\");"), vec![literal("")]);
    assert_eq!(arguments_of("a.py", "hash('')"), vec![literal("")]);
    assert_eq!(
        arguments_of("a.rs", "fn a() { hash(\"\"); }"),
        vec![literal("")]
    );
}

#[test]
fn reads_a_non_string_literal_as_written() {
    assert_eq!(
        arguments_of("a.js", "hash(1, true, null);"),
        vec![literal("1"), literal("true"), literal("null")]
    );
    assert_eq!(
        arguments_of("a.py", "hash(1, True, None)"),
        vec![literal("1"), literal("True"), literal("None")]
    );
    assert_eq!(
        arguments_of("a.rs", "fn a() { hash(1, true, 'c'); }"),
        vec![literal("1"), literal("true"), literal("'c'")]
    );
}

#[test]
fn reads_a_value_it_cannot_see_as_present_and_unknown() {
    assert_eq!(arguments_of("a.js", "createHash(algo);"), vec![opaque()]);
    assert_eq!(arguments_of("a.py", "new(algo)"), vec![opaque()]);
    assert_eq!(
        arguments_of("a.js", "createHash(pick() + suffix);"),
        vec![opaque()],
        "a computed value is never guessed at"
    );
}

#[test]
fn reads_a_python_keyword_argument_by_name() {
    assert_eq!(
        arguments_of("a.py", "run(cmd, shell=True)"),
        vec![
            opaque(),
            (Some("shell".to_owned()), Some("True".to_owned()))
        ]
    );
}

#[test]
fn counts_arguments_as_it_did_before_reading_them() {
    let facts = analyze(&source("a.js", "setTimeout(work /*, 100 */);"))
        .unwrap_or_else(|error| panic!("analyzes: {error}"));

    assert_eq!(
        facts.calls()[0].argument_count(),
        1,
        "a comment inside the argument list is not an argument"
    );
}
