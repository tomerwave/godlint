use tree_sitter::Node;

use super::vocabulary::{Callee, Cognition, Condition, ErrorHandler, Vocabulary};
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
    callee,
    error_handler,
    condition,
    cognition,
    is_access,
    import,
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

fn callee(node: Node<'_>) -> Option<Callee<'_>> {
    let field = match node.kind() {
        "call_expression" => "function",
        "new_expression" => "constructor",
        _ => return None,
    };

    node.child_by_field_name(field).map(|node| Callee {
        node,
        is_macro: false,
    })
}

fn error_handler(node: Node<'_>) -> Option<ErrorHandler<'_>> {
    (node.kind() == "catch_clause")
        .then(|| node.child_by_field_name("body"))
        .flatten()
        .map(|body| {
            let mut statements = body.walk();

            ErrorHandler {
                node,
                body_is_empty: body
                    .named_children(&mut statements)
                    .all(|statement| matches!(statement.kind(), "empty_statement" | "comment")),
            }
        })
}

fn condition(node: Node<'_>) -> Option<Condition<'_>> {
    let condition = match node.kind() {
        "if_statement" | "while_statement" => node.child_by_field_name("condition")?,
        _ => return None,
    };

    Some(Condition {
        node: condition,
        operator_count: count_condition_operators(condition),
    })
}

fn cognition(node: Node<'_>) -> Option<Cognition> {
    match node.kind() {
        "if_statement" if is_else_if(node) => Some(Cognition::Hybrid),
        "if_statement" | "switch_statement" | "for_statement" | "for_in_statement"
        | "while_statement" | "do_statement" | "ternary_expression" | "catch_clause" => {
            Some(Cognition::Structural)
        }
        "else_clause" if holds_an_if(node) => None,
        "else_clause" => Some(Cognition::Hybrid),
        "binary_expression" if opens_operator_sequence(node) => Some(Cognition::Fundamental),
        _ => None,
    }
}

fn is_else_if(node: Node<'_>) -> bool {
    node.parent()
        .is_some_and(|parent| parent.kind() == "else_clause")
}

fn holds_an_if(node: Node<'_>) -> bool {
    let mut cursor = node.walk();

    node.named_children(&mut cursor)
        .any(|child| child.kind() == "if_statement")
}

fn logical_operator(node: Node<'_>) -> Option<&'static str> {
    if node.kind() != "binary_expression" {
        return None;
    }

    match node.child_by_field_name("operator")?.kind() {
        "&&" => Some("&&"),
        "||" => Some("||"),
        _ => None,
    }
}

fn opens_operator_sequence(node: Node<'_>) -> bool {
    let Some(operator) = logical_operator(node) else {
        return false;
    };

    node.parent()
        .and_then(logical_operator)
        .is_none_or(|enclosing| enclosing != operator)
}

fn count_condition_operators(node: Node<'_>) -> u32 {
    let is_logical_operator = node.kind() == "binary_expression"
        && node
            .child_by_field_name("operator")
            .is_some_and(|operator| matches!(operator.kind(), "&&" | "||"));
    let is_ternary = node.kind() == "ternary_expression";

    let mut cursor = node.walk();
    let children = node
        .children(&mut cursor)
        .filter(|child| !is_function(child.kind()))
        .map(count_condition_operators)
        .sum::<u32>();

    u32::from(is_logical_operator || is_ternary) + children
}

fn is_access(kind: &str) -> bool {
    kind == "member_expression"
}

fn is_decision(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "catch_clause"
            | "do_statement"
            | "for_in_statement"
            | "for_statement"
            | "if_statement"
            | "switch_statement"
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

const COMMENT_PREFIXES: [(&str, CommentKind); 4] = [
    ("//", CommentKind::Line),
    ("/**/", CommentKind::Block),
    ("/**", CommentKind::Doc),
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
    if node.kind() != "arrow_function" {
        return false;
    }

    node.child_by_field_name("body")
        .is_some_and(|body| body.kind() != "statement_block")
}

fn import(node: Node<'_>) -> Option<Node<'_>> {
    matches!(node.kind(), "import_statement" | "export_statement")
        .then(|| node.child_by_field_name("source"))
        .flatten()
        .and_then(|string| string.named_child(0))
}
