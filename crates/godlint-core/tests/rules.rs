#![allow(clippy::expect_used, clippy::unwrap_used)]

#[path = "rules/support.rs"]
mod support;

#[path = "rules/bot_conditions.rs"]
mod bot_conditions;

#[path = "rules/accountable_suppression.rs"]
mod accountable_suppression;

#[path = "rules/cognitive_complexity.rs"]
mod cognitive_complexity;

#[path = "rules/condition_complexity.rs"]
mod condition_complexity;

#[path = "rules/decision_complexity.rs"]
mod decision_complexity;

#[path = "rules/dependency_boundary.rs"]
mod dependency_boundary;

#[path = "rules/direct_environment_read.rs"]
mod direct_environment_read;

#[path = "rules/errors.rs"]
mod errors;

#[path = "rules/empty_function.rs"]
mod empty_function;

#[path = "rules/empty_error_handler.rs"]
mod empty_error_handler;

#[path = "rules/explicit_timer_delay.rs"]
mod explicit_timer_delay;

#[path = "rules/filename_case.rs"]
mod filename_case;

#[path = "rules/file_size.rs"]
mod file_size;

#[path = "rules/forbidden_dependency.rs"]
mod forbidden_dependency;

#[path = "rules/function_nesting.rs"]
mod function_nesting;

#[path = "rules/function_size.rs"]
mod function_size;

#[path = "rules/function_statements.rs"]
mod function_statements;

#[path = "rules/module_independence.rs"]
mod module_independence;

#[path = "rules/no_comments.rs"]
mod no_comments;

#[path = "rules/no_insecure_random.rs"]
mod no_insecure_random;

#[path = "rules/assertion_required.rs"]
mod assertion_required;

#[path = "rules/no_empty_test.rs"]
mod no_empty_test;

#[path = "rules/no_focused_test.rs"]
mod no_focused_test;

#[path = "rules/no_production_log.rs"]
mod no_production_log;

#[path = "rules/no_network_in_unit_test.rs"]
mod no_network_in_unit_test;

#[path = "rules/no_randomness_without_seed.rs"]
mod no_randomness_without_seed;

#[path = "rules/no_inline_script.rs"]
mod no_inline_script;
#[path = "rules/no_internal_import.rs"]
mod no_internal_import;
#[path = "rules/no_monolithic_job.rs"]
mod no_monolithic_job;

#[path = "rules/no_shell_command.rs"]
mod no_shell_command;

#[path = "rules/no_test_helper_in_production.rs"]
mod no_test_helper_in_production;

#[path = "rules/no_skipped_test.rs"]
mod no_skipped_test;

#[path = "rules/no_sleep_in_test.rs"]
mod no_sleep_in_test;

#[path = "rules/no_weak_hash.rs"]
mod no_weak_hash;

#[path = "rules/no_dynamic_execution.rs"]
mod no_dynamic_execution;

#[path = "rules/parameter_count.rs"]
mod parameter_count;

#[path = "rules/explicit_workflow_permissions.rs"]
mod explicit_workflow_permissions;

#[path = "rules/secrets_inherit.rs"]
mod secrets_inherit;

#[path = "rules/overprovisioned_secrets.rs"]
mod overprovisioned_secrets;

#[path = "rules/hardcoded_container_credentials.rs"]
mod hardcoded_container_credentials;
#[path = "rules/unredacted_secrets.rs"]
mod unredacted_secrets;

#[path = "rules/no_workflow_comments.rs"]
mod no_workflow_comments;

#[path = "rules/pin_third_party_actions.rs"]
mod pin_third_party_actions;

#[path = "rules/registry.rs"]
mod registry;

#[path = "rules/return_count.rs"]
mod return_count;

#[path = "rules/restricted_import.rs"]
mod restricted_import;

#[path = "rules/restricted_call.rs"]
mod restricted_call;

#[path = "rules/todo_requires_reference.rs"]
mod todo_requires_reference;

#[path = "rules/template_injection.rs"]
mod template_injection;

#[path = "rules/unused_suppression.rs"]
mod unused_suppression;
