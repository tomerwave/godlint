use crate::{
    config::{Config, Severity},
    rules::{
        Languages, Rule, accountable_suppression::AccountableSuppression,
        assertion_required::AssertionRequired, bot_conditions::BotConditions,
        branch_naming::BranchNaming, cognitive_complexity::CognitiveComplexity,
        condition_complexity::ConditionComplexity, decision_complexity::DecisionComplexity,
        dependency_boundary::DependencyBoundary, direct_environment_read::DirectEnvironmentRead,
        empty_error_handler::EmptyErrorHandler, empty_function::EmptyFunction,
        explicit_timer_delay::ExplicitTimerDelay,
        explicit_workflow_permissions::ExplicitWorkflowPermissions, file_size::FileSize,
        filename_case::FilenameCase, forbidden_dependency::ForbiddenDependency,
        function_nesting::FunctionNesting, function_size::FunctionSize,
        function_statements::FunctionStatements,
        hardcoded_container_credentials::HardcodedContainerCredentials,
        lockfile_version_drift::LockfileVersionDrift, module_independence::ModuleIndependence,
        no_comments::NoComments, no_dynamic_execution::NoDynamicExecution,
        no_empty_test::NoEmptyTest, no_focused_test::NoFocusedTest,
        no_inline_script::NoInlineScript, no_insecure_random::NoInsecureRandom,
        no_internal_import::NoInternalImport, no_monolithic_job::NoMonolithicJob,
        no_network_in_unit_test::NoNetworkInUnitTest, no_production_log::NoProductionLog,
        no_randomness_without_seed::NoRandomnessWithoutSeed, no_shell_command::NoShellCommand,
        no_silenced_failure::NoSilencedFailure, no_skipped_test::NoSkippedTest,
        no_sleep_in_test::NoSleepInTest, no_test_helper_in_production::NoTestHelperInProduction,
        no_weak_hash::NoWeakHash, no_workflow_comments::NoWorkflowComments,
        overprovisioned_secrets::OverprovisionedSecrets, parameter_count::ParameterCount,
        pin_third_party_actions::PinThirdPartyActions, restricted_call::RestrictedCall,
        restricted_import::RestrictedImport, return_count::ReturnCount,
        secrets_inherit::SecretsInherit, stale_action_refs::StaleActionRefs,
        template_injection::TemplateInjection, todo_requires_reference::TodoRequiresReference,
        unredacted_secrets::UnredactedSecrets, untrusted_github_env::UntrustedGithubEnv,
        unused_suppression::UnusedSuppression,
    },
};

struct Registration {
    id: &'static str,
    severity: fn(&Config) -> Severity,
    suppressible: bool,
    languages: Languages,
}

macro_rules! registrations {
    ($($rule:ty => $field:ident, $suppressible:literal);+ $(;)?) => {
        const REGISTRATIONS: &[Registration] = &[
            $(
                Registration {
                    id: <$rule>::ID,
                    languages: <$rule>::LANGUAGES,
                    severity: |config| {
                        config
                            .rules
                            .$field
                            .as_ref()
                            .map_or(Severity::Off, <$rule as Rule>::severity)
                    },
                    suppressible: $suppressible,
                }
            ),+
        ];
    };
}

registrations! {
    FunctionSize => function_size, true;
    FunctionNesting => function_nesting, true;
    FileSize => file_size, true;
    EmptyFunction => empty_function, true;
    TodoRequiresReference => todo_requires_reference, true;
    ParameterCount => parameter_count, true;
    DecisionComplexity => decision_complexity, true;
    ConditionComplexity => condition_complexity, true;
    CognitiveComplexity => cognitive_complexity, true;
    ReturnCount => return_count, true;
    FunctionStatements => function_statements, true;
    NoComments => no_comments, true;
    AccountableSuppression => accountable_suppression, false;
    UnusedSuppression => unused_suppression, false;
    RestrictedCall => restricted_call, true;
    NoDynamicExecution => no_dynamic_execution, true;
    DirectEnvironmentRead => direct_environment_read, true;
    ExplicitTimerDelay => explicit_timer_delay, true;
    EmptyErrorHandler => empty_error_handler, true;
    AssertionRequired => assertion_required, true;
    NoEmptyTest => no_empty_test, true;
    NoFocusedTest => no_focused_test, true;
    NoSkippedTest => no_skipped_test, true;
    NoSleepInTest => no_sleep_in_test, true;
    NoRandomnessWithoutSeed => no_randomness_without_seed, true;
    NoNetworkInUnitTest => no_network_in_unit_test, true;
    NoInternalImport => no_internal_import, true;
    NoShellCommand => no_shell_command, true;
    NoTestHelperInProduction => no_test_helper_in_production, true;
    NoWeakHash => no_weak_hash, true;
    NoInsecureRandom => no_insecure_random, true;
    NoProductionLog => no_production_log, true;
    RestrictedImport => restricted_import, true;
    DependencyBoundary => dependency_boundary, true;
    ForbiddenDependency => forbidden_dependency, true;
    ModuleIndependence => module_independence, true;
    FilenameCase => filename_case, true;
    BranchNaming => branch_naming, false;
    LockfileVersionDrift => lockfile_version_drift, true;
    ExplicitWorkflowPermissions => explicit_workflow_permissions, true;
    PinThirdPartyActions => pin_third_party_actions, true;
    StaleActionRefs => stale_action_refs, true;
    TemplateInjection => template_injection, true;
    BotConditions => bot_conditions, true;
    NoInlineScript => no_inline_script, true;
    NoMonolithicJob => no_monolithic_job, true;
    SecretsInherit => secrets_inherit, true;
    OverprovisionedSecrets => overprovisioned_secrets, true;
    UnredactedSecrets => unredacted_secrets, true;
    UntrustedGithubEnv => untrusted_github_env, true;
    NoSilencedFailure => no_silenced_failure, true;
    NoWorkflowComments => no_workflow_comments, true;
    HardcodedContainerCredentials => hardcoded_container_credentials, true;
}

pub fn rule_ids() -> impl Iterator<Item = &'static str> {
    REGISTRATIONS.iter().map(|registration| registration.id)
}

pub fn is_known_rule(rule_id: &str) -> bool {
    registration(rule_id).is_some()
}

fn registration(rule_id: &str) -> Option<&'static Registration> {
    REGISTRATIONS
        .iter()
        .find(|registration| registration.id == rule_id)
}

pub fn configured_severity(config: &Config, rule_id: &str) -> Severity {
    registration(rule_id).map_or(Severity::Off, |registration| {
        (registration.severity)(config)
    })
}

pub fn is_suppressible_rule(rule_id: &str) -> bool {
    registration(rule_id).is_some_and(|registration| registration.suppressible)
}

pub fn rule_languages(rule_id: &str) -> Option<Languages> {
    registration(rule_id).map(|registration| registration.languages)
}

const NEAR: usize = 4;

pub fn closest_rule_id(name: &str) -> Option<&'static str> {
    rule_ids()
        .map(|rule_id| (distance(name, rule_id), rule_id))
        .filter(|(distance, _)| *distance < NEAR)
        .min()
        .map(|(_, rule_id)| rule_id)
}

fn distance(left: &str, right: &str) -> usize {
    let target: Vec<char> = right.chars().collect();
    let mut row: Vec<usize> = (0..=target.len()).collect();

    for (index, character) in left.chars().enumerate() {
        let mut diagonal = row[0];

        row[0] = index + 1;

        for column in 0..target.len() {
            let replace = diagonal + usize::from(character != target[column]);

            diagonal = row[column + 1];
            row[column + 1] = replace.min(row[column] + 1).min(diagonal + 1);
        }
    }

    row[target.len()]
}
