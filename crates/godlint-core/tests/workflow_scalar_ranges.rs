#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use godlint_core::{analyzers::workflow, source::TextFile};

fn workflow(body: &str) -> workflow::WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow file: {error}"));

    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn expression_details(facts: &workflow::WorkflowFacts) -> Vec<(usize, usize, &str, &str)> {
    facts
        .expressions()
        .iter()
        .map(|fact| {
            (
                fact.range().start(),
                fact.range().end(),
                fact.body(),
                fact.context(),
            )
        })
        .collect()
}

#[test]
fn commands_are_the_script_inside_each_yaml_scalar() {
    let cases = [
        (
            "|\n          echo one\n          echo two",
            "echo one\n          echo two",
        ),
        (
            ">\n          echo one\n          echo two",
            "echo one\n          echo two",
        ),
        ("'echo one'", "echo one"),
        ("\"echo one\"", "echo one"),
        ("echo one", "echo one"),
    ];

    for (scalar, expected) in cases {
        let facts = workflow(&format!(
            "jobs:\n  build:\n    steps:\n      - run: {scalar}\n"
        ));
        let run = facts.steps()[0].run().unwrap();
        let range = facts.steps()[0].run_range().unwrap();

        assert_eq!(run, expected, "{scalar:?}");
        let ranged = scalar.strip_prefix(['\'', '"']).map_or(expected, |body| {
            body.strip_suffix(['\'', '"']).unwrap_or(body)
        });
        assert_eq!(facts.file().slice(range), ranged, "{scalar:?} range");
    }
}

#[test]
fn expression_context_paths_are_case_insensitive() {
    let facts = workflow(concat!(
        "jobs:\n  build:\n    steps:\n",
        "      - run: echo ${{ GITHUB.ACTOR }}\n",
        "      - run: echo ${{ github.Event.Pull_Request.Title }}\n",
        "      - run: echo ${{ SECRETS.TOKEN }}\n",
    ));
    let expressions = facts.expressions();

    assert_eq!(
        expressions
            .iter()
            .map(|fact| fact.context())
            .collect::<Vec<_>>(),
        [
            "github.actor",
            "github.event.pull_request.title",
            "secrets.token"
        ]
    );
    assert_eq!(
        expressions
            .iter()
            .map(|fact| fact.body())
            .collect::<Vec<_>>(),
        [
            "GITHUB.ACTOR",
            "github.Event.Pull_Request.Title",
            "SECRETS.TOKEN"
        ]
    );
}

#[test]
fn reads_a_job_condition_as_its_scalar_range() {
    let facts = workflow(concat!(
        "jobs:\n",
        "  guarded:\n",
        "    if: ${{ GITHUB.ACTOR == 'dependabot[bot]' }}\n",
        "    runs-on: ubuntu-latest\n",
        "    steps: []\n",
        "  unguarded:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps: []\n",
    ));
    let condition = facts.jobs()[0].condition().unwrap();

    assert_eq!(
        &facts.file().text()[condition.start()..condition.end()],
        "${{ GITHUB.ACTOR == 'dependabot[bot]' }}"
    );
    assert_eq!(facts.jobs()[1].condition(), None);
}

#[test]
fn finds_both_expressions_on_one_run_line() {
    let facts = workflow(concat!(
        "jobs:\n  build:\n    steps:\n",
        "      - run: echo \"${{ github.actor }} pushed ${{ github.sha }}\"\n",
    ));

    assert_eq!(
        expression_details(&facts),
        [
            (45, 64, "github.actor", "github.actor"),
            (72, 89, "github.sha", "github.sha"),
        ]
    );
}

#[test]
fn finds_expressions_on_different_lines_of_one_block_scalar() {
    let facts = workflow(concat!(
        "jobs:\n  build:\n    steps:\n",
        "      - run: |\n",
        "          echo \"${{ github.actor }}\"\n",
        "          echo \"${{ github.sha }}\"\n",
    ));

    assert_eq!(
        expression_details(&facts),
        [
            (57, 76, "github.actor", "github.actor"),
            (94, 111, "github.sha", "github.sha"),
        ]
    );
}

#[test]
fn pins_expression_offsets_with_text_before_between_and_after() {
    let facts = workflow(concat!(
        "jobs:\n  build:\n    steps:\n",
        "      - run: before-${{ github.ref }}-between-${{ runner.os }}-after\n",
    ));

    assert_eq!(
        expression_details(&facts),
        [
            (46, 63, "github.ref", "github.ref"),
            (72, 88, "runner.os", "runner.os"),
        ]
    );
}
