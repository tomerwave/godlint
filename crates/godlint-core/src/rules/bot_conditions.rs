use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{BotConditionsRule, Config, Severity},
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
    },
    source::{SourceRange, range_contains},
};

const ACTOR_CONTEXTS: [&str; 2] = ["github.actor", "github.triggering_actor"];

pub struct BotConditions;

impl Rule for BotConditions {
    const ID: &'static str = "ci/bot-conditions";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = BotConditionsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for BotConditions {
    fn check(
        workflow: &WorkflowFacts,
        configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        let conditions = workflow
            .steps()
            .iter()
            .filter_map(|step| step.condition())
            .chain(workflow.jobs().iter().filter_map(|job| job.condition()));
        conditions
            .flat_map(|condition| violations_in_condition(workflow, condition, configuration))
            .collect()
    }
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.bot_conditions.as_ref(), |rule| {
        evaluate_workflow_rule::<BotConditions>(workflows, rule)
    })
}

fn violations_in_condition(
    workflow: &WorkflowFacts,
    condition: SourceRange,
    configuration: &BotConditionsRule,
) -> Vec<(SourceRange, Violation)> {
    let expressions = workflow
        .expressions()
        .iter()
        .filter(|expression| range_contains(condition, expression.range()))
        .collect::<Vec<_>>();

    if expressions.is_empty() {
        let body = &workflow.file().text()[condition.start()..condition.end()];
        return violation(condition, body, configuration)
            .into_iter()
            .collect();
    }

    expressions
        .into_iter()
        .filter_map(|expression| violation(expression.range(), expression.body(), configuration))
        .collect()
}

fn violation(
    range: SourceRange,
    body: &str,
    configuration: &BotConditionsRule,
) -> Option<(SourceRange, Violation)> {
    compared_bot(body, configuration).map(|_| {
        (
            range,
            Violation::AttackerInfluencedBotCondition {
                expression: body.trim().to_owned(),
            },
        )
    })
}

fn compared_bot<'configuration>(
    expression: &str,
    configuration: &'configuration BotConditionsRule,
) -> Option<&'configuration str> {
    let normalized = expression.trim().to_ascii_lowercase();

    ACTOR_CONTEXTS.iter().find_map(|actor| {
        compared_identity(&normalized, actor, configuration)
            .or_else(|| contained_identity(&normalized, actor, configuration))
    })
}

fn compared_identity<'configuration>(
    expression: &str,
    actor: &str,
    configuration: &'configuration BotConditionsRule,
) -> Option<&'configuration str> {
    let comparison = expression.strip_prefix(actor)?.trim_start();
    let operator = comparison.get(..2)?;

    if !matches!(operator, "==" | "!=") {
        return None;
    }

    let identity = quoted_value(comparison[2..].trim_start())?;

    configuration
        .bots
        .iter()
        .find(|bot| identity == bot.to_ascii_lowercase())
        .map(String::as_str)
}

fn contained_identity<'configuration>(
    expression: &str,
    actor: &str,
    configuration: &'configuration BotConditionsRule,
) -> Option<&'configuration str> {
    let arguments = function_arguments(expression, "contains")?;
    let identity_fragment = actor_fragment(arguments, actor)?;

    configuration
        .bots
        .iter()
        .find(|bot| bot.to_ascii_lowercase().contains(identity_fragment))
        .map(String::as_str)
}

fn function_arguments<'expression>(
    expression: &'expression str,
    name: &str,
) -> Option<&'expression str> {
    expression
        .strip_prefix(name)?
        .strip_prefix('(')?
        .strip_suffix(')')
}

fn actor_fragment<'arguments>(arguments: &'arguments str, actor: &str) -> Option<&'arguments str> {
    let (search, item) = arguments.split_once(',')?;
    (search.trim() == actor)
        .then(|| quoted_value(item.trim()))
        .flatten()
        .filter(|fragment| !fragment.is_empty())
}

fn quoted_value(value: &str) -> Option<&str> {
    let quote = value.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }

    let quoted = &value[quote.len_utf8()..];
    let end = quoted.find(quote)?;
    quoted[end + quote.len_utf8()..]
        .trim()
        .is_empty()
        .then_some(&quoted[..end])
}
