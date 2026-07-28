use tree_sitter::Node;

use crate::{
    analyzers::{
        Analyzer, AnalyzerError, SourceFacts,
        vocabulary::{Callee, Vocabulary},
    },
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
    callee,
    is_access,
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

fn callee(node: Node<'_>) -> Option<Callee<'_>> {
    let is_macro = match node.kind() {
        "call_expression" => false,
        "macro_invocation" => true,
        _ => return None,
    };
    let field = if is_macro { "macro" } else { "function" };

    node.child_by_field_name(field)
        .map(|node| Callee { node, is_macro })
}

fn is_access(_kind: &str) -> bool {
    false
}

fn is_decision(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "for_expression"
            | "if_expression"
            | "match_expression"
            | "try_expression"
            | "while_expression"
    ) || is_match_guard(node)
        || is_refutable_let(node)
}

fn is_match_guard(node: Node<'_>) -> bool {
    node.kind() == "match_pattern" && node.child_by_field_name("condition").is_some()
}

fn is_refutable_let(node: Node<'_>) -> bool {
    node.kind() == "let_declaration" && node.child_by_field_name("alternative").is_some()
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

const COMMENT_PREFIXES: [(&str, CommentKind); 7] = [
    ("///", CommentKind::Doc),
    ("//!", CommentKind::Doc),
    ("//", CommentKind::Line),
    ("/**/", CommentKind::Block),
    ("/**", CommentKind::Doc),
    ("/*!", CommentKind::Doc),
    ("/*", CommentKind::Block),
];

fn comment_kind(node: Node<'_>, source: &str) -> Option<CommentKind> {
    if !node.is_extra() {
        return None;
    }

    let text = source.get(node.byte_range())?;

    COMMENT_PREFIXES
        .iter()
        .find(|(prefix, _)| text.starts_with(prefix))
        .map(|(_, kind)| *kind)
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
