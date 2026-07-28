//! Shared scaffolding for rule tests.
//!
//! Every rule test analyzes real source rather than injecting metric values, so these
//! tests fail when the analyzer miscounts and not only when a comparison is wrong.

use std::{num::NonZeroU32, path::PathBuf};

use godlint_core::source::SourceFile;
use godlint_core::{
    analyzers::{SourceFacts, analyze},
    facts::FunctionFact,
};

/// Parses `source` and returns its facts.
pub(super) fn facts(path: &str, source: &str) -> SourceFacts {
    let source = SourceFile::new(PathBuf::from(path), source.into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));

    analyze(&source).unwrap_or_else(|error| panic!("analyzes {path}: {error}"))
}

/// Parses `source` and returns its facts with the function at `index`.
pub(super) fn nth_function(path: &str, source: &str, index: usize) -> (SourceFacts, FunctionFact) {
    let facts = facts(path, source);
    let function = facts
        .functions()
        .get(index)
        .unwrap_or_else(|| panic!("{path} has no function at index {index}"))
        .clone();

    (facts, function)
}

/// Parses `source` and returns its facts with its first function.
pub(super) fn function(path: &str, source: &str) -> (SourceFacts, FunctionFact) {
    nth_function(path, source, 0)
}

pub(super) fn limit(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap_or_else(|| panic!("{value} is not a valid limit"))
}
