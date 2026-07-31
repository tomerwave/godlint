use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, ExplicitWorkflowPermissionsRule, Severity},
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
    },
    source::SourceRange,
};

pub struct ExplicitWorkflowPermissions;

impl Rule for ExplicitWorkflowPermissions {
    const ID: &'static str = "ci/explicit-workflow-permissions";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = ExplicitWorkflowPermissionsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for ExplicitWorkflowPermissions {
    fn check(
        workflow: &WorkflowFacts,
        configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        if declares_nothing_anywhere(workflow) {
            return vec![(
                workflow.file().full_range(),
                Violation::UndeclaredPermissions,
            )];
        }

        workflow
            .jobs()
            .iter()
            .filter(|job| !job.declares_permissions())
            .filter(|_| configuration.require_per_job || !workflow.declares_permissions())
            .map(|job| {
                (
                    job.range(),
                    Violation::InheritedPermissions {
                        job: job.name().to_owned(),
                    },
                )
            })
            .collect()
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(
        config.rules.explicit_workflow_permissions.as_ref(),
        |rule| evaluate_workflow_rule::<ExplicitWorkflowPermissions>(workflows, rule),
    )
}

fn declares_nothing_anywhere(workflow: &WorkflowFacts) -> bool {
    !workflow.declares_permissions()
        && !workflow.jobs().is_empty()
        && workflow
            .jobs()
            .iter()
            .all(|job| !job.declares_permissions())
}
