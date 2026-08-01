use crate::{
    config::{Config, Severity},
    rules::{
        Rule, accountable_suppression::AccountableSuppression,
        assertion_required::AssertionRequired, bot_conditions::BotConditions,
        cognitive_complexity::CognitiveComplexity, condition_complexity::ConditionComplexity,
        decision_complexity::DecisionComplexity, dependency_boundary::DependencyBoundary,
        direct_environment_read::DirectEnvironmentRead, empty_error_handler::EmptyErrorHandler,
        empty_function::EmptyFunction, explicit_timer_delay::ExplicitTimerDelay,
        explicit_workflow_permissions::ExplicitWorkflowPermissions, file_size::FileSize,
        filename_case::FilenameCase, forbidden_dependency::ForbiddenDependency,
        function_nesting::FunctionNesting, function_size::FunctionSize,
        function_statements::FunctionStatements,
        hardcoded_container_credentials::HardcodedContainerCredentials,
        module_independence::ModuleIndependence, no_comments::NoComments,
        no_dynamic_execution::NoDynamicExecution, no_empty_test::NoEmptyTest,
        no_focused_test::NoFocusedTest, no_inline_script::NoInlineScript,
        no_insecure_random::NoInsecureRandom, no_internal_import::NoInternalImport,
        no_monolithic_job::NoMonolithicJob, no_network_in_unit_test::NoNetworkInUnitTest,
        no_production_log::NoProductionLog, no_randomness_without_seed::NoRandomnessWithoutSeed,
        no_shell_command::NoShellCommand, no_silenced_failure::NoSilencedFailure,
        no_skipped_test::NoSkippedTest, no_sleep_in_test::NoSleepInTest,
        no_test_helper_in_production::NoTestHelperInProduction, no_weak_hash::NoWeakHash,
        no_workflow_comments::NoWorkflowComments, overprovisioned_secrets::OverprovisionedSecrets,
        parameter_count::ParameterCount, pin_third_party_actions::PinThirdPartyActions,
        restricted_call::RestrictedCall, restricted_import::RestrictedImport,
        return_count::ReturnCount, secrets_inherit::SecretsInherit,
        stale_action_refs::StaleActionRefs, template_injection::TemplateInjection,
        todo_requires_reference::TodoRequiresReference, unredacted_secrets::UnredactedSecrets,
        unused_suppression::UnusedSuppression,
    },
};

macro_rules! severity {
    ($name:ident, $rule:ty, $field:ident) => {
        pub(super) fn $name(config: &Config) -> Severity {
            config
                .rules
                .$field
                .as_ref()
                .map_or(Severity::Off, <$rule as Rule>::severity)
        }
    };
}

severity!(function_size_severity, FunctionSize, function_size);
severity!(function_nesting_severity, FunctionNesting, function_nesting);
severity!(file_size_severity, FileSize, file_size);
severity!(empty_function_severity, EmptyFunction, empty_function);
severity!(
    todo_requires_reference_severity,
    TodoRequiresReference,
    todo_requires_reference
);
severity!(parameter_count_severity, ParameterCount, parameter_count);
severity!(
    decision_complexity_severity,
    DecisionComplexity,
    decision_complexity
);
severity!(
    condition_complexity_severity,
    ConditionComplexity,
    condition_complexity
);
severity!(
    cognitive_complexity_severity,
    CognitiveComplexity,
    cognitive_complexity
);
severity!(return_count_severity, ReturnCount, return_count);
severity!(
    function_statements_severity,
    FunctionStatements,
    function_statements
);
severity!(no_comments_severity, NoComments, no_comments);
severity!(
    accountable_suppression_severity,
    AccountableSuppression,
    accountable_suppression
);
severity!(
    unused_suppression_severity,
    UnusedSuppression,
    unused_suppression
);

severity!(restricted_call_severity, RestrictedCall, restricted_call);
severity!(
    no_dynamic_execution_severity,
    NoDynamicExecution,
    no_dynamic_execution
);
severity!(
    direct_environment_read_severity,
    DirectEnvironmentRead,
    direct_environment_read
);
severity!(
    explicit_timer_delay_severity,
    ExplicitTimerDelay,
    explicit_timer_delay
);
severity!(
    empty_error_handler_severity,
    EmptyErrorHandler,
    empty_error_handler
);
severity!(no_weak_hash_severity, NoWeakHash, no_weak_hash);
severity!(no_focused_test_severity, NoFocusedTest, no_focused_test);
severity!(
    assertion_required_severity,
    AssertionRequired,
    assertion_required
);
severity!(no_empty_test_severity, NoEmptyTest, no_empty_test);
severity!(no_skipped_test_severity, NoSkippedTest, no_skipped_test);
severity!(
    no_internal_import_severity,
    NoInternalImport,
    no_internal_import
);
severity!(no_shell_command_severity, NoShellCommand, no_shell_command);
severity!(
    no_test_helper_in_production_severity,
    NoTestHelperInProduction,
    no_test_helper_in_production
);
severity!(no_sleep_in_test_severity, NoSleepInTest, no_sleep_in_test);
severity!(
    no_network_in_unit_test_severity,
    NoNetworkInUnitTest,
    no_network_in_unit_test
);
severity!(
    no_randomness_without_seed_severity,
    NoRandomnessWithoutSeed,
    no_randomness_without_seed
);
severity!(
    no_insecure_random_severity,
    NoInsecureRandom,
    no_insecure_random
);
severity!(
    no_production_log_severity,
    NoProductionLog,
    no_production_log
);
severity!(
    restricted_import_severity,
    RestrictedImport,
    restricted_import
);
severity!(
    dependency_boundary_severity,
    DependencyBoundary,
    dependency_boundary
);
severity!(
    forbidden_dependency_severity,
    ForbiddenDependency,
    forbidden_dependency
);
severity!(
    module_independence_severity,
    ModuleIndependence,
    module_independence
);
severity!(filename_case_severity, FilenameCase, filename_case);
severity!(
    explicit_workflow_permissions_severity,
    ExplicitWorkflowPermissions,
    explicit_workflow_permissions
);
severity!(
    pin_third_party_actions_severity,
    PinThirdPartyActions,
    pin_third_party_actions
);
severity!(
    stale_action_refs_severity,
    StaleActionRefs,
    stale_action_refs
);
severity!(
    template_injection_severity,
    TemplateInjection,
    template_injection
);
severity!(bot_conditions_severity, BotConditions, bot_conditions);
severity!(no_inline_script_severity, NoInlineScript, no_inline_script);
severity!(
    no_monolithic_job_severity,
    NoMonolithicJob,
    no_monolithic_job
);
severity!(secrets_inherit_severity, SecretsInherit, secrets_inherit);
severity!(
    overprovisioned_secrets_severity,
    OverprovisionedSecrets,
    overprovisioned_secrets
);
severity!(
    unredacted_secrets_severity,
    UnredactedSecrets,
    unredacted_secrets
);
severity!(
    no_silenced_failure_severity,
    NoSilencedFailure,
    no_silenced_failure
);
severity!(
    no_workflow_comments_severity,
    NoWorkflowComments,
    no_workflow_comments
);
severity!(
    hardcoded_container_credentials_severity,
    HardcodedContainerCredentials,
    hardcoded_container_credentials
);
