use std::{error::Error, fmt, path::PathBuf};

use tree_sitter::{Language as TreeSitterLanguage, Node, Parser};

use crate::{
    facts::{FunctionFact, FunctionFactError},
    source::{Language, SourceFile, SourceRange, SourceRangeError},
};

pub struct Analyzer;

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

impl Analyzer {
    pub fn extract_functions(source: &SourceFile) -> Result<Vec<FunctionFact>, AnalyzerError> {
        let mut parser = Parser::new();
        let language = Self::tree_sitter_language(source);
        let path = source.path().to_path_buf();

        parser
            .set_language(&language)
            .map_err(|source| AnalyzerError::ConfiguresParser { path, source })?;

        let tree = parser.parse(source.source(), None).ok_or_else(|| {
            AnalyzerError::MissingSyntaxTree {
                path: source.path().to_path_buf(),
            }
        })?;

        if tree.root_node().has_error() {
            return Err(AnalyzerError::InvalidSyntax {
                path: source.path().to_path_buf(),
            });
        }

        let mut functions = Vec::new();

        Self::collect_functions(tree.root_node(), source, 0, &mut functions)?;

        Ok(functions)
    }

    fn tree_sitter_language(source: &SourceFile) -> TreeSitterLanguage {
        match source.language() {
            Language::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Language::Python => tree_sitter_python::LANGUAGE.into(),
            Language::Rust => tree_sitter_rust::LANGUAGE.into(),
            Language::TypeScript
                if source
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "tsx") =>
            {
                tree_sitter_typescript::LANGUAGE_TSX.into()
            }
            Language::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        }
    }

    fn collect_functions(
        node: Node<'_>,
        source: &SourceFile,
        nesting_depth: u32,
        functions: &mut Vec<FunctionFact>,
    ) -> Result<(), AnalyzerError> {
        let is_function = Self::is_function_node(source.language(), node.kind());
        let child_nesting_depth = nesting_depth + u32::from(is_function);

        if is_function {
            functions.push(Self::function_fact(node, source, nesting_depth)?);
        }

        let mut cursor = node.walk();

        for child in node.children(&mut cursor) {
            Self::collect_functions(child, source, child_nesting_depth, functions)?;
        }

        Ok(())
    }

    fn is_function_node(language: Language, kind: &str) -> bool {
        match language {
            Language::JavaScript | Language::TypeScript => matches!(
                kind,
                "arrow_function"
                    | "function_declaration"
                    | "function_expression"
                    | "generator_function_declaration"
                    | "method_definition"
            ),
            Language::Python => kind == "function_definition",
            Language::Rust => kind == "function_item",
        }
    }

    fn function_fact(
        node: Node<'_>,
        source: &SourceFile,
        nesting_depth: u32,
    ) -> Result<FunctionFact, AnalyzerError> {
        let range = Self::range(node, source)?;
        let body_range = node
            .child_by_field_name("body")
            .map(|body| Self::range(body, source))
            .transpose()?
            .unwrap_or(range);
        let name = node
            .child_by_field_name("name")
            .and_then(|name| source.source().get(name.byte_range()))
            .map(str::to_owned);

        let path = source.path().to_path_buf();

        FunctionFact::new(source.clone(), name, range, body_range, nesting_depth)
            .map_err(|source| AnalyzerError::InvalidFunction { path, source })
    }

    fn range(node: Node<'_>, source: &SourceFile) -> Result<SourceRange, AnalyzerError> {
        let path = source.path().to_path_buf();

        SourceRange::new(node.start_byte(), node.end_byte())
            .map_err(|source| AnalyzerError::InvalidRange { path, source })
    }
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
