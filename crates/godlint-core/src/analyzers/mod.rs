use std::{error::Error, fmt, path::PathBuf};

use tree_sitter::{Language as TreeSitterLanguage, Node, Parser};

use crate::{
    facts::{
        AccessFact, AccessFactError, CallFact, CallFactError, CommentFact, CommentFactError,
        CommentKind, FunctionFact, FunctionFactDetails, FunctionFactError,
    },
    source::{Language, SourceFile, SourceRange, SourceRangeError},
};

use self::vocabulary::Vocabulary;

mod ecmascript;
mod javascript;
mod metrics;
mod python;
mod rust;
mod typescript;
mod vocabulary;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFacts {
    source: SourceFile,
    accesses: Vec<AccessFact>,
    comments: Vec<CommentFact>,
    calls: Vec<CallFact>,
    functions: Vec<FunctionFact>,
}

impl SourceFacts {
    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn functions(&self) -> &[FunctionFact] {
        &self.functions
    }

    pub fn accesses(&self) -> &[AccessFact] {
        &self.accesses
    }

    pub fn comments(&self) -> &[CommentFact] {
        &self.comments
    }

    pub fn calls(&self) -> &[CallFact] {
        &self.calls
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
    InvalidCall {
        path: PathBuf,
        source: CallFactError,
    },
    InvalidAccess {
        path: PathBuf,
        source: AccessFactError,
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

pub(crate) fn analyze_with(
    source: &SourceFile,
    language: TreeSitterLanguage,
    vocabulary: Vocabulary,
) -> Result<SourceFacts, AnalyzerError> {
    let tree = parse(source, language)?;
    let mut collected = Collected::default();

    collect_source_facts(tree.root_node(), source, &vocabulary, &mut collected)?;

    Ok(SourceFacts {
        source: source.clone(),
        accesses: collected.accesses,
        comments: collected.comments,
        calls: collected.calls,
        functions: collected.functions,
    })
}

fn parse(
    source: &SourceFile,
    language: TreeSitterLanguage,
) -> Result<tree_sitter::Tree, AnalyzerError> {
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

    Ok(tree)
}

#[derive(Default)]
struct Collected {
    accesses: Vec<AccessFact>,
    functions: Vec<FunctionFact>,
    comments: Vec<CommentFact>,
    calls: Vec<CallFact>,
}

impl Collected {
    fn absorb_declarations(
        &mut self,
        node: Node<'_>,
        source: &SourceFile,
        vocabulary: &Vocabulary,
    ) -> Result<(), AnalyzerError> {
        if metrics::is_function(node, vocabulary) {
            self.functions
                .push(function_fact(node, source, vocabulary)?);
        }

        if let Some(kind) = (vocabulary.comment_kind)(node, source.source()) {
            self.comments.push(comment_fact(node, source, kind)?);
        }

        Ok(())
    }

    fn absorb_references(
        &mut self,
        node: Node<'_>,
        source: &SourceFile,
        vocabulary: &Vocabulary,
    ) -> Result<(), AnalyzerError> {
        if let Some(call) = call_fact(node, source, vocabulary)? {
            self.calls.push(call);
        }

        if let Some(access) = access_fact(node, source, vocabulary)? {
            self.accesses.push(access);
        }

        Ok(())
    }
}

fn collect_source_facts(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
    collected: &mut Collected,
) -> Result<(), AnalyzerError> {
    collected.absorb_declarations(node, source, vocabulary)?;
    collected.absorb_references(node, source, vocabulary)?;

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        collect_source_facts(child, source, vocabulary, collected)?;
    }

    Ok(())
}

fn call_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<CallFact>, AnalyzerError> {
    let Some(callee) = (vocabulary.callee)(node) else {
        return Ok(None);
    };
    let callee_text = source
        .source()
        .get(callee.node.byte_range())
        .unwrap_or_default();

    if !is_direct_path(callee_text) {
        return Ok(None);
    }

    let range = node_range(callee.node, source)?;
    let fact = CallFact::new(
        source.clone(),
        range,
        callee_text.to_owned(),
        callee.is_macro,
    )
    .map_err(|error| AnalyzerError::InvalidCall {
        path: source.path().to_path_buf(),
        source: error,
    })?;

    Ok(Some(fact))
}

fn access_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<AccessFact>, AnalyzerError> {
    if !node.is_named() || !(vocabulary.is_access)(node.kind()) {
        return Ok(None);
    }

    let text = source.source().get(node.byte_range()).unwrap_or_default();

    if !is_direct_path(text) {
        return Ok(None);
    }

    let range = node_range(node, source)?;
    let fact = AccessFact::new(source.clone(), range, text.to_owned()).map_err(|error| {
        AnalyzerError::InvalidAccess {
            path: source.path().to_path_buf(),
            source: error,
        }
    })?;

    Ok(Some(fact))
}

fn is_direct_path(text: &str) -> bool {
    !text.is_empty()
        && text
            .chars()
            .all(|character| character.is_alphanumeric() || "_.:".contains(character))
}

fn comment_fact(
    node: Node<'_>,
    source: &SourceFile,
    kind: CommentKind,
) -> Result<CommentFact, AnalyzerError> {
    let range = node_range(node, source)?;

    CommentFact::new(source.clone(), range, kind).map_err(|error| AnalyzerError::InvalidComment {
        path: source.path().to_path_buf(),
        source: error,
    })
}

fn function_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<FunctionFact, AnalyzerError> {
    let range = node_range(node, source)?;
    let body_range = node
        .child_by_field_name("body")
        .map(|body| node_range(body, source))
        .transpose()?
        .unwrap_or(range);
    let text = source.source();
    let name = node
        .child_by_field_name("name")
        .and_then(|name| text.get(name.byte_range()))
        .map(str::to_owned);

    FunctionFact::new(
        source.clone(),
        name,
        FunctionFactDetails {
            range,
            body_range,
            parameter_count: metrics::parameter_count(node, text, vocabulary),
            decision_points: metrics::decision_points(node, vocabulary),
            return_paths: metrics::return_paths(node, vocabulary),
            statement_count: metrics::statement_count(node, vocabulary),
            block_depth: metrics::block_depth(node, vocabulary),
            body_is_empty: metrics::body_is_empty(node, text, vocabulary),
            is_abstract: (vocabulary.is_abstract)(node, text),
        },
    )
    .map_err(|error| AnalyzerError::InvalidFunction {
        path: source.path().to_path_buf(),
        source: error,
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
            Self::InvalidComment { path, source } => {
                write!(formatter, "invalid comment in {}: {source}", path.display())
            }
            Self::InvalidCall { path, source } => {
                write!(formatter, "invalid call in {}: {source}", path.display())
            }
            Self::InvalidAccess { path, source } => {
                write!(formatter, "invalid access in {}: {source}", path.display())
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
            Self::InvalidCall { source, .. } => Some(source),
            Self::InvalidAccess { source, .. } => Some(source),
            Self::MissingSyntaxTree { .. } | Self::InvalidSyntax { .. } => None,
        }
    }
}
