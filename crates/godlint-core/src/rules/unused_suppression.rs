use crate::{
    config::{Config, Severity, UnusedSuppressionRule},
    rules::{
        Finding, Rule, RuleError, Violation, configured_severity, is_suppressible_rule,
        when_configured,
    },
    source::SourceFile,
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
) -> Result<Vec<Finding>, RuleError> {
    when_configured(config.rules.unused_suppression.as_ref(), |configuration| {
        let severity = UnusedSuppression::severity(configuration);

        if severity == Severity::Off {
            return Ok(Vec::new());
        }

        suppressions
            .iter()
            .filter(|suppression| is_unused(suppression, findings, config))
            .map(|suppression| finding(suppression.source(), suppression, severity))
            .collect()
    })
}

fn is_unused(suppression: &Suppression, findings: &[Finding], config: &Config) -> bool {
    let has_enabled_rule = suppression.rules().iter().any(|rule_id| {
        is_suppressible_rule(rule_id) && configured_severity(config, rule_id) != Severity::Off
    });

    has_enabled_rule && !findings.iter().any(|finding| suppression.covers(finding))
}

fn finding(
    source: &SourceFile,
    suppression: &Suppression,
    severity: Severity,
) -> Result<Finding, RuleError> {
    let location = source
        .location(suppression.range())
        .map_err(|source| RuleError::LocatesSource { source })?;

    Ok(Finding {
        path: source.path().to_path_buf(),
        line: location.start.line,
        column: location.start.column,
        severity,
        rule_id: UnusedSuppression::ID,
        violation: Violation::UnusedSuppression,
    })
}
