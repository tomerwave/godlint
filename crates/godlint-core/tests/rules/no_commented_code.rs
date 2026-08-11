use super::support::rule_violations;
use godlint_core::rules::{Violation, no_commented_code};

#[test]
fn reports_disabled_return_comment() {
    let config = "version: 1\nrules:\n  style/no-commented-code:\n    severity: error\n";
    assert!(matches!(
        rule_violations(
            no_commented_code::evaluate,
            "x.py",
            "# return value\n",
            config
        )
        .as_slice(),
        [Violation::CommentedCode]
    ));
}

#[test]
fn ignores_directives_and_shebangs() {
    let config = "version: 1\nrules:\n  style/no-commented-code:\n    severity: error\n";
    assert!(
        rule_violations(
            no_commented_code::evaluate,
            "x.py",
            "#!/usr/bin/env python\n# godlint-ignore-next-line style/no-comments\n",
            config
        )
        .is_empty()
    );
}
