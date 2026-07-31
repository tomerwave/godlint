#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    facts::{JobFact, Secrets, StepFact},
    source::{SourceRange, TextFile, Workflow},
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

fn text(facts: &WorkflowFacts, range: SourceRange) -> &str {
    &facts.file().text()[range.start()..range.end()]
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

const DETAILED_WORKFLOW: &str = r#"name: Detailed CI

# workflow comment
on:
  pull_request:

jobs:
  prepare: # job comment
    runs-on: ubuntu-latest
  build:
    needs: prepare
    runs-on: ubuntu-latest
    container:
      image: registry.example.test/build:latest
      credentials:
        username: builder
        password: 'literal: password'
    services:
      database:
        image: postgres:17
        credentials:
          username: ${{ secrets.SERVICE_USER }}
          password: ${{ secrets.SERVICE_PASSWORD }}
    steps:
      # step comment
      - name: 'Checkout source'
        if: ${{ GitHub.ref == 'refs/heads/main' }}
        uses: actions/checkout@v4
        with:
          token: ${{ secrets.TOKEN }}
          label: "${{ format('{0}', github.actor) }}"
        env:
          MODE: test
      - run: |
          echo "${{ github.actor }}"
  matrix:
    needs: [prepare, build]
    runs-on: ubuntu-latest
    steps: []
  release:
    needs:
      - build
      - matrix
    uses: ./.github/workflows/release.yml
    secrets: inherit
  deploy:
    needs: release
    uses: owner/repository/.github/workflows/deploy.yml@v1
    secrets:
      token: ${{ secrets.DEPLOY_TOKEN }}

# ignored expression ${{ secrets.COMMENT_TOKEN }}
"#;

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

#[test]
fn reads_each_step_as_a_job_owned_site_with_its_settings() {
    let facts = workflow(DETAILED_WORKFLOW);

    assert!(facts.unparsed().is_empty());
    assert_eq!(facts.steps().len(), 2);
    assert_checkout_step(&facts, &facts.steps()[0]);
    assert_command_step(&facts, &facts.steps()[1]);
}

fn assert_checkout_step(facts: &WorkflowFacts, checkout: &StepFact) {
    assert_eq!(checkout.file(), facts.file());
    assert_eq!(text(facts, checkout.range()), "name");
    assert_eq!(checkout.job(), "build");
    assert_eq!(checkout.name(), Some("Checkout source"));
    assert_eq!(text(facts, checkout.uses().unwrap()), "actions/checkout@v4");
    assert_eq!(
        text(facts, checkout.condition().unwrap()),
        "${{ GitHub.ref == 'refs/heads/main' }}"
    );
    assert_eq!(
        checkout
            .inputs()
            .iter()
            .map(|setting| (setting.key(), text(facts, setting.range())))
            .collect::<Vec<_>>(),
        vec![
            ("token", "${{ secrets.TOKEN }}"),
            ("label", "\"${{ format('{0}', github.actor) }}\"")
        ]
    );
    assert_eq!(checkout.inputs()[0].file(), facts.file());
    assert_eq!(checkout.environment()[0].key(), "MODE");
    assert_eq!(text(facts, checkout.environment()[0].range()), "test");
}

fn assert_command_step(facts: &WorkflowFacts, command: &StepFact) {
    assert_eq!(text(facts, command.range()), "run");
    assert_eq!(command.name(), None);
    assert!(text(facts, command.run().unwrap()).contains("${{ github.actor }}"));
}

#[test]
fn reads_job_dependencies_calls_secrets_bodies_and_step_counts() {
    let facts = workflow(DETAILED_WORKFLOW);
    let jobs = facts.jobs();

    assert_eq!(jobs.len(), 5);
    assert_build_job(&facts, &jobs[1]);
    assert_job_needs(&jobs[2], &["prepare", "build"]);
    assert_job_needs(&jobs[3], &["build", "matrix"]);
    assert_reusable_job(&facts, &jobs[3]);
    assert_named_secrets(&facts, &jobs[4]);
}

fn assert_build_job(facts: &WorkflowFacts, build: &JobFact) {
    assert!(text(facts, build.body()).contains("needs: prepare"));
    assert_eq!(build.step_count(), 2);
    assert_eq!(build.needs()[0].key(), "prepare");
    assert_eq!(text(facts, build.needs()[0].range()), "prepare");
    assert_eq!(build.needs()[0].file(), facts.file());
}

fn assert_job_needs(job: &JobFact, expected: &[&str]) {
    assert_eq!(
        job.needs()
            .iter()
            .map(|need| need.key())
            .collect::<Vec<_>>(),
        expected
    );
}

fn assert_reusable_job(facts: &WorkflowFacts, release: &JobFact) {
    assert_eq!(release.step_count(), 0);
    assert_eq!(
        text(facts, release.calls_workflow().unwrap()),
        "./.github/workflows/release.yml"
    );
    match release.secrets() {
        Some(Secrets::Inherit { range }) => assert_eq!(text(facts, *range), "inherit"),
        other => panic!("expected inherited secrets, got {other:?}"),
    }
}

fn assert_named_secrets(facts: &WorkflowFacts, deploy: &JobFact) {
    assert_eq!(
        text(facts, deploy.calls_workflow().unwrap()),
        "owner/repository/.github/workflows/deploy.yml@v1"
    );
    match deploy.secrets() {
        Some(Secrets::Named(settings)) => {
            assert_eq!(settings[0].key(), "token");
            assert_eq!(
                text(facts, settings[0].range()),
                "${{ secrets.DEPLOY_TOKEN }}"
            );
        }
        other => panic!("expected named secrets, got {other:?}"),
    }
}

#[test]
fn expressions_are_ordered_scalar_ranges_with_matchable_contexts() {
    let facts = workflow(DETAILED_WORKFLOW);
    let expressions = facts.expressions();

    assert_eq!(
        expressions
            .iter()
            .map(|fact| fact.body())
            .collect::<Vec<_>>(),
        vec![
            "secrets.SERVICE_USER",
            "secrets.SERVICE_PASSWORD",
            "GitHub.ref == 'refs/heads/main'",
            "secrets.TOKEN",
            "format('{0}', github.actor)",
            "github.actor",
            "secrets.DEPLOY_TOKEN"
        ]
    );
    assert_eq!(
        expressions
            .iter()
            .map(|fact| fact.context())
            .collect::<Vec<_>>(),
        vec![
            "secrets.service_user",
            "secrets.service_password",
            "github.ref",
            "secrets.token",
            "format('{0}', github.actor)",
            "github.actor",
            "secrets.deploy_token"
        ]
    );
    assert_eq!(
        text(&facts, expressions[2].range()),
        "${{ GitHub.ref == 'refs/heads/main' }}"
    );
    let condition = facts.steps()[0].condition().unwrap();
    let run = facts.steps()[1].run().unwrap();
    assert!(condition.start() <= expressions[2].range().start());
    assert!(expressions[2].range().end() <= condition.end());
    assert!(run.start() <= expressions[5].range().start());
    assert!(expressions[5].range().end() <= run.end());
}

#[test]
fn an_expression_written_in_a_yaml_comment_is_not_an_expression() {
    let facts =
        workflow("jobs:\n  build:\n    # ${{ secrets.TOKEN }}\n    runs-on: ubuntu-latest\n");

    assert!(facts.unparsed().is_empty());
    assert!(facts.expressions().is_empty());
    assert_eq!(facts.comments().len(), 1);
}

#[test]
fn comments_are_source_ordered_ranges_including_inline_comments() {
    let facts = workflow(DETAILED_WORKFLOW);
    let comments = facts
        .comments()
        .iter()
        .map(|range| text(&facts, *range))
        .collect::<Vec<_>>();

    assert_eq!(
        comments,
        vec![
            "# workflow comment",
            "# job comment",
            "# step comment",
            "# ignored expression ${{ secrets.COMMENT_TOKEN }}"
        ]
    );
}

#[test]
fn credentials_are_scoped_to_their_job_and_classified_by_interpolation() {
    let facts = workflow(DETAILED_WORKFLOW);
    let credentials = facts.credentials();

    assert_eq!(credentials.len(), 4);
    assert_eq!(
        credentials
            .iter()
            .map(|fact| (
                fact.key(),
                fact.job(),
                text(&facts, fact.range()),
                fact.is_literal()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("username", "build", "builder", true),
            ("password", "build", "'literal: password'", true),
            ("username", "build", "${{ secrets.SERVICE_USER }}", false),
            (
                "password",
                "build",
                "${{ secrets.SERVICE_PASSWORD }}",
                false
            )
        ]
    );
}

#[test]
fn empty_workflow_shapes_produce_empty_collections_without_parse_gaps() {
    let no_jobs = workflow("name: Empty\non:\n  push:\n");
    let no_steps = workflow("jobs:\n  build:\n    runs-on: ubuntu-latest\n");
    let run_only = workflow("jobs:\n  build:\n    steps:\n      - run: echo ok\n");

    for facts in [&no_jobs, &no_steps, &run_only] {
        assert!(facts.unparsed().is_empty());
    }
    assert_empty_workflow(&no_jobs);
    assert_eq!(no_steps.jobs()[0].step_count(), 0);
    assert!(no_steps.steps().is_empty());
    assert_eq!(run_only.steps()[0].name(), None);
    assert_eq!(text(&run_only, run_only.steps()[0].range()), "run");
    assert_eq!(
        text(&run_only, run_only.steps()[0].run().unwrap()),
        "echo ok"
    );
}

fn assert_empty_workflow(facts: &WorkflowFacts) {
    assert!(facts.jobs().is_empty());
    assert!(facts.steps().is_empty());
    assert!(facts.expressions().is_empty());
    assert!(facts.comments().is_empty());
    assert!(facts.credentials().is_empty());
}
