/*
godlint-ignore-next-line maintainability/function-nesting owner=tomer expires=2999-12-31 -- reaches past the closing delimiter, #486
*/
fn wrapped(flag: bool) {
    if flag {
        if flag {
            let _ = flag;
        }
    }
}
