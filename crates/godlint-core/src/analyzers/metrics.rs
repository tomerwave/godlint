use tree_sitter::Node;

use crate::{
    analyzers::vocabulary::Vocabulary,
    facts::{BlockDepth, DecisionPoints, ParameterCount, ReturnPaths, StatementCount},
};

pub(super) fn is_function(node: Node<'_>, vocabulary: &Vocabulary) -> bool {
    node.is_named() && (vocabulary.is_function)(node.kind())
}

fn matches(node: Node<'_>, predicate: fn(&str) -> bool) -> bool {
    node.is_named() && predicate(node.kind())
}

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

pub(super) fn decision_points(function: Node<'_>, vocabulary: &Vocabulary) -> DecisionPoints {
    let is_decision = vocabulary.is_decision;

    DecisionPoints::new(count_own_nodes(function, vocabulary, |node| {
        node.is_named() && is_decision(node)
    }))
}

pub(super) fn return_paths(function: Node<'_>, vocabulary: &Vocabulary) -> ReturnPaths {
    let is_return = vocabulary.is_return;
    let explicit = count_own_nodes(function, vocabulary, |node| matches(node, is_return));
    let implicit = u32::from((vocabulary.has_implicit_tail_return)(function));

    ReturnPaths::new(explicit + implicit)
}

fn count_own_nodes(
    function: Node<'_>,
    vocabulary: &Vocabulary,
    predicate: impl Fn(Node<'_>) -> bool + Copy,
) -> u32 {
    let mut cursor = function.walk();

    function
        .children(&mut cursor)
        .map(|child| count_nodes_in(child, vocabulary, predicate))
        .sum()
}

fn count_nodes_in(
    node: Node<'_>,
    vocabulary: &Vocabulary,
    predicate: impl Fn(Node<'_>) -> bool + Copy,
) -> u32 {
    if is_function(node, vocabulary) {
        return 0;
    }

    let mut cursor = node.walk();

    u32::from(predicate(node))
        + node
            .children(&mut cursor)
            .map(|child| count_nodes_in(child, vocabulary, predicate))
            .sum::<u32>()
}

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

fn child_opens_block(child: Node<'_>, in_else: bool, vocabulary: &Vocabulary) -> bool {
    if !matches(child, vocabulary.is_nesting) {
        return false;
    }

    !(in_else && (vocabulary.is_conditional)(child.kind()))
}

pub(super) fn body_is_empty(function: Node<'_>, source: &str, vocabulary: &Vocabulary) -> bool {
    let Some(body) = function.child_by_field_name("body") else {
        return false;
    };

    if !(vocabulary.is_block)(body.kind()) {
        return false;
    }

    let mut cursor = body.walk();

    body.named_children(&mut cursor)
        .all(|child| !child.is_extra() && is_placeholder(child, source, vocabulary))
}

fn is_placeholder(node: Node<'_>, source: &str, vocabulary: &Vocabulary) -> bool {
    let text = source.get(node.byte_range()).unwrap_or_default();

    (vocabulary.is_placeholder)(node.kind(), text)
}
