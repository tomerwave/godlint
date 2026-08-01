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

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Focus {
    Ordinary,
    Only,
    Skipped,
}

#[derive(Clone, Copy)]
pub(crate) struct TestDeclaration<'tree> {
    pub node: Node<'tree>,
    pub name: Option<Node<'tree>>,
    pub marker: Node<'tree>,
    pub focus: Focus,
}

#[derive(Clone)]
pub(crate) struct Assertion<'tree> {
    pub node: Node<'tree>,
    pub name: String,
    pub is_macro: bool,
    pub operands: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct Argument<'tree> {
    pub name: Option<Node<'tree>>,
    pub value: Node<'tree>,
}

#[derive(Clone, Copy)]
pub(crate) struct Condition<'tree> {
    pub node: Node<'tree>,
    pub operator_count: u32,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Cognition {
    Structural,
    Hybrid,
    Fundamental,
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
    pub cognition: fn(Node<'_>) -> Option<Cognition>,
    pub is_access: fn(&str) -> bool,
    pub direct_path: fn(&str) -> Option<String>,
    pub argument: fn(Node<'_>) -> Option<Argument<'_>>,
    pub literal: fn(Node<'_>, &str) -> Option<String>,
    pub test: for<'tree> fn(Node<'tree>, &str) -> Option<TestDeclaration<'tree>>,
    pub assertion: for<'tree> fn(Node<'tree>, &str) -> Option<Assertion<'tree>>,
    pub import: fn(Node<'_>) -> Option<Node<'_>>,
    pub comment_kind: fn(Node<'_>, &str) -> Option<CommentKind>,
    pub has_implicit_tail_return: fn(Node<'_>) -> bool,
}

pub(crate) fn literal_value(node: Node<'_>, source: &str, content: &str) -> String {
    let mut cursor = node.walk();

    node.children(&mut cursor)
        .find(|child| child.kind() == content)
        .and_then(|child| source.get(child.byte_range()))
        .unwrap_or_default()
        .to_owned()
}

pub(crate) fn named_operands(node: Node<'_>) -> usize {
    let mut cursor = node.walk();

    node.named_children(&mut cursor)
        .filter(|child| !child.is_extra())
        .count()
}

pub(crate) fn argument_operands(node: Node<'_>) -> usize {
    node.child_by_field_name("arguments")
        .map_or(0, named_operands)
}

pub(crate) fn plain_path(text: &str) -> Option<String> {
    let is_path = !text.is_empty()
        && text
            .chars()
            .all(|character| character.is_alphanumeric() || "_.:".contains(character));

    is_path.then(|| text.to_owned())
}

pub(crate) fn is_leading_block_statement(
    node: Node<'_>,
    statement_kinds: &[&str],
    block_kinds: &[&str],
) -> bool {
    let Some(statement) = node.parent() else {
        return false;
    };
    let Some(block) = statement.parent() else {
        return false;
    };

    statement_kinds.contains(&statement.kind())
        && block_kinds.contains(&block.kind())
        && statement.named_child(0) == Some(node)
        && first_statement(block) == Some(statement)
}

fn first_statement(block: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = block.walk();

    block
        .named_children(&mut cursor)
        .find(|child| !child.is_extra())
}
