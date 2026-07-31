use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, PinThirdPartyActionsRule, Severity},
    facts::ActionFact,
    rules::{
        ActionRule, Finding, Languages, Rule, Violation, evaluate_action_rule, when_configured,
    },
};

pub struct PinThirdPartyActions;

impl Rule for PinThirdPartyActions {
    const ID: &'static str = "ci/pin-third-party-actions";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = PinThirdPartyActionsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl ActionRule for PinThirdPartyActions {
    fn check(action: &ActionFact, configuration: &Self::Configuration) -> Option<Violation> {
        let owner = action.owner()?;

        if action.is_commit() || is_trusted(owner, configuration) {
            return None;
        }

        Some(Violation::MutableActionReference {
            reference: action.reference().to_owned(),
            unversioned: action.version().is_none(),
        })
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.pin_third_party_actions.as_ref(), |rule| {
        evaluate_action_rule::<PinThirdPartyActions>(workflows, rule)
    })
}

fn is_trusted(owner: &str, configuration: &PinThirdPartyActionsRule) -> bool {
    configuration
        .trusted_owners
        .iter()
        .any(|trusted| trusted.eq_ignore_ascii_case(owner))
}
