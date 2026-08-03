use tree_sitter::Node;

use crate::{
    analyzers::AnalyzerError,
    facts::{ActionFact, CredentialFact, ExpressionFact, JobFact, Secrets, Setting, StepFact},
    source::{SourceRange, TextFile},
};

use crate::facts::workflow::{JobFactDetails, StepFactDetails, unbraced};

use super::syntax::{
    content, declared, first_pair, key_of, mapping, node_text, pairs, range, sequence_items,
    value_of,
};

const COMMENT: &str = "comment";
const CONTAINER: &str = "container";
const CREDENTIALS: &str = "credentials";
const CONTINUE_ON_ERROR: &str = "continue-on-error";
const ENV: &str = "env";
const IF: &str = "if";
const ID: &str = "id";
const NAME: &str = "name";
const NEEDS: &str = "needs";
const PASSWORD: &str = "password";
const PERMISSIONS: &str = "permissions";
const RUN: &str = "run";
const SCALARS: [&str; 4] = [
    "block_scalar",
    "double_quote_scalar",
    "plain_scalar",
    "single_quote_scalar",
];
const SECRETS: &str = "secrets";
const SERVICES: &str = "services";
const STEPS: &str = "steps";
const USERNAME: &str = "username";
const USES: &str = "uses";
const WITH: &str = "with";

pub(super) struct JobCollection {
    pub jobs: Vec<JobFact>,
    pub steps: Vec<StepFact>,
    pub credentials: Vec<CredentialFact>,
}

struct JobNodes<'tree> {
    name: String,
    key: Node<'tree>,
    value: Option<Node<'tree>>,
    body: Option<Node<'tree>>,
}

struct JobLinks {
    needs: Vec<Setting>,
    secrets: Option<Secrets>,
    calls_workflow: Option<SourceRange>,
}

struct StepSites {
    range: SourceRange,
    id: Option<SourceRange>,
    name: Option<SourceRange>,
    run: Option<SourceRange>,
    uses: Option<SourceRange>,
    condition: Option<SourceRange>,
    continue_on_error: Option<SourceRange>,
}

struct StepSettings {
    inputs: Vec<Setting>,
    environment: Vec<Setting>,
}

struct StepPolicySites {
    id: Option<SourceRange>,
    name: Option<SourceRange>,
    continue_on_error: Option<SourceRange>,
}

pub(super) fn jobs(
    listed: Option<Node<'_>>,
    file: &TextFile,
) -> Result<JobCollection, AnalyzerError> {
    let mut collected = JobCollection {
        jobs: Vec::new(),
        steps: Vec::new(),
        credentials: Vec::new(),
    };

    for pair in pairs(listed.and_then(mapping)) {
        collect_job(pair, file, &mut collected)?;
    }

    Ok(collected)
}

fn collect_job(
    pair: Node<'_>,
    file: &TextFile,
    collected: &mut JobCollection,
) -> Result<(), AnalyzerError> {
    let (Some(name), Some(key)) = (key_of(pair, file), pair.child_by_field_name("key")) else {
        return Ok(());
    };
    let value = pair.child_by_field_name("value");
    let body = value.and_then(mapping);
    let steps = collect_steps(name, value_of(body, STEPS, file), file)?;
    let credentials = collect_credentials(name, body, file)?;
    let nodes = JobNodes {
        name: name.to_owned(),
        key,
        value,
        body,
    };
    let details = job_details(&nodes, steps.len(), file)?;

    collected
        .jobs
        .push(JobFact::from_details(file.clone(), details));
    collected.steps.extend(steps);
    collected.credentials.extend(credentials);

    Ok(())
}

fn job_details(
    nodes: &JobNodes<'_>,
    step_count: usize,
    file: &TextFile,
) -> Result<JobFactDetails, AnalyzerError> {
    let (range, body) = job_ranges(nodes, file)?;
    let links = job_links(nodes.body, file)?;

    Ok(JobFactDetails {
        range,
        name: nodes.name.clone(),
        body,
        condition: optional_range(value_of(nodes.body, IF, file), file)?,
        continue_on_error: optional_range(value_of(nodes.body, CONTINUE_ON_ERROR, file), file)?,
        declares_permissions: declared(nodes.body, PERMISSIONS, file),
        needs: links.needs,
        secrets: links.secrets,
        calls_workflow: links.calls_workflow,
        step_count,
    })
}

fn job_ranges(
    nodes: &JobNodes<'_>,
    file: &TextFile,
) -> Result<(SourceRange, SourceRange), AnalyzerError> {
    Ok((
        range(nodes.key, file)?,
        range(nodes.value.unwrap_or(nodes.key), file)?,
    ))
}

fn job_links(body: Option<Node<'_>>, file: &TextFile) -> Result<JobLinks, AnalyzerError> {
    Ok(JobLinks {
        needs: needs(value_of(body, NEEDS, file), file)?,
        secrets: secrets(value_of(body, SECRETS, file), file)?,
        calls_workflow: optional_range(value_of(body, USES, file), file)?,
    })
}

fn collect_steps(
    job: &str,
    listed: Option<Node<'_>>,
    file: &TextFile,
) -> Result<Vec<StepFact>, AnalyzerError> {
    let mut steps = Vec::new();

    for item in listed.into_iter().flat_map(sequence_items) {
        if let Some(step) = step(job, mapping(item), file)? {
            steps.push(step);
        }
    }

    Ok(steps)
}

fn step(
    job: &str,
    body: Option<Node<'_>>,
    file: &TextFile,
) -> Result<Option<StepFact>, AnalyzerError> {
    let Some(first) = first_pair(body) else {
        return Ok(None);
    };
    let Some(key) = first.child_by_field_name("key") else {
        return Ok(None);
    };
    let sites = step_sites(key, body, file)?;
    let settings = step_settings(body, file)?;
    let details = StepFactDetails {
        range: sites.range,
        job: job.to_owned(),
        id: sites.id,
        name: sites.name,
        run: sites.run,
        uses: sites.uses,
        condition: sites.condition,
        continue_on_error: sites.continue_on_error,
        inputs: settings.inputs,
        environment: settings.environment,
    };

    Ok(Some(StepFact::new(file.clone(), details)))
}

fn step_sites(
    key: Node<'_>,
    body: Option<Node<'_>>,
    file: &TextFile,
) -> Result<StepSites, AnalyzerError> {
    let policy = step_policy_sites(body, file)?;
    let (uses, condition) = step_action_sites(body, file)?;

    Ok(StepSites {
        range: range(key, file)?,
        id: policy.id,
        name: policy.name,
        run: value_of(body, RUN, file)
            .map(|node| script_range(node, file))
            .transpose()?,
        uses,
        condition,
        continue_on_error: policy.continue_on_error,
    })
}

fn script_range(node: Node<'_>, file: &TextFile) -> Result<SourceRange, AnalyzerError> {
    let node = content(node).unwrap_or(node);
    let bytes = node.byte_range();
    let (start, end) = match node.kind() {
        "block_scalar" => {
            let text = &file.text()[bytes.clone()];
            let content = text.find('\n').map_or(text.len(), |line_end| {
                line_end
                    + 1
                    + text[line_end + 1..]
                        .bytes()
                        .take_while(|byte| matches!(byte, b' ' | b'\t'))
                        .count()
            });
            let start = bytes.start + content;
            let end = file.text().as_bytes()[..bytes.end]
                .strip_suffix(b"\n")
                .map_or(bytes.end, |without_newline| {
                    without_newline
                        .strip_suffix(b"\r")
                        .map_or(bytes.end - 1, |without_return| without_return.len())
                });
            (start, end.max(start))
        }
        "double_quote_scalar" | "single_quote_scalar" => (bytes.start + 1, bytes.end - 1),
        _ => (bytes.start, bytes.end),
    };

    file.range(start, end)
        .map_err(|source| AnalyzerError::InvalidRange {
            path: file.path().to_path_buf(),
            source,
        })
}

fn step_policy_sites(
    body: Option<Node<'_>>,
    file: &TextFile,
) -> Result<StepPolicySites, AnalyzerError> {
    Ok(StepPolicySites {
        id: optional_range(value_of(body, ID, file), file)?,
        name: optional_range(value_of(body, NAME, file), file)?,
        continue_on_error: optional_range(value_of(body, CONTINUE_ON_ERROR, file), file)?,
    })
}

fn step_action_sites(
    body: Option<Node<'_>>,
    file: &TextFile,
) -> Result<(Option<SourceRange>, Option<SourceRange>), AnalyzerError> {
    Ok((
        optional_range(value_of(body, USES, file), file)?,
        optional_range(value_of(body, IF, file), file)?,
    ))
}

fn step_settings(body: Option<Node<'_>>, file: &TextFile) -> Result<StepSettings, AnalyzerError> {
    Ok(StepSettings {
        inputs: settings(value_of(body, WITH, file), file)?,
        environment: settings(value_of(body, ENV, file), file)?,
    })
}

fn needs(node: Option<Node<'_>>, file: &TextFile) -> Result<Vec<Setting>, AnalyzerError> {
    let Some(node) = node else {
        return Ok(Vec::new());
    };

    sequence_items(node)
        .into_iter()
        .map(|item| setting(node_text(item, file), item, file))
        .collect()
}

fn secrets(node: Option<Node<'_>>, file: &TextFile) -> Result<Option<Secrets>, AnalyzerError> {
    let Some(node) = node else {
        return Ok(None);
    };

    if mapping(node).is_some() {
        return settings(Some(node), file).map(Secrets::Named).map(Some);
    }

    Ok((node_text(node, file) == "inherit")
        .then(|| range(node, file))
        .transpose()?
        .map(|range| Secrets::Inherit { range }))
}

fn settings(node: Option<Node<'_>>, file: &TextFile) -> Result<Vec<Setting>, AnalyzerError> {
    pairs(node.and_then(mapping))
        .into_iter()
        .filter_map(|pair| setting_pair(pair, file).transpose())
        .collect()
}

fn setting_pair(pair: Node<'_>, file: &TextFile) -> Result<Option<Setting>, AnalyzerError> {
    let (Some(key), Some(value)) = (key_of(pair, file), pair.child_by_field_name("value")) else {
        return Ok(None);
    };

    setting(key, value, file).map(Some)
}

fn setting(key: &str, node: Node<'_>, file: &TextFile) -> Result<Setting, AnalyzerError> {
    Ok(Setting::new(
        file.clone(),
        key.to_owned(),
        range(node, file)?,
    ))
}

fn collect_credentials(
    job: &str,
    body: Option<Node<'_>>,
    file: &TextFile,
) -> Result<Vec<CredentialFact>, AnalyzerError> {
    let mut found = credentials_in(job, value_of(body, CONTAINER, file), file)?;
    let services = value_of(body, SERVICES, file).and_then(mapping);

    for service in pairs(services) {
        let service_body = service.child_by_field_name("value");
        found.extend(credentials_in(job, service_body, file)?);
    }

    Ok(found)
}

fn credentials_in(
    job: &str,
    holder: Option<Node<'_>>,
    file: &TextFile,
) -> Result<Vec<CredentialFact>, AnalyzerError> {
    let body = holder.and_then(mapping);
    let listed = value_of(body, CREDENTIALS, file).and_then(mapping);

    pairs(listed)
        .into_iter()
        .filter(|pair| credential_key(*pair, file).is_some())
        .filter_map(|pair| credential(job, pair, file).transpose())
        .collect()
}

fn credential(
    job: &str,
    pair: Node<'_>,
    file: &TextFile,
) -> Result<Option<CredentialFact>, AnalyzerError> {
    let (Some(key), Some(value)) = (
        credential_key(pair, file),
        pair.child_by_field_name("value"),
    ) else {
        return Ok(None);
    };
    let literal = !file.text()[value.byte_range()].contains("${{");

    Ok(Some(CredentialFact::new(
        range(value, file)?,
        key.to_owned(),
        job.to_owned(),
        literal,
    )))
}

fn credential_key<'text>(pair: Node<'_>, file: &'text TextFile) -> Option<&'text str> {
    key_of(pair, file).filter(|key| matches!(*key, USERNAME | PASSWORD))
}

pub(super) fn actions(root: Node<'_>, file: &TextFile) -> Result<Vec<ActionFact>, AnalyzerError> {
    let mut found = Vec::new();

    collect_used(root, file, &mut found);
    found
        .into_iter()
        .map(|node| Ok(ActionFact::new(file.clone(), range(node, file)?)))
        .collect()
}

fn collect_used<'tree>(node: Node<'tree>, file: &TextFile, found: &mut Vec<Node<'tree>>) {
    if key_of(node, file) == Some(USES) {
        found.extend(node.child_by_field_name("value"));
    }

    walk(node, |child| collect_used(child, file, found));
}

pub(super) fn expressions(
    root: Node<'_>,
    file: &TextFile,
) -> Result<Vec<ExpressionFact>, AnalyzerError> {
    let mut found = Vec::new();

    collect_expressions(root, file, &mut found)?;
    Ok(found)
}

fn collect_expressions(
    node: Node<'_>,
    file: &TextFile,
    found: &mut Vec<ExpressionFact>,
) -> Result<(), AnalyzerError> {
    if SCALARS.contains(&node.kind()) {
        return expressions_in(node, file, found);
    }
    if matches!(node.kind(), "block_mapping_pair" | "flow_pair") {
        return node
            .child_by_field_name("value")
            .map_or(Ok(()), |value| collect_expressions(value, file, found));
    }

    let mut result = Ok(());
    walk(node, |child| {
        if result.is_ok() {
            result = collect_expressions(child, file, found);
        }
    });
    result
}

fn expressions_in(
    scalar: Node<'_>,
    file: &TextFile,
    found: &mut Vec<ExpressionFact>,
) -> Result<(), AnalyzerError> {
    let text = &file.text()[scalar.byte_range()];
    let mut offset = 0;

    while let Some(start) = text[offset..].find("${{").map(|start| start + offset) {
        let Some(end) = text[start + 3..].find("}}").map(|end| end + start + 5) else {
            break;
        };
        let expression = file
            .range(scalar.start_byte() + start, scalar.start_byte() + end)
            .map_err(|source| AnalyzerError::InvalidRange {
                path: file.path().to_path_buf(),
                source,
            })?;
        let body = unbraced(file.slice(expression));

        found.push(ExpressionFact::new(file.clone(), expression, context(body)));
        offset = end;
    }

    Ok(())
}

fn context(body: &str) -> String {
    let length = body
        .char_indices()
        .take_while(|(_, character)| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '.')
        })
        .map(|(index, character)| index + character.len_utf8())
        .last()
        .unwrap_or(0);
    let path = &body[..length];

    if path.is_empty() || body[length..].starts_with('(') {
        return body.to_owned();
    }

    path.to_ascii_lowercase()
}

pub(super) fn comments(root: Node<'_>, file: &TextFile) -> Result<Vec<SourceRange>, AnalyzerError> {
    let mut found = Vec::new();

    collect_nodes(root, COMMENT, &mut found);
    found.into_iter().map(|node| range(node, file)).collect()
}

pub(super) fn unparsed(root: Node<'_>, file: &TextFile) -> Result<Vec<SourceRange>, AnalyzerError> {
    let mut found = Vec::new();

    collect_torn(root, &mut found);
    found.into_iter().map(|node| range(node, file)).collect()
}

fn collect_torn<'tree>(node: Node<'tree>, found: &mut Vec<Node<'tree>>) {
    if node.is_error() || node.is_missing() {
        found.push(node);
    } else if node.has_error() {
        walk(node, |child| collect_torn(child, found));
    }
}

fn collect_nodes<'tree>(node: Node<'tree>, kind: &str, found: &mut Vec<Node<'tree>>) {
    if node.kind() == kind {
        found.push(node);
    }

    walk(node, |child| collect_nodes(child, kind, found));
}

fn optional_range(
    node: Option<Node<'_>>,
    file: &TextFile,
) -> Result<Option<SourceRange>, AnalyzerError> {
    node.map(|node| range(node, file)).transpose()
}

fn walk<'tree>(node: Node<'tree>, mut visit: impl FnMut(Node<'tree>)) {
    let mut cursor = node.walk();

    for child in node.children(&mut cursor) {
        visit(child);
    }
}
