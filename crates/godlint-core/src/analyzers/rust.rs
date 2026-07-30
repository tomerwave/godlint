use tree_sitter::Node;

use crate::{
    analyzers::{
        Analyzer, AnalyzerError, SourceFacts,
        vocabulary::{
            Argument, Callee, Cognition, Condition, ErrorHandler, Vocabulary, literal_value,
            plain_path,
        },
    },
    facts::CommentKind,
    source::SourceFile,
};

pub(super) struct Rust;

impl Analyzer for Rust {
    fn analyze(&self, source: &SourceFile) -> Result<SourceFacts, AnalyzerError> {
        super::analyze_with(source, tree_sitter_rust::LANGUAGE.into(), VOCABULARY)
    }
}

const VOCABULARY: Vocabulary = Vocabulary {
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
    direct_path,
    argument,
    literal,
    import,
    comment_kind,
    has_implicit_tail_return,
};

fn is_function(kind: &str) -> bool {
    matches!(kind, "closure_expression" | "function_item")
}

fn is_nesting(kind: &str) -> bool {
    matches!(
        kind,
        "for_expression"
            | "if_expression"
            | "loop_expression"
            | "match_expression"
            | "while_expression"
    )
}

fn is_block(kind: &str) -> bool {
    kind == "block"
}

fn is_conditional(kind: &str) -> bool {
    kind == "if_expression"
}

fn callee(node: Node<'_>) -> Option<Callee<'_>> {
    let is_macro = match node.kind() {
        "call_expression" => false,
        "macro_invocation" => true,
        _ => return None,
    };
    let field = if is_macro { "macro" } else { "function" };

    node.child_by_field_name(field)
        .map(|node| Callee { node, is_macro })
}

fn error_handler(_node: Node<'_>) -> Option<ErrorHandler<'_>> {
    None
}

fn condition(node: Node<'_>) -> Option<Condition<'_>> {
    let condition = match node.kind() {
        "if_expression" | "while_expression" => node.child_by_field_name("condition")?,
        _ => return None,
    };

    Some(Condition {
        node: condition,
        operator_count: count_condition_operators(condition),
    })
}

fn cognition(node: Node<'_>) -> Option<Cognition> {
    match node.kind() {
        "if_expression" if is_else_if(node) => Some(Cognition::Hybrid),
        "if_expression" | "match_expression" | "for_expression" | "while_expression"
        | "loop_expression" => Some(Cognition::Structural),
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
        .any(|child| child.kind() == "if_expression")
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

    let mut cursor = node.walk();
    let children = node
        .children(&mut cursor)
        .filter(|child| !is_function(child.kind()))
        .map(count_condition_operators)
        .sum::<u32>();

    u32::from(is_logical_operator) + children
}

fn import(node: Node<'_>) -> Option<Node<'_>> {
    match node.kind() {
        "use_declaration" => spelled_path(node.child_by_field_name("argument")?),
        "extern_crate_declaration" => node.child_by_field_name("name"),
        _ => None,
    }
}

fn spelled_path(argument: Node<'_>) -> Option<Node<'_>> {
    match argument.kind() {
        "scoped_use_list" | "use_as_clause" => argument.child_by_field_name("path"),
        "use_list" => None,
        _ => Some(argument),
    }
}

fn direct_path(text: &str) -> Option<String> {
    plain_path(text)
}

fn argument(node: Node<'_>) -> Option<Argument<'_>> {
    Some(Argument {
        name: None,
        value: node,
    })
}

fn literal(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "string_literal" | "raw_string_literal" => {
            Some(literal_value(node, source, "string_content"))
        }
        "integer_literal" | "float_literal" | "boolean_literal" | "char_literal" => {
            Some(source.get(node.byte_range())?.to_owned())
        }
        _ => None,
    }
}

fn is_access(_kind: &str) -> bool {
    false
}

fn is_decision(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "for_expression"
            | "if_expression"
            | "match_expression"
            | "try_expression"
            | "while_expression"
    ) || is_match_guard(node)
        || is_refutable_let(node)
}

fn is_match_guard(node: Node<'_>) -> bool {
    node.kind() == "match_pattern" && node.child_by_field_name("condition").is_some()
}

fn is_refutable_let(node: Node<'_>) -> bool {
    node.kind() == "let_declaration" && node.child_by_field_name("alternative").is_some()
}

fn is_return(kind: &str) -> bool {
    matches!(kind, "return_expression" | "try_expression")
}

fn is_placeholder(_kind: &str, _text: &str) -> bool {
    false
}

fn is_receiver(kind: &str, _text: &str) -> bool {
    kind == "self_parameter"
}

fn is_abstract(_node: Node<'_>, _source: &str) -> bool {
    false
}

const COMMENT_PREFIXES: [(&str, CommentKind); 7] = [
    ("///", CommentKind::Doc),
    ("//!", CommentKind::Doc),
    ("//", CommentKind::Line),
    ("/**/", CommentKind::Block),
    ("/**", CommentKind::Doc),
    ("/*!", CommentKind::Doc),
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
    let Some(body) = node.child_by_field_name("body") else {
        return false;
    };

    if body.kind() != "block" {
        return true;
    }

    let mut cursor = body.walk();

    body.named_children(&mut cursor)
        .filter(|child| !child.is_extra())
        .last()
        .is_some_and(|last| !is_statement(last.kind()))
}

fn is_statement(kind: &str) -> bool {
    matches!(
        kind,
        "empty_statement" | "expression_statement" | "let_declaration"
    ) || kind.ends_with("_item")
}
