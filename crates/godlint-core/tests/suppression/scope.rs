use super::support::{EMPTY_FUNCTION, across_files, surviving};

#[test]
fn an_enclosing_directive_resolves_to_the_innermost_function() {
    let directive =
        "        // godlint-ignore-enclosing maintainability/empty-function -- inner is a stub\n";
    let body = "fn outer() {\n    let inner = || {\nBODY    };\n    let _ = inner;\n}\n";

    assert_eq!(
        surviving("src/a.rs", &body.replace("BODY", ""), EMPTY_FUNCTION),
        vec![(2, 17)],
        "without a directive the closure is reported"
    );
    assert_eq!(
        surviving("src/b.rs", &body.replace("BODY", directive), EMPTY_FUNCTION),
        Vec::new(),
        "a directive inside the closure covers the closure"
    );
}

#[test]
fn an_enclosing_directive_resolves_to_a_late_nested_function() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  maintainability/parameter-count:\n",
        "    severity: error\n",
        "    max-parameters: 0\n",
    );
    let source = concat!(
        "fn outer(outer_value: u32) {\n",
        "    prepare_a_very_long_prefix_so_the_nested_declaration_is_near_the_end();\n",
        "    prepare_another_very_long_prefix_so_range_position_cannot_choose_the_outer_function();\n",
        "    let inner = |inner_value: u32| {\n",
        "        // godlint-ignore-enclosing maintainability/parameter-count -- callback signature\n",
        "        consume(inner_value);\n",
        "    };\n",
        "    let _ = inner;\n",
        "    consume(outer_value);\n",
        "}\n",
    );

    assert_eq!(
        surviving("src/example.rs", source, configuration),
        vec![(1, 1)],
        "only the late nested function is covered; the outer function still reports"
    );
}

#[test]
fn an_enclosing_directive_does_not_reach_a_nested_declaration() {
    let directive =
        "    // godlint-ignore-enclosing maintainability/empty-function -- outer is a stub\n";
    let body = "fn outer() {\nBODY    let inner = || {};\n    let _ = inner;\n}\n";
    let without = surviving("src/a.rs", &body.replace("BODY", ""), EMPTY_FUNCTION);
    let with = surviving("src/b.rs", &body.replace("BODY", directive), EMPTY_FUNCTION);

    assert_eq!(without.len(), 1, "the closure is reported: {without:?}");
    assert_eq!(
        with.len(),
        1,
        "a justification for the enclosing function does not describe a closure inside it: \
         {with:?}"
    );
}

#[test]
fn an_enclosing_directive_does_not_reach_a_neighbour_on_its_line() {
    let directive =
        " /* godlint-ignore-enclosing maintainability/empty-function -- a is a no-op */ ";
    let body = "export const a = (): void => {BODY}; export const b = (): void => {};\n";
    let without = surviving("src/a.ts", &body.replace("BODY", ""), EMPTY_FUNCTION);
    let with = surviving("src/b.ts", &body.replace("BODY", directive), EMPTY_FUNCTION);

    assert_eq!(without.len(), 2, "both arrows are reported: {without:?}");
    assert_eq!(
        with.len(),
        1,
        "b shares a line with a but is not the declaration a justified: {with:?}"
    );
}

#[test]
fn an_enclosing_directive_does_not_reach_a_comment_inside_a_nested_declaration() {
    let source = concat!(
        "fn outer() {\n",
        "    // godlint-ignore-enclosing style/no-comments -- outer is generated\n",
        "    let inner = || {\n",
        "        // reported, because it is inside the closure\n",
        "        1\n",
        "    };\n",
        "    let _ = inner;\n",
        "}\n"
    );
    let body = concat!(
        "version: 1\n",
        "rules:\n",
        "  style/no-comments:\n",
        "    severity: error\n",
        "    allow-doc-comments: false\n"
    );

    assert_eq!(
        surviving("src/example.rs", source, body).len(),
        1,
        "the exclusion is by range, so it covers any finding inside a nested declaration"
    );
}

#[test]
fn an_enclosing_directive_stops_where_its_declaration_ends() {
    let directive = "/* godlint-ignore-enclosing maintainability/empty-function -- a */";
    let body = format!("fn a() {{{directive}}}fn b() {{}}\n");

    assert_eq!(
        surviving("src/example.rs", &body, EMPTY_FUNCTION).len(),
        1,
        "b begins at the byte a ends on, and is a different declaration"
    );
}

#[test]
fn an_enclosing_directive_cannot_reach_a_file_level_finding() {
    let source = concat!(
        "fn a() {\n",
        "    // godlint-ignore-enclosing maintainability/file-size -- reaches the file?\n",
        "    work();\n",
        "}\n"
    );
    let body = concat!(
        "version: 1\n",
        "rules:\n",
        "  maintainability/file-size:\n",
        "    severity: error\n",
        "    max-lines: 1\n"
    );

    assert_eq!(
        surviving("src/example.rs", source, body).len(),
        1,
        "a file-level finding spans the whole file, so no declaration encloses it"
    );
}

#[test]
fn a_directive_covers_only_the_file_that_holds_it() {
    let directive = "// godlint-ignore-next-line maintainability/empty-function -- a stub\n";
    let stub = "fn example() {}\n";

    assert_eq!(
        surviving("src/plain.rs", stub, EMPTY_FUNCTION),
        vec![(1, 1)],
        "without a directive the empty function is reported"
    );
    assert_eq!(
        surviving(
            "src/covered.rs",
            &format!("{directive}{stub}"),
            EMPTY_FUNCTION
        ),
        Vec::new(),
        "the directive covers the function beneath it"
    );
    assert_eq!(
        across_files(
            ("src/covered.rs", &format!("{directive}{stub}")),
            ("src/other.rs", stub),
            EMPTY_FUNCTION
        ),
        vec![("src/other.rs".to_owned(), 1, 1)],
        "a directive in one file leaves an identical function in another file reported"
    );
}
