use std::num::NonZeroU32;

use crate::config::{
    AccountableSuppressionRule, AssertionRequiredRule, BotConditionsRule, CognitiveComplexityRule,
    ConditionComplexityRule, DecisionComplexityRule, DependencyBoundaryRule,
    DirectEnvironmentReadRule, EmptyErrorHandlerRule, EmptyFunctionRule, ExplicitTimerDelayRule,
    ExplicitWorkflowPermissionsRule, FilenameCaseRule, ForbiddenDependencyRule,
    FunctionNestingRule, FunctionStatementsRule, HardcodedContainerCredentialsRule, LineLimitRule,
    ModuleIndependenceRule, NoCommentsRule, NoDynamicExecutionRule, NoEmptyTestRule,
    NoFocusedTestRule, NoInsecureRandomRule, NoInternalImportRule, NoMonolithicJobRule,
    NoNetworkInUnitTestRule, NoProductionLogRule, NoRandomnessWithoutSeedRule, NoShellCommandRule,
    NoSkippedTestRule, NoSleepInTestRule, NoTestHelperInProductionRule, NoWeakHashRule,
    NoWorkflowCommentsRule, OverprovisionedSecretsRule, ParameterCountRule,
    PinThirdPartyActionsRule, RestrictedCallRule, RestrictedImportRule, ReturnCountRule, Rules,
    SecretsInheritRule, Severity, TemplateInjectionRule, TodoRequiresReferenceRule,
    UnredactedSecretsRule, UnusedSuppressionRule, default_bots, default_configuration_paths,
    default_markers, default_reference_prefixes, default_test_helpers, default_test_paths,
    default_trusted_owners,
};

pub const RECOMMENDED: &str = "recommended@1";

type Expand = fn(&mut Rules);

const SUITES: &[(&str, Expand)] = &[(RECOMMENDED, recommended)];

const FUNCTION_LINES: NonZeroU32 = NonZeroU32::new(50).expect("50 is not zero");

const FILE_LINES: NonZeroU32 = NonZeroU32::new(500).expect("500 is not zero");

const INLINE_SCRIPT_LINES: NonZeroU32 = NonZeroU32::new(8).expect("8 is not zero");

const JOB_STEPS: u32 = 7;

pub fn names() -> impl Iterator<Item = &'static str> {
    SUITES.iter().map(|(name, _)| *name)
}

pub fn lookup(name: &str) -> Option<Expand> {
    SUITES
        .iter()
        .find(|(known, _)| *known == name)
        .map(|(_, expand)| *expand)
}

fn recommended(rules: &mut Rules) {
    maintainability(rules);
    policy(rules);
    security(rules);
    reliability(rules);
    logging(rules);
    testing(rules);
    continuous_integration(rules);
}

fn continuous_integration(rules: &mut Rules) {
    workflow_shape(rules);
    workflow_secrets(rules);
    workflow_hygiene(rules);
}

fn workflow_shape(rules: &mut Rules) {
    rules.no_inline_script.get_or_insert(LineLimitRule {
        severity: Severity::Error,
        max_lines: INLINE_SCRIPT_LINES,
        skip_blank_lines: true,
        skip_comments: true,
    });
    rules.no_monolithic_job.get_or_insert(NoMonolithicJobRule {
        severity: Severity::Error,
        max_steps: JOB_STEPS,
        allow_in: Vec::new(),
    });
}

fn workflow_secrets(rules: &mut Rules) {
    rules.secrets_inherit.get_or_insert(SecretsInheritRule {
        severity: Severity::Error,
        allow_in: Vec::new(),
    });
    rules
        .overprovisioned_secrets
        .get_or_insert(OverprovisionedSecretsRule {
            severity: Severity::Error,
        });
    rules
        .unredacted_secrets
        .get_or_insert(UnredactedSecretsRule {
            severity: Severity::Error,
        });
    rules
        .template_injection
        .get_or_insert(TemplateInjectionRule {
            severity: Severity::Error,
            allow_in: Vec::new(),
        });
    rules
        .bot_conditions
        .get_or_insert_with(|| BotConditionsRule {
            severity: Severity::Error,
            bots: default_bots(),
        });
    rules
        .explicit_workflow_permissions
        .get_or_insert(ExplicitWorkflowPermissionsRule {
            severity: Severity::Error,
            require_per_job: false,
        });
    rules
        .pin_third_party_actions
        .get_or_insert(PinThirdPartyActionsRule {
            severity: Severity::Error,
            trusted_owners: default_trusted_owners(),
        });
}

fn workflow_hygiene(rules: &mut Rules) {
    rules
        .no_workflow_comments
        .get_or_insert(NoWorkflowCommentsRule {
            severity: Severity::Error,
        });
    rules
        .hardcoded_container_credentials
        .get_or_insert(HardcodedContainerCredentialsRule {
            severity: Severity::Error,
        });
}

fn maintainability(rules: &mut Rules) {
    size(rules);
    complexity(rules);
}

fn size(rules: &mut Rules) {
    let error = Severity::Error;

    rules.function_size.get_or_insert(LineLimitRule {
        severity: error,
        max_lines: FUNCTION_LINES,
        skip_blank_lines: true,
        skip_comments: true,
    });
    rules.file_size.get_or_insert(LineLimitRule {
        severity: error,
        max_lines: FILE_LINES,
        skip_blank_lines: true,
        skip_comments: true,
    });
    rules.function_nesting.get_or_insert(FunctionNestingRule {
        severity: error,
        max_depth: 2,
    });
    rules.parameter_count.get_or_insert(ParameterCountRule {
        severity: error,
        max_parameters: 4,
    });
}

fn complexity(rules: &mut Rules) {
    let error = Severity::Error;

    rules
        .decision_complexity
        .get_or_insert(DecisionComplexityRule {
            severity: error,
            max_complexity: 5,
        });
    rules
        .condition_complexity
        .get_or_insert(ConditionComplexityRule {
            severity: error,
            max_operators: 3,
        });
    rules
        .cognitive_complexity
        .get_or_insert(CognitiveComplexityRule {
            severity: error,
            max_score: 15,
        });
    rules.return_count.get_or_insert(ReturnCountRule {
        severity: error,
        max_returns: 6,
    });
    rules
        .function_statements
        .get_or_insert(FunctionStatementsRule {
            severity: error,
            max_statements: 14,
        });
    rules.empty_function.get_or_insert(EmptyFunctionRule {
        severity: error,
        allow_names: Vec::new(),
    });
}

fn policy(rules: &mut Rules) {
    let error = Severity::Error;

    rules
        .todo_requires_reference
        .get_or_insert_with(|| TodoRequiresReferenceRule {
            severity: error,
            markers: default_markers(),
            reference_prefixes: default_reference_prefixes(),
        });
    rules.no_comments.get_or_insert(NoCommentsRule {
        severity: error,
        allow_doc_comments: false,
    });
    rules
        .accountable_suppression
        .get_or_insert(AccountableSuppressionRule {
            severity: error,
            require_owner: true,
            require_expiry: true,
        });
    rules
        .unused_suppression
        .get_or_insert(UnusedSuppressionRule { severity: error });
}

fn security(rules: &mut Rules) {
    boundaries(rules);
    dangerous_apis(rules);
}

fn boundaries(rules: &mut Rules) {
    let error = Severity::Error;

    rules.restricted_call.get_or_insert(RestrictedCallRule {
        severity: error,
        calls: Vec::new(),
    });
    rules.filename_case.get_or_insert(FilenameCaseRule {
        severity: error,
        scopes: Vec::new(),
        allow: Vec::new(),
    });
    rules.restricted_import.get_or_insert(RestrictedImportRule {
        severity: error,
        modules: Vec::new(),
    });
    rules
        .no_internal_import
        .get_or_insert(NoInternalImportRule {
            severity: error,
            allow: Vec::new(),
        });
    rules
        .dependency_boundary
        .get_or_insert(DependencyBoundaryRule {
            severity: error,
            layers: Vec::new(),
        });
    rules
        .forbidden_dependency
        .get_or_insert(ForbiddenDependencyRule {
            severity: error,
            packages: Vec::new(),
        });
    rules
        .module_independence
        .get_or_insert(ModuleIndependenceRule {
            severity: error,
            sets: Vec::new(),
        });
}

fn dangerous_apis(rules: &mut Rules) {
    let error = Severity::Error;

    rules
        .no_dynamic_execution
        .get_or_insert(NoDynamicExecutionRule { severity: error });
    rules.no_shell_command.get_or_insert(NoShellCommandRule {
        severity: error,
        allow_in: Vec::new(),
    });
    rules
        .no_insecure_random
        .get_or_insert(NoInsecureRandomRule {
            severity: error,
            allow_in: Vec::new(),
        });
    rules.no_weak_hash.get_or_insert(NoWeakHashRule {
        severity: error,
        allow_in: Vec::new(),
    });
    rules
        .direct_environment_read
        .get_or_insert_with(|| DirectEnvironmentReadRule {
            severity: error,
            allow_in: default_configuration_paths(),
        });
}

fn reliability(rules: &mut Rules) {
    rules
        .explicit_timer_delay
        .get_or_insert(ExplicitTimerDelayRule {
            severity: Severity::Error,
        });
    rules
        .empty_error_handler
        .get_or_insert(EmptyErrorHandlerRule {
            severity: Severity::Error,
        });
}

fn testing(rules: &mut Rules) {
    let error = Severity::Error;

    rules
        .no_focused_test
        .get_or_insert(NoFocusedTestRule { severity: error });
    rules
        .no_empty_test
        .get_or_insert(NoEmptyTestRule { severity: error });
    rules
        .assertion_required
        .get_or_insert(AssertionRequiredRule {
            severity: error,
            extra_assertions: Vec::new(),
        });
    rules
        .no_skipped_test
        .get_or_insert(NoSkippedTestRule { severity: error });
    rules
        .no_sleep_in_test
        .get_or_insert(NoSleepInTestRule { severity: error });
    rules
        .no_randomness_without_seed
        .get_or_insert(NoRandomnessWithoutSeedRule {
            severity: error,
            allow_in: Vec::new(),
        });
    rules
        .no_network_in_unit_test
        .get_or_insert(NoNetworkInUnitTestRule {
            severity: error,
            unit_paths: Vec::new(),
            allow_in: Vec::new(),
        });
    rules
        .no_test_helper_in_production
        .get_or_insert(NoTestHelperInProductionRule {
            severity: error,
            test_paths: default_test_paths(),
            helpers: default_test_helpers(),
        });
}

fn logging(rules: &mut Rules) {
    rules.no_production_log.get_or_insert(NoProductionLogRule {
        severity: Severity::Error,
        allow_in: Vec::new(),
    });
}
