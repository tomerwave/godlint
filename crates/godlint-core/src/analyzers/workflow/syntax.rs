use tree_sitter::Node;

use crate::{
    analyzers::AnalyzerError,
    source::{SourceRange, TextFile},
};

const BLOCK_MAPPING: &str = "block_mapping";
const BLOCK_PAIR: &str = "block_mapping_pair";
const BLOCK_SEQUENCE: &str = "block_sequence";
const BLOCK_SEQUENCE_ITEM: &str = "block_sequence_item";
const FLOW_MAPPING: &str = "flow_mapping";
const FLOW_NODE: &str = "flow_node";
const FLOW_PAIR: &str = "flow_pair";
const FLOW_SEQUENCE: &str = "flow_sequence";
const WRAPPERS: [&str; 4] = ["stream", "document", "block_node", FLOW_NODE];
const QUOTES: [char; 2] = ['"', '\''];

pub(super) fn mapping(node: Node<'_>) -> Option<Node<'_>> {
    let current = content(node)?;

    matches!(current.kind(), BLOCK_MAPPING | FLOW_MAPPING).then_some(current)
}

pub(super) fn pairs(mapping: Option<Node<'_>>) -> Vec<Node<'_>> {
    let Some(mapping) = mapping else {
        return Vec::new();
    };
    let mut cursor = mapping.walk();

    mapping
        .named_children(&mut cursor)
        .filter(|child| matches!(child.kind(), BLOCK_PAIR | FLOW_PAIR))
        .collect()
}

pub(super) fn value_of<'tree>(
    mapping: Option<Node<'tree>>,
    key: &str,
    file: &TextFile,
) -> Option<Node<'tree>> {
    pair_of(mapping, key, file).and_then(|pair| pair.child_by_field_name("value"))
}

pub(super) fn pair_of<'tree>(
    mapping: Option<Node<'tree>>,
    key: &str,
    file: &TextFile,
) -> Option<Node<'tree>> {
    pairs(mapping)
        .into_iter()
        .find(|pair| key_of(*pair, file) == Some(key))
}

pub(super) fn declared(mapping: Option<Node<'_>>, key: &str, file: &TextFile) -> bool {
    pair_of(mapping, key, file).is_some()
}

pub(super) fn key_of<'text>(pair: Node<'_>, file: &'text TextFile) -> Option<&'text str> {
    pair.child_by_field_name("key")
        .map(|key| node_text(key, file))
}

pub(super) fn node_text<'text>(node: Node<'_>, file: &'text TextFile) -> &'text str {
    file.text()[node.byte_range()].trim().trim_matches(QUOTES)
}

pub(super) fn sequence_items(node: Node<'_>) -> Vec<Node<'_>> {
    let Some(sequence) = content(node) else {
        return Vec::new();
    };

    match sequence.kind() {
        BLOCK_SEQUENCE => children(sequence, BLOCK_SEQUENCE_ITEM),
        FLOW_SEQUENCE => named_children(sequence),
        _ => vec![sequence],
    }
}

pub(super) fn content(mut node: Node<'_>) -> Option<Node<'_>> {
    while WRAPPERS.contains(&node.kind()) || node.kind() == BLOCK_SEQUENCE_ITEM {
        node = first_named(node)?;
    }

    Some(node)
}

pub(super) fn first_pair(mapping: Option<Node<'_>>) -> Option<Node<'_>> {
    pairs(mapping).into_iter().next()
}

pub(super) fn range(node: Node<'_>, file: &TextFile) -> Result<SourceRange, AnalyzerError> {
    file.range(node.start_byte(), node.end_byte())
        .map_err(|source| AnalyzerError::InvalidRange {
            path: file.path().to_path_buf(),
            source,
        })
}

fn first_named(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();

    node.named_children(&mut cursor)
        .find(|child| !child.is_extra())
}

fn children<'tree>(node: Node<'tree>, kind: &str) -> Vec<Node<'tree>> {
    named_children(node)
        .into_iter()
        .filter(|child| child.kind() == kind)
        .filter_map(content)
        .collect()
}

fn named_children(node: Node<'_>) -> Vec<Node<'_>> {
    let mut cursor = node.walk();

    node.named_children(&mut cursor)
        .filter(|child| !child.is_extra())
        .collect()
}
