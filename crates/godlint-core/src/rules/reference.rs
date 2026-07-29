use crate::{
    analyzers::SourceFacts,
    facts::{AccessFact, CallFact, ErrorHandlerFact, ImportFact},
    rules::{Finding, Ranged, Reporting, Rule, Violation, collect_ranged},
    source::SourceRange,
};

pub trait CallRule: Rule {
    fn check(call: &CallFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub trait AccessRule: Rule {
    fn check(access: &AccessFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub trait ImportRule: Rule {
    fn check(import: &ImportFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub trait ErrorHandlerRule: Rule {
    fn check(
        error_handler: &ErrorHandlerFact,
        configuration: &Self::Configuration,
    ) -> Option<Violation>;
}

pub fn evaluate_call_rule<R: CallRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
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
) -> Vec<Finding> {
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

impl Ranged for ImportFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

impl Ranged for AccessFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

pub fn evaluate_import_rule<R: ImportRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::imports,
        |import, _| R::check(import, configuration),
    )
}

impl Ranged for ErrorHandlerFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

pub fn evaluate_error_handler_rule<R: ErrorHandlerRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::error_handlers,
        |error_handler, _| R::check(error_handler, configuration),
    )
}
