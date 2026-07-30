use tree_sitter::Node;

use crate::facts::CommentKind;

#[derive(Clone, Copy)]
pub(crate) struct Callee<'tree> {
    pub node: Node<'tree>,
    pub is_macro: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct ErrorHandler<'tree> {
    pub node: Node<'tree>,
    pub body_is_empty: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct Condition<'tree> {
    pub node: Node<'tree>,
    pub operator_count: u32,
}

#[derive(Clone, Copy)]
pub(crate) struct Vocabulary {
    pub is_function: fn(&str) -> bool,
    pub is_nesting: fn(&str) -> bool,
    pub is_block: fn(&str) -> bool,
    pub is_conditional: fn(&str) -> bool,
    pub is_decision: fn(Node<'_>) -> bool,
    pub is_return: fn(&str) -> bool,
    pub is_placeholder: fn(&str, &str) -> bool,
    pub is_receiver: fn(&str, &str) -> bool,
    pub is_abstract: fn(Node<'_>, &str) -> bool,
    pub callee: fn(Node<'_>) -> Option<Callee<'_>>,
    pub error_handler: fn(Node<'_>) -> Option<ErrorHandler<'_>>,
    pub condition: fn(Node<'_>) -> Option<Condition<'_>>,
    pub is_access: fn(&str) -> bool,
    pub import: fn(Node<'_>) -> Option<Node<'_>>,
    pub comment_kind: fn(Node<'_>, &str) -> Option<CommentKind>,
    pub has_implicit_tail_return: fn(Node<'_>) -> bool,
}

pub(crate) fn is_leading_block_statement(node: Node<'_>, block_kinds: &[&str]) -> bool {
    let Some(statement) = node.parent() else {
        return false;
    };
    let Some(block) = statement.parent() else {
        return false;
    };

    block_kinds.contains(&block.kind())
        && statement.named_child(0) == Some(node)
        && first_statement(block) == Some(statement)
}

fn first_statement(block: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = block.walk();

    block
        .named_children(&mut cursor)
        .find(|child| !child.is_extra())
}
