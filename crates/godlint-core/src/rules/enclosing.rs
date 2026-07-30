use crate::{
    analyzers::SourceFacts,
    facts::{FunctionFact, TestFact},
    source::SourceRange,
};

pub(super) fn in_test(facts: &SourceFacts, range: SourceRange) -> bool {
    facts.tests().iter().any(|test| test.contains(range))
}

pub(super) fn test_body<'facts>(
    facts: &'facts SourceFacts,
    test: &TestFact,
) -> Option<&'facts FunctionFact> {
    facts
        .functions()
        .iter()
        .filter(|function| test.contains(function.range()))
        .max_by_key(|function| width(function.range()))
}

fn width(range: SourceRange) -> usize {
    range.end() - range.start()
}
