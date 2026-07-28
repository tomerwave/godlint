use crate::{
    analyzers::{Analyzer, AnalyzerError, SourceFacts},
    source::SourceFile,
};

pub(super) struct Python;

impl Analyzer for Python {
    fn analyze(&self, source: &SourceFile) -> Result<SourceFacts, AnalyzerError> {
        super::analyze_with(
            source,
            tree_sitter_python::LANGUAGE.into(),
            is_function_node,
        )
    }
}

fn is_function_node(kind: &str) -> bool {
    kind == "function_definition"
}
