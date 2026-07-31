use tree_sitter::Parser;

use crate::{
    analyzers::AnalyzerError,
    facts::{ActionFact, CredentialFact, ExpressionFact, JobFact, StepFact},
    source::{SourceRange, TextFile},
};

use self::{collect::JobCollection, syntax::value_of};

mod collect;
mod syntax;

const CONCURRENCY: &str = "concurrency";
const JOBS: &str = "jobs";
const PERMISSIONS: &str = "permissions";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowFacts {
    file: TextFile,
    unparsed: Vec<SourceRange>,
    actions: Vec<ActionFact>,
    jobs: Vec<JobFact>,
    steps: Vec<StepFact>,
    expressions: Vec<ExpressionFact>,
    comments: Vec<SourceRange>,
    credentials: Vec<CredentialFact>,
    declares_permissions: bool,
    declares_concurrency: bool,
}

impl WorkflowFacts {
    pub fn file(&self) -> &TextFile {
        &self.file
    }

    pub fn unparsed(&self) -> &[SourceRange] {
        &self.unparsed
    }

    pub fn actions(&self) -> &[ActionFact] {
        &self.actions
    }

    pub fn jobs(&self) -> &[JobFact] {
        &self.jobs
    }

    pub fn steps(&self) -> &[StepFact] {
        &self.steps
    }

    pub fn expressions(&self) -> &[ExpressionFact] {
        &self.expressions
    }

    pub fn comments(&self) -> &[SourceRange] {
        &self.comments
    }

    pub fn credentials(&self) -> &[CredentialFact] {
        &self.credentials
    }

    pub fn declares_permissions(&self) -> bool {
        self.declares_permissions
    }

    pub fn declares_concurrency(&self) -> bool {
        self.declares_concurrency
    }
}

pub fn read(file: &TextFile) -> Result<WorkflowFacts, AnalyzerError> {
    let tree = parse(file)?;
    let root = tree.root_node();
    let workflow = syntax::mapping(root);
    let collected = collect::jobs(value_of(workflow, JOBS, file), file)?;

    facts(file, root, workflow, collected)
}

fn facts(
    file: &TextFile,
    root: tree_sitter::Node<'_>,
    workflow: Option<tree_sitter::Node<'_>>,
    collected: JobCollection,
) -> Result<WorkflowFacts, AnalyzerError> {
    Ok(WorkflowFacts {
        file: file.clone(),
        unparsed: collect::unparsed(root, file)?,
        actions: collect::actions(root, file)?,
        jobs: collected.jobs,
        steps: collected.steps,
        expressions: collect::expressions(root, file)?,
        comments: collect::comments(root, file)?,
        credentials: collected.credentials,
        declares_permissions: syntax::declared(workflow, PERMISSIONS, file),
        declares_concurrency: syntax::declared(workflow, CONCURRENCY, file),
    })
}

fn parse(file: &TextFile) -> Result<tree_sitter::Tree, AnalyzerError> {
    let mut parser = Parser::new();

    parser
        .set_language(&tree_sitter_yaml::LANGUAGE.into())
        .map_err(|source| AnalyzerError::ConfiguresParser {
            path: file.path().to_path_buf(),
            source,
        })?;

    parser
        .parse(file.text(), None)
        .ok_or_else(|| AnalyzerError::MissingSyntaxTree {
            path: file.path().to_path_buf(),
        })
}
