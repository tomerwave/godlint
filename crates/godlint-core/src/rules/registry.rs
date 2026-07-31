mod severity;

use severity::*;

use crate::{
    config::{Config, Severity},
    rules::{
        Languages, Rule, accountable_suppression::AccountableSuppression,
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
        no_shell_command::NoShellCommand, no_skipped_test::NoSkippedTest,
        no_sleep_in_test::NoSleepInTest, no_test_helper_in_production::NoTestHelperInProduction,
        no_weak_hash::NoWeakHash, no_workflow_comments::NoWorkflowComments,
        overprovisioned_secrets::OverprovisionedSecrets, parameter_count::ParameterCount,
        pin_third_party_actions::PinThirdPartyActions, restricted_call::RestrictedCall,
        restricted_import::RestrictedImport, return_count::ReturnCount,
        secrets_inherit::SecretsInherit, template_injection::TemplateInjection,
        todo_requires_reference::TodoRequiresReference, unredacted_secrets::UnredactedSecrets,
        unused_suppression::UnusedSuppression,
    },
};

struct Registration {
    id: &'static str,
    severity: fn(&Config) -> Severity,
    suppressible: bool,
    languages: Languages,
}

const REGISTRATIONS: &[Registration] = &[
    Registration {
        id: FunctionSize::ID,
        languages: FunctionSize::LANGUAGES,
        severity: function_size_severity,
        suppressible: true,
    },
    Registration {
        id: FunctionNesting::ID,
        languages: FunctionNesting::LANGUAGES,
        severity: function_nesting_severity,
        suppressible: true,
    },
    Registration {
        id: FileSize::ID,
        languages: FileSize::LANGUAGES,
        severity: file_size_severity,
        suppressible: true,
    },
    Registration {
        id: EmptyFunction::ID,
        languages: EmptyFunction::LANGUAGES,
        severity: empty_function_severity,
        suppressible: true,
    },
    Registration {
        id: TodoRequiresReference::ID,
        languages: TodoRequiresReference::LANGUAGES,
        severity: todo_requires_reference_severity,
        suppressible: true,
    },
    Registration {
        id: ParameterCount::ID,
        languages: ParameterCount::LANGUAGES,
        severity: parameter_count_severity,
        suppressible: true,
    },
    Registration {
        id: DecisionComplexity::ID,
        languages: DecisionComplexity::LANGUAGES,
        severity: decision_complexity_severity,
        suppressible: true,
    },
    Registration {
        id: ConditionComplexity::ID,
        languages: ConditionComplexity::LANGUAGES,
        severity: condition_complexity_severity,
        suppressible: true,
    },
    Registration {
        id: CognitiveComplexity::ID,
        languages: CognitiveComplexity::LANGUAGES,
        severity: cognitive_complexity_severity,
        suppressible: true,
    },
    Registration {
        id: ReturnCount::ID,
        languages: ReturnCount::LANGUAGES,
        severity: return_count_severity,
        suppressible: true,
    },
    Registration {
        id: FunctionStatements::ID,
        languages: FunctionStatements::LANGUAGES,
        severity: function_statements_severity,
        suppressible: true,
    },
    Registration {
        id: NoComments::ID,
        languages: NoComments::LANGUAGES,
        severity: no_comments_severity,
        suppressible: true,
    },
    Registration {
        id: AccountableSuppression::ID,
        languages: AccountableSuppression::LANGUAGES,
        severity: accountable_suppression_severity,
        suppressible: false,
    },
    Registration {
        id: UnusedSuppression::ID,
        languages: UnusedSuppression::LANGUAGES,
        severity: unused_suppression_severity,
        suppressible: false,
    },
    Registration {
        id: RestrictedCall::ID,
        languages: RestrictedCall::LANGUAGES,
        severity: restricted_call_severity,
        suppressible: true,
    },
    Registration {
        id: NoDynamicExecution::ID,
        languages: NoDynamicExecution::LANGUAGES,
        severity: no_dynamic_execution_severity,
        suppressible: true,
    },
    Registration {
        id: DirectEnvironmentRead::ID,
        languages: DirectEnvironmentRead::LANGUAGES,
        severity: direct_environment_read_severity,
        suppressible: true,
    },
    Registration {
        id: ExplicitTimerDelay::ID,
        languages: ExplicitTimerDelay::LANGUAGES,
        severity: explicit_timer_delay_severity,
        suppressible: true,
    },
    Registration {
        id: EmptyErrorHandler::ID,
        languages: EmptyErrorHandler::LANGUAGES,
        severity: empty_error_handler_severity,
        suppressible: true,
    },
    Registration {
        id: AssertionRequired::ID,
        languages: AssertionRequired::LANGUAGES,
        severity: assertion_required_severity,
        suppressible: true,
    },
    Registration {
        id: NoEmptyTest::ID,
        languages: NoEmptyTest::LANGUAGES,
        severity: no_empty_test_severity,
        suppressible: true,
    },
    Registration {
        id: NoFocusedTest::ID,
        languages: NoFocusedTest::LANGUAGES,
        severity: no_focused_test_severity,
        suppressible: true,
    },
    Registration {
        id: NoSkippedTest::ID,
        languages: NoSkippedTest::LANGUAGES,
        severity: no_skipped_test_severity,
        suppressible: true,
    },
    Registration {
        id: NoSleepInTest::ID,
        languages: NoSleepInTest::LANGUAGES,
        severity: no_sleep_in_test_severity,
        suppressible: true,
    },
    Registration {
        id: NoRandomnessWithoutSeed::ID,
        languages: NoRandomnessWithoutSeed::LANGUAGES,
        severity: no_randomness_without_seed_severity,
        suppressible: true,
    },
    Registration {
        id: NoNetworkInUnitTest::ID,
        languages: NoNetworkInUnitTest::LANGUAGES,
        severity: no_network_in_unit_test_severity,
        suppressible: true,
    },
    Registration {
        id: NoInternalImport::ID,
        languages: NoInternalImport::LANGUAGES,
        severity: no_internal_import_severity,
        suppressible: true,
    },
    Registration {
        id: NoShellCommand::ID,
        languages: NoShellCommand::LANGUAGES,
        severity: no_shell_command_severity,
        suppressible: true,
    },
    Registration {
        id: NoTestHelperInProduction::ID,
        languages: NoTestHelperInProduction::LANGUAGES,
        severity: no_test_helper_in_production_severity,
        suppressible: true,
    },
    Registration {
        id: NoWeakHash::ID,
        languages: NoWeakHash::LANGUAGES,
        severity: no_weak_hash_severity,
        suppressible: true,
    },
    Registration {
        id: NoInsecureRandom::ID,
        languages: NoInsecureRandom::LANGUAGES,
        severity: no_insecure_random_severity,
        suppressible: true,
    },
    Registration {
        id: NoProductionLog::ID,
        languages: NoProductionLog::LANGUAGES,
        severity: no_production_log_severity,
        suppressible: true,
    },
    Registration {
        id: RestrictedImport::ID,
        languages: RestrictedImport::LANGUAGES,
        severity: restricted_import_severity,
        suppressible: true,
    },
    Registration {
        id: DependencyBoundary::ID,
        languages: DependencyBoundary::LANGUAGES,
        severity: dependency_boundary_severity,
        suppressible: true,
    },
    Registration {
        id: ForbiddenDependency::ID,
        languages: ForbiddenDependency::LANGUAGES,
        severity: forbidden_dependency_severity,
        suppressible: true,
    },
    Registration {
        id: ModuleIndependence::ID,
        languages: ModuleIndependence::LANGUAGES,
        severity: module_independence_severity,
        suppressible: true,
    },
    Registration {
        id: FilenameCase::ID,
        languages: FilenameCase::LANGUAGES,
        severity: filename_case_severity,
        suppressible: true,
    },
    Registration {
        id: ExplicitWorkflowPermissions::ID,
        languages: ExplicitWorkflowPermissions::LANGUAGES,
        severity: explicit_workflow_permissions_severity,
        suppressible: true,
    },
    Registration {
        id: PinThirdPartyActions::ID,
        languages: PinThirdPartyActions::LANGUAGES,
        severity: pin_third_party_actions_severity,
        suppressible: true,
    },
    Registration {
        id: TemplateInjection::ID,
        languages: TemplateInjection::LANGUAGES,
        severity: template_injection_severity,
        suppressible: true,
    },
    Registration {
        id: BotConditions::ID,
        languages: BotConditions::LANGUAGES,
        severity: bot_conditions_severity,
        suppressible: true,
    },
    Registration {
        id: NoInlineScript::ID,
        languages: NoInlineScript::LANGUAGES,
        severity: no_inline_script_severity,
        suppressible: true,
    },
    Registration {
        id: NoMonolithicJob::ID,
        languages: NoMonolithicJob::LANGUAGES,
        severity: no_monolithic_job_severity,
        suppressible: true,
    },
    Registration {
        id: SecretsInherit::ID,
        languages: SecretsInherit::LANGUAGES,
        severity: secrets_inherit_severity,
        suppressible: true,
    },
    Registration {
        id: OverprovisionedSecrets::ID,
        languages: OverprovisionedSecrets::LANGUAGES,
        severity: overprovisioned_secrets_severity,
        suppressible: true,
    },
    Registration {
        id: UnredactedSecrets::ID,
        languages: UnredactedSecrets::LANGUAGES,
        severity: unredacted_secrets_severity,
        suppressible: true,
    },
    Registration {
        id: NoWorkflowComments::ID,
        languages: NoWorkflowComments::LANGUAGES,
        severity: no_workflow_comments_severity,
        suppressible: true,
    },
    Registration {
        id: HardcodedContainerCredentials::ID,
        languages: HardcodedContainerCredentials::LANGUAGES,
        severity: hardcoded_container_credentials_severity,
        suppressible: true,
    },
];

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
