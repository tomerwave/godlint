use crate::{analyzers::AnalyzerError, source::SourceFile};

pub(super) fn extract_functions(
    source: &SourceFile,
) -> Result<Vec<crate::facts::FunctionFact>, AnalyzerError> {
    let language = if source
        .path()
        .extension()
        .is_some_and(|extension| extension == "tsx")
    {
        tree_sitter_typescript::LANGUAGE_TSX.into()
    } else {
        tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
    };

    super::extract_functions_with(source, language, is_function_node)
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
