use godlint_core::{
    config::{FunctionNestingRule, Severity},
    rules::{Metric, Rule, Violation, function_nesting::FunctionNesting},
};

use super::support::{function, function_limits, nth_function};

fn configuration(max_depth: u32) -> FunctionNestingRule {
    FunctionNestingRule {
        severity: Severity::Error,
        only_in: Vec::new(),
        allow_in: Vec::new(),
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
fn an_ordinary_branch_inside_else_still_adds_a_level() {
    for (path, source) in [
        (
            "src/example.js",
            "function example() {\n  if (a) {\n    run();\n  } else {\n    while (b) run();\n  }\n}",
        ),
        (
            "src/example.py",
            "def example():\n    if a:\n        run()\n    else:\n        while b:\n            run()",
        ),
    ] {
        assert_eq!(
            depth(path, source),
            2,
            "only an else-if stays flat; a loop inside else remains nested: {path}"
        );
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
    assert_eq!(
        function_limits::<FunctionNesting>(
            "src/example.rs",
            "fn example() {\n    if a {\n        if b {\n            run();\n        }\n    }\n}",
            &configuration(1),
        ),
        vec![Violation::limit(Metric::BlockDepth, 2, 1)]
    );
}

#[test]
fn accepts_a_function_at_its_limit() {
    assert!(
        function_limits::<FunctionNesting>(
            "src/example.rs",
            "fn example() {\n    if a {\n        if b {\n            run();\n        }\n    }\n}",
            &configuration(2),
        )
        .is_empty()
    );
}

const NESTED_IN_ELSE: [(&str, &str); 3] = [
    (
        "src/example.js",
        "function classify(a, items) {\n  if (a) {\n    return 1;\n  } else {\n    for (const item of items) {\n      if (item) {\n        return 2;\n      }\n    }\n  }\n  return 0;\n}\n",
    ),
    (
        "src/example.rs",
        "fn classify(a: bool, items: &[bool]) -> u32 {\n    if a {\n        return 1;\n    } else {\n        for item in items {\n            if *item {\n                return 2;\n            }\n        }\n    }\n    0\n}\n",
    ),
    (
        "src/example.py",
        "def classify(a, items):\n    if a:\n        return 1\n    else:\n        for item in items:\n            if item:\n                return 2\n    return 0\n",
    ),
];

const CHAINED_ELSE_IF: [(&str, &str); 3] = [
    (
        "src/chain.js",
        "function classify(a, b) {\n  if (a) {\n    return 1;\n  } else if (b) {\n    return 2;\n  }\n  return 0;\n}\n",
    ),
    (
        "src/chain.rs",
        "fn classify(a: bool, b: bool) -> u32 {\n    if a {\n        return 1;\n    } else if b {\n        return 2;\n    }\n    0\n}\n",
    ),
    (
        "src/chain.py",
        "def classify(a, b):\n    if a:\n        return 1\n    elif b:\n        return 2\n    return 0\n",
    ),
];

#[test]
fn a_loop_inside_an_else_nests() {
    for (path, source) in NESTED_IN_ELSE {
        assert_eq!(
            depth(path, source),
            3,
            "the else, the loop inside it and the loop's own branch each nest: {path}"
        );
    }
}

#[test]
fn an_else_if_continues_the_chain_rather_than_nesting() {
    for (path, source) in CHAINED_ELSE_IF {
        assert_eq!(
            depth(path, source),
            1,
            "an else-if is the same decision continued, not a level deeper: {path}"
        );
    }
}

#[test]
fn a_braceless_else_holding_a_loop_still_nests() {
    let source = concat!(
        "function classify(a, items) {\n",
        "  if (a) return 1;\n",
        "  else for (const item of items) {\n",
        "    if (item) return 2;\n",
        "  }\n",
        "  return 0;\n",
        "}\n",
    );

    assert_eq!(
        depth("src/braceless.js", source),
        3,
        "the loop is the else itself rather than a block inside it, and it still opens a level \
         — only an else-if is the same decision continued"
    );
}

#[test]
fn a_curried_function_nests_in_the_closure_that_holds_the_blocks() {
    let source = "const curried = a => b => {\n    if (a) {\n        if (b) {\n            return 1;\n        }\n    }\n    return 0;\n};\n";

    assert_eq!(
        nth_function("src/curried.ts", source, 0)
            .1
            .block_depth()
            .value(),
        0
    );
    assert_eq!(
        nth_function("src/curried.ts", source, 1)
            .1
            .block_depth()
            .value(),
        2
    );
}

#[test]
fn a_block_in_a_parameter_does_not_nest_the_function() {
    let depth = depth(
        "src/parameter.rs",
        "fn example(value: [u8; { if SIZE > 0 { 1 } else { 2 } }]) {\n    run(value);\n}",
    );

    assert_eq!(depth, 0);
}
