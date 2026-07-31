use tree_sitter::{Node, Parser};

use crate::{
    analyzers::AnalyzerError,
    facts::{ActionFact, JobFact},
    source::{SourceRange, TextFile},
};

const MAPPING: &str = "block_mapping";
const PAIR: &str = "block_mapping_pair";
const NESTED: [&str; 3] = ["stream", "document", "block_node"];

const CONCURRENCY: &str = "concurrency";
const JOBS: &str = "jobs";
const PERMISSIONS: &str = "permissions";
const USES: &str = "uses";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowFacts {
    file: TextFile,
    unparsed: Vec<SourceRange>,
    actions: Vec<ActionFact>,
    jobs: Vec<JobFact>,
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

    pub fn declares_permissions(&self) -> bool {
        self.declares_permissions
    }

    pub fn declares_concurrency(&self) -> bool {
        self.declares_concurrency
    }
}

pub fn read(file: &TextFile) -> Result<WorkflowFacts, AnalyzerError> {
    let tree = parse(file)?;
    let workflow = mapping(tree.root_node());

    Ok(WorkflowFacts {
        file: file.clone(),
        unparsed: unparsed(tree.root_node(), file)?,
        actions: actions(tree.root_node(), file)?,
        jobs: jobs(workflow, file)?,
        declares_permissions: declared(workflow, PERMISSIONS, file),
        declares_concurrency: declared(workflow, CONCURRENCY, file),
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

fn unparsed(root: Node<'_>, file: &TextFile) -> Result<Vec<SourceRange>, AnalyzerError> {
    let mut torn = Vec::new();

    collect_torn(root, &mut torn);

    torn.into_iter().map(|node| range(node, file)).collect()
}

fn collect_torn<'tree>(node: Node<'tree>, torn: &mut Vec<Node<'tree>>) {
    if node.is_error() || node.is_missing() {
        torn.push(node);

        return;
    }

    if !node.has_error() {
        return;
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        collect_torn(child, torn);
    }
}

fn actions(root: Node<'_>, file: &TextFile) -> Result<Vec<ActionFact>, AnalyzerError> {
    let mut used = Vec::new();

    collect_used(root, file, &mut used);

    used.into_iter()
        .map(|node| Ok(ActionFact::new(file.clone(), range(node, file)?)))
        .collect()
}

fn collect_used<'tree>(node: Node<'tree>, file: &TextFile, found: &mut Vec<Node<'tree>>) {
    if node.kind() == PAIR && key_of(node, file) == Some(USES) {
        found.extend(node.child_by_field_name("value"));
    }

    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        collect_used(child, file, found);
    }
}

fn jobs(workflow: Option<Node<'_>>, file: &TextFile) -> Result<Vec<JobFact>, AnalyzerError> {
    let Some(listed) = value_of(workflow, JOBS, file) else {
        return Ok(Vec::new());
    };
    let mut jobs = Vec::new();

    for pair in pairs(mapping(listed)) {
        let (Some(name), Some(key)) = (key_of(pair, file), pair.child_by_field_name("key")) else {
            continue;
        };
        let body = pair.child_by_field_name("value").and_then(mapping);

        jobs.push(JobFact::new(
            file.clone(),
            range(key, file)?,
            name.to_owned(),
            declared(body, PERMISSIONS, file),
        ));
    }

    Ok(jobs)
}

fn mapping(node: Node<'_>) -> Option<Node<'_>> {
    let mut current = node;

    while NESTED.contains(&current.kind()) {
        current = first_named(current)?;
    }

    (current.kind() == MAPPING).then_some(current)
}

fn first_named(node: Node<'_>) -> Option<Node<'_>> {
    let mut cursor = node.walk();

    node.named_children(&mut cursor)
        .find(|child| !child.is_extra())
}

fn pairs(mapping: Option<Node<'_>>) -> Vec<Node<'_>> {
    let Some(mapping) = mapping else {
        return Vec::new();
    };
    let mut cursor = mapping.walk();

    mapping
        .named_children(&mut cursor)
        .filter(|child| child.kind() == PAIR)
        .collect()
}

fn value_of<'tree>(
    mapping: Option<Node<'tree>>,
    key: &str,
    file: &TextFile,
) -> Option<Node<'tree>> {
    pairs(mapping)
        .into_iter()
        .find(|pair| key_of(*pair, file) == Some(key))
        .and_then(|pair| pair.child_by_field_name("value"))
}

fn declared(mapping: Option<Node<'_>>, key: &str, file: &TextFile) -> bool {
    pairs(mapping)
        .iter()
        .any(|pair| key_of(*pair, file) == Some(key))
}

fn key_of<'text>(pair: Node<'_>, file: &'text TextFile) -> Option<&'text str> {
    let key = pair.child_by_field_name("key")?;

    file.text().get(key.byte_range()).map(str::trim)
}

fn range(node: Node<'_>, file: &TextFile) -> Result<SourceRange, AnalyzerError> {
    file.range(node.start_byte(), node.end_byte())
        .map_err(|source| AnalyzerError::InvalidRange {
            path: file.path().to_path_buf(),
            source,
        })
}
