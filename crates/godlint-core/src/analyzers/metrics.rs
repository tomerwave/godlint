use tree_sitter::Node;

use crate::{
    analyzers::vocabulary::{Cognition, Vocabulary},
    facts::{
        BlockDepth, CognitiveScore, DecisionPoints, ParameterCount, ReturnPaths, StatementCount,
    },
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

pub(super) struct Measured {
    pub decisions: DecisionPoints,
    pub cognitive: CognitiveScore,
    pub returns: ReturnPaths,
    pub statements: StatementCount,
    pub depth: BlockDepth,
}

#[derive(Clone, Copy)]
struct Place {
    nesting: u32,
    depth: u32,
    in_else: bool,
    in_body: bool,
}

#[derive(Default)]
struct Totals {
    decisions: u32,
    cognitive: u32,
    returns: u32,
    statements: u32,
    depth: u32,
}

pub(super) fn measure(function: Node<'_>, vocabulary: &Vocabulary) -> Measured {
    let body = function.child_by_field_name("body");
    let mut totals = Totals::default();
    let mut cursor = function.walk();

    for child in function.children(&mut cursor) {
        totals.absorb(
            child,
            Place {
                nesting: 0,
                depth: 0,
                in_else: false,
                in_body: body == Some(child),
            },
            vocabulary,
        );
    }

    let implicit = u32::from((vocabulary.has_implicit_tail_return)(function));

    Measured {
        decisions: DecisionPoints::new(totals.decisions),
        cognitive: CognitiveScore::new(totals.cognitive),
        returns: ReturnPaths::new(totals.returns + implicit),
        statements: StatementCount::new(counted_statements(body, totals.statements, vocabulary)),
        depth: BlockDepth::new(totals.depth),
    }
}

fn counted_statements(body: Option<Node<'_>>, walked: u32, vocabulary: &Vocabulary) -> u32 {
    let Some(body) = body else {
        return 0;
    };

    if (vocabulary.is_block)(body.kind()) {
        walked
    } else {
        1
    }
}

impl Totals {
    fn absorb(&mut self, node: Node<'_>, place: Place, vocabulary: &Vocabulary) {
        if is_function(node, vocabulary) {
            return;
        }

        let class = (vocabulary.cognition)(node);
        let (own, inner) = cognition_weights(class, place.nesting);

        self.decisions += u32::from(node.is_named() && (vocabulary.is_decision)(node));
        self.returns += u32::from(matches(node, vocabulary.is_return));
        self.cognitive += own;

        if place.in_body {
            self.depth = self.depth.max(place.depth);

            if matches(node, vocabulary.is_block) {
                self.statements += declared_statements(node);
            }
        }

        self.descend(node, place, inner, vocabulary);
    }

    fn descend(&mut self, node: Node<'_>, place: Place, inner: u32, vocabulary: &Vocabulary) {
        let mut alternatives = node.walk();
        let alternatives: Vec<Node<'_>> = node
            .children_by_field_name("alternative", &mut alternatives)
            .collect();
        let alternative = node.child_by_field_name("alternative");
        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            self.absorb(
                child,
                Place {
                    nesting: if alternatives.contains(&child) {
                        place.nesting
                    } else {
                        inner
                    },
                    depth: place.depth
                        + u32::from(child_opens_block(child, place.in_else, vocabulary)),
                    in_else: alternative == Some(child),
                    in_body: place.in_body,
                },
                vocabulary,
            );
        }
    }
}

fn cognition_weights(class: Option<Cognition>, nesting: u32) -> (u32, u32) {
    match class {
        Some(Cognition::Structural) => (1 + nesting, nesting + 1),
        Some(Cognition::Hybrid) => (1, nesting + 1),
        Some(Cognition::Fundamental) => (1, nesting),
        None => (0, nesting),
    }
}

fn declared_statements(block: Node<'_>) -> u32 {
    let mut cursor = block.walk();
    let declared = block
        .named_children(&mut cursor)
        .filter(|statement| !statement.is_extra())
        .count();

    u32::try_from(declared).unwrap_or(u32::MAX)
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
