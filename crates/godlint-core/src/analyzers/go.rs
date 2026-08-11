use tree_sitter::Node;

use crate::{
    analyzers::{
        Analyzer, AnalyzerError, SourceFacts,
        vocabulary::{
            Argument, Assertion, Callee, Cognition, Condition, ErrorHandler, Focus,
            TestDeclaration, Vocabulary, plain_path,
        },
    },
    facts::CommentKind,
    source::SourceFile,
};

pub(super) struct Go;

impl Analyzer for Go {
    fn analyze(&self, source: &SourceFile) -> Result<SourceFacts, AnalyzerError> {
        super::analyze_with(source, tree_sitter_go::LANGUAGE.into(), VOCABULARY)
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
    assertion,
    import,
    comment_kind,
    has_implicit_tail_return,
};

fn is_function(kind: &str) -> bool {
    matches!(
        kind,
        "function_declaration" | "method_declaration" | "func_literal"
    )
}

fn is_nesting(kind: &str) -> bool {
    matches!(
        kind,
        "for_statement"
            | "if_statement"
            | "expression_switch_statement"
            | "type_switch_statement"
            | "select_statement"
    )
}

fn is_block(kind: &str) -> bool {
    kind == "block"
}

fn is_conditional(_kind: &str) -> bool {
    true
}

fn is_decision(node: Node<'_>) -> bool {
    matches!(
        node.kind(),
        "if_statement"
            | "for_statement"
            | "expression_switch_statement"
            | "type_switch_statement"
            | "select_statement"
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

fn is_abstract(_node: Node<'_>, _source: &str) -> bool {
    false
}

fn callee(node: Node<'_>) -> Option<Callee<'_>> {
    (node.kind() == "call_expression")
        .then(|| node.child_by_field_name("function"))
        .flatten()
        .map(|node| Callee {
            node,
            is_macro: false,
        })
}

fn error_handler(_node: Node<'_>) -> Option<ErrorHandler<'_>> {
    None
}

fn condition(node: Node<'_>) -> Option<Condition<'_>> {
    let condition = match node.kind() {
        "if_statement" => node.child_by_field_name("condition")?,
        _ => return None,
    };
    Some(Condition {
        node: condition,
        operator_count: count_condition_operators(condition),
    })
}

fn count_condition_operators(node: Node<'_>) -> u32 {
    let current = u32::from(
        node.kind() == "binary_expression"
            && node
                .child_by_field_name("operator")
                .is_some_and(|operator| matches!(operator.kind(), "&&" | "||")),
    );
    let mut cursor = node.walk();
    current
        + node
            .children(&mut cursor)
            .map(count_condition_operators)
            .sum::<u32>()
}

fn cognition(node: Node<'_>) -> Option<Cognition> {
    match node.kind() {
        "if_statement"
        | "for_statement"
        | "expression_switch_statement"
        | "type_switch_statement"
        | "select_statement" => Some(Cognition::Structural),
        "binary_expression"
            if node
                .child_by_field_name("operator")
                .is_some_and(|operator| matches!(operator.kind(), "&&" | "||")) =>
        {
            Some(Cognition::Fundamental)
        }
        _ => None,
    }
}

fn is_access(kind: &str) -> bool {
    kind == "selector_expression"
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
        "interpreted_string_literal" | "raw_string_literal" => Some(
            source
                .get(node.byte_range())?
                .trim_matches(['"', '`'])
                .to_owned(),
        ),
        "int_literal" | "float_literal" | "imaginary_literal" | "rune_literal" | "true"
        | "false" => Some(source.get(node.byte_range())?.to_owned()),
        _ => None,
    }
}

fn test<'tree>(node: Node<'tree>, source: &str) -> Option<TestDeclaration<'tree>> {
    let (name, focus) = test_details(node, source)?;
    Some(TestDeclaration {
        node,
        name: Some(name),
        marker: name,
        focus,
    })
}

fn test_details<'tree>(node: Node<'tree>, source: &str) -> Option<(Node<'tree>, Focus)> {
    if node.kind() != "function_declaration" {
        return None;
    }
    let (name, text, signature) = node
        .child_by_field_name("name")
        .and_then(|name| {
            name.utf8_text(source.as_bytes())
                .ok()
                .map(|text| (name, text))
        })
        .and_then(|(name, text)| {
            node.child_by_field_name("parameters")
                .and_then(|parameters| source.get(parameters.byte_range()))
                .map(|signature| (name, text, signature))
        })?;
    if !is_test_name(text) || (text.starts_with("Test") && !signature.contains("*testing.T")) {
        return None;
    }
    let focus = if contains_skip_call(node, source) {
        Focus::Skipped
    } else {
        Focus::Ordinary
    };
    Some((name, focus))
}

fn is_test_name(name: &str) -> bool {
    name.starts_with("Test") || name.starts_with("Benchmark") || name.starts_with("Example")
}

fn contains_skip_call(node: Node<'_>, source: &str) -> bool {
    if node.kind() == "call_expression"
        && node
            .child_by_field_name("function")
            .and_then(|function| function.child_by_field_name("field"))
            .and_then(|field| field.utf8_text(source.as_bytes()).ok())
            .is_some_and(|field| matches!(field, "Skip" | "Skipf" | "SkipNow"))
    {
        return true;
    }

    let mut cursor = node.walk();
    node.children(&mut cursor)
        .any(|child| contains_skip_call(child, source))
}

fn assertion<'tree>(node: Node<'tree>, source: &str) -> Option<Assertion<'tree>> {
    if node.kind() != "call_expression" {
        return None;
    }
    let callee = node.child_by_field_name("function")?;
    let name = source.get(callee.byte_range())?;
    let receiver = name.split('.').next().unwrap_or_default();
    let is_assertion = [
        "Error", "Errorf", "Fatal", "Fatalf", "Fail", "FailNow", "Log", "Logf",
    ]
    .iter()
    .any(|suffix| name.ends_with(suffix))
        && matches!(receiver, "t" | "b" | "tb" | "test");
    is_assertion.then(|| Assertion {
        node,
        name: name.to_owned(),
        is_macro: false,
        operands: node
            .child_by_field_name("arguments")
            .map_or(0, |args| args.named_child_count()),
    })
}

fn import(node: Node<'_>) -> Option<Node<'_>> {
    (node.kind() == "import_spec")
        .then(|| {
            node.child_by_field_name("path")
                .or_else(|| node.named_child(0))
        })
        .flatten()
}

const COMMENT_PREFIXES: [(&str, CommentKind); 4] = [
    ("///", CommentKind::Doc),
    ("//", CommentKind::Line),
    ("/**", CommentKind::Doc),
    ("/*", CommentKind::Block),
];

fn comment_kind(node: Node<'_>, source: &str) -> Option<CommentKind> {
    super::vocabulary::prefixed_comment(node, source, &COMMENT_PREFIXES)
}

fn has_implicit_tail_return(_node: Node<'_>) -> bool {
    false
}
