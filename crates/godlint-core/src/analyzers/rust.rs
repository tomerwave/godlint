use crate::{
    analyzers::{Analyzer, AnalyzerError, SourceFacts},
    source::SourceFile,
};

pub(super) struct Rust;

impl Analyzer for Rust {
    fn analyze(&self, source: &SourceFile) -> Result<SourceFacts, AnalyzerError> {
        super::analyze_with(source, tree_sitter_rust::LANGUAGE.into(), is_function_node)
    }
}

fn is_function_node(kind: &str) -> bool {
    kind == "function_item"
}
