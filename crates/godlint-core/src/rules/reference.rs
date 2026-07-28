use crate::{
    analyzers::SourceFacts,
    config::Severity,
    facts::{AccessFact, CallFact},
    rules::{Finding, RuleError, Violation, finding},
    source::{SourceFile, SourceRange},
};

pub trait Reference {
    fn source(&self) -> &SourceFile;

    fn range(&self) -> SourceRange;
}

impl Reference for CallFact {
    fn source(&self) -> &SourceFile {
        CallFact::source(self)
    }

    fn range(&self) -> SourceRange {
        CallFact::range(self)
    }
}

impl Reference for AccessFact {
    fn source(&self) -> &SourceFile {
        AccessFact::source(self)
    }

    fn range(&self) -> SourceRange {
        AccessFact::range(self)
    }
}

pub fn evaluate<R: Reference>(
    facts: &[SourceFacts],
    severity: Severity,
    rule_id: &'static str,
    references: impl Fn(&SourceFacts) -> &[R],
    check: impl Fn(&R) -> Option<Violation>,
) -> Result<Vec<Finding>, RuleError> {
    if severity == Severity::Off {
        return Ok(Vec::new());
    }

    let mut findings = Vec::new();

    for source_facts in facts {
        for reference in references(source_facts) {
            let Some(violation) = check(reference) else {
                continue;
            };

            findings.push(finding(
                Reference::source(reference),
                Reference::range(reference),
                severity,
                rule_id,
                violation,
            )?);
        }
    }

    Ok(findings)
}
