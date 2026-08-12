use crate::{
    analyzers::SourceFacts,
    config::{Config, RestrictedCall as RestrictedCallConfiguration, RestrictedCallRule, Severity},
    facts::CallFact,
    rules::{
        CallRule, Finding, Rule, Violation,
        catalogue::{Catalogue, matches, spelled},
        evaluate_call_rule, when_configured,
    },
    source::Dialect,
};

const BUILT_INS: Catalogue = Catalogue(&[
    ("process.exit", Dialect::JavaScript),
    ("sys.exit", Dialect::Python),
    ("os._exit", Dialect::Python),
    ("std::process::exit", Dialect::Rust),
    ("os.Exit", Dialect::Go),
]);

pub struct RestrictedCall;

impl Rule for RestrictedCall {
    const ID: &'static str = "architecture/restricted-call";

    type Configuration = RestrictedCallRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CallRule for RestrictedCall {
    fn check(call: &CallFact, configuration: &Self::Configuration) -> Option<Violation> {
        is_restricted(call, &configuration.calls).then(|| Violation::RestrictedCall {
            callee: spelled(call),
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.restricted_call.as_ref(), |rule| {
        evaluate_call_rule::<RestrictedCall>(facts, rule)
    })
}

fn is_restricted(call: &CallFact, restrictions: &[RestrictedCallConfiguration]) -> bool {
    let name = spelled(call);
    let configured = restrictions
        .iter()
        .find(|restriction| restriction.name == name);
    let restricted = if BUILT_INS.lists(&name) {
        BUILT_INS.speaks(call.source().language(), &name)
    } else {
        configured.is_some()
    };

    restricted
        && !configured.is_some_and(|restriction| matches(call.source(), &restriction.allow_in))
}
