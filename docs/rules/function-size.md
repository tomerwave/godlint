# `maintainability/function-size`

This rule reports functions whose effective line count exceeds `max-lines`.

The count uses the whole function range, including its declaration and delimiters.
Lines with code count even when they also contain a comment. `skip-blank-lines` excludes
whitespace-only lines. `skip-comments` excludes comment-only lines using `//` and block
comments for Rust and TypeScript/JavaScript, and `#` for Python. Python docstrings are
code, not comments.

The rule is currently evaluated against shared `FunctionFact` values. Language parsers
and CLI reporting will be connected in later slices.
