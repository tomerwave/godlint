use tree_sitter::Node;

use crate::{
    analyzers::{
        Analyzer, AnalyzerError, SourceFacts,
        vocabulary::{Vocabulary, is_leading_block_statement},
    },
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
    is_docstring,
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

fn is_decision(kind: &str) -> bool {
    matches!(
        kind,
        "case_clause"
            | "conditional_expression"
            | "elif_clause"
            | "except_clause"
            | "for_statement"
            | "if_statement"
            | "while_statement"
    )
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

fn is_docstring(node: Node<'_>) -> bool {
    node.kind() == "string" && is_leading_block_statement(node, &DOCSTRING_BLOCKS)
}

fn has_implicit_tail_return(node: Node<'_>) -> bool {
    node.kind() == "lambda"
}
