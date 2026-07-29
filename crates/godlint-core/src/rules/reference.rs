use crate::{
    analyzers::SourceFacts,
    facts::{AccessFact, CallFact},
    rules::{Finding, Ranged, Reporting, Rule, RuleError, Violation, collect_ranged},
    source::SourceRange,
};

pub trait CallRule: Rule {
    fn check(call: &CallFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub trait AccessRule: Rule {
    fn check(access: &AccessFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub fn evaluate_call_rule<R: CallRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::calls,
        |call, _| R::check(call, configuration),
    )
}

pub fn evaluate_access_rule<R: AccessRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::accesses,
        |access, _| R::check(access, configuration),
    )
}

impl Ranged for CallFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

impl Ranged for AccessFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}
