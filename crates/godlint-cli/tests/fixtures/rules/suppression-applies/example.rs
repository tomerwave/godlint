// godlint-ignore-next-line maintainability/function-nesting owner=tomer expires=2999-12-31 -- flattening in #482
fn nested(flag: bool) {
    if flag {
        if flag {
            let _ = flag;
        }
    }
}

fn blank() {
    // godlint-ignore-enclosing maintainability/empty-function owner=tomer expires=2999-12-31 -- awaiting #483
}
