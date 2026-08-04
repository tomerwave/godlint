use godlint_core::{
    config::{AccountableSuppressionRule, Severity},
    date::Date,
    rules::{
        Rule, SuppressionDefect, SuppressionRule, Violation,
        accountable_suppression::AccountableSuppression, is_suppressible_rule, rule_ids,
        unused_suppression::UnusedSuppression,
    },
};

use super::support::suppressions;

const TODAY: &str = "2026-07-28";

fn configuration(require_owner: bool, require_expiry: bool) -> AccountableSuppressionRule {
    AccountableSuppressionRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
        require_owner,
        require_expiry,
    }
}

fn today() -> Date {
    Date::parse(TODAY).unwrap_or_else(|error| panic!("parses {TODAY}: {error}"))
}

fn defects(source: &str, require_owner: bool, require_expiry: bool) -> Vec<SuppressionDefect> {
    suppressions("src/example.rs", source)
        .iter()
        .flat_map(|suppression| {
            AccountableSuppression::check(
                suppression,
                &configuration(require_owner, require_expiry),
                today(),
            )
        })
        .map(|violation| match violation {
            Violation::UnaccountableSuppression { defect } => defect,
            other => panic!("unexpected violation: {other:?}"),
        })
        .collect()
}

fn enclosing(directive: &str) -> Vec<SuppressionDefect> {
    defects(
        &format!("fn example() {{\n    // godlint-ignore-enclosing {directive}\n}}\n"),
        false,
        false,
    )
}

#[test]
fn an_accountable_suppression_has_no_defects() {
    assert_eq!(AccountableSuppression::ID, "policy/accountable-suppression");
    assert!(
        enclosing("maintainability/empty-function -- awaiting #482").is_empty(),
        "a justified suppression of a known rule is accountable"
    );
}

#[test]
fn requires_a_justification() {
    assert_eq!(
        enclosing("maintainability/empty-function"),
        vec![SuppressionDefect::MissingJustification]
    );
}

#[test]
fn treats_an_empty_justification_as_missing() {
    assert_eq!(
        enclosing("maintainability/empty-function --"),
        vec![SuppressionDefect::MissingJustification]
    );
}

#[test]
fn requires_at_least_one_rule() {
    assert_eq!(
        enclosing("-- nothing named"),
        vec![SuppressionDefect::NoRules]
    );
}

#[test]
fn reports_an_unknown_rule() {
    assert_eq!(
        enclosing("maintainability/no-such-rule -- typo"),
        vec![SuppressionDefect::UnknownRule {
            rule: "maintainability/no-such-rule".to_owned()
        }]
    );
}

#[test]
fn reports_every_unknown_rule_in_a_list() {
    assert_eq!(
        enclosing("one/two,maintainability/empty-function,three/four -- mixed").len(),
        2
    );
}

#[test]
fn refuses_to_suppress_the_rule_that_holds_suppressions_to_account() {
    assert!(!is_suppressible_rule(AccountableSuppression::ID));
    assert_eq!(
        enclosing("policy/accountable-suppression -- circular"),
        vec![SuppressionDefect::NotSuppressible {
            rule: "policy/accountable-suppression".to_owned()
        }]
    );
}

#[test]
fn refuses_to_suppress_the_rule_that_removes_stale_exceptions() {
    assert!(!is_suppressible_rule(UnusedSuppression::ID));
    assert_eq!(
        enclosing("policy/unused-suppression -- circular"),
        vec![SuppressionDefect::NotSuppressible {
            rule: "policy/unused-suppression".to_owned()
        }]
    );
}

#[test]
fn reports_an_unrecognised_option() {
    assert_eq!(
        enclosing("maintainability/empty-function ownr=tomer -- misspelt"),
        vec![SuppressionDefect::UnknownOption {
            option: "ownr".to_owned()
        }]
    );
}

#[test]
fn reports_a_stray_argument_as_an_unrecognised_option() {
    assert_eq!(
        enclosing("maintainability/empty-function stray -- extra word"),
        vec![SuppressionDefect::UnknownOption {
            option: "stray".to_owned()
        }]
    );
}

#[test]
fn reports_an_option_given_twice() {
    assert_eq!(
        enclosing(
            "maintainability/empty-function expires=2999-01-01 expires=2999-12-31 -- renewed"
        ),
        vec![SuppressionDefect::RepeatedOption {
            option: "expires".to_owned()
        }]
    );
    assert_eq!(
        enclosing("maintainability/empty-function owner=one owner=two -- two owners"),
        vec![SuppressionDefect::RepeatedOption {
            option: "owner".to_owned()
        }]
    );
}

#[test]
fn an_empty_option_value_is_absent_rather_than_satisfied() {
    let directive = "fn example() {\n    // godlint-ignore-enclosing \
                     maintainability/empty-function owner= -- blank\n}\n";

    assert_eq!(
        defects(directive, true, false),
        vec![SuppressionDefect::MissingOwner]
    );
}

#[test]
fn reports_an_expiry_that_is_not_a_calendar_date() {
    assert_eq!(
        enclosing("maintainability/empty-function expires=31-12-2999 -- day first"),
        vec![SuppressionDefect::InvalidExpiry {
            value: "31-12-2999".to_owned()
        }]
    );
    assert_eq!(
        enclosing("maintainability/empty-function expires=2999-02-30 -- no such day"),
        vec![SuppressionDefect::InvalidExpiry {
            value: "2999-02-30".to_owned()
        }]
    );
}

#[test]
fn reports_an_expiry_in_the_past() {
    assert_eq!(
        enclosing("maintainability/empty-function expires=2020-01-01 -- overdue"),
        vec![SuppressionDefect::Expired {
            expires: "2020-01-01".to_owned()
        }]
    );
}

#[test]
fn accepts_an_expiry_today_and_later() {
    assert!(
        enclosing(&format!(
            "maintainability/empty-function expires={TODAY} -- due today"
        ))
        .is_empty(),
        "an expiry is reported the day after it passes, not on the day itself"
    );
    assert!(enclosing("maintainability/empty-function expires=2999-12-31 -- distant").is_empty());
}

#[test]
fn requires_an_owner_only_when_configured() {
    let directive = "fn example() {\n    // godlint-ignore-enclosing \
                     maintainability/empty-function -- unowned\n}\n";

    assert!(defects(directive, false, false).is_empty());
    assert_eq!(
        defects(directive, true, false),
        vec![SuppressionDefect::MissingOwner]
    );
}

#[test]
fn requires_an_expiry_only_when_configured() {
    let directive = "fn example() {\n    // godlint-ignore-enclosing \
                     maintainability/empty-function -- undated\n}\n";

    assert!(defects(directive, false, false).is_empty());
    assert_eq!(
        defects(directive, false, true),
        vec![SuppressionDefect::MissingExpiry]
    );
}

#[test]
fn reports_an_enclosing_directive_with_nothing_to_enclose() {
    assert_eq!(
        defects(
            "// godlint-ignore-enclosing maintainability/empty-function -- detached\nfn a() {}\n",
            false,
            false
        ),
        vec![SuppressionDefect::Unresolved]
    );
}

#[test]
fn a_next_line_directive_always_resolves() {
    assert!(
        defects(
            "// godlint-ignore-next-line maintainability/empty-function -- above\nfn a() {}\n",
            false,
            false
        )
        .is_empty()
    );
}

#[test]
fn every_registered_rule_can_be_named_by_a_suppression() {
    let mut suppressible = 0;

    for identifier in rule_ids() {
        if !is_suppressible_rule(identifier) {
            continue;
        }

        suppressible += 1;

        assert!(
            enclosing(&format!("{identifier} -- registered")).is_empty(),
            "{identifier} is registered but a suppression cannot name it"
        );
    }

    assert!(
        suppressible > 0,
        "the registry lists no suppressible rule, so this test proves nothing"
    );
}
