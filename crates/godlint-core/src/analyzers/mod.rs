use std::{error::Error, fmt, path::PathBuf};

use tree_sitter::{Language as TreeSitterLanguage, Node, Parser};

use crate::{
    facts::{CommentFact, CommentFactError, FunctionFact, FunctionFactDetails, FunctionFactError},
    source::{Language, SourceFile, SourceRange, SourceRangeError},
};

mod ecmascript;
mod javascript;
mod python;
mod rust;
mod typescript;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFacts {
    source: SourceFile,
    comments: Vec<CommentFact>,
    functions: Vec<FunctionFact>,
}

impl SourceFacts {
    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn functions(&self) -> &[FunctionFact] {
        &self.functions
    }

    pub fn comments(&self) -> &[CommentFact] {
        &self.comments
    }
}

pub trait Analyzer {
    fn analyze(&self, source: &SourceFile) -> Result<SourceFacts, AnalyzerError>;
}

#[derive(Debug)]
pub enum AnalyzerError {
    ConfiguresParser {
        path: PathBuf,
        source: tree_sitter::LanguageError,
    },
    MissingSyntaxTree {
        path: PathBuf,
    },
    InvalidSyntax {
        path: PathBuf,
    },
    InvalidRange {
        path: PathBuf,
        source: SourceRangeError,
    },
    InvalidFunction {
        path: PathBuf,
        source: FunctionFactError,
    },
    InvalidComment {
        path: PathBuf,
        source: CommentFactError,
    },
}

pub fn analyze(source: &SourceFile) -> Result<SourceFacts, AnalyzerError> {
    match source.language() {
        Language::JavaScript => javascript::JavaScript.analyze(source),
        Language::Python => python::Python.analyze(source),
        Language::Rust => rust::Rust.analyze(source),
        Language::TypeScript => typescript::TypeScript.analyze(source),
    }
}

pub(super) fn analyze_with(
    source: &SourceFile,
    language: TreeSitterLanguage,
    is_function_node: fn(&str) -> bool,
) -> Result<SourceFacts, AnalyzerError> {
    let mut parser = Parser::new();

    parser
        .set_language(&language)
        .map_err(|error| AnalyzerError::ConfiguresParser {
            path: source.path().to_path_buf(),
            source: error,
        })?;

    let tree =
        parser
            .parse(source.source(), None)
            .ok_or_else(|| AnalyzerError::MissingSyntaxTree {
                path: source.path().to_path_buf(),
            })?;

    if tree.root_node().has_error() {
        return Err(AnalyzerError::InvalidSyntax {
            path: source.path().to_path_buf(),
        });
    }

    let mut functions = Vec::new();
    let mut comments = Vec::new();

    collect_source_facts(
        tree.root_node(),
        source,
        0,
        is_function_node,
        &mut functions,
        &mut comments,
    )?;

    Ok(SourceFacts {
        source: source.clone(),
        comments,
        functions,
    })
}

fn collect_source_facts(
    node: Node<'_>,
    source: &SourceFile,
    nesting_depth: u32,
    is_function_node: fn(&str) -> bool,
    functions: &mut Vec<FunctionFact>,
    comments: &mut Vec<CommentFact>,
) -> Result<(), AnalyzerError> {
    let is_function = is_function_node(node.kind());
    let child_nesting_depth = nesting_depth + u32::from(is_function);

    if is_function {
        functions.push(function_fact(
            node,
            source,
            nesting_depth,
            is_function_node,
        )?);
    }

    if node_is_comment(node, source) {
        comments.push(comment_fact(node, source)?);
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        collect_source_facts(
            child,
            source,
            child_nesting_depth,
            is_function_node,
            functions,
            comments,
        )?;
    }

    Ok(())
}

fn node_is_comment(node: Node<'_>, source: &SourceFile) -> bool {
    if !node.is_extra() {
        return false;
    }

    source
        .source()
        .get(node.byte_range())
        .is_some_and(|text| match source.language() {
            Language::JavaScript | Language::Rust | Language::TypeScript => {
                text.starts_with("//") || text.starts_with("/*")
            }
            Language::Python => text.starts_with('#'),
        })
}

fn comment_fact(node: Node<'_>, source: &SourceFile) -> Result<CommentFact, AnalyzerError> {
    let range = node_range(node, source)?;

    CommentFact::new(source.clone(), range).map_err(|error| AnalyzerError::InvalidComment {
        path: source.path().to_path_buf(),
        source: error,
    })
}

fn function_fact(
    node: Node<'_>,
    source: &SourceFile,
    nesting_depth: u32,
    is_function_node: fn(&str) -> bool,
) -> Result<FunctionFact, AnalyzerError> {
    let range = node_range(node, source)?;
    let body_range = node
        .child_by_field_name("body")
        .map(|body| node_range(body, source))
        .transpose()?
        .unwrap_or(range);
    let body_is_empty = node
        .child_by_field_name("body")
        .is_some_and(|body| body_is_empty(body, source.language()));
    let parameter_count = parameter_count(node);
    let decision_points = decision_points(node, source.language(), is_function_node);
    let return_count = return_count(node, source.language(), is_function_node);
    let name = node
        .child_by_field_name("name")
        .and_then(|name| source.source().get(name.byte_range()))
        .map(str::to_owned);

    FunctionFact::new(
        source.clone(),
        name,
        FunctionFactDetails {
            range,
            body_range,
            parameter_count,
            decision_points,
            return_count,
            body_is_empty,
            nesting_depth,
        },
    )
    .map_err(|error| AnalyzerError::InvalidFunction {
        path: source.path().to_path_buf(),
        source: error,
    })
}

fn decision_points(
    function: Node<'_>,
    language: Language,
    is_function_node: fn(&str) -> bool,
) -> u32 {
    let mut cursor = function.walk();

    function
        .children(&mut cursor)
        .map(|child| decision_points_in(child, language, is_function_node))
        .sum()
}

fn decision_points_in(
    node: Node<'_>,
    language: Language,
    is_function_node: fn(&str) -> bool,
) -> u32 {
    if is_function_node(node.kind()) {
        return 0;
    }

    let mut cursor = node.walk();

    decision_point(node.kind(), language)
        + node
            .children(&mut cursor)
            .map(|child| decision_points_in(child, language, is_function_node))
            .sum::<u32>()
}

fn decision_point(kind: &str, language: Language) -> u32 {
    let is_decision = match language {
        Language::JavaScript | Language::TypeScript => matches!(
            kind,
            "catch_clause"
                | "do_statement"
                | "for_in_statement"
                | "for_statement"
                | "if_statement"
                | "switch_case"
                | "ternary_expression"
                | "while_statement"
        ),
        Language::Python => matches!(
            kind,
            "case_clause"
                | "conditional_expression"
                | "elif_clause"
                | "except_clause"
                | "for_statement"
                | "if_statement"
                | "while_statement"
        ),
        Language::Rust => matches!(
            kind,
            "for_expression"
                | "if_expression"
                | "loop_expression"
                | "match_arm"
                | "while_expression"
        ),
    };

    u32::from(is_decision)
}

fn return_count(function: Node<'_>, language: Language, is_function_node: fn(&str) -> bool) -> u32 {
    let mut cursor = function.walk();

    function
        .children(&mut cursor)
        .map(|child| return_count_in(child, language, is_function_node))
        .sum()
}

fn return_count_in(node: Node<'_>, language: Language, is_function_node: fn(&str) -> bool) -> u32 {
    if is_function_node(node.kind()) {
        return 0;
    }

    let mut cursor = node.walk();

    u32::from(is_return(node.kind(), language))
        + node
            .children(&mut cursor)
            .map(|child| return_count_in(child, language, is_function_node))
            .sum::<u32>()
}

fn is_return(kind: &str, language: Language) -> bool {
    match language {
        Language::JavaScript | Language::Python | Language::TypeScript => {
            kind == "return_statement"
        }
        Language::Rust => kind == "return_expression",
    }
}

fn parameter_count(node: Node<'_>) -> u32 {
    if node.child_by_field_name("parameter").is_some() {
        return 1;
    }

    node.child_by_field_name("parameters")
        .map_or(0, |parameters| parameters.named_child_count() as u32)
}

fn body_is_empty(body: Node<'_>, language: Language) -> bool {
    if language == Language::Python {
        return body.named_child_count() == 1
            && body
                .named_child(0)
                .is_some_and(|statement| statement.kind() == "pass_statement");
    }

    let mut cursor = body.walk();

    body.named_children(&mut cursor)
        .all(|child| child.is_extra())
}

fn node_range(node: Node<'_>, source: &SourceFile) -> Result<SourceRange, AnalyzerError> {
    SourceRange::new(node.start_byte(), node.end_byte()).map_err(|error| {
        AnalyzerError::InvalidRange {
            path: source.path().to_path_buf(),
            source: error,
        }
    })
}

impl fmt::Display for AnalyzerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfiguresParser { path, source } => {
                write!(
                    formatter,
                    "unable to configure parser for {}: {source}",
                    path.display()
                )
            }
            Self::MissingSyntaxTree { path } => {
                write!(
                    formatter,
                    "parser produced no syntax tree for {}",
                    path.display()
                )
            }
            Self::InvalidSyntax { path } => {
                write!(formatter, "invalid syntax in {}", path.display())
            }
            Self::InvalidRange { path, source } => {
                write!(formatter, "invalid range in {}: {source}", path.display())
            }
            Self::InvalidFunction { path, source } => {
                write!(
                    formatter,
                    "invalid function in {}: {source}",
                    path.display()
                )
            }
            Self::InvalidComment { path, source } => {
                write!(formatter, "invalid comment in {}: {source}", path.display())
            }
        }
    }
}

impl Error for AnalyzerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ConfiguresParser { source, .. } => Some(source),
            Self::InvalidRange { source, .. } => Some(source),
            Self::InvalidFunction { source, .. } => Some(source),
            Self::InvalidComment { source, .. } => Some(source),
            Self::MissingSyntaxTree { .. } | Self::InvalidSyntax { .. } => None,
        }
    }
}
