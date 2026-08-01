use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, de::IgnoredAny};

use crate::suites;

mod error;
mod rules;
mod validate;

pub use error::ConfigError;
pub use rules::*;

pub const DEFAULT_EXCLUDES: [&str; 12] = [
    ".git",
    ".mypy_cache",
    ".next",
    ".tox",
    ".venv",
    "__pycache__",
    "build",
    "coverage",
    "dist",
    "node_modules",
    "target",
    "vendor",
];

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u8,
    #[serde(default = "default_fail_on", rename = "fail-on")]
    pub fail_on: Severity,
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub suites: Vec<String>,
    #[serde(default)]
    pub rules: Rules,
}

#[derive(Debug, Default, Deserialize)]
pub struct Rules {
    #[serde(rename = "maintainability/function-size")]
    pub function_size: Option<LineLimitRule>,
    #[serde(rename = "maintainability/function-nesting")]
    pub function_nesting: Option<FunctionNestingRule>,
    #[serde(rename = "maintainability/file-size")]
    pub file_size: Option<LineLimitRule>,
    #[serde(rename = "maintainability/empty-function")]
    pub empty_function: Option<EmptyFunctionRule>,
    #[serde(rename = "policy/todo-requires-reference")]
    pub todo_requires_reference: Option<TodoRequiresReferenceRule>,
    #[serde(rename = "maintainability/parameter-count")]
    pub parameter_count: Option<ParameterCountRule>,
    #[serde(rename = "maintainability/decision-complexity")]
    pub decision_complexity: Option<DecisionComplexityRule>,
    #[serde(rename = "maintainability/condition-complexity")]
    pub condition_complexity: Option<ConditionComplexityRule>,
    #[serde(rename = "maintainability/cognitive-complexity")]
    pub cognitive_complexity: Option<CognitiveComplexityRule>,
    #[serde(rename = "maintainability/return-count")]
    pub return_count: Option<ReturnCountRule>,
    #[serde(rename = "maintainability/function-statements")]
    pub function_statements: Option<FunctionStatementsRule>,
    #[serde(rename = "style/no-comments")]
    pub no_comments: Option<NoCommentsRule>,
    #[serde(rename = "policy/accountable-suppression")]
    pub accountable_suppression: Option<AccountableSuppressionRule>,
    #[serde(rename = "policy/unused-suppression")]
    pub unused_suppression: Option<UnusedSuppressionRule>,
    #[serde(rename = "architecture/restricted-call")]
    pub restricted_call: Option<RestrictedCallRule>,
    #[serde(rename = "security/no-dynamic-execution")]
    pub no_dynamic_execution: Option<NoDynamicExecutionRule>,
    #[serde(rename = "security/direct-environment-read")]
    pub direct_environment_read: Option<DirectEnvironmentReadRule>,
    #[serde(rename = "reliability/explicit-timer-delay")]
    pub explicit_timer_delay: Option<ExplicitTimerDelayRule>,
    #[serde(rename = "reliability/empty-error-handler")]
    pub empty_error_handler: Option<EmptyErrorHandlerRule>,
    #[serde(rename = "testing/assertion-required")]
    pub assertion_required: Option<AssertionRequiredRule>,
    #[serde(rename = "testing/no-empty-test")]
    pub no_empty_test: Option<NoEmptyTestRule>,
    #[serde(rename = "testing/no-focused-test")]
    pub no_focused_test: Option<NoFocusedTestRule>,
    #[serde(rename = "testing/no-skipped-test")]
    pub no_skipped_test: Option<NoSkippedTestRule>,
    #[serde(rename = "testing/no-test-helper-in-production")]
    pub no_test_helper_in_production: Option<NoTestHelperInProductionRule>,
    #[serde(rename = "architecture/no-internal-import")]
    pub no_internal_import: Option<NoInternalImportRule>,
    #[serde(rename = "security/no-shell-command")]
    pub no_shell_command: Option<NoShellCommandRule>,
    #[serde(rename = "testing/no-sleep-in-test")]
    pub no_sleep_in_test: Option<NoSleepInTestRule>,
    #[serde(rename = "testing/no-randomness-without-seed")]
    pub no_randomness_without_seed: Option<NoRandomnessWithoutSeedRule>,
    #[serde(rename = "testing/no-network-in-unit-test")]
    pub no_network_in_unit_test: Option<NoNetworkInUnitTestRule>,
    #[serde(rename = "security/no-weak-hash")]
    pub no_weak_hash: Option<NoWeakHashRule>,
    #[serde(rename = "security/no-insecure-random")]
    pub no_insecure_random: Option<NoInsecureRandomRule>,
    #[serde(rename = "logging/no-production-log")]
    pub no_production_log: Option<NoProductionLogRule>,
    #[serde(rename = "architecture/restricted-import")]
    pub restricted_import: Option<RestrictedImportRule>,
    #[serde(rename = "architecture/dependency-boundary")]
    pub dependency_boundary: Option<DependencyBoundaryRule>,
    #[serde(rename = "architecture/module-independence")]
    pub module_independence: Option<ModuleIndependenceRule>,
    #[serde(rename = "security/forbidden-dependency")]
    pub forbidden_dependency: Option<ForbiddenDependencyRule>,
    #[serde(rename = "architecture/filename-case")]
    pub filename_case: Option<FilenameCaseRule>,
    #[serde(rename = "ci/pin-third-party-actions")]
    pub pin_third_party_actions: Option<PinThirdPartyActionsRule>,
    #[serde(rename = "ci/stale-action-refs")]
    pub stale_action_refs: Option<StaleActionRefsRule>,
    #[serde(rename = "ci/explicit-workflow-permissions")]
    pub explicit_workflow_permissions: Option<ExplicitWorkflowPermissionsRule>,
    #[serde(rename = "ci/no-comments")]
    pub no_workflow_comments: Option<NoWorkflowCommentsRule>,
    #[serde(rename = "ci/hardcoded-container-credentials")]
    pub hardcoded_container_credentials: Option<HardcodedContainerCredentialsRule>,
    #[serde(rename = "ci/template-injection")]
    pub template_injection: Option<TemplateInjectionRule>,
    #[serde(rename = "ci/bot-conditions")]
    pub bot_conditions: Option<BotConditionsRule>,
    #[serde(rename = "ci/no-inline-script")]
    pub no_inline_script: Option<LineLimitRule>,
    #[serde(rename = "ci/no-monolithic-job")]
    pub no_monolithic_job: Option<NoMonolithicJobRule>,
    #[serde(rename = "ci/secrets-inherit")]
    pub secrets_inherit: Option<SecretsInheritRule>,
    #[serde(rename = "ci/overprovisioned-secrets")]
    pub overprovisioned_secrets: Option<OverprovisionedSecretsRule>,
    #[serde(rename = "ci/unredacted-secrets")]
    pub unredacted_secrets: Option<UnredactedSecretsRule>,
    #[serde(rename = "ci/no-silenced-failure")]
    pub no_silenced_failure: Option<NoSilencedFailureRule>,
    #[serde(flatten)]
    unrecognised: BTreeMap<String, IgnoredAny>,
}

impl Rules {
    pub fn unrecognised(&self) -> impl Iterator<Item = &str> {
        self.unrecognised.keys().map(String::as_str)
    }
}

const fn default_fail_on() -> Severity {
    Severity::Error
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Off,
    Info,
    Warning,
    Error,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let mut config: Self =
            yaml_serde::from_str(&source).map_err(|source| ConfigError::Parse {
                path: path.to_path_buf(),
                source,
            })?;

        config.expand_suites()?;
        config.validate()?;

        Ok(config)
    }

    pub fn excludes(&self) -> Vec<String> {
        if self.exclude.is_empty() {
            return DEFAULT_EXCLUDES.iter().map(|name| (*name).into()).collect();
        }

        self.exclude.clone()
    }

    fn expand_suites(&mut self) -> Result<(), ConfigError> {
        for name in &self.suites {
            let expand = suites::lookup(name)
                .ok_or_else(|| ConfigError::UnknownSuite { name: name.clone() })?;

            expand(&mut self.rules);
        }

        Ok(())
    }
}
