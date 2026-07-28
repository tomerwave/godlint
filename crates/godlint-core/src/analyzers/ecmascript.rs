//! Function-node vocabulary shared by the ECMAScript-family grammars.
//!
//! The JavaScript, TypeScript, and TSX grammars name function nodes identically, so
//! both analyzers resolve their node kinds here to stay in lockstep.

pub(super) fn is_function_node(kind: &str) -> bool {
    matches!(
        kind,
        "arrow_function"
            | "function_declaration"
            | "function_expression"
            | "generator_function_declaration"
            | "method_definition"
    )
}
