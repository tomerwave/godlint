use tree_sitter::Node;

use crate::{
    analyzers::{Analyzer, AnalyzerError, SourceFacts, vocabulary::Vocabulary},
    facts::CommentKind,
    source::SourceFile,
};

pub(super) struct Rust;

impl Analyzer for Rust {
    fn analyze(&self, source: &SourceFile) -> Result<SourceFacts, AnalyzerError> {
        super::analyze_with(source, tree_sitter_rust::LANGUAGE.into(), VOCABULARY)
    }
}

const VOCABULARY: Vocabulary = Vocabulary {
    is_function,
    is_nesting,
    is_block,
    is_conditional,
    is_decision,
    is_return,
    is_placeholder,
    is_receiver,
    is_abstract,
    comment_kind,
    has_implicit_tail_return,
};

fn is_function(kind: &str) -> bool {
    matches!(kind, "closure_expression" | "function_item")
}

fn is_nesting(kind: &str) -> bool {
    matches!(
        kind,
        "for_expression"
            | "if_expression"
            | "loop_expression"
            | "match_expression"
            | "while_expression"
    )
}

fn is_block(kind: &str) -> bool {
    kind == "block"
}

fn is_conditional(kind: &str) -> bool {
    kind == "if_expression"
}

fn is_decision(kind: &str) -> bool {
    matches!(
        kind,
        "for_expression" | "if_expression" | "match_arm" | "try_expression" | "while_expression"
    )
}

fn is_return(kind: &str) -> bool {
    matches!(kind, "return_expression" | "try_expression")
}

fn is_placeholder(_kind: &str, _text: &str) -> bool {
    false
}

fn is_receiver(kind: &str, _text: &str) -> bool {
    kind == "self_parameter"
}

fn is_abstract(_node: Node<'_>, _source: &str) -> bool {
    false
}

fn comment_kind(node: Node<'_>, source: &str) -> Option<CommentKind> {
    if !node.is_extra() {
        return None;
    }

    let text = source.get(node.byte_range())?;

    if text.starts_with("//!") || text.starts_with("///") {
        return Some(CommentKind::Doc);
    }

    if text.starts_with("//") {
        return Some(CommentKind::Line);
    }

    if text.starts_with("/*!") || (text.starts_with("/**") && !text.starts_with("/**/")) {
        return Some(CommentKind::Doc);
    }

    text.starts_with("/*").then_some(CommentKind::Block)
}

fn has_implicit_tail_return(node: Node<'_>) -> bool {
    let Some(body) = node.child_by_field_name("body") else {
        return false;
    };

    if body.kind() != "block" {
        return true;
    }

    let mut cursor = body.walk();

    body.named_children(&mut cursor)
        .filter(|child| !child.is_extra())
        .last()
        .is_some_and(|last| !is_statement(last.kind()))
}

fn is_statement(kind: &str) -> bool {
    matches!(
        kind,
        "empty_statement" | "expression_statement" | "let_declaration"
    ) || kind.ends_with("_item")
}
