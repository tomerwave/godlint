use crate::{
    config::{Config, Severity, UnusedSuppressionRule},
    rules::{
        Finding, Reporting, Rule, Violation, configured_severity, is_suppressible_rule,
        when_configured,
    },
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
        let reporting = Reporting::of::<UnusedSuppression>(configuration);

        if reporting.severity == Severity::Off {
            return Vec::new();
        }

        suppressions
            .iter()
            .filter(|suppression| is_unused(suppression, findings, config))
            .map(|suppression| {
                super::finding(
                    suppression.source(),
                    suppression.range(),
                    reporting,
                    Violation::UnusedSuppression,
                )
            })
            .collect()
    })
}

fn is_unused(suppression: &Suppression, findings: &[Finding], config: &Config) -> bool {
    let has_enabled_rule = suppression.rules().iter().any(|rule_id| {
        is_suppressible_rule(rule_id) && configured_severity(config, rule_id) != Severity::Off
    });

    suppression.resolves()
        && has_enabled_rule
        && !findings.iter().any(|finding| suppression.covers(finding))
}
