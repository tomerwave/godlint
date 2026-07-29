use tree_sitter::Node;

use crate::{
    analyzers::{
        Analyzer, AnalyzerError, SourceFacts,
        vocabulary::{Callee, Vocabulary, is_leading_block_statement},
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
    is_access,
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
