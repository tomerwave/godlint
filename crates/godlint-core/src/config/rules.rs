use std::num::NonZeroU32;

use serde::Deserialize;

use crate::config::{Scoped, Severity};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountableSuppressionRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default, rename = "require-owner")]
    pub require_owner: bool,
    #[serde(default, rename = "require-expiry")]
    pub require_expiry: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedCallRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default)]
    pub calls: Vec<RestrictedCall>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedCall {
    pub name: String,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedImportRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default)]
    pub modules: Vec<RestrictedImport>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilenameCaseRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default)]
    pub scopes: Vec<NamingScope>,
    #[serde(default)]
    pub allow: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BranchNamingRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default = "default_branch_types")]
    pub types: Vec<String>,
    #[serde(default)]
    pub allow: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamingScope {
    pub paths: Vec<String>,
    pub case: NamingCase,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum NamingCase {
    Kebab,
    Snake,
    Camel,
    Pascal,
}

impl NamingCase {
    pub fn describe(self) -> &'static str {
        match self {
            Self::Kebab => "kebab-case",
            Self::Snake => "snake_case",
            Self::Camel => "camelCase",
            Self::Pascal => "PascalCase",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForbiddenDependencyRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default)]
    pub packages: Vec<ForbiddenDependency>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ForbiddenDependency {
    pub name: String,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyBoundaryRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default)]
    pub layers: Vec<Layer>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleIndependenceRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default)]
    pub sets: Vec<IndependentSet>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IndependentSet {
    pub name: String,
    #[serde(default)]
    pub members: Vec<Layer>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub name: String,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub modules: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedImport {
    pub name: String,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectEnvironmentReadRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,

    #[serde(default = "default_configuration_paths", rename = "allow-in")]
    pub allow_in: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssertionRequiredRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default, rename = "extra-assertions")]
    pub extra_assertions: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoTestHelperInProductionRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default = "default_test_paths", rename = "test-paths")]
    pub test_paths: Vec<String>,
    #[serde(default = "default_test_helpers")]
    pub helpers: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoInternalImportRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default)]
    pub allow: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoNetworkInUnitTestRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,

    #[serde(default, rename = "unit-paths")]
    pub unit_paths: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplicitWorkflowPermissionsRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default, rename = "require-per-job")]
    pub require_per_job: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PinThirdPartyActionsRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default = "default_trusted_owners", rename = "trusted-owners")]
    pub trusted_owners: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BotConditionsRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default = "default_bots")]
    pub bots: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineLimitRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(rename = "max-lines")]
    pub max_lines: NonZeroU32,
    #[serde(default = "enabled", rename = "skip-blank-lines")]
    pub skip_blank_lines: bool,
    #[serde(default = "enabled", rename = "skip-comments")]
    pub skip_comments: bool,
}

macro_rules! scoped_rules {
    ($($name:ident),+ $(,)?) => {
        $(
            #[derive(Debug, Deserialize)]
            #[serde(deny_unknown_fields)]
            pub struct $name {
                pub severity: Severity,
                #[serde(default, rename = "only-in")]
                pub only_in: Vec<String>,
                #[serde(default, rename = "allow-in")]
                pub allow_in: Vec<String>,
            }

            impl Scoped for $name {
                fn only_in(&self) -> &[String] {
                    &self.only_in
                }

                fn allow_in(&self) -> &[String] {
                    &self.allow_in
                }
            }
        )+
    };
}

macro_rules! scoped {
    ($($name:ident),+ $(,)?) => {
        $(
            impl Scoped for $name {
                fn only_in(&self) -> &[String] {
                    &self.only_in
                }

                fn allow_in(&self) -> &[String] {
                    &self.allow_in
                }
            }
        )+
    };
}

scoped_rules! {
    EmptyErrorHandlerRule,
    ExplicitTimerDelayRule,
    HardcodedContainerCredentialsRule,
    NoDynamicExecutionRule,
    NoEmptyTestRule,
    NoFocusedTestRule,
    NoInsecureRandomRule,
    NoProductionLogRule,
    NoRandomnessWithoutSeedRule,
    NoShellCommandRule,
    NoSilencedFailureRule,
    NoSkippedTestRule,
    NoSleepInTestRule,
    NoWeakHashRule,
    NoWorkflowCommentsRule,
    OverprovisionedSecretsRule,
    SecretsInheritRule,
    StaleActionRefsRule,
    TemplateInjectionRule,
    UnredactedSecretsRule,
    UntrustedGithubEnvRule,
    UnusedSuppressionRule,
    LockfileVersionDriftRule,
}

pub type NetworkTimeoutRequiredRule = NoProductionLogRule;

macro_rules! count_limit_rules {
    ($($name:ident { $key:literal => $field:ident }),+ $(,)?) => {
        $(
            #[derive(Debug, Deserialize)]
            #[serde(deny_unknown_fields)]
            pub struct $name {
                pub severity: Severity,
                #[serde(default, rename = "only-in")]
                pub only_in: Vec<String>,
                #[serde(default, rename = "allow-in")]
                pub allow_in: Vec<String>,
                #[serde(rename = $key)]
                pub $field: u32,
            }

            impl $name {
                pub fn limit(&self) -> u32 {
                    self.$field
                }
            }

            impl Scoped for $name {
                fn only_in(&self) -> &[String] {
                    &self.only_in
                }

                fn allow_in(&self) -> &[String] {
                    &self.allow_in
                }
            }
        )+
    };
}

count_limit_rules! {
    FunctionNestingRule { "max-depth" => max_depth },
    ParameterCountRule { "max-parameters" => max_parameters },
    DecisionComplexityRule { "max-complexity" => max_complexity },
    ReturnCountRule { "max-returns" => max_returns },
    FunctionStatementsRule { "max-statements" => max_statements },
    ConditionComplexityRule { "max-operators" => max_operators },
    CognitiveComplexityRule { "max-score" => max_score },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoMonolithicJobRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,

    #[serde(rename = "max-steps")]
    pub max_steps: u32,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
}

impl NoMonolithicJobRule {
    pub fn limit(&self) -> u32 {
        self.max_steps
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptyFunctionRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default, rename = "allow-names")]
    pub allow_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoCommentsRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default = "enabled", rename = "allow-doc-comments")]
    pub allow_doc_comments: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TodoRequiresReferenceRule {
    pub severity: Severity,
    #[serde(default, rename = "only-in")]
    pub only_in: Vec<String>,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
    #[serde(default = "default_markers")]
    pub markers: Vec<String>,
    #[serde(default = "default_reference_prefixes", rename = "reference-prefixes")]
    pub reference_prefixes: Vec<String>,
}

pub(crate) fn default_reference_prefixes() -> Vec<String> {
    vec!["#".into()]
}

pub(crate) fn default_markers() -> Vec<String> {
    vec!["TODO".into(), "FIXME".into(), "HACK".into(), "XXX".into()]
}

pub(crate) fn default_test_paths() -> Vec<String> {
    vec![
        "**/tests/**".into(),
        "**/test/**".into(),
        "**/__tests__/**".into(),
        "**/*.test.*".into(),
        "**/*.spec.*".into(),
        "**/test_*.py".into(),
        "**/*_test.py".into(),
        "**/conftest.py".into(),
    ]
}

pub(crate) fn default_test_helpers() -> Vec<String> {
    vec![
        "tests".into(),
        "test".into(),
        "__tests__".into(),
        "__mocks__".into(),
        "fixtures".into(),
        "mocks".into(),
        "conftest".into(),
    ]
}

pub(crate) fn default_trusted_owners() -> Vec<String> {
    vec!["actions".into(), "github".into()]
}

pub(crate) fn default_branch_types() -> Vec<String> {
    vec![
        "feat".into(),
        "fix".into(),
        "perf".into(),
        "docs".into(),
        "chore".into(),
        "refactor".into(),
        "style".into(),
        "build".into(),
        "revert".into(),
        "test".into(),
        "ci".into(),
        "release".into(),
    ]
}

pub(crate) fn default_bots() -> Vec<String> {
    vec![
        "dependabot[bot]".into(),
        "github-actions[bot]".into(),
        "renovate[bot]".into(),
    ]
}

pub(crate) fn default_configuration_paths() -> Vec<String> {
    vec!["**/config.*".into(), "**/config/**".into()]
}

const fn enabled() -> bool {
    true
}

scoped! {
    AccountableSuppressionRule,
    AssertionRequiredRule,
    BotConditionsRule,
    BranchNamingRule,
    DependencyBoundaryRule,
    DirectEnvironmentReadRule,
    EmptyFunctionRule,
    ExplicitWorkflowPermissionsRule,
    FilenameCaseRule,
    ForbiddenDependencyRule,
    LineLimitRule,
    ModuleIndependenceRule,
    NoCommentsRule,
    NoInternalImportRule,
    NoMonolithicJobRule,
    NoNetworkInUnitTestRule,
    NoTestHelperInProductionRule,
    PinThirdPartyActionsRule,
    RestrictedCallRule,
    RestrictedImportRule,
    TodoRequiresReferenceRule,
}
