use godlint_core::analyzers::analyze;

use super::source;

fn found(path: &str, contents: &str) -> Vec<(String, bool, usize)> {
    let facts =
        analyze(&source(path, contents)).unwrap_or_else(|error| panic!("analyzes {path}: {error}"));

    facts
        .assertions()
        .iter()
        .map(|assertion| {
            (
                assertion.name().to_owned(),
                assertion.is_macro(),
                assertion.operands(),
            )
        })
        .collect()
}

fn names(path: &str, contents: &str) -> Vec<String> {
    found(path, contents)
        .into_iter()
        .map(|(name, _, _)| name)
        .collect()
}

#[test]
fn reads_a_python_assert_statement_and_its_operands() {
    assert_eq!(
        found("a.py", "def test_a():\n    assert value == 1\n"),
        vec![("assert".to_owned(), false, 1)]
    );
    assert_eq!(
        found("a.py", "def test_a():\n    assert value == 1, 'explains'\n"),
        vec![("assert".to_owned(), false, 2)],
        "the message is an operand, which is what assertion-message-required will read"
    );
}

#[test]
fn reads_every_python_assertion_call_form() {
    let cases = [
        ("self.assertEqual(a, b)", "self.assertEqual", 2),
        ("assertTrue(value)", "assertTrue", 1),
        ("case.assertIsNone(value)", "case.assertIsNone", 1),
        (
            "client.assert_called_once_with(url)",
            "client.assert_called_once_with",
            1,
        ),
        ("assert_that(value)", "assert_that", 1),
    ];

    for (call, name, operands) in cases {
        let contents = format!("def test_a():\n    {call}\n");

        assert_eq!(
            found("a.py", &contents),
            vec![(name.to_owned(), false, operands)],
            "{call}"
        );
    }
}

#[test]
fn reads_a_pytest_raises_context_manager_as_an_assertion() {
    assert_eq!(
        names(
            "a.py",
            "def test_a():\n    with pytest.raises(ValueError):\n        parse('x')\n"
        ),
        vec!["pytest.raises".to_owned()],
        "asserting that something raises is an assertion, and a test doing only this is not empty"
    );
}

#[test]
fn does_not_treat_a_domain_function_as_an_assertion() {
    assert!(
        names("a.py", "def test_a():\n    assert_invariant(order)\n").is_empty(),
        "matching any name beginning with assert would report a domain helper; #90 rejected that"
    );
    assert!(names("a.py", "def test_a():\n    asserting(order)\n").is_empty());
    assert!(names("a.py", "def test_a():\n    verify(order)\n").is_empty());
}

#[test]
fn reads_every_rust_assertion_macro() {
    let cases = [
        ("assert!(value);", "assert", 1),
        ("assert_eq!(a, b);", "assert_eq", 2),
        ("assert_ne!(a, b);", "assert_ne", 2),
        ("debug_assert!(value);", "debug_assert", 1),
        ("debug_assert_eq!(a, b);", "debug_assert_eq", 2),
        ("debug_assert_ne!(a, b);", "debug_assert_ne", 2),
    ];

    for (call, name, operands) in cases {
        let contents = format!("#[test]\nfn a() {{\n    {call}\n}}\n");

        assert_eq!(
            found("a.rs", &contents),
            vec![(name.to_owned(), true, operands)],
            "{call}"
        );
    }
}

#[test]
fn counts_a_rust_assertion_message_as_an_operand() {
    assert_eq!(
        found(
            "a.rs",
            "#[test]\nfn a() {\n    assert!(value, \"explains\");\n}\n"
        ),
        vec![("assert".to_owned(), true, 2)]
    );
    assert_eq!(
        found(
            "a.rs",
            "#[test]\nfn a() {\n    assert_eq!(a, b, \"explains {x}\", x = 1);\n}\n"
        ),
        vec![("assert_eq".to_owned(), true, 4)]
    );
}

#[test]
fn does_not_miscount_a_rust_trailing_comma() {
    assert_eq!(
        found("a.rs", "#[test]\nfn a() {\n    assert_eq!(a, b,);\n}\n"),
        vec![("assert_eq".to_owned(), true, 2)],
        "a trailing comma is punctuation, not an operand"
    );
}

#[test]
fn counts_a_rust_nested_comma_once() {
    assert_eq!(
        found(
            "a.rs",
            "#[test]\nfn a() {\n    assert_eq!(f(a, b), c);\n}\n"
        ),
        vec![("assert_eq".to_owned(), true, 2)],
        "a comma inside a nested token tree belongs to that tree"
    );
}

#[test]
fn does_not_count_a_comma_inside_a_turbofish() {
    let cases = [
        (
            "#[test]\nfn a() {\n    assert_eq!(HashMap::<String, u32>::new(), m);\n}\n",
            2,
        ),
        (
            "#[test]\nfn a() {\n    assert!(x == Foo::<A, B>::new());\n}\n",
            1,
        ),
        ("#[test]\nfn a() {\n    assert_eq!(f::<A, B>(), c);\n}\n", 2),
        (
            "#[test]\nfn a() {\n    assert_eq!(HashMap::<String, Vec<u32>>::new(), m);\n}\n",
            2,
        ),
        (
            "#[test]\nfn a() {\n    assert_eq!(f::<Vec<u8>, Vec<u16>>(), c);\n}\n",
            2,
        ),
    ];

    for (source, operands) in cases {
        assert_eq!(
            found("a.rs", source).first().map(|(_, _, count)| *count),
            Some(operands),
            "a comma separating type arguments is not separating operands: {source}"
        );
    }
}

#[test]
fn still_counts_a_comparison_and_the_message_after_it() {
    assert_eq!(
        found("a.rs", "#[test]\nfn a() {\n    assert!(a < b);\n}\n"),
        vec![("assert".to_owned(), true, 1)],
        "a less-than is not a generic opener, so nothing may be swallowed after it"
    );
    assert_eq!(
        found("a.rs", "#[test]\nfn a() {\n    assert_eq!(a >> b, c);\n}\n"),
        vec![("assert_eq".to_owned(), true, 2)],
        "a shift closes no generic, and tree-sitter spells it with the same token"
    );
    assert_eq!(
        found(
            "a.rs",
            "#[test]\nfn a() {\n    assert!(a < b, \"explains\");\n}\n"
        ),
        vec![("assert".to_owned(), true, 2)]
    );
}

#[test]
fn does_not_treat_another_rust_macro_as_an_assertion() {
    assert!(names("a.rs", "#[test]\nfn a() {\n    println!(\"{a}\");\n}\n").is_empty());
    assert!(names("a.rs", "#[test]\nfn a() {\n    vec![1, 2];\n}\n").is_empty());
    assert!(
        names("a.rs", "#[test]\nfn a() {\n    assert_invariant!(a);\n}\n").is_empty(),
        "a macro whose name merely begins with assert is not one of the six"
    );
}

#[test]
fn reads_a_rust_should_panic_attribute_as_the_assertion() {
    assert_eq!(
        found(
            "a.rs",
            "#[test]\n#[should_panic]\nfn a() {\n    boom();\n}\n"
        ),
        vec![("should_panic".to_owned(), false, 0)],
        "the attribute is the assertion; without this every should_panic test asserts nothing"
    );
    assert_eq!(
        names(
            "a.rs",
            "#[test]\n#[should_panic(expected = \"overflow\")]\nfn a() {\n    boom();\n}\n"
        ),
        vec!["should_panic".to_owned()],
        "the expected message is an argument to the attribute, not a different attribute"
    );
    assert!(names("a.rs", "#[test]\nfn a() {\n    boom();\n}\n").is_empty());
}

#[test]
fn a_should_panic_assertion_falls_inside_its_own_test() {
    let facts = analyze(&source(
        "a.rs",
        "#[test]\n#[should_panic]\nfn a() {\n    boom();\n}\n",
    ))
    .unwrap_or_else(|error| panic!("analyzes: {error}"));
    let test = facts.tests().first().expect("finds the test");

    assert!(
        test.contains(facts.assertions()[0].range()),
        "the attribute precedes the function, so the assertion is recorded at the function's range"
    );
}

#[test]
fn reads_a_javascript_expect_once_per_chain() {
    assert_eq!(
        found(
            "a.spec.js",
            "it('a', () => {\n  expect(value).toBe(1);\n});\n"
        ),
        vec![("expect".to_owned(), false, 1)],
        "the matcher is a second call on the same chain; counting it twice would double every count"
    );
}

#[test]
fn reads_every_javascript_assertion_form() {
    let cases = [
        ("expect(value).toBe(1);", "expect", 1),
        ("chai.expect(value).to.equal(1);", "chai.expect", 1),
        ("assert(value);", "assert", 1),
        ("assert.equal(a, b);", "assert.equal", 2),
        (
            "assert.deepStrictEqual(a, b, 'explains');",
            "assert.deepStrictEqual",
            3,
        ),
    ];

    for (call, name, operands) in cases {
        let contents = format!("it('a', () => {{\n  {call}\n}});\n");

        assert_eq!(
            found("a.spec.ts", &contents),
            vec![(name.to_owned(), false, operands)],
            "{call}"
        );
    }
}

#[test]
fn reads_a_type_level_assertion() {
    assert_eq!(
        names(
            "a.spec.ts",
            "it('a', () => {\n  expectTypeOf<A>().toEqualTypeOf<B>();\n});\n"
        ),
        vec!["expectTypeOf".to_owned()],
        "a type assertion is an assertion; a typed suite may have no other kind"
    );
    assert_eq!(
        names(
            "a.spec.ts",
            "it('a', () => {\n  assertType<A>(value);\n});\n"
        ),
        vec!["assertType".to_owned()]
    );
    assert!(names("a.spec.ts", "it('a', () => {\n  expectation(value);\n});\n").is_empty());
}

#[test]
fn treats_an_optional_call_as_the_same_assertion() {
    assert_eq!(
        names(
            "a.spec.ts",
            "it('a', () => {\n  assert?.equal(a, b);\n});\n"
        ),
        vec!["assert.equal".to_owned()]
    );
}

#[test]
fn does_not_treat_a_domain_call_as_a_javascript_assertion() {
    assert!(names("a.spec.js", "it('a', () => {\n  inspect(value);\n});\n").is_empty());
    assert!(
        names("a.spec.js", "it('a', () => {\n  expectation(value);\n});\n").is_empty(),
        "a name that merely contains expect is not the assertion"
    );
    assert!(names("a.spec.js", "it('a', () => {\n  asserted(value);\n});\n").is_empty());
}

#[test]
fn records_no_assertion_where_a_file_has_none() {
    for (path, contents) in [
        ("a.py", "def helper():\n    return 1\n"),
        ("a.rs", "fn helper() -> u32 {\n    1\n}\n"),
        ("a.js", "function helper() {\n  return 1;\n}\n"),
    ] {
        assert!(names(path, contents).is_empty(), "{path}");
    }
}

#[test]
fn an_assertions_text_is_the_whole_assertion() {
    let facts = analyze(&source("a.py", "def test_a():\n    assert value == 1\n"))
        .unwrap_or_else(|error| panic!("analyzes: {error}"));

    assert_eq!(
        facts.assertions()[0].text(),
        "assert value == 1",
        "no-duplicate-assertion compares assertions, so the fact must span the whole one"
    );
}

#[test]
fn an_assertion_falls_inside_the_test_that_encloses_it() {
    let facts = analyze(&source("a.py", "def test_a():\n    assert value == 1\n"))
        .unwrap_or_else(|error| panic!("analyzes: {error}"));
    let test = facts.tests().first().expect("finds the test");

    assert!(
        test.contains(facts.assertions()[0].range()),
        "assertion-required counts assertions per test, so containment must hold"
    );
}
