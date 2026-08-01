#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::PathBuf;

use godlint_core::{analyzers::workflow, source::TextFile};

fn workflow(body: &str) -> godlint_core::analyzers::workflow::WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));
    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

#[test]
fn step_ids_and_continue_on_error_values_stay_literal() {
    let facts = workflow(concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - id: probe\n",
        "        continue-on-error: false\n",
        "        run: check\n",
        "      - run: report\n",
    ));
    let configured = &facts.steps()[0];
    let range = configured.continue_on_error().unwrap();
    let absent = &facts.steps()[1];

    assert_eq!(configured.id(), Some("probe"));
    assert_eq!(&facts.file().text()[range.start()..range.end()], "false");
    assert_eq!(absent.id(), None);
    assert_eq!(absent.continue_on_error(), None);
}

#[test]
fn job_continue_on_error_values_stay_literal() {
    let facts = workflow(concat!(
        "jobs:\n",
        "  optional:\n",
        "    continue-on-error: ${{ inputs.optional }}\n",
        "    runs-on: ubuntu-latest\n",
        "    steps: []\n",
        "  required:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps: []\n",
    ));
    let range = facts.jobs()[0].continue_on_error().unwrap();

    assert_eq!(
        &facts.file().text()[range.start()..range.end()],
        "${{ inputs.optional }}"
    );
    assert_eq!(facts.jobs()[1].continue_on_error(), None);
}
