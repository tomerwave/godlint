use std::path::PathBuf;

use godlint_core::{analyzers::manifest::dependency_specs, source::TextFile};

fn read(text: &str) -> Vec<(String, String, String)> {
    let file = TextFile::new(PathBuf::from("package.json"), text.to_owned())
        .unwrap_or_else(|error| panic!("creates manifest: {error}"));

    dependency_specs(&file)
        .iter()
        .map(|fact| {
            (
                fact.section().to_owned(),
                fact.package().to_owned(),
                fact.spec().to_owned(),
            )
        })
        .collect()
}

fn entry(section: &str, package: &str, spec: &str) -> (String, String, String) {
    (section.to_owned(), package.to_owned(), spec.to_owned())
}

#[test]
fn reads_every_section_that_installs_a_package() {
    assert_eq!(
        read(
            r#"{"dependencies":{"a":"1"},"devDependencies":{"b":"2"},"optionalDependencies":{"c":"3"}}"#
        ),
        vec![
            entry("dependencies", "a", "1"),
            entry("devDependencies", "b", "2"),
            entry("optionalDependencies", "c", "3"),
        ]
    );
}

#[test]
fn ignores_peer_dependencies_and_every_other_key() {
    assert_eq!(
        read(
            r#"{"peerDependencies":{"a":"^1"},"overrides":{"b":"^2"},"name":"dependencies","dependencies":{}}"#
        ),
        Vec::new()
    );
}

#[test]
fn skips_values_of_every_shape_around_the_sections() {
    let manifest = "{\n\t\"name\": \"web\",\r\n  \"private\": true,\n  \"count\": -1.5e3,\n  \"nothing\": null,\n  \"files\": [\"a\", {\"b\": \"}\"}, [1, 2]],\n  \"scripts\": { \"x\": \"say \\\"}\\\"\" },\n  \"dependencies\": { \"a\" : \"1.0.0\" , \"b\": { \"version\": \"2\" }, \"c\": [\"3\"], \"d\": \"4\" }\n}\n";

    assert_eq!(
        read(manifest),
        vec![
            entry("dependencies", "a", "1.0.0"),
            entry("dependencies", "d", "4")
        ]
    );
}

#[test]
fn keeps_an_escaped_quote_inside_a_spec() {
    assert_eq!(
        read(r#"{"dependencies":{"a":"1\"2"}}"#),
        vec![entry("dependencies", "a", "1\\\"2")]
    );
}

#[test]
fn reads_an_empty_section_and_an_empty_spec() {
    assert_eq!(
        read(r#"{"dependencies":{},"devDependencies":{"a":""}}"#),
        vec![entry("devDependencies", "a", "")]
    );
}

#[test]
fn ignores_a_section_that_is_not_an_object() {
    assert_eq!(
        read(r#"{"dependencies":["a"],"devDependencies":"b","optionalDependencies":{"c":"1"}}"#),
        vec![entry("optionalDependencies", "c", "1")]
    );
}

#[test]
fn records_the_quoted_spec_as_the_range() {
    let text = r#"{"dependencies":{"a":"^1"}}"#;
    let file = TextFile::new(PathBuf::from("package.json"), text.to_owned())
        .unwrap_or_else(|error| panic!("creates manifest: {error}"));
    let facts = dependency_specs(&file);
    let range = facts
        .first()
        .unwrap_or_else(|| panic!("reads one spec"))
        .range();

    assert_eq!(&text[range.start()..range.end()], "\"^1\"");
    assert_eq!(facts.len(), 1);
}

#[test]
fn reads_nothing_from_a_document_that_is_not_an_object() {
    for text in [
        "",
        "   ",
        "[]",
        "\"dependencies\"",
        "{",
        "{\"dependencies\"",
        "{\"dependencies\":",
        "{\"dependencies\":{\"a\":\"1\"",
        "{\"dependencies\":{\"a\":\"1\"}",
        "{\"dependencies\":{\"a\":\"1\"},",
        "{\"dependencies\":{\"a\":\"1",
        "{\"dependencies\":{\"a\":\"1\\",
        "{\"dependencies\":{\"a\" \"1\"}}",
        "{\"dependencies\":{\"a\":\"1\" \"b\":\"2\"}}",
        "{\"dependencies\":{\"a\":\"1\"} \"devDependencies\":{}}",
        "{dependencies:{\"a\":\"1\"}}",
        "{\"dependencies\":{a:\"1\"}}",
        "{\"name\":[1,2",
        "{\"name\":\"web",
        "{\"name\"",
    ] {
        assert_eq!(read(text), Vec::new(), "{text:?}");
    }
}

#[test]
fn reads_a_document_whose_last_value_is_a_scalar() {
    assert_eq!(
        read("{\"dependencies\":{\"a\":\"1\"},\"private\":true}"),
        vec![entry("dependencies", "a", "1")]
    );
    assert_eq!(
        read("{\"private\":true,\"dependencies\":{\"a\":\"1\"}}"),
        vec![entry("dependencies", "a", "1")]
    );
    assert_eq!(
        read("{\"private\" : true , \"dependencies\":{\"a\":\"1\"}}"),
        vec![entry("dependencies", "a", "1")]
    );
}
