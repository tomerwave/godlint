use std::collections::{BTreeMap, BTreeSet};

use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, Severity, StaleActionRefsRule},
    facts::ActionFact,
    glob,
    rules::{Finding, Languages, Reporting, Rule, Violation, report, when_configured},
};

pub struct StaleActionRefs;

impl Rule for StaleActionRefs {
    const ID: &'static str = "ci/stale-action-refs";

    const LANGUAGES: Languages = Languages::WORKFLOWS;

    type Configuration = StaleActionRefsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

struct Occurrence<'a> {
    action: &'a ActionFact,
    name: String,
    sha: String,
    label: Option<String>,
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.stale_action_refs.as_ref(), |configuration| {
        evaluate_configured(workflows, configuration)
    })
}

fn evaluate_configured(
    workflows: &[WorkflowFacts],
    configuration: &StaleActionRefsRule,
) -> Vec<Finding> {
    let occurrences = collect(workflows, configuration);
    let labels_by_pin = labels_by_pin(&occurrences);
    let pins_by_label = pins_by_label(&occurrences);

    report(
        Reporting::of::<StaleActionRefs>(configuration),
        occurrences.iter().flat_map(|occurrence| {
            violations(occurrence, &labels_by_pin, &pins_by_label).map(|violation| {
                (
                    occurrence.action.file(),
                    occurrence.action.range(),
                    violation,
                )
            })
        }),
    )
}

fn collect<'a>(
    workflows: &'a [WorkflowFacts],
    configuration: &StaleActionRefsRule,
) -> Vec<Occurrence<'a>> {
    workflows
        .iter()
        .filter(|workflow| {
            !glob::matches_any(
                configuration.allow_in.iter().map(String::as_str),
                workflow.file().path_text(),
            )
        })
        .flat_map(|workflow| {
            workflow
                .actions()
                .iter()
                .filter(|action| action.is_commit() && !action.is_local() && !action.is_container())
                .map(|action| Occurrence {
                    action,
                    name: action.name().to_ascii_lowercase(),
                    sha: action.version().unwrap_or_default().to_ascii_lowercase(),
                    label: trailing_label(workflow, action).map(str::to_ascii_lowercase),
                })
        })
        .collect()
}

fn trailing_label<'a>(workflow: &'a WorkflowFacts, action: &ActionFact) -> Option<&'a str> {
    let range = workflow.comments().iter().find(|comment| {
        action.range().end() <= comment.start()
            && workflow.file().line(action.range().end()) == workflow.file().line(comment.start())
    })?;
    let label = workflow.file().text()[range.start()..range.end()]
        .trim_start_matches('#')
        .trim();

    (!label.is_empty()).then_some(label)
}

fn labels_by_pin(occurrences: &[Occurrence<'_>]) -> BTreeMap<(String, String), BTreeSet<String>> {
    occurrences
        .iter()
        .filter_map(|occurrence| {
            occurrence.label.as_ref().map(|label| {
                (
                    (occurrence.name.clone(), occurrence.sha.clone()),
                    label.clone(),
                )
            })
        })
        .fold(BTreeMap::new(), |mut groups, (key, label)| {
            groups.entry(key).or_default().insert(label);
            groups
        })
}

fn pins_by_label(occurrences: &[Occurrence<'_>]) -> BTreeMap<(String, String), BTreeSet<String>> {
    occurrences
        .iter()
        .filter_map(|occurrence| {
            occurrence.label.as_ref().map(|label| {
                (
                    (occurrence.name.clone(), comparable_label(label).to_owned()),
                    occurrence.sha.clone(),
                )
            })
        })
        .fold(BTreeMap::new(), |mut groups, (key, sha)| {
            groups.entry(key).or_default().insert(sha);
            groups
        })
}

fn comparable_label(label: &str) -> &str {
    label.strip_prefix('v').unwrap_or(label)
}

fn violations(
    occurrence: &Occurrence<'_>,
    labels_by_pin: &BTreeMap<(String, String), BTreeSet<String>>,
    pins_by_label: &BTreeMap<(String, String), BTreeSet<String>>,
) -> impl Iterator<Item = Violation> {
    let mut violations = Vec::new();

    let Some(label) = occurrence.label.as_ref() else {
        violations.push(Violation::UnlabelledActionPin {
            action: occurrence.action.reference().to_owned(),
        });
        return violations.into_iter();
    };

    let labels = &labels_by_pin[&(occurrence.name.clone(), occurrence.sha.clone())];
    let comparable_labels = labels
        .iter()
        .map(|label| comparable_label(label))
        .collect::<BTreeSet<_>>();
    if comparable_labels.len() > 1 {
        violations.push(Violation::ContradictoryActionLabels {
            action: occurrence.name.clone(),
            sha: occurrence.sha.clone(),
            labels: labels.iter().cloned().collect(),
        });
    }

    let pins = &pins_by_label[&(occurrence.name.clone(), comparable_label(label).to_owned())];
    if pins.len() > 1 {
        violations.push(Violation::ContradictoryActionPins {
            action: occurrence.name.clone(),
            label: label.clone(),
            shas: pins.iter().cloned().collect(),
        });
    }

    violations.into_iter()
}
