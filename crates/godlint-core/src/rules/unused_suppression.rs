use std::{collections::BTreeMap, path::Path};

use crate::{
    config::{Config, Severity, UnusedSuppressionRule},
    rules::{Finding, Reporting, Rule, Violation, is_suppressible_rule, when_configured},
    suppression::Suppression,
};

pub struct UnusedSuppression;

impl Rule for UnusedSuppression {
    const ID: &'static str = "policy/unused-suppression";

    type Configuration = UnusedSuppressionRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(
    suppressions: &[Suppression],
    findings: &[Finding],
    config: &Config,
) -> Vec<Finding> {
    when_configured(config.rules.unused_suppression.as_ref(), |configuration| {
        let by_file = grouped_by_path(findings);

        super::report(
            Reporting::of::<UnusedSuppression>(configuration),
            suppressions
                .iter()
                .filter(|suppression| {
                    let listed = by_file
                        .get(suppression.source().path())
                        .map_or(&[][..], Vec::as_slice);

                    is_unused(suppression, listed)
                })
                .map(|suppression| {
                    (
                        suppression.source().text_file(),
                        suppression.range(),
                        Violation::UnusedSuppression,
                    )
                }),
        )
    })
}

fn grouped_by_path(findings: &[Finding]) -> BTreeMap<&Path, Vec<&Finding>> {
    let mut grouped: BTreeMap<&Path, Vec<&Finding>> = BTreeMap::new();

    for finding in findings {
        grouped
            .entry(finding.path.as_path())
            .or_default()
            .push(finding);
    }

    grouped
}

fn is_unused(suppression: &Suppression, findings: &[&Finding]) -> bool {
    let names_a_suppressible_rule = suppression
        .rules()
        .iter()
        .any(|rule_id| is_suppressible_rule(rule_id));

    suppression.resolves()
        && names_a_suppressible_rule
        && !findings.iter().any(|finding| suppression.covers(finding))
}
