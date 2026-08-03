use tree_sitter::Node;

use super::vocabulary::{
    Argument, Assertion, Callee, Cognition, Condition, ErrorHandler, Focus, TestDeclaration,
    Vocabulary, argument_operands, is_else_if, literal_value, opens_operator_sequence, plain_path,
};
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
    direct_path,
    argument,
    literal,
    test,
    assertion,
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

fn holds_an_if(node: Node<'_>) -> bool {
    let mut cursor = node.walk();

    node.named_children(&mut cursor)
        .any(|child| child.kind() == "if_statement")
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

fn direct_path(text: &str) -> Option<String> {
    plain_path(&text.replace("?.", "."))
}

const RUNNERS: [&str; 5] = ["it", "test", "describe", "suite", "context"];

fn test<'tree>(node: Node<'tree>, source: &str) -> Option<TestDeclaration<'tree>> {
    if node.kind() != "call_expression" {
        return None;
    }

    let callee = node.child_by_field_name("function")?;
    let spelled = source.get(callee.byte_range())?;
    let (runner, suffix) = spelled.split_once('.').unwrap_or((spelled, ""));

    if !RUNNERS.contains(&runner) {
        return None;
    }

    Some(TestDeclaration {
        node,
        name: first_string(node),
        marker: callee,
        focus: focus_of(suffix),
    })
}

fn focus_of(suffix: &str) -> Focus {
    match suffix {
        "only" => Focus::Only,
        "skip" | "todo" => Focus::Skipped,
        _ => Focus::Ordinary,
    }
}

fn first_string(node: Node<'_>) -> Option<Node<'_>> {
    let arguments = node.child_by_field_name("arguments")?;
    let mut cursor = arguments.walk();

    arguments
        .named_children(&mut cursor)
        .find(|argument| argument.kind() == "string")
        .and_then(|string| {
            let mut inner = string.walk();

            string
                .children(&mut inner)
                .find(|child| child.kind() == "string_fragment")
        })
}

fn argument(node: Node<'_>) -> Option<Argument<'_>> {
    Some(Argument {
        name: None,
        value: node,
    })
}

fn literal(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "string" | "template_string" => Some(literal_value(node, source, "string_fragment")),
        "number" | "true" | "false" | "null" | "undefined" => {
            Some(source.get(node.byte_range())?.to_owned())
        }
        _ => None,
    }
}

fn assertion<'tree>(node: Node<'tree>, source: &str) -> Option<Assertion<'tree>> {
    if node.kind() != "call_expression" {
        return None;
    }

    let callee = node.child_by_field_name("function")?;
    let text = source.get(callee.byte_range())?.replace("?.", ".");

    names_an_assertion(&text).then(|| Assertion {
        node: chained(node),
        is_macro: false,
        operands: argument_operands(node),
        name: text,
    })
}

fn chained(node: Node<'_>) -> Node<'_> {
    let mut outermost = node;

    while let Some(parent) = outermost.parent() {
        let carried = match parent.kind() {
            "member_expression" => parent.child_by_field_name("object"),
            "call_expression" => parent.child_by_field_name("function"),
            _ => None,
        };

        if carried != Some(outermost) {
            break;
        }

        outermost = parent;
    }

    outermost
}

const ASSERTING: [&str; 3] = ["expect", "expectTypeOf", "assertType"];

fn names_an_assertion(path: &str) -> bool {
    let mut segments = path.split('.');
    let first = segments.next();

    first == Some("assert")
        || path
            .rsplit('.')
            .next()
            .is_some_and(|last| ASSERTING.contains(&last))
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
    super::vocabulary::prefixed_comment(node, source, &COMMENT_PREFIXES)
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
