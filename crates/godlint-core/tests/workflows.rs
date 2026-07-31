#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    source::{TextFile, Workflow},
};

fn workflow(body: &str) -> WorkflowFacts {
    read(".github/workflows/ci.yml", body)
}

fn read(path: &str, body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(path), body.into())
        .unwrap_or_else(|error| panic!("creates workflow file: {error}"));

    workflow::read(&file).unwrap_or_else(|error| panic!("reads {path}: {error}"))
}

fn references(facts: &WorkflowFacts) -> Vec<&str> {
    facts
        .actions()
        .iter()
        .map(|action| action.reference())
        .collect()
}

const WORKFLOW: &str = "name: CI

on:
  pull_request:

permissions:
  contents: read

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

jobs:
  build:
    runs-on: ubuntu-latest
    permissions:
      contents: read
    steps:
      - uses: actions/checkout@v4
      - name: Build
        run: cargo build
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: some/action@0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c
";

#[test]
fn only_a_file_in_the_workflow_directory_is_a_workflow() {
    let cases = [
        (".github/workflows/ci.yml", true),
        (".github/workflows/release.yaml", true),
        ("nested/.github/workflows/ci.yml", true),
        (".github/dependabot.yml", false),
        (".github/workflows/notes.md", false),
        ("workflows/ci.yml", false),
        ("godlint.yaml", false),
        (".github/workflows", false),
    ];

    for (path, expected) in cases {
        assert_eq!(
            Workflow::names(&PathBuf::from(path)),
            expected,
            "{path}: a workflow is a YAML file GitHub will run, not any YAML file"
        );
    }
}

#[test]
fn reads_every_action_a_workflow_uses() {
    let facts = workflow(WORKFLOW);

    assert_eq!(
        references(&facts),
        vec![
            "actions/checkout@v4",
            "some/action@0f1a2b3c4d5e6f708192a3b4c5d6e7f8091a2b3c"
        ]
    );
}

#[test]
fn tells_a_commit_from_a_tag() {
    let facts = workflow(WORKFLOW);
    let pinned: Vec<bool> = facts
        .actions()
        .iter()
        .map(|action| action.is_commit())
        .collect();

    assert_eq!(
        pinned,
        vec![false, true],
        "a tag can be moved and a commit cannot, which is the whole of the distinction"
    );
}

#[test]
fn separates_an_action_into_the_parts_a_policy_names() {
    let facts = workflow(WORKFLOW);
    let first = &facts.actions()[0];

    assert_eq!(first.reference(), "actions/checkout@v4");
    assert_eq!(first.name(), "actions/checkout");
    assert_eq!(first.version(), Some("v4"));
    assert_eq!(first.owner(), Some("actions"));
}

#[test]
fn a_reference_without_a_version_has_a_name_and_no_version() {
    let facts = workflow("jobs:\n  a:\n    steps:\n      - uses: actions/checkout\n");
    let action = &facts.actions()[0];

    assert_eq!(action.name(), "actions/checkout");
    assert_eq!(action.version(), None);
    assert!(!action.is_commit(), "no version cannot be a pinned one");
}

#[test]
fn a_local_action_and_a_container_have_no_owner() {
    let facts = workflow(concat!(
        "jobs:\n",
        "  a:\n",
        "    steps:\n",
        "      - uses: ./.github/actions/setup\n",
        "      - uses: docker://alpine:3.20\n",
    ));
    let owners: Vec<Option<&str>> = facts
        .actions()
        .iter()
        .map(|action| action.owner())
        .collect();

    assert_eq!(
        owners,
        vec![None, None],
        "a path in this repository and an image are not somebody's action"
    );
    assert!(facts.actions()[0].is_local());
    assert!(facts.actions()[1].is_container());
}

#[test]
fn a_quoted_reference_reads_the_same_as_a_bare_one() {
    let facts = workflow(concat!(
        "jobs:\n",
        "  a:\n",
        "    steps:\n",
        "      - uses: \"actions/checkout@v4\"\n",
        "      - uses: 'actions/cache@v4'\n",
    ));

    assert_eq!(
        references(&facts),
        vec!["actions/checkout@v4", "actions/cache@v4"]
    );
}

#[test]
fn a_job_calling_a_reusable_workflow_is_an_action_like_any_other() {
    let facts = workflow("jobs:\n  release:\n    uses: ./.github/workflows/publish.yml\n");

    assert_eq!(references(&facts), vec!["./.github/workflows/publish.yml"]);
}

#[test]
fn reads_where_permissions_are_declared() {
    let facts = workflow(WORKFLOW);
    let declared: Vec<(&str, bool)> = facts
        .jobs()
        .iter()
        .map(|job| (job.name(), job.declares_permissions()))
        .collect();

    assert!(
        facts.declares_permissions(),
        "the workflow declares its own"
    );
    assert_eq!(
        declared,
        vec![("build", true), ("publish", false)],
        "a job either narrows the workflow's permissions or inherits them"
    );
}

#[test]
fn a_workflow_declaring_nothing_says_so_rather_than_failing() {
    let facts = workflow("name: CI\non:\n  push:\n");

    assert!(facts.actions().is_empty());
    assert!(facts.jobs().is_empty());
    assert!(!facts.declares_permissions());
    assert!(!facts.declares_concurrency());
}

#[test]
fn reads_whether_a_workflow_declares_concurrency() {
    assert!(workflow(WORKFLOW).declares_concurrency());
    assert!(!workflow("name: CI\njobs:\n  a:\n    steps: []\n").declares_concurrency());
}

#[test]
fn a_finding_can_point_at_the_line_and_column_of_a_use() {
    let facts = workflow(WORKFLOW);
    let location = facts.file().location(facts.actions()[0].range());

    assert_eq!(location.start.line, 19);
    assert_eq!(location.start.column, 15);
}

#[test]
fn a_use_written_in_a_comment_or_a_string_is_not_a_use() {
    let facts = workflow(concat!(
        "jobs:\n",
        "  a:\n",
        "    steps:\n",
        "      # - uses: evil/action@v1\n",
        "      - name: 'uses: evil/action@v2'\n",
        "        run: 'echo \"uses: evil/action@v3\"'\n",
    ));

    assert!(
        facts.unparsed().is_empty(),
        "the fixture must be YAML GitHub would accept, or it proves nothing"
    );
    assert!(
        facts.actions().is_empty(),
        "this is what reading the syntax buys over grepping the text: {:?}",
        references(&facts)
    );
}

#[test]
fn a_workflow_that_is_not_a_mapping_is_read_as_declaring_nothing() {
    for body in ["", "- one\n- two\n", "just a string\n"] {
        let facts = workflow(body);

        assert!(facts.jobs().is_empty(), "{body:?}");
        assert!(!facts.declares_permissions(), "{body:?}");
    }
}

#[test]
fn records_where_it_stopped_understanding_a_workflow() {
    let file = TextFile::new(
        PathBuf::from(".github/workflows/torn.yml"),
        "jobs:\n  build:\n    steps:\n      - run: echo \"a: b\"\n".into(),
    )
    .unwrap_or_else(|error| panic!("creates workflow file: {error}"));
    let facts = workflow::read(&file).unwrap_or_else(|error| panic!("still reads: {error}"));

    assert!(
        !facts.unparsed().is_empty(),
        "a plain scalar containing a colon is not valid YAML, and a workflow Godlint cannot \
         read must say so rather than report nothing"
    );
}

#[test]
fn a_workflow_it_understands_records_nothing_unparsed() {
    assert!(workflow(WORKFLOW).unparsed().is_empty());
}
