use tree_sitter::Node;

use crate::{
    facts::{ErrorHandlerFact, FinallyFact},
    source::{Language, SourceFile},
};

use super::{AnalyzerError, node_range, vocabulary::Vocabulary};

pub(super) fn finally_fact(
    node: Node<'_>,
    source: &SourceFile,
) -> Result<Option<FinallyFact>, AnalyzerError> {
    if !matches!(node.kind(), "finally_clause" | "finally_block") {
        return Ok(None);
    }
    let range = node_range(node, source)?;
    let text = source.source().get(node.byte_range()).unwrap_or_default();
    let control_flow = ["return", "break", "continue"]
        .iter()
        .any(|word| text.contains(word));
    Ok(Some(FinallyFact::new(source.clone(), range, control_flow)))
}

pub(super) fn error_handler_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<ErrorHandlerFact>, AnalyzerError> {
    let Some(handler) = (vocabulary.error_handler)(node) else {
        return Ok(None);
    };
    let text = source
        .source()
        .get(handler.node.byte_range())
        .unwrap_or_default();
    let python = matches!(source.language(), Language::Python)
        && text
            .lines()
            .map(str::trim)
            .any(|body| body == "raise" || body.starts_with("raise "));
    let javascript = matches!(
        source.language(),
        Language::JavaScript | Language::TypeScript
    ) && text
        .replace(';', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .windows(2)
        .any(|pair| pair[0] == "throw");
    Ok(Some(ErrorHandlerFact::with_rethrows(
        source.clone(),
        node_range(handler.node, source)?,
        handler.body_is_empty,
        python || javascript,
    )))
}
