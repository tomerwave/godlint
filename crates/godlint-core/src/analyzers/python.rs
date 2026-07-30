use tree_sitter::Node;

use crate::{
    analyzers::{
        Analyzer, AnalyzerError, SourceFacts,
        vocabulary::{
            Argument, Callee, Cognition, Condition, ErrorHandler, Focus, TestDeclaration,
            Vocabulary, is_leading_block_statement, literal_value, plain_path,
        },
    },
    facts::CommentKind,
    source::SourceFile,
};

pub(super) struct Python;

impl Analyzer for Python {
    fn analyze(&self, source: &SourceFile) -> Result<SourceFacts, AnalyzerError> {
        super::analyze_with(source, tree_sitter_python::LANGUAGE.into(), VOCABULARY)
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
    test,
    import,
    comment_kind,
    has_implicit_tail_return,
};

const DOCSTRING_BLOCKS: [&str; 2] = ["block", "module"];

const ABSTRACT_DECORATORS: [&str; 2] = ["abstractmethod", "overload"];

fn is_function(kind: &str) -> bool {
    matches!(kind, "function_definition" | "lambda")
}

fn is_nesting(kind: &str) -> bool {
    matches!(
        kind,
        "for_statement"
            | "if_statement"
            | "match_statement"
            | "try_statement"
            | "while_statement"
            | "with_statement"
    )
}

fn is_block(kind: &str) -> bool {
    kind == "block"
}

fn is_conditional(kind: &str) -> bool {
    kind == "if_statement"
}

fn callee(node: Node<'_>) -> Option<Callee<'_>> {
    (node.kind() == "call")
        .then(|| node.child_by_field_name("function"))
        .flatten()
        .map(|node| Callee {
            node,
            is_macro: false,
        })
}

fn error_handler(node: Node<'_>) -> Option<ErrorHandler<'_>> {
    if node.kind() != "except_clause" {
        return None;
    }

    let mut clause = node.walk();
    let body = node
        .named_children(&mut clause)
        .find(|child| child.kind() == "block")?;
    let mut statements = body.walk();

    Some(ErrorHandler {
        node,
        body_is_empty: body
            .named_children(&mut statements)
            .all(is_placeholder_statement),
    })
}

fn condition(node: Node<'_>) -> Option<Condition<'_>> {
    let condition = match node.kind() {
        "if_statement" | "while_statement" | "elif_clause" => {
            node.child_by_field_name("condition")?
        }
        _ => return None,
    };

    Some(Condition {
        node: condition,
        operator_count: count_condition_operators(condition),
    })
}

fn cognition(node: Node<'_>) -> Option<Cognition> {
    match node.kind() {
        "if_statement"
        | "for_statement"
        | "while_statement"
        | "match_statement"
        | "conditional_expression"
        | "except_clause" => Some(Cognition::Structural),
        "elif_clause" | "else_clause" => Some(Cognition::Hybrid),
        "boolean_operator" if opens_operator_sequence(node) => Some(Cognition::Fundamental),
        _ => None,
    }
}

fn logical_operator(node: Node<'_>) -> Option<&'static str> {
    if node.kind() != "boolean_operator" {
        return None;
    }

    match node.child_by_field_name("operator")?.kind() {
        "and" => Some("and"),
        "or" => Some("or"),
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
    let is_operator = matches!(node.kind(), "boolean_operator" | "conditional_expression");

    let mut cursor = node.walk();
    let children = node
        .children(&mut cursor)
        .filter(|child| !is_function(child.kind()))
        .map(count_condition_operators)
        .sum::<u32>();

    u32::from(is_operator) + children
}

fn is_placeholder_statement(statement: Node<'_>) -> bool {
    match statement.kind() {
        "pass_statement" => true,
        "expression_statement" => statement
            .named_child(0)
            .is_some_and(|value| value.kind() == "ellipsis"),
        _ => false,
    }
}

fn import(node: Node<'_>) -> Option<Node<'_>> {
    match node.kind() {
        "import_statement" => spelled_module(node.child_by_field_name("name")?),
        "import_from_statement" => node.child_by_field_name("module_name"),
        _ => None,
    }
}

fn spelled_module(name: Node<'_>) -> Option<Node<'_>> {
    if name.kind() == "aliased_import" {
        return name.child_by_field_name("name");
    }

    Some(name)
}

fn direct_path(text: &str) -> Option<String> {
    plain_path(text)
}

fn test<'tree>(node: Node<'tree>, source: &str) -> Option<TestDeclaration<'tree>> {
    if node.kind() != "function_definition" {
        return None;
    }

    let name = node.child_by_field_name("name")?;
    let decorators = decorators_of(node, source);
    let marked = decorators
        .iter()
        .find(|(text, _)| text.starts_with("pytest.mark."));
    let named = source
        .get(name.byte_range())
        .is_some_and(|text| text.starts_with("test_") || text == "test");

    if !named && marked.is_none() {
        return None;
    }

    Some(TestDeclaration {
        node,
        name: Some(name),
        marker: marked.map_or(name, |(_, node)| *node),
        focus: python_focus(&decorators),
    })
}

fn python_focus(decorators: &[(&str, Node<'_>)]) -> Focus {
    let skipped = decorators
        .iter()
        .any(|(text, _)| text.starts_with("pytest.mark.skip") || text.starts_with("unittest.skip"));

    if skipped {
        Focus::Skipped
    } else {
        Focus::Ordinary
    }
}

fn decorators_of<'tree, 'source>(
    node: Node<'tree>,
    source: &'source str,
) -> Vec<(&'source str, Node<'tree>)> {
    let Some(parent) = node.parent().filter(|p| p.kind() == "decorated_definition") else {
        return Vec::new();
    };
    let mut cursor = parent.walk();

    parent
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "decorator")
        .filter_map(|decorator| {
            let inner = decorator.named_child(0)?;
            let marker = if inner.kind() == "call" {
                inner.child_by_field_name("function")?
            } else {
                inner
            };

            Some((source.get(marker.byte_range())?, marker))
        })
        .collect()
}

fn argument(node: Node<'_>) -> Option<Argument<'_>> {
    if node.kind() == "keyword_argument" {
        return Some(Argument {
            name: node.child_by_field_name("name"),
            value: node.child_by_field_name("value")?,
        });
    }

    Some(Argument {
        name: None,
        value: node,
    })
}

fn literal(node: Node<'_>, source: &str) -> Option<String> {
    match node.kind() {
        "string" | "concatenated_string" => Some(literal_value(node, source, "string_content")),
        "integer" | "float" | "true" | "false" | "none" => {
            Some(source.get(node.byte_range())?.to_owned())
        }
        _ => None,
    }
}

fn is_access(kind: &str) -> bool {
    kind == "attribute"
}

fn is_decision(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "conditional_expression"
            | "elif_clause"
            | "except_clause"
            | "for_statement"
            | "if_statement"
            | "match_statement"
            | "while_statement"
    ) || is_case_guard(node)
}

fn is_case_guard(node: Node<'_>) -> bool {
    node.kind() == "if_clause"
        && node
            .parent()
            .is_some_and(|parent| parent.kind() == "case_clause")
}

fn is_return(kind: &str) -> bool {
    kind == "return_statement"
}

fn is_placeholder(kind: &str, text: &str) -> bool {
    kind == "pass_statement" || (kind == "expression_statement" && text.trim() == "...")
}

fn is_receiver(_kind: &str, text: &str) -> bool {
    matches!(text, "cls" | "self")
}

fn is_abstract(node: Node<'_>, source: &str) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };

    if parent.kind() != "decorated_definition" {
        return false;
    }

    let mut cursor = parent.walk();

    parent
        .named_children(&mut cursor)
        .filter(|child| child.kind() == "decorator")
        .filter_map(|decorator| source.get(decorator.byte_range()))
        .any(|text| {
            ABSTRACT_DECORATORS
                .iter()
                .any(|decorator| text.contains(decorator))
        })
}

fn comment_kind(node: Node<'_>, source: &str) -> Option<CommentKind> {
    if node.kind() == "string" && is_leading_block_statement(node, &DOCSTRING_BLOCKS) {
        return Some(CommentKind::Docstring);
    }

    hash_comment(node, source)
}

fn hash_comment(node: Node<'_>, source: &str) -> Option<CommentKind> {
    if !node.is_extra() {
        return None;
    }

    let text = source.get(node.byte_range())?;

    if !text.starts_with('#') {
        return None;
    }

    if node.start_byte() == 0 && text.starts_with("#!") {
        return Some(CommentKind::Shebang);
    }

    Some(CommentKind::Line)
}

fn has_implicit_tail_return(node: Node<'_>) -> bool {
    node.kind() == "lambda"
}
