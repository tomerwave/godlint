use godlint_core::{analyzers::analyze, facts::TestFocus};

use super::source;

fn found(path: &str, contents: &str) -> Vec<(Option<String>, String, TestFocus)> {
    let facts =
        analyze(&source(path, contents)).unwrap_or_else(|error| panic!("analyzes {path}: {error}"));

    facts
        .tests()
        .iter()
        .map(|test| {
            (
                test.name().map(str::to_owned),
                test.marker().to_owned(),
                test.focus(),
            )
        })
        .collect()
}

fn named(name: &str, marker: &str, focus: TestFocus) -> (Option<String>, String, TestFocus) {
    (Some(name.to_owned()), marker.to_owned(), focus)
}

#[test]
fn recognises_a_rust_test_by_its_attribute() {
    assert_eq!(
        found("a.rs", "#[test]\nfn works() {}"),
        vec![named("works", "test", TestFocus::Ordinary)]
    );
    assert_eq!(
        found("a.rs", "#[tokio::test]\nasync fn works() {}"),
        vec![named("works", "tokio::test", TestFocus::Ordinary)],
        "an attribute path ending in test is a test, whichever runner owns it"
    );
}

#[test]
fn reads_a_rust_attribute_that_is_not_adjacent_to_the_function() {
    assert_eq!(
        found("a.rs", "#[test]\n#[ignore]\nfn slow() {}"),
        vec![named("slow", "test", TestFocus::Skipped)],
        "attributes stack, and ignore is the Rust way of skipping"
    );
    assert_eq!(
        found("a.rs", "#[ignore]\n#[test]\nfn slow() {}"),
        vec![named("slow", "test", TestFocus::Skipped)],
        "the order they are written in is the author's business, not the rule's"
    );
}

#[test]
fn leaves_an_ordinary_rust_function_alone() {
    assert!(found("a.rs", "fn works() {}").is_empty());
    assert!(
        found("a.rs", "#[allow(dead_code)]\nfn works() {}").is_empty(),
        "an attribute that is not a test attribute does not make a test"
    );
    assert!(
        found("a.rs", "#[test]\nfn first() {}\nfn second() {}").len() == 1,
        "the attribute belongs to the function it precedes and not to the next one"
    );
}

#[test]
fn recognises_a_python_test_by_name_or_decorator() {
    assert_eq!(
        found("a.py", "def test_works():\n    pass"),
        vec![named("test_works", "test_works", TestFocus::Ordinary)]
    );
    assert_eq!(
        found(
            "a.py",
            "@pytest.mark.parametrize('x', [1])\ndef check(x):\n    pass"
        ),
        vec![named(
            "check",
            "pytest.mark.parametrize",
            TestFocus::Ordinary
        )],
        "a pytest marker makes a test of a function whose name says nothing"
    );
    assert_eq!(
        found("a.py", "@pytest.mark.skip\ndef test_slow():\n    pass"),
        vec![named("test_slow", "pytest.mark.skip", TestFocus::Skipped)]
    );
}

#[test]
fn leaves_an_ordinary_python_function_alone() {
    assert!(found("a.py", "def works():\n    pass").is_empty());
    assert!(
        found("a.py", "def testing_helper():\n    pass").is_empty(),
        "the convention is test_ and not test as a prefix of any word"
    );
    assert!(
        found("a.py", "@functools.cache\ndef helper():\n    pass").is_empty(),
        "a decorator that is not a pytest marker does not make a test"
    );
}

#[test]
fn recognises_a_javascript_test_by_its_runner() {
    assert_eq!(
        found("a.js", "it('works', () => {});"),
        vec![named("works", "it", TestFocus::Ordinary)]
    );
    assert_eq!(
        found("a.ts", "test('works', () => {});"),
        vec![named("works", "test", TestFocus::Ordinary)]
    );
    assert_eq!(
        found("a.js", "describe('group', () => {});"),
        vec![named("group", "describe", TestFocus::Ordinary)]
    );
}

#[test]
fn reads_focus_and_skipping_from_the_runner_member() {
    assert_eq!(
        found("a.js", "it.only('focused', () => {});"),
        vec![named("focused", "it.only", TestFocus::Only)]
    );
    assert_eq!(
        found("a.js", "it.skip('skipped', () => {});"),
        vec![named("skipped", "it.skip", TestFocus::Skipped)]
    );
    assert_eq!(
        found("a.js", "describe.only('group', () => {});"),
        vec![named("group", "describe.only", TestFocus::Only)]
    );
    assert_eq!(
        found("a.js", "it.todo('later');"),
        vec![named("later", "it.todo", TestFocus::Skipped)],
        "a todo test does not run, which is what a skipped test means"
    );
}

#[test]
fn leaves_an_ordinary_javascript_call_alone() {
    assert!(found("a.js", "run('works', () => {});").is_empty());
    assert!(
        found("a.js", "iterate('works', () => {});").is_empty(),
        "a runner name is the whole callee and not a prefix of it"
    );
}

#[test]
fn a_test_knows_what_it_encloses() {
    let facts = analyze(&source("a.js", "it('works', () => {\n  sleep(10);\n});"))
        .unwrap_or_else(|error| panic!("analyzes: {error}"));
    let test = &facts.tests()[0];
    let call = facts
        .calls()
        .iter()
        .find(|call| call.callee() == "sleep")
        .unwrap_or_else(|| panic!("finds the inner call"));

    assert!(
        test.contains(call.range()),
        "a rule asks whether a call is inside a test by range, which is what keeps the fact small"
    );
}
