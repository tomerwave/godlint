fn no_reason() {
    // godlint-ignore-enclosing maintainability/empty-function owner=tomer expires=2999-12-31
}

fn no_rule() {
    // godlint-ignore-enclosing owner=tomer expires=2999-12-31 -- names nothing
}

fn unknown_rule() {
    // godlint-ignore-enclosing maintainability/no-such-rule owner=tomer expires=2999-12-31 -- typo
}

fn not_suppressible() {
    // godlint-ignore-enclosing policy/accountable-suppression owner=tomer expires=2999-12-31 -- circular
}

fn stale() {
    // godlint-ignore-enclosing maintainability/empty-function owner=tomer expires=2020-01-01 -- overdue
}

fn misdated() {
    // godlint-ignore-enclosing maintainability/empty-function owner=tomer expires=31-12-2999 -- day first
}

fn misspelt_option() {
    // godlint-ignore-enclosing maintainability/empty-function ownr=tomer expires=2999-12-31 -- bad key
}

fn anonymous() {
    // godlint-ignore-enclosing maintainability/empty-function expires=2999-12-31 -- no owner
}

fn undated() {
    // godlint-ignore-enclosing maintainability/empty-function owner=tomer -- no expiry
}

// godlint-ignore-enclosing maintainability/empty-function owner=tomer expires=2999-12-31 -- nothing encloses this
fn detached() {}
