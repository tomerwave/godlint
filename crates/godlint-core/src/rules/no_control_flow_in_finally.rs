use crate::{
    analyzers::SourceFacts,
    config::{Config, NoProductionLogRule, Severity},
    facts::FinallyFact,
    rules::{
        Absence, FinallyRule, Finding, Languages, Rule, Violation, evaluate_finally_rule,
        when_configured,
    },
    source::Dialect,
};

pub struct NoControlFlowInFinally;

impl Rule for NoControlFlowInFinally {
    const ID: &'static str = "reliability/no-control-flow-in-finally";
    const LANGUAGES: Languages = Languages::all_but(&[
        (Dialect::Go, Absence::NoSuchConstruct),
        (Dialect::Rust, Absence::NoSuchConstruct),
        (Dialect::Workflow, Absence::NoSuchConstruct),
    ]);
    type Configuration = NoProductionLogRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl FinallyRule for NoControlFlowInFinally {
    fn check(
        finally_block: &FinallyFact,
        _configuration: &Self::Configuration,
    ) -> Option<Violation> {
        finally_block
            .has_control_flow()
            .then_some(Violation::NoControlFlowInFinally)
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_control_flow_in_finally.as_ref(), |rule| {
        evaluate_finally_rule::<NoControlFlowInFinally>(facts, rule)
    })
}
