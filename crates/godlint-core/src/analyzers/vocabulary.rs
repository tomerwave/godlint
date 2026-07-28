use tree_sitter::Node;

#[derive(Clone, Copy)]
pub(crate) struct Vocabulary {
    pub is_function: fn(&str) -> bool,
    pub is_nesting: fn(&str) -> bool,
    pub is_block: fn(&str) -> bool,
    pub is_conditional: fn(&str) -> bool,
    pub is_decision: fn(&str) -> bool,
    pub is_return: fn(&str) -> bool,
    pub is_placeholder: fn(&str, &str) -> bool,
    pub is_receiver: fn(&str, &str) -> bool,
    pub is_abstract: fn(Node<'_>, &str) -> bool,
    pub is_docstring: fn(Node<'_>) -> bool,
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
        && block.named_child(0) == Some(statement)
}
