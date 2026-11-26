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

fn properties_of(path: &str, contents: &str) -> Option<Vec<(Option<String>, Option<String>)>> {
    let facts =
        analyze(&source(path, contents)).unwrap_or_else(|error| panic!("analyzes {path}: {error}"));
    let call = facts
        .calls()
        .first()
        .unwrap_or_else(|| panic!("no call in {path}"));

    call.positional(0)
        .and_then(|argument| argument.properties.as_ref())
        .map(|properties| {
            properties
                .iter()
                .map(|property| (property.name.clone(), property.literal.clone()))
                .collect()
        })
}

fn property(name: &str, value: Option<&str>) -> (Option<String>, Option<String>) {
    (Some(name.to_owned()), value.map(str::to_owned))
}

#[test]
fn reads_the_properties_of_an_object_literal_argument() {
    assert_eq!(
        properties_of(
            "a.ts",
            "configure({ protect: true, \"name\": 'db', retain });"
        ),
        Some(vec![
            property("protect", Some("true")),
            property("name", Some("db")),
            property("retain", None),
        ])
    );
    assert_eq!(
        properties_of("a.js", "configure({ /* note */ protect: flag });"),
        Some(vec![property("protect", None)]),
        "a comment is not a property, and a computed value is never guessed at"
    );
    assert_eq!(properties_of("a.js", "configure({});"), Some(Vec::new()));
}

#[test]
fn reads_no_properties_from_an_object_it_cannot_see_in_full() {
    assert_eq!(
        properties_of("a.ts", "configure({ ...defaults, protect: true });"),
        None
    );
    assert_eq!(properties_of("a.ts", "configure({ [key]: true });"), None);
    assert_eq!(properties_of("a.ts", "configure({ 1: true });"), None);
    assert_eq!(properties_of("a.ts", "configure({ \"\": true });"), None);
    assert_eq!(properties_of("a.ts", "configure({ run() {} });"), None);
    assert_eq!(properties_of("a.ts", "configure(options);"), None);
}

#[test]
fn reads_nested_object_properties() {
    let facts = analyze(&source("a.ts", "configure({ outer: { inner: true } });"))
        .unwrap_or_else(|error| panic!("analyzes: {error}"));
    let outer = facts.calls()[0]
        .positional(0)
        .and_then(|argument| argument.property("outer"));

    assert_eq!(
        outer
            .and_then(|outer| outer.property("inner"))
            .and_then(|inner| inner.literal.as_deref()),
        Some("true")
    );
    assert!(outer.and_then(|outer| outer.property("missing")).is_none());
}

#[test]
fn reads_no_properties_in_languages_without_object_literals() {
    assert_eq!(properties_of("a.py", "configure({'protect': True})"), None);
    assert_eq!(
        properties_of("a.rs", "fn a() { configure(Options { protect: true }); }"),
        None
    );
    assert_eq!(
        properties_of(
            "a.go",
            "package a\n\nfunc f() { configure(Options{Protect: true}) }"
        ),
        None
    );
}
