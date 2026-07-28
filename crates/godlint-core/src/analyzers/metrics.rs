//! Derivation of the language-neutral function metrics from a parsed function node.
//!
//! Nothing here names a grammar node kind. Every question about what a node means is
//! delegated to the [`Vocabulary`] the owning language module supplied, so a new
//! language is added by describing it rather than by editing these walks.

use tree_sitter::Node;

use crate::{
    analyzers::vocabulary::Vocabulary,
    facts::{BlockDepth, DecisionPoints, ParameterCount, ReturnPaths, StatementCount},
};

/// Reports whether `node` is a function declaration rather than a keyword token.
///
/// Grammars name a keyword token after the construct it introduces, so Python's `lambda`
/// keyword has the same node kind as the lambda itself. Only named nodes are structure.
pub(super) fn is_function(node: Node<'_>, vocabulary: &Vocabulary) -> bool {
    node.is_named() && (vocabulary.is_function)(node.kind())
}

fn matches(node: Node<'_>, predicate: fn(&str) -> bool) -> bool {
    node.is_named() && predicate(node.kind())
}

/// Counts parameters the author declared, excluding a receiver such as `self`.
pub(super) fn parameter_count(
    function: Node<'_>,
    source: &str,
    vocabulary: &Vocabulary,
) -> ParameterCount {
    if function.child_by_field_name("parameter").is_some() {
        return ParameterCount::new(1);
    }

    let Some(parameters) = function.child_by_field_name("parameters") else {
        return ParameterCount::new(0);
    };
    let mut cursor = parameters.walk();
    let declared = parameters
        .named_children(&mut cursor)
        .filter(|parameter| !parameter.is_extra())
        .enumerate()
        .filter(|(index, parameter)| *index > 0 || !is_receiver(*parameter, source, vocabulary))
        .count();

    ParameterCount::new(u32::try_from(declared).unwrap_or(u32::MAX))
}

fn is_receiver(parameter: Node<'_>, source: &str, vocabulary: &Vocabulary) -> bool {
    let text = source.get(parameter.byte_range()).unwrap_or_default();

    (vocabulary.is_receiver)(parameter.kind(), text.trim())
}

/// Counts branch points the function owns, excluding those inside nested functions.
pub(super) fn decision_points(function: Node<'_>, vocabulary: &Vocabulary) -> DecisionPoints {
    DecisionPoints::new(count_own_nodes(
        function,
        vocabulary,
        vocabulary.is_decision,
    ))
}

/// Counts the paths that leave the function, including an implicit trailing expression.
pub(super) fn return_paths(function: Node<'_>, vocabulary: &Vocabulary) -> ReturnPaths {
    let explicit = count_own_nodes(function, vocabulary, vocabulary.is_return);
    let implicit = u32::from((vocabulary.has_implicit_tail_return)(function));

    ReturnPaths::new(explicit + implicit)
}

/// Counts nodes matching `predicate` within the function but not within nested ones.
fn count_own_nodes(
    function: Node<'_>,
    vocabulary: &Vocabulary,
    predicate: fn(&str) -> bool,
) -> u32 {
    let mut cursor = function.walk();

    function
        .children(&mut cursor)
        .map(|child| count_nodes_in(child, vocabulary, predicate))
        .sum()
}

fn count_nodes_in(node: Node<'_>, vocabulary: &Vocabulary, predicate: fn(&str) -> bool) -> u32 {
    if is_function(node, vocabulary) {
        return 0;
    }

    let mut cursor = node.walk();

    u32::from(matches(node, predicate))
        + node
            .children(&mut cursor)
            .map(|child| count_nodes_in(child, vocabulary, predicate))
            .sum::<u32>()
}

/// Counts statements in the body through nested blocks but not nested functions.
///
/// A body that is a bare expression rather than a block is one statement: an arrow
/// function or lambda does exactly one thing.
pub(super) fn statement_count(function: Node<'_>, vocabulary: &Vocabulary) -> StatementCount {
    let Some(body) = function.child_by_field_name("body") else {
        return StatementCount::new(0);
    };

    if !(vocabulary.is_block)(body.kind()) {
        return StatementCount::new(1);
    }

    StatementCount::new(count_block_statements(body, vocabulary))
}

fn count_block_statements(block: Node<'_>, vocabulary: &Vocabulary) -> u32 {
    let mut cursor = block.walk();

    block
        .named_children(&mut cursor)
        .filter(|statement| !statement.is_extra())
        .map(|statement| 1 + count_nested_statements(statement, vocabulary))
        .sum()
}

fn count_nested_statements(node: Node<'_>, vocabulary: &Vocabulary) -> u32 {
    if is_function(node, vocabulary) {
        return 0;
    }

    if matches(node, vocabulary.is_block) {
        return count_block_statements(node, vocabulary);
    }

    let mut cursor = node.walk();

    node.children(&mut cursor)
        .map(|child| count_nested_statements(child, vocabulary))
        .sum()
}

/// Measures the deepest run of nested control-flow blocks inside the body.
pub(super) fn block_depth(function: Node<'_>, vocabulary: &Vocabulary) -> BlockDepth {
    let Some(body) = function.child_by_field_name("body") else {
        return BlockDepth::new(0);
    };

    BlockDepth::new(deepest_block(body, 0, false, vocabulary))
}

fn deepest_block(node: Node<'_>, depth: u32, in_else: bool, vocabulary: &Vocabulary) -> u32 {
    let mut cursor = node.walk();
    let alternative = node.child_by_field_name("alternative");
    let mut deepest = depth;

    for child in node.children(&mut cursor) {
        if is_function(child, vocabulary) {
            continue;
        }

        let child_depth = depth + u32::from(child_opens_block(child, in_else, vocabulary));
        let child_in_else = alternative == Some(child);

        deepest = deepest.max(deepest_block(child, child_depth, child_in_else, vocabulary));
    }

    deepest
}

/// Reports whether `child` deepens nesting.
///
/// A conditional reached as the `else` branch of another conditional continues a flat
/// chain, so `else if` reads as one level however long the chain grows.
fn child_opens_block(child: Node<'_>, in_else: bool, vocabulary: &Vocabulary) -> bool {
    if !matches(child, vocabulary.is_nesting) {
        return false;
    }

    !(in_else && (vocabulary.is_conditional)(child.kind()))
}

/// Reports whether the body declares no work at all.
///
/// A body holding only a comment is not empty: the comment is the author recording that
/// the emptiness is deliberate, which is exactly what an allow-list would otherwise say.
pub(super) fn body_is_empty(function: Node<'_>, source: &str, vocabulary: &Vocabulary) -> bool {
    let Some(body) = function.child_by_field_name("body") else {
        return false;
    };

    if !(vocabulary.is_block)(body.kind()) {
        return false;
    }

    let mut cursor = body.walk();

    // A comment is a named child too, and it fails this test on purpose.
    body.named_children(&mut cursor)
        .all(|child| !child.is_extra() && is_placeholder(child, source, vocabulary))
}

fn is_placeholder(node: Node<'_>, source: &str, vocabulary: &Vocabulary) -> bool {
    let text = source.get(node.byte_range()).unwrap_or_default();

    (vocabulary.is_placeholder)(node.kind(), text)
}
