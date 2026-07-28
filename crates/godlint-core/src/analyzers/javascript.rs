use crate::{
    analyzers::{Analyzer, AnalyzerError, SourceFacts},
    source::SourceFile,
};

pub(super) struct JavaScript;

impl Analyzer for JavaScript {
    fn analyze(&self, source: &SourceFile) -> Result<SourceFacts, AnalyzerError> {
        super::analyze_with(
            source,
            tree_sitter_javascript::LANGUAGE.into(),
            is_function_node,
        )
    }
}

fn is_function_node(kind: &str) -> bool {
    matches!(
        kind,
        "arrow_function"
            | "function_declaration"
            | "function_expression"
            | "generator_function_declaration"
            | "method_definition"
    )
}
