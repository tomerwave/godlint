use tree_sitter::Node;

use super::vocabulary::Vocabulary;
use crate::facts::CommentKind;

pub(super) const VOCABULARY: Vocabulary = Vocabulary {
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
    matches!(
        kind,
        "arrow_function"
            | "function_declaration"
            | "function_expression"
            | "generator_function"
            | "generator_function_declaration"
            | "method_definition"
    )
}

fn is_nesting(kind: &str) -> bool {
    matches!(
        kind,
        "do_statement"
            | "for_in_statement"
            | "for_statement"
            | "if_statement"
            | "switch_statement"
            | "try_statement"
            | "while_statement"
    )
}

fn is_block(kind: &str) -> bool {
    kind == "statement_block"
}

fn is_conditional(kind: &str) -> bool {
    kind == "if_statement"
}

fn is_decision(kind: &str) -> bool {
    matches!(
        kind,
        "catch_clause"
            | "do_statement"
            | "for_in_statement"
            | "for_statement"
            | "if_statement"
            | "switch_case"
            | "ternary_expression"
            | "while_statement"
    )
}

fn is_return(kind: &str) -> bool {
    kind == "return_statement"
}

fn is_placeholder(_kind: &str, _text: &str) -> bool {
    false
}

fn is_receiver(_kind: &str, _text: &str) -> bool {
    false
}

fn is_abstract(node: Node<'_>, source: &str) -> bool {
    if node.kind() != "method_definition" {
        return false;
    }

    let is_constructor = node
        .child_by_field_name("name")
        .and_then(|name| source.get(name.byte_range()))
        .is_some_and(|name| name == "constructor");

    is_constructor && declares_parameter_property(node, source)
}

fn declares_parameter_property(node: Node<'_>, source: &str) -> bool {
    let Some(parameters) = node.child_by_field_name("parameters") else {
        return false;
    };
    let mut cursor = parameters.walk();

    parameters
        .named_children(&mut cursor)
        .any(|parameter| parameter_carries_modifier(parameter, source))
}

fn parameter_carries_modifier(parameter: Node<'_>, source: &str) -> bool {
    let mut cursor = parameter.walk();

    parameter.children(&mut cursor).any(|child| {
        child.kind() == "accessibility_modifier"
            || source
                .get(child.byte_range())
                .is_some_and(|text| text == "readonly")
    })
}

fn comment_kind(node: Node<'_>, source: &str) -> Option<CommentKind> {
    if !node.is_extra() {
        return None;
    }

    let text = source.get(node.byte_range())?;

    if text.starts_with("//") {
        return Some(CommentKind::Line);
    }

    if text.starts_with("/**") && !text.starts_with("/**/") {
        return Some(CommentKind::Doc);
    }

    text.starts_with("/*").then_some(CommentKind::Block)
}

fn has_implicit_tail_return(node: Node<'_>) -> bool {
    if node.kind() != "arrow_function" {
        return false;
    }

    node.child_by_field_name("body")
        .is_some_and(|body| body.kind() != "statement_block")
}
