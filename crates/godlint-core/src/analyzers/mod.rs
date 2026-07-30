use std::{error::Error, fmt, path::PathBuf};

use tree_sitter::{Language as TreeSitterLanguage, Node, Parser};

use crate::{
    facts::{
        AccessFact, CallFact, CommentFact, ConditionFact, ErrorHandlerFact, FunctionFact,
        FunctionFactDetails, FunctionFactError, ImportFact,
    },
    source::{Language, SourceFile, SourceFileError, SourceRange},
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
    conditions: Vec<ConditionFact>,
    error_handlers: Vec<ErrorHandlerFact>,
    functions: Vec<FunctionFact>,
    imports: Vec<ImportFact>,
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

    pub fn error_handlers(&self) -> &[ErrorHandlerFact] {
        &self.error_handlers
    }

    pub fn conditions(&self) -> &[ConditionFact] {
        &self.conditions
    }

    pub fn imports(&self) -> &[ImportFact] {
        &self.imports
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
        source: SourceFileError,
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
        conditions: collected.conditions,
        error_handlers: collected.error_handlers,
        functions: collected.functions,
        imports: collected.imports,
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
    conditions: Vec<ConditionFact>,
    error_handlers: Vec<ErrorHandlerFact>,
    imports: Vec<ImportFact>,
}

impl Collected {
    fn absorb(
        &mut self,
        node: Node<'_>,
        source: &SourceFile,
        vocabulary: &Vocabulary,
    ) -> Result<(), AnalyzerError> {
        self.absorb_declarations(node, source, vocabulary)?;
        self.absorb_references(node, source, vocabulary)
    }

    fn absorb_declarations(
        &mut self,
        node: Node<'_>,
        source: &SourceFile,
        vocabulary: &Vocabulary,
    ) -> Result<(), AnalyzerError> {
        self.functions
            .extend(function_fact(node, source, vocabulary)?);
        self.comments
            .extend(comment_fact(node, source, vocabulary)?);

        Ok(())
    }

    fn absorb_references(
        &mut self,
        node: Node<'_>,
        source: &SourceFile,
        vocabulary: &Vocabulary,
    ) -> Result<(), AnalyzerError> {
        self.absorb_expressions(node, source, vocabulary)?;
        self.absorb_paths(node, source, vocabulary)
    }

    fn absorb_expressions(
        &mut self,
        node: Node<'_>,
        source: &SourceFile,
        vocabulary: &Vocabulary,
    ) -> Result<(), AnalyzerError> {
        self.calls.extend(call_fact(node, source, vocabulary)?);
        self.conditions
            .extend(condition_fact(node, source, vocabulary)?);
        self.error_handlers
            .extend(error_handler_fact(node, source, vocabulary)?);

        Ok(())
    }

    fn absorb_paths(
        &mut self,
        node: Node<'_>,
        source: &SourceFile,
        vocabulary: &Vocabulary,
    ) -> Result<(), AnalyzerError> {
        self.accesses.extend(access_fact(node, source, vocabulary)?);
        self.imports.extend(import_fact(node, source, vocabulary)?);

        Ok(())
    }
}

fn error_handler_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<ErrorHandlerFact>, AnalyzerError> {
    let Some(handler) = (vocabulary.error_handler)(node) else {
        return Ok(None);
    };

    Ok(Some(ErrorHandlerFact::new(
        source.clone(),
        node_range(handler.node, source)?,
        handler.body_is_empty,
    )))
}

fn condition_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<ConditionFact>, AnalyzerError> {
    let Some(condition) = (vocabulary.condition)(node) else {
        return Ok(None);
    };

    Ok(Some(ConditionFact::new(
        source.clone(),
        node_range(condition.node, source)?,
        condition.operator_count,
    )))
}

fn collect_source_facts(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
    collected: &mut Collected,
) -> Result<(), AnalyzerError> {
    collected.absorb(node, source, vocabulary)?;

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
    if direct_path(callee.node, source).is_none() {
        return Ok(None);
    }

    let range = node_range(callee.node, source)?;
    Ok(Some(CallFact::new(
        source.clone(),
        range,
        callee.is_macro,
        argument_count(node),
    )))
}

fn argument_count(node: Node<'_>) -> usize {
    let Some(arguments) = node.child_by_field_name("arguments") else {
        return 0;
    };
    let mut cursor = arguments.walk();

    arguments
        .named_children(&mut cursor)
        .filter(|argument| !argument.is_extra())
        .count()
}

fn access_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<AccessFact>, AnalyzerError> {
    if !(vocabulary.is_access)(node.kind()) {
        return Ok(None);
    }

    if direct_path(node, source).is_none() {
        return Ok(None);
    }

    let range = node_range(node, source)?;

    Ok(Some(AccessFact::new(source.clone(), range)))
}

fn import_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<ImportFact>, AnalyzerError> {
    let Some(module) = (vocabulary.import)(node) else {
        return Ok(None);
    };
    let range = node_range(module, source)?;

    Ok(Some(ImportFact::new(source.clone(), range)))
}

fn direct_path<'source>(node: Node<'_>, source: &'source SourceFile) -> Option<&'source str> {
    source
        .source()
        .get(node.byte_range())
        .filter(|text| is_direct_path(text))
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
    vocabulary: &Vocabulary,
) -> Result<Option<CommentFact>, AnalyzerError> {
    let Some(kind) = (vocabulary.comment_kind)(node, source.source()) else {
        return Ok(None);
    };
    let range = node_range(node, source)?;

    Ok(Some(CommentFact::new(source.clone(), range, kind)))
}

fn function_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<FunctionFact>, AnalyzerError> {
    if !metrics::is_function(node, vocabulary) {
        return Ok(None);
    }

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
            cognitive_score: metrics::cognitive_score(node, vocabulary),
            return_paths: metrics::return_paths(node, vocabulary),
            statement_count: metrics::statement_count(node, vocabulary),
            block_depth: metrics::block_depth(node, vocabulary),
            body_is_empty: metrics::body_is_empty(node, text, vocabulary),
            is_abstract: (vocabulary.is_abstract)(node, text),
        },
    )
    .map(Some)
    .map_err(|error| AnalyzerError::InvalidFunction {
        path: source.path().to_path_buf(),
        source: error,
    })
}

fn node_range(node: Node<'_>, source: &SourceFile) -> Result<SourceRange, AnalyzerError> {
    source
        .range(node.start_byte(), node.end_byte())
        .map_err(|error| AnalyzerError::InvalidRange {
            path: source.path().to_path_buf(),
            source: error,
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
