//! The node vocabulary each grammar supplies to the shared fact extractor.
//!
//! Rules consume language-neutral facts, and the extractor that builds them stays
//! language-neutral too: every judgement about what a given node *means* is answered
//! here by the owning language module. Adding a language means filling this in, not
//! editing the extractor.

use tree_sitter::Node;

/// Answers the questions the fact extractor needs to ask about a node.
#[derive(Clone, Copy)]
pub(crate) struct Vocabulary {
    /// Declares a callable whose metrics are reported against itself.
    ///
    /// This must include closures and lambdas wherever the language has them, so that
    /// one `max-lines` or `max-complexity` means the same thing across languages.
    pub is_function: fn(&str) -> bool,
    /// Introduces a nested control-flow block for depth accounting.
    pub is_nesting: fn(&str) -> bool,
    /// Holds a sequence of statements, such as a braced block or an indented suite.
    pub is_block: fn(&str) -> bool,
    /// Is a conditional, so that an `else if` chain reads as one level rather than many.
    pub is_conditional: fn(&str) -> bool,
    /// Adds a branch to the control-flow graph.
    pub is_decision: fn(&str) -> bool,
    /// Leaves the function early.
    pub is_return: fn(&str) -> bool,
    /// Stands in for a body the author left deliberately unimplemented.
    pub is_placeholder: fn(&str, &str) -> bool,
    /// Is the method receiver rather than a declared parameter, such as `self`.
    pub is_receiver: fn(&str, &str) -> bool,
    /// Declares no implementation on purpose, such as an abstract or overload signature.
    pub is_abstract: fn(Node<'_>, &str) -> bool,
    /// Plays the role of a comment without being one, as a Python docstring does.
    pub is_docstring: fn(Node<'_>) -> bool,
    /// Yields a value by falling off the end of the body rather than returning.
    pub has_implicit_tail_return: fn(Node<'_>) -> bool,
}

/// Reports whether `node` opens one of `block_kinds` as that block's first statement.
///
/// Documentation strings are positional: only the leading string of a module, class, or
/// function body carries documentation meaning, and a string anywhere else is data.
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
