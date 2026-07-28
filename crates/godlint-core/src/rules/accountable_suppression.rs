use crate::{
    config::{AccountableSuppressionRule, Config, Severity},
    date::Date,
    rules::{
        Finding, RULE_IDS, Rule, RuleError, SuppressionDefect, SuppressionRule, Violation,
        evaluate_suppression_rule, when_configured,
    },
    suppression::Suppression,
};

pub struct AccountableSuppression;

impl Rule for AccountableSuppression {
    const ID: &'static str = "policy/accountable-suppression";

    type Configuration = AccountableSuppressionRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl SuppressionRule for AccountableSuppression {
    fn check(
        suppression: &Suppression,
        configuration: &Self::Configuration,
        today: Date,
    ) -> Vec<Violation> {
        defects(suppression, configuration, today)
            .into_iter()
            .map(|defect| Violation::UnaccountableSuppression { defect })
            .collect()
    }
}

fn defects(
    suppression: &Suppression,
    configuration: &AccountableSuppressionRule,
    today: Date,
) -> Vec<SuppressionDefect> {
    let mut defects = named_rules(suppression);

    defects.extend(suppression.unknown_options().iter().map(|option| {
        SuppressionDefect::UnknownOption {
            option: option.clone(),
        }
    }));

    if suppression.justification().is_none() {
        defects.push(SuppressionDefect::MissingJustification);
    }

    if configuration.require_owner && suppression.owner().is_none() {
        defects.push(SuppressionDefect::MissingOwner);
    }

    defects.extend(expiry(suppression, configuration, today));

    if !suppression.resolves() {
        defects.push(SuppressionDefect::Unresolved);
    }

    defects
}

fn named_rules(suppression: &Suppression) -> Vec<SuppressionDefect> {
    if suppression.rules().is_empty() {
        return vec![SuppressionDefect::NoRules];
    }

    suppression
        .rules()
        .iter()
        .filter_map(|rule| named_rule(rule))
        .collect()
}

fn named_rule(rule: &str) -> Option<SuppressionDefect> {
    if !RULE_IDS.contains(&rule) {
        return Some(SuppressionDefect::UnknownRule {
            rule: rule.to_owned(),
        });
    }

    if !crate::rules::is_suppressible_rule(rule) {
        return Some(SuppressionDefect::NotSuppressible {
            rule: rule.to_owned(),
        });
    }

    None
}

fn expiry(
    suppression: &Suppression,
    configuration: &AccountableSuppressionRule,
    today: Date,
) -> Option<SuppressionDefect> {
    let Some(value) = suppression.expires() else {
        return configuration
            .require_expiry
            .then_some(SuppressionDefect::MissingExpiry);
    };

    let Ok(expires) = Date::parse(value) else {
        return Some(SuppressionDefect::InvalidExpiry {
            value: value.to_owned(),
        });
    };

    (expires < today).then(|| SuppressionDefect::Expired {
        expires: expires.to_string(),
    })
}

pub fn evaluate(
    suppressions: &[Suppression],
    config: &Config,
    today: Date,
) -> Result<Vec<Finding>, RuleError> {
    when_configured(
        config.rules.accountable_suppression.as_ref(),
        |configuration| {
            evaluate_suppression_rule::<AccountableSuppression>(suppressions, configuration, today)
        },
    )
}
