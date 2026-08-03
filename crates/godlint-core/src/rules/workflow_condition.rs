use crate::{
    analyzers::workflow::WorkflowFacts,
    source::{SourceRange, range_contains},
};

pub(crate) fn expressions_in_condition(
    workflow: &WorkflowFacts,
    condition: SourceRange,
) -> Vec<(SourceRange, &str)> {
    let expressions = workflow
        .expressions()
        .iter()
        .filter(|expression| range_contains(condition, expression.range()))
        .map(|expression| (expression.range(), expression.body()))
        .collect::<Vec<_>>();

    if expressions.is_empty() {
        vec![(condition, workflow.file().slice(condition))]
    } else {
        expressions
    }
}
