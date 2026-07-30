use std::{error::Error, fmt, path::PathBuf};

use tree_sitter::{Language as TreeSitterLanguage, Node, Parser};

use crate::{
    facts::{
        AccessFact, CallArgument, CallFact, CallFactDetails, CallTarget, CommentFact,
        ConditionFact, ErrorHandlerFact, FunctionFact, FunctionFactDetails, FunctionFactError,
        ImportFact, TestFact, TestFactDetails, TestFocus,
        AccessFact, AssertionFact, AssertionFactDetails, CallArgument, CallFact, CallTarget,
        CommentFact, ConditionFact, ErrorHandlerFact, FunctionFact, FunctionFactDetails,
        FunctionFactError, ImportFact, TestFact, TestFactDetails, TestFocus,
    },
    source::{Language, SourceFile, SourceFileError, SourceRange},
};

use self::vocabulary::{Focus, Vocabulary};

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
    unparsed: Vec<SourceRange>,
    accesses: Vec<AccessFact>,
    comments: Vec<CommentFact>,
    calls: Vec<CallFact>,
    conditions: Vec<ConditionFact>,
    error_handlers: Vec<ErrorHandlerFact>,
    functions: Vec<FunctionFact>,
    imports: Vec<ImportFact>,
    tests: Vec<TestFact>,
    assertions: Vec<AssertionFact>,
}

impl SourceFacts {
    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn unparsed(&self) -> &[SourceRange] {
        &self.unparsed
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

    pub fn tests(&self) -> &[TestFact] {
        &self.tests
    }

    pub fn assertions(&self) -> &[AssertionFact] {
        &self.assertions
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
        unparsed: collected.unparsed,
        accesses: collected.accesses,
        comments: collected.comments,
        calls: collected.calls,
        conditions: collected.conditions,
        error_handlers: collected.error_handlers,
        functions: collected.functions,
        imports: collected.imports,
        tests: collected.tests,
        assertions: collected.assertions,
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

    Ok(tree)
}

#[derive(Default)]
struct Collected {
    unparsed: Vec<SourceRange>,
    accesses: Vec<AccessFact>,
    functions: Vec<FunctionFact>,
    tests: Vec<TestFact>,
    comments: Vec<CommentFact>,
    calls: Vec<CallFact>,
    conditions: Vec<ConditionFact>,
    error_handlers: Vec<ErrorHandlerFact>,
    imports: Vec<ImportFact>,
    assertions: Vec<AssertionFact>,
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
        self.tests.extend(test_fact(node, source, vocabulary)?);
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
        self.assertions
            .extend(assertion_fact(node, source, vocabulary)?);
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

fn assertion_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<AssertionFact>, AnalyzerError> {
    let Some(assertion) = (vocabulary.assertion)(node, source.source()) else {
        return Ok(None);
    };

    Ok(Some(AssertionFact::new(
        source.clone(),
        node_range(assertion.node, source)?,
        AssertionFactDetails {
            target: CallTarget {
                path: assertion.name,
                is_macro: assertion.is_macro,
            },
            operands: assertion.operands,
        },
    )))
}

fn test_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<TestFact>, AnalyzerError> {
    let Some(declaration) = (vocabulary.test)(node, source.source()) else {
        return Ok(None);
    };

    Ok(Some(TestFact::new(
        source.clone(),
        node_range(declaration.node, source)?,
        TestFactDetails {
            name: text_of(declaration.name, source).map(str::to_owned),
            marker: text_of(Some(declaration.marker), source)
                .unwrap_or_default()
                .to_owned(),
            focus: focus_of(declaration.focus),
        },
    )))
}

fn focus_of(focus: Focus) -> TestFocus {
    match focus {
        Focus::Ordinary => TestFocus::Ordinary,
        Focus::Only => TestFocus::Only,
        Focus::Skipped => TestFocus::Skipped,
    }
}

fn text_of<'source>(node: Option<Node<'_>>, source: &'source SourceFile) -> Option<&'source str> {
    source.source().get(node?.byte_range())
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
    if !node.has_error() {
        collected.absorb(node, source, vocabulary)?;

        return absorb_children(node, source, vocabulary, collected);
    }

    if node.is_error() || node.is_missing() {
        collected.unparsed.push(node_range(node, source)?);
    }

    absorb_children(node, source, vocabulary, collected)
}

fn absorb_children(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
    collected: &mut Collected,
) -> Result<(), AnalyzerError> {
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
    let Some(path) = direct_path(callee.node, source, vocabulary) else {
        return Ok(None);
    };
    let range = node_range(callee.node, source)?;

    Ok(Some(CallFact::new(
        source.clone(),
        range,
        node_range(node, source)?,
        CallFactDetails {
            target: CallTarget {
                path,
                is_macro: callee.is_macro,
            },
            arguments: call_arguments(node, source, vocabulary),
        },
    )))
}

fn call_arguments(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Vec<CallArgument> {
    let Some(arguments) = node.child_by_field_name("arguments") else {
        return Vec::new();
    };
    let mut cursor = arguments.walk();

    arguments
        .named_children(&mut cursor)
        .filter(|argument| !argument.is_extra())
        .filter_map(|argument| call_argument(argument, source, vocabulary))
        .collect()
}

fn call_argument(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Option<CallArgument> {
    let argument = (vocabulary.argument)(node)?;

    Some(CallArgument {
        name: argument
            .name
            .and_then(|name| source.source().get(name.byte_range()))
            .map(str::to_owned),
        literal: (vocabulary.literal)(argument.value, source.source()),
    })
}

fn access_fact(
    node: Node<'_>,
    source: &SourceFile,
    vocabulary: &Vocabulary,
) -> Result<Option<AccessFact>, AnalyzerError> {
    if !(vocabulary.is_access)(node.kind()) {
        return Ok(None);
    }

    let Some(path) = direct_path(node, source, vocabulary) else {
        return Ok(None);
    };
    let range = node_range(node, source)?;

    Ok(Some(AccessFact::new(source.clone(), range, path)))
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

fn direct_path(node: Node<'_>, source: &SourceFile, vocabulary: &Vocabulary) -> Option<String> {
    source
        .source()
        .get(node.byte_range())
        .and_then(|text| (vocabulary.direct_path)(text))
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
            Self::MissingSyntaxTree { .. } => None,
        }
    }
}
