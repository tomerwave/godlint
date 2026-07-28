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
            super::ecmascript::VOCABULARY,
        )
    }
}
