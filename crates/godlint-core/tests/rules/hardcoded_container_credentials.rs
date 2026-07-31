use std::path::PathBuf;

use godlint_core::{
    analyzers::workflow::{self, WorkflowFacts},
    config::{HardcodedContainerCredentialsRule, Severity},
    rules::{
        Violation, evaluate_workflow_rule,
        hardcoded_container_credentials::HardcodedContainerCredentials,
    },
    source::TextFile,
};

fn workflow(body: &str) -> WorkflowFacts {
    let file = TextFile::new(PathBuf::from(".github/workflows/ci.yml"), body.into())
        .unwrap_or_else(|error| panic!("creates workflow: {error}"));

    workflow::read(&file).unwrap_or_else(|error| panic!("reads workflow: {error}"))
}

fn violations(body: &str) -> Vec<Violation> {
    let facts = workflow(body);
    let configuration = HardcodedContainerCredentialsRule {
        severity: Severity::Error,
    };

    evaluate_workflow_rule::<HardcodedContainerCredentials>(
        std::slice::from_ref(&facts),
        &configuration,
    )
    .into_iter()
    .map(|finding| finding.violation)
    .collect()
}

#[test]
fn literal_usernames_and_passwords_in_job_containers_and_services_are_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    container:\n",
        "      image: registry.example.test/build:latest\n",
        "      credentials:\n",
        "        username: builder\n",
        "        password: 'literal: password'\n",
        "    services:\n",
        "      database:\n",
        "        image: registry.example.test/database:latest\n",
        "        credentials:\n",
        "          username: service-user\n",
        "          password: service-password\n",
        "    steps:\n",
        "      - run: cargo build\n",
    );

    assert_eq!(
        violations(body),
        vec![
            Violation::HardcodedContainerCredential {
                key: "username".to_owned(),
                job: "build".to_owned(),
            },
            Violation::HardcodedContainerCredential {
                key: "password".to_owned(),
                job: "build".to_owned(),
            },
            Violation::HardcodedContainerCredential {
                key: "username".to_owned(),
                job: "build".to_owned(),
            },
            Violation::HardcodedContainerCredential {
                key: "password".to_owned(),
                job: "build".to_owned(),
            },
        ]
    );
}

#[test]
fn interpolated_container_and_service_credentials_are_not_reported() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    container:\n",
        "      image: registry.example.test/build:latest\n",
        "      credentials:\n",
        "        username: ${{ secrets.CONTAINER_USER }}\n",
        "        password: prefix-${{ secrets.CONTAINER_PASSWORD }}\n",
        "    services:\n",
        "      database:\n",
        "        image: registry.example.test/database:latest\n",
        "        credentials:\n",
        "          username: ${{ secrets.SERVICE_USER }}\n",
        "          password: ${{ secrets.SERVICE_PASSWORD }}\n",
        "    steps:\n",
        "      - run: cargo build\n",
    );

    assert!(violations(body).is_empty());
}

#[test]
fn unrelated_container_keys_are_not_credentials() {
    let body = concat!(
        "jobs:\n",
        "  build:\n",
        "    container:\n",
        "      image: private-image\n",
        "      credentials:\n",
        "        token: literal-token\n",
        "    steps:\n",
        "      - run: cargo build\n",
    );

    assert!(violations(body).is_empty());
}

#[test]
fn the_rule_is_silent_when_it_is_switched_off() {
    let facts = workflow(concat!(
        "jobs:\n",
        "  build:\n",
        "    container:\n",
        "      image: private-image\n",
        "      credentials:\n",
        "        username: builder\n",
    ));
    let configuration = HardcodedContainerCredentialsRule {
        severity: Severity::Off,
    };

    assert!(
        evaluate_workflow_rule::<HardcodedContainerCredentials>(
            std::slice::from_ref(&facts),
            &configuration
        )
        .is_empty()
    );
}
