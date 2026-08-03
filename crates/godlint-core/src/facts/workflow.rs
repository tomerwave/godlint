use crate::source::{SourceRange, TextFile};

const QUOTES: [char; 2] = ['"', '\''];
const COMMIT_LENGTH: usize = 40;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionFact {
    file: TextFile,
    range: SourceRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Setting {
    file: TextFile,
    key: String,
    range: SourceRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Secrets {
    Inherit { range: SourceRange },
    Named(Vec<Setting>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobFact {
    file: TextFile,
    range: SourceRange,
    name: String,
    body: SourceRange,
    condition: Option<SourceRange>,
    continue_on_error: Option<SourceRange>,
    declares_permissions: bool,
    needs: Vec<Setting>,
    secrets: Option<Secrets>,
    calls_workflow: Option<SourceRange>,
    step_count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepFact {
    file: TextFile,
    range: SourceRange,
    job: String,
    id: Option<SourceRange>,
    name: Option<SourceRange>,
    run: Option<SourceRange>,
    uses: Option<SourceRange>,
    condition: Option<SourceRange>,
    continue_on_error: Option<SourceRange>,
    inputs: Vec<Setting>,
    environment: Vec<Setting>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpressionFact {
    file: TextFile,
    range: SourceRange,
    context: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CredentialFact {
    range: SourceRange,
    key: String,
    job: String,
    literal: bool,
}

pub(crate) struct JobFactDetails {
    pub range: SourceRange,
    pub name: String,
    pub body: SourceRange,
    pub condition: Option<SourceRange>,
    pub continue_on_error: Option<SourceRange>,
    pub declares_permissions: bool,
    pub needs: Vec<Setting>,
    pub secrets: Option<Secrets>,
    pub calls_workflow: Option<SourceRange>,
    pub step_count: usize,
}

pub(crate) struct StepFactDetails {
    pub range: SourceRange,
    pub job: String,
    pub id: Option<SourceRange>,
    pub name: Option<SourceRange>,
    pub run: Option<SourceRange>,
    pub uses: Option<SourceRange>,
    pub condition: Option<SourceRange>,
    pub continue_on_error: Option<SourceRange>,
    pub inputs: Vec<Setting>,
    pub environment: Vec<Setting>,
}

impl ActionFact {
    pub fn new(file: TextFile, range: SourceRange) -> Self {
        Self { file, range }
    }

    pub fn file(&self) -> &TextFile {
        &self.file
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn reference(&self) -> &str {
        text(&self.file, self.range)
    }

    pub fn name(&self) -> &str {
        match self.reference().split_once('@') {
            Some((name, _)) => name,
            None => self.reference(),
        }
    }

    pub fn version(&self) -> Option<&str> {
        self.reference().split_once('@').map(|(_, version)| version)
    }

    pub fn owner(&self) -> Option<&str> {
        (!self.is_local() && !self.is_container())
            .then(|| self.name().split('/').next())
            .flatten()
            .filter(|owner| !owner.is_empty())
    }

    pub fn is_commit(&self) -> bool {
        self.version().is_some_and(is_commit)
    }

    pub fn is_local(&self) -> bool {
        self.reference().starts_with("./") || self.reference().starts_with(".\\")
    }

    pub fn is_container(&self) -> bool {
        self.reference().starts_with("docker://")
    }
}

impl Setting {
    pub(crate) fn new(file: TextFile, key: String, range: SourceRange) -> Self {
        Self { file, key, range }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn file(&self) -> &TextFile {
        &self.file
    }
}

impl JobFact {
    pub(crate) fn from_details(file: TextFile, details: JobFactDetails) -> Self {
        Self {
            file,
            range: details.range,
            name: details.name,
            body: details.body,
            condition: details.condition,
            continue_on_error: details.continue_on_error,
            declares_permissions: details.declares_permissions,
            needs: details.needs,
            secrets: details.secrets,
            calls_workflow: details.calls_workflow,
            step_count: details.step_count,
        }
    }

    pub fn file(&self) -> &TextFile {
        &self.file
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn declares_permissions(&self) -> bool {
        self.declares_permissions
    }

    pub fn body(&self) -> SourceRange {
        self.body
    }

    pub fn condition(&self) -> Option<SourceRange> {
        self.condition
    }

    pub fn continue_on_error(&self) -> Option<SourceRange> {
        self.continue_on_error
    }

    pub fn needs(&self) -> &[Setting] {
        &self.needs
    }

    pub fn secrets(&self) -> Option<&Secrets> {
        self.secrets.as_ref()
    }

    pub fn calls_workflow(&self) -> Option<SourceRange> {
        self.calls_workflow
    }

    pub fn step_count(&self) -> usize {
        self.step_count
    }
}

impl StepFact {
    pub(crate) fn new(file: TextFile, details: StepFactDetails) -> Self {
        Self {
            file,
            range: details.range,
            job: details.job,
            id: details.id,
            name: details.name,
            run: details.run,
            uses: details.uses,
            condition: details.condition,
            continue_on_error: details.continue_on_error,
            inputs: details.inputs,
            environment: details.environment,
        }
    }

    pub fn file(&self) -> &TextFile {
        &self.file
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn job(&self) -> &str {
        &self.job
    }

    pub fn id(&self) -> Option<&str> {
        self.id.map(|range| text(&self.file, range))
    }

    pub fn name(&self) -> Option<&str> {
        self.name.map(|range| text(&self.file, range))
    }

    pub fn run(&self) -> Option<SourceRange> {
        self.run
    }

    pub fn uses(&self) -> Option<SourceRange> {
        self.uses
    }

    pub fn condition(&self) -> Option<SourceRange> {
        self.condition
    }

    pub fn continue_on_error(&self) -> Option<SourceRange> {
        self.continue_on_error
    }

    pub fn inputs(&self) -> &[Setting] {
        &self.inputs
    }

    pub fn environment(&self) -> &[Setting] {
        &self.environment
    }
}

impl ExpressionFact {
    pub(crate) fn new(file: TextFile, range: SourceRange, context: String) -> Self {
        Self {
            file,
            range,
            context,
        }
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn body(&self) -> &str {
        unbraced(self.file.slice(self.range))
    }

    pub fn context(&self) -> &str {
        &self.context
    }
}

impl CredentialFact {
    pub(crate) fn new(range: SourceRange, key: String, job: String, literal: bool) -> Self {
        Self {
            range,
            key,
            job,
            literal,
        }
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn job(&self) -> &str {
        &self.job
    }

    pub fn is_literal(&self) -> bool {
        self.literal
    }
}

pub(crate) fn unbraced(interpolation: &str) -> &str {
    interpolation
        .strip_prefix("${{")
        .and_then(|body| body.strip_suffix("}}"))
        .unwrap_or(interpolation)
        .trim()
}

fn text(file: &TextFile, range: SourceRange) -> &str {
    file.slice(range).trim_matches(QUOTES)
}

fn is_commit(version: &str) -> bool {
    version.len() == COMMIT_LENGTH
        && version
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}
