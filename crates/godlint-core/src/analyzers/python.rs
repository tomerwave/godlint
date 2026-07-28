use crate::{analyzers::AnalyzerError, source::SourceFile};

pub(super) fn extract_functions(
    source: &SourceFile,
) -> Result<Vec<crate::facts::FunctionFact>, AnalyzerError> {
    super::extract_functions_with(
        source,
        tree_sitter_python::LANGUAGE.into(),
        is_function_node,
    )
}

fn is_function_node(kind: &str) -> bool {
    kind == "function_definition"
}
