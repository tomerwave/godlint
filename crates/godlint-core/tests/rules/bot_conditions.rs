use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{BotConditionsRule, Severity},
    rules::{Violation, bot_conditions::BotConditions, evaluate_workflow_rule},
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));

    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn violations(body: &str, bots: &[&str]) -> Vec<Violation> {
    let facts = workflow(body);
    let configuration = BotConditionsRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        bots: bots.iter().map(|bot| (*bot).to_owned()).collect(),
    };

    evaluate_workflow_rule::<BotConditions>(std::slice::from_ref(&facts), &configuration)
        .into_iter()
        .map(|finding| finding.violation)
        .collect()
}

#[test]
fn a_braced_step_condition_comparing_the_actor_to_a_bot_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - if: ${{ github.actor == 'dependabot[bot]' }}\n",
        "        run: publish\n",
    );

    assert_eq!(
        violations(body, &["dependabot[bot]"]),
        vec![Violation::AttackerInfluencedBotCondition {
            expression: "github.actor == 'dependabot[bot]'".to_owned(),
        }]
    );
}

#[test]
fn an_unbraced_step_condition_comparing_the_actor_to_a_bot_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - if: github.actor == 'dependabot[bot]'\n",
        "        run: publish\n",
    );

    assert_eq!(violations(body, &["dependabot[bot]"]).len(), 1);
}

#[test]
fn a_braced_job_condition_comparing_the_actor_to_a_bot_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  publish:\n",
        "    if: ${{ github.actor == 'github-actions[bot]' }}\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - run: publish\n",
    );

    assert_eq!(violations(body, &["github-actions[bot]"]).len(), 1);
}

#[test]
fn an_unbraced_job_condition_comparing_the_actor_to_a_bot_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  publish:\n",
        "    if: github.actor == 'github-actions[bot]'\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - run: publish\n",
    );

    assert_eq!(violations(body, &["github-actions[bot]"]).len(), 1);
}

#[test]
fn contains_with_the_actor_and_a_bot_name_fragment_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  publish:\n",
        "    if: ${{ contains(github.actor, 'dependabot') }}\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - run: publish\n",
    );

    assert_eq!(violations(body, &["dependabot[bot]"]).len(), 1);
}

#[test]
fn contains_requires_the_actor_as_the_search_value() {
    let body = concat!(
        "jobs:\n",
        "  publish:\n",
        "    if: ${{ contains(github.sha, 'dependabot') }}\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - run: publish\n",
    );

    assert!(violations(body, &["dependabot[bot]"]).is_empty());
}

#[test]
fn contains_requires_a_non_empty_bot_name_fragment() {
    let body = concat!(
        "jobs:\n",
        "  publish:\n",
        "    if: ${{ contains(github.actor, '') }}\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - run: publish\n",
    );

    assert!(violations(body, &["dependabot[bot]"]).is_empty());
}

#[test]
fn malformed_actor_checks_are_silent() {
    let conditions = [
        "github.actor == dependabot",
        "contains github.actor, 'dependabot')",
        "contains(github.actor)",
        "contains(github.actor, )",
        "contains(github.actor, dependabot)",
        "contains(github.actor, 'dependabot)",
    ];

    for condition in conditions {
        let body = format!(
            "jobs:\n  publish:\n    if: {condition}\n    runs-on: ubuntu-latest\n    steps: []\n"
        );

        assert!(
            violations(&body, &["dependabot[bot]"]).is_empty(),
            "{condition}"
        );
    }
}

#[test]
fn a_double_quoted_bot_identity_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  publish:\n",
        "    if: github.actor == \"dependabot[bot]\"\n",
        "    runs-on: ubuntu-latest\n",
        "    steps: []\n",
    );

    assert_eq!(violations(body, &["dependabot[bot]"]).len(), 1);
}

#[test]
fn a_triggering_actor_comparison_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  publish:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - if: ${{ github.triggering_actor != 'renovate[bot]' }}\n",
        "        run: publish\n",
    );

    assert_eq!(violations(body, &["renovate[bot]"]).len(), 1);
}

#[test]
fn context_normalisation_makes_an_uppercase_actor_match() {
    let body = concat!(
        "jobs:\n",
        "  publish:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - if: ${{ GITHUB.ACTOR == 'dependabot[bot]' }}\n",
        "        run: publish\n",
    );

    assert_eq!(violations(body, &["dependabot[bot]"]).len(), 1);
}

#[test]
fn a_configured_repository_bot_is_reported() {
    let body = concat!(
        "jobs:\n",
        "  publish:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - if: ${{ github.actor == 'release-machine[bot]' }}\n",
        "        run: publish\n",
    );

    assert_eq!(violations(body, &["release-machine[bot]"]).len(), 1);
    assert!(violations(body, &["dependabot[bot]"]).is_empty());
}

#[test]
fn actor_reads_outside_conditions_are_not_reported() {
    let body = concat!(
        "jobs:\n",
        "  publish:\n",
        "    runs-on: ubuntu-latest\n",
        "    env:\n",
        "      BOT: ${{ github.actor == 'dependabot[bot]' }}\n",
        "    steps:\n",
        "      - run: echo ${{ github.actor == 'dependabot[bot]' }}\n",
    );

    assert!(violations(body, &["dependabot[bot]"]).is_empty());
}

#[test]
fn an_actor_condition_without_a_configured_bot_is_not_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - if: ${{ github.actor == 'trusted-human' }}\n",
        "        run: build\n",
    );

    assert!(violations(body, &["dependabot[bot]"]).is_empty());
}

#[test]
fn an_actor_read_that_is_not_a_bot_comparison_is_not_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - if: ${{ github.actor }}\n",
        "        run: inspect\n",
        "      - if: ${{ github.actor && true }}\n",
        "        run: inspect\n",
    );

    assert!(violations(body, &["dependabot[bot]"]).is_empty());
}

#[test]
fn a_bot_name_that_is_only_a_substring_is_not_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    runs-on: ubuntu-latest\n",
        "    steps:\n",
        "      - if: ${{ github.actor == 'not-dependabot[bot]' }}\n",
        "        run: build\n",
    );

    assert!(violations(body, &["dependabot[bot]"]).is_empty());
}

#[test]
fn the_message_quotes_the_expression_and_explains_why_it_proves_nothing() {
    let violation = Violation::AttackerInfluencedBotCondition {
        expression: "github.actor == 'dependabot[bot]'".to_owned(),
    };
    let message = violation.to_string();

    assert!(
        message.contains("\"github.actor == 'dependabot[bot]'\""),
        "{message}"
    );
    assert!(
        message.contains("attacker-influenced on the trigger"),
        "{message}"
    );
    assert!(message.contains("passing it proves nothing"), "{message}");
    assert!(
        message.contains("github.event.pull_request.user.login"),
        "{message}"
    );
    assert!(message.contains("verify the app"), "{message}");
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    let facts = workflow(concat!(
        "jobs:\n",
        "  build:\n",
        "    if: ${{ github.actor == 'dependabot[bot]' }}\n",
        "    runs-on: ubuntu-latest\n",
        "    steps: []\n",
    ));
    let configuration = BotConditionsRule {
        severity: Severity::Off,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        bots: vec!["dependabot[bot]".to_owned()],
    };

    assert!(
        evaluate_workflow_rule::<BotConditions>(std::slice::from_ref(&facts), &configuration)
            .is_empty()
    );
}
