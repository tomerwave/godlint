use godlint_core::{
    config::{FunctionNestingRule, Severity},
    rules::{FunctionRule, Metric, Rule, Violation, function_nesting::FunctionNesting},
};

use super::support::function;

fn configuration(max_depth: u32) -> FunctionNestingRule {
    FunctionNestingRule {
        severity: Severity::Error,
        max_depth,
    }
}

fn depth(path: &str, source: &str) -> u32 {
    function(path, source).1.block_depth().value()
}

#[test]
fn measures_nesting_inside_the_function() {
    assert_eq!(FunctionNesting::ID, "maintainability/function-nesting");
    assert_eq!(depth("src/flat.rs", "fn example() {\n    run();\n}"), 0);
    assert_eq!(
        depth(
            "src/one.rs",
            "fn example() {\n    if a {\n        run();\n    }\n}"
        ),
        1
    );
    assert_eq!(
        depth(
            "src/three.rs",
            "fn example() {\n    if a {\n        for b in c {\n            while d {\n                run();\n            }\n        }\n    }\n}"
        ),
        3
    );
}

#[test]
fn measures_the_same_depth_in_every_language() {
    let cases = [
        (
            "src/example.rs",
            "fn example() {\n    if a {\n        for b in c {\n            run();\n        }\n    }\n}",
        ),
        (
            "src/example.ts",
            "function example() {\n  if (a) {\n    for (const b of c) {\n      run();\n    }\n  }\n}",
        ),
        (
            "src/example.js",
            "function example() {\n  if (a) {\n    for (const b of c) {\n      run();\n    }\n  }\n}",
        ),
        (
            "src/example.py",
            "def example():\n    if a:\n        for b in c:\n            run()",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(depth(path, source), 2, "{path}");
    }
}

#[test]
fn reads_an_else_if_chain_as_one_level() {
    let cases = [
        (
            "src/example.rs",
            "fn example() {\n    if a {\n    } else if b {\n    } else if c {\n        run();\n    }\n}",
        ),
        (
            "src/example.ts",
            "function example() {\n  if (a) {\n  } else if (b) {\n  } else if (c) {\n    run();\n  }\n}",
        ),
        (
            "src/example.py",
            "def example():\n    if a:\n        pass\n    elif b:\n        pass\n    elif c:\n        run()",
        ),
    ];

    for (path, source) in cases {
        assert_eq!(depth(path, source), 1, "{path}");
    }
}

#[test]
fn attributes_nesting_inside_a_closure_to_the_closure() {
    let (_, host) = function(
        "src/example.rs",
        "fn host() {\n    let f = || {\n        if a {\n            run();\n        }\n    };\n}",
    );

    assert_eq!(host.block_depth().value(), 0);
}

#[test]
fn reports_a_function_deeper_than_its_limit() {
    let (facts, deep) = function(
        "src/example.rs",
        "fn example() {\n    if a {\n        if b {\n            run();\n        }\n    }\n}",
    );

    assert_eq!(
        FunctionNesting::check(&deep, &facts, &configuration(1)),
        Some(Violation::Limit {
            metric: Metric::BlockDepth,
            actual: 2,
            max: 1
        })
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    let (facts, deep) = function(
        "src/example.rs",
        "fn example() {\n    if a {\n        if b {\n            run();\n        }\n    }\n}",
    );

    assert_eq!(
        FunctionNesting::check(&deep, &facts, &configuration(2)),
        None
    );
}
