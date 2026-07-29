use std::{fs, num::NonZeroU32, path::Path};

use serde::Deserialize;

use crate::suites;

mod error;
mod validate;

pub use error::ConfigError;

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
#[serde(deny_unknown_fields)]
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
    #[serde(rename = "logging/no-production-log")]
    pub no_production_log: Option<NoProductionLogRule>,
    #[serde(rename = "architecture/restricted-import")]
    pub restricted_import: Option<RestrictedImportRule>,
    #[serde(rename = "architecture/dependency-boundary")]
    pub dependency_boundary: Option<DependencyBoundaryRule>,
    #[serde(rename = "security/forbidden-dependency")]
    pub forbidden_dependency: Option<ForbiddenDependencyRule>,
    #[serde(rename = "architecture/filename-case")]
    pub filename_case: Option<FilenameCaseRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountableSuppressionRule {
    pub severity: Severity,
    #[serde(default, rename = "require-owner")]
    pub require_owner: bool,
    #[serde(default, rename = "require-expiry")]
    pub require_expiry: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnusedSuppressionRule {
    pub severity: Severity,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RestrictedCallRule {
    pub severity: Severity,
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
    #[serde(default)]
    pub modules: Vec<RestrictedImport>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FilenameCaseRule {
    pub severity: Severity,
    #[serde(default)]
    pub scopes: Vec<NamingScope>,
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
    #[serde(default)]
    pub layers: Vec<Layer>,
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
pub struct NoDynamicExecutionRule {
    pub severity: Severity,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectEnvironmentReadRule {
    pub severity: Severity,
    #[serde(default = "default_configuration_paths", rename = "allow-in")]
    pub allow_in: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoProductionLogRule {
    pub severity: Severity,
    #[serde(default, rename = "allow-in")]
    pub allow_in: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplicitTimerDelayRule {
    pub severity: Severity,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LineLimitRule {
    pub severity: Severity,
    #[serde(rename = "max-lines")]
    pub max_lines: NonZeroU32,
    #[serde(default = "enabled", rename = "skip-blank-lines")]
    pub skip_blank_lines: bool,
    #[serde(default = "enabled", rename = "skip-comments")]
    pub skip_comments: bool,
}

macro_rules! count_limit_rules {
    ($($name:ident { $key:literal => $field:ident }),+ $(,)?) => {
        $(
            #[derive(Debug, Deserialize)]
            #[serde(deny_unknown_fields)]
            pub struct $name {
                pub severity: Severity,
                #[serde(rename = $key)]
                pub $field: u32,
            }

            impl $name {
                pub fn limit(&self) -> u32 {
                    self.$field
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
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptyFunctionRule {
    pub severity: Severity,
    #[serde(default, rename = "allow-names")]
    pub allow_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoCommentsRule {
    pub severity: Severity,
    #[serde(default = "enabled", rename = "allow-doc-comments")]
    pub allow_doc_comments: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TodoRequiresReferenceRule {
    pub severity: Severity,
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

pub(crate) fn default_configuration_paths() -> Vec<String> {
    vec!["**/config.*".into(), "**/config/**".into()]
}

const fn default_fail_on() -> Severity {
    Severity::Error
}

const fn enabled() -> bool {
    true
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
