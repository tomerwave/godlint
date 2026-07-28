use crate::{
    analyzers::SourceFacts,
    config::{Severity, TodoRequiresReferenceRule},
    facts::CommentFact,
    rules::{Finding, Rule, RuleError},
};

pub struct TodoRequiresReference;

pub fn evaluate(
    facts: &[SourceFacts],
    configuration: &TodoRequiresReferenceRule,
) -> Result<Vec<Finding>, RuleError> {
    let mut findings = Vec::new();

    for source_facts in facts {
        for comment in source_facts.comments() {
            if TodoRequiresReference::evaluate(comment, configuration).is_some() {
                findings.push(finding(comment, configuration)?);
            }
        }
    }

    Ok(findings)
}

impl Rule for TodoRequiresReference {
    type Input = CommentFact;
    type Configuration = TodoRequiresReferenceRule;
    type Violation = ();

    const ID: &'static str = "policy/todo-requires-reference";

    fn evaluate(
        comment: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation> {
        if configuration.severity == Severity::Off || !comment.text().contains("TODO") {
            return None;
        }

        (!has_reference(comment.text(), &configuration.reference_prefixes)).then_some(())
    }
}

fn has_reference(comment: &str, prefixes: &[String]) -> bool {
    prefixes
        .iter()
        .any(|prefix| has_reference_with_prefix(comment, prefix))
}

fn has_reference_with_prefix(comment: &str, prefix: &str) -> bool {
    comment.match_indices(prefix).any(|(index, _)| {
        comment[index + prefix.len()..]
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_digit())
    })
}

fn finding(
    comment: &CommentFact,
    configuration: &TodoRequiresReferenceRule,
) -> Result<Finding, RuleError> {
    let location = comment
        .source()
        .location(comment.range())
        .map_err(|source| RuleError::LocatesSource { source })?;

    Ok(Finding {
        path: comment.source().path().to_path_buf(),
        line: location.start.line,
        column: location.start.column,
        severity: configuration.severity,
        rule_id: TodoRequiresReference::ID,
        message: "TODO comment requires an issue reference.".into(),
    })
}
