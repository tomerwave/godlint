use std::{error::Error, fmt, path::PathBuf};

use tree_sitter::{Language as TreeSitterLanguage, Node, Parser};

use crate::{
    facts::{FunctionFact, FunctionFactError},
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
    functions: Vec<FunctionFact>,
}

impl SourceFacts {
    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn functions(&self) -> &[FunctionFact] {
        &self.functions
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

    collect_functions(
        tree.root_node(),
        source,
        0,
        is_function_node,
        &mut functions,
    )?;

    Ok(SourceFacts {
        source: source.clone(),
        functions,
    })
}

fn collect_functions(
    node: Node<'_>,
    source: &SourceFile,
    nesting_depth: u32,
    is_function_node: fn(&str) -> bool,
    functions: &mut Vec<FunctionFact>,
) -> Result<(), AnalyzerError> {
    let is_function = is_function_node(node.kind());
    let child_nesting_depth = nesting_depth + u32::from(is_function);

    if is_function {
        functions.push(function_fact(node, source, nesting_depth)?);
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        collect_functions(
            child,
            source,
            child_nesting_depth,
            is_function_node,
            functions,
        )?;
    }

    Ok(())
}

fn function_fact(
    node: Node<'_>,
    source: &SourceFile,
    nesting_depth: u32,
) -> Result<FunctionFact, AnalyzerError> {
    let range = node_range(node, source)?;
    let body_range = node
        .child_by_field_name("body")
        .map(|body| node_range(body, source))
        .transpose()?
        .unwrap_or(range);
    let name = node
        .child_by_field_name("name")
        .and_then(|name| source.source().get(name.byte_range()))
        .map(str::to_owned);

    FunctionFact::new(source.clone(), name, range, body_range, nesting_depth).map_err(|error| {
        AnalyzerError::InvalidFunction {
            path: source.path().to_path_buf(),
            source: error,
        }
    })
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
        }
    }
}

impl Error for AnalyzerError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ConfiguresParser { source, .. } => Some(source),
            Self::InvalidRange { source, .. } => Some(source),
            Self::InvalidFunction { source, .. } => Some(source),
            Self::MissingSyntaxTree { .. } | Self::InvalidSyntax { .. } => None,
        }
    }
}
