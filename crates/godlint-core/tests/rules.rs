#![allow(clippy::expect_used, clippy::unwrap_used)]

#[path = "rules/support.rs"]
mod support;

#[path = "rules/accountable_suppression.rs"]
mod accountable_suppression;

#[path = "rules/cyclomatic_complexity.rs"]
mod cyclomatic_complexity;

#[path = "rules/errors.rs"]
mod errors;

#[path = "rules/empty_function.rs"]
mod empty_function;

#[path = "rules/file_size.rs"]
mod file_size;

#[path = "rules/function_nesting.rs"]
mod function_nesting;

#[path = "rules/function_size.rs"]
mod function_size;

#[path = "rules/function_statements.rs"]
mod function_statements;

#[path = "rules/no_comments.rs"]
mod no_comments;

#[path = "rules/parameter_count.rs"]
mod parameter_count;

#[path = "rules/return_count.rs"]
mod return_count;

#[path = "rules/todo_requires_reference.rs"]
mod todo_requires_reference;
