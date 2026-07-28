fn host() {
    let inner = || {
        // godlint-ignore-enclosing maintainability/empty-function owner=tomer expires=2999-12-31 -- the closure is the deliberate no-op, #35
    };
    let _ = inner;
}
