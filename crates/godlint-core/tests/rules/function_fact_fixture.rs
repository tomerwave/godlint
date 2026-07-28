use std::path::PathBuf;

use godlint_core::{
    facts::{FunctionFact, FunctionFactDetails},
    source::{SourceFile, SourceRange},
};

pub(super) struct FunctionFactFixture {
    source: SourceFile,
    name: Option<String>,
    range: SourceRange,
    body_range: SourceRange,
    parameter_count: u32,
    decision_points: u32,
    return_count: u32,
    statement_count: u32,
    body_is_empty: bool,
    nesting_depth: u32,
}

impl FunctionFactFixture {
    pub(super) fn new() -> Self {
        Self::with_source("src/example.rs", "fn example() {}")
    }

    pub(super) fn with_source(path: &str, contents: &str) -> Self {
        let source = SourceFile::new(PathBuf::from(path), contents.into())
            .unwrap_or_else(|error| panic!("creates source file: {error}"));
        let range = SourceRange::new(0, source.source().len())
            .unwrap_or_else(|error| panic!("creates source range: {error}"));

        Self {
            source,
            name: Some("example".into()),
            range,
            body_range: range,
            parameter_count: 0,
            decision_points: 0,
            return_count: 0,
            statement_count: 0,
            body_is_empty: false,
            nesting_depth: 0,
        }
    }

    pub(super) fn with_decision_points(mut self, decision_points: u32) -> Self {
        self.decision_points = decision_points;

        self
    }

    pub(super) fn with_return_count(mut self, return_count: u32) -> Self {
        self.return_count = return_count;

        self
    }

    pub(super) fn with_statement_count(mut self, statement_count: u32) -> Self {
        self.statement_count = statement_count;

        self
    }

    pub(super) fn with_nesting_depth(mut self, nesting_depth: u32) -> Self {
        self.nesting_depth = nesting_depth;

        self
    }

    pub(super) fn build(self) -> FunctionFact {
        FunctionFact::new(
            self.source,
            self.name,
            FunctionFactDetails {
                range: self.range,
                body_range: self.body_range,
                parameter_count: self.parameter_count,
                decision_points: self.decision_points,
                return_count: self.return_count,
                statement_count: self.statement_count,
                body_is_empty: self.body_is_empty,
                nesting_depth: self.nesting_depth,
            },
        )
        .unwrap_or_else(|error| panic!("creates function fact: {error}"))
    }
}
