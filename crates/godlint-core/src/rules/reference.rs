use crate::{
    analyzers::SourceFacts,
    config::Severity,
    facts::{AccessFact, CallFact},
    rules::{Finding, Reporting, Rule, RuleError, Violation, finding},
    source::{SourceFile, SourceRange},
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
    evaluate(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::calls,
        |call| R::check(call, configuration),
    )
}

pub fn evaluate_access_rule<R: AccessRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Result<Vec<Finding>, RuleError> {
    evaluate(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::accesses,
        |access| R::check(access, configuration),
    )
}

pub trait Reference {
    fn source_file(&self) -> &SourceFile;

    fn source_range(&self) -> SourceRange;
}

impl Reference for CallFact {
    fn source_file(&self) -> &SourceFile {
        self.source()
    }

    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

impl Reference for AccessFact {
    fn source_file(&self) -> &SourceFile {
        self.source()
    }

    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

pub fn evaluate<R: Reference>(
    facts: &[SourceFacts],
    reporting: Reporting,
    references: impl Fn(&SourceFacts) -> &[R],
    check: impl Fn(&R) -> Option<Violation>,
) -> Result<Vec<Finding>, RuleError> {
    if reporting.severity == Severity::Off {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();

    for source_facts in facts {
        for reference in references(source_facts) {
            let Some(violation) = check(reference) else {
                continue;
            };

            findings.push(finding(
                reference.source_file(),
                reference.source_range(),
                reporting,
                violation,
            )?);
        }
    }

    Ok(findings)
}
