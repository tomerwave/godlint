use std::{
    collections::BTreeSet,
    error::Error,
    fmt, fs,
    num::NonZeroU32,
    path::{Path, PathBuf},
};

use serde::Deserialize;

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

fn default_reference_prefixes() -> Vec<String> {
    vec!["#".into()]
}

fn default_markers() -> Vec<String> {
    vec!["TODO".into(), "FIXME".into(), "HACK".into(), "XXX".into()]
}

fn default_configuration_paths() -> Vec<String> {
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

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: yaml_serde::Error,
    },
    UnsupportedVersion {
        version: u8,
    },
    InvalidComplexityLimit,
    InvalidTodoMarkers,
    InvalidTodoReferencePrefixes,
    InvalidRestrictedCallName,
    DuplicateRestrictedCallName {
        name: String,
    },
    BlankAllowIn {
        rule: &'static str,
    },
    InvalidExclude {
        pattern: String,
    },
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let source = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        let config: Self = yaml_serde::from_str(&source).map_err(|source| ConfigError::Parse {
            path: path.to_path_buf(),
            source,
        })?;

        config.validate()?;

        Ok(config)
    }

    pub fn excludes(&self) -> Vec<String> {
        if self.exclude.is_empty() {
            return DEFAULT_EXCLUDES.iter().map(|name| (*name).into()).collect();
        }

        self.exclude.clone()
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.version != 1 {
            return Err(ConfigError::UnsupportedVersion {
                version: self.version,
            });
        }

        if let Some(pattern) = self
            .exclude
            .iter()
            .find(|pattern| pattern.trim().is_empty())
        {
            return Err(ConfigError::InvalidExclude {
                pattern: pattern.clone(),
            });
        }

        if self
            .rules
            .decision_complexity
            .as_ref()
            .is_some_and(|rule| rule.limit() == 0)
        {
            return Err(ConfigError::InvalidComplexityLimit);
        }

        self.validate_todo_rule()?;
        self.validate_restricted_call_rule()?;

        self.validate_direct_environment_read_rule()
    }

    fn validate_todo_rule(&self) -> Result<(), ConfigError> {
        let Some(rule) = &self.rules.todo_requires_reference else {
            return Ok(());
        };

        if rule.markers.is_empty() || any_blank(&rule.markers) {
            return Err(ConfigError::InvalidTodoMarkers);
        }

        if rule.reference_prefixes.is_empty()
            || rule
                .reference_prefixes
                .iter()
                .any(|prefix| prefix_is_unusable(prefix))
        {
            return Err(ConfigError::InvalidTodoReferencePrefixes);
        }

        Ok(())
    }

    fn validate_restricted_call_rule(&self) -> Result<(), ConfigError> {
        let Some(rule) = &self.rules.restricted_call else {
            return Ok(());
        };

        let mut seen = BTreeSet::new();

        for call in &rule.calls {
            if call.name.trim().is_empty() {
                return Err(ConfigError::InvalidRestrictedCallName);
            }

            if any_blank(&call.allow_in) {
                return Err(ConfigError::BlankAllowIn {
                    rule: "architecture/restricted-call",
                });
            }

            if !seen.insert(call.name.as_str()) {
                return Err(ConfigError::DuplicateRestrictedCallName {
                    name: call.name.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_direct_environment_read_rule(&self) -> Result<(), ConfigError> {
        if self
            .rules
            .direct_environment_read
            .as_ref()
            .is_some_and(|rule| any_blank(&rule.allow_in))
        {
            return Err(ConfigError::BlankAllowIn {
                rule: "security/direct-environment-read",
            });
        }

        Ok(())
    }
}

fn any_blank(values: &[String]) -> bool {
    values.iter().any(|value| value.trim().is_empty())
}

fn prefix_is_unusable(prefix: &str) -> bool {
    let trimmed = prefix.trim();

    trimmed.is_empty() || trimmed.chars().all(|character| character.is_ascii_digit())
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Parse { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::UnsupportedVersion { version } => {
                write!(formatter, "unsupported configuration version: {version}")
            }
            Self::InvalidComplexityLimit => {
                write!(
                    formatter,
                    "maintainability/decision-complexity max-complexity must be at least 1"
                )
            }
            Self::InvalidTodoMarkers => {
                write!(
                    formatter,
                    "policy/todo-requires-reference markers must not be empty"
                )
            }
            Self::InvalidTodoReferencePrefixes => {
                write!(
                    formatter,
                    "policy/todo-requires-reference reference-prefixes must not be empty or numeric"
                )
            }
            Self::InvalidRestrictedCallName => {
                write!(
                    formatter,
                    "architecture/restricted-call call names must not be blank"
                )
            }
            Self::DuplicateRestrictedCallName { name } => {
                write!(
                    formatter,
                    "architecture/restricted-call lists {name} more than once; \
                     one entry decides its allow-in boundary"
                )
            }
            Self::BlankAllowIn { rule } => {
                write!(formatter, "{rule} allow-in paths must not be blank")
            }
            Self::InvalidExclude { pattern } => {
                write!(formatter, "exclude pattern must not be blank: {pattern:?}")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::UnsupportedVersion { .. }
            | Self::InvalidComplexityLimit
            | Self::InvalidTodoMarkers
            | Self::InvalidTodoReferencePrefixes
            | Self::InvalidRestrictedCallName
            | Self::DuplicateRestrictedCallName { .. }
            | Self::BlankAllowIn { .. }
            | Self::InvalidExclude { .. } => None,
        }
    }
}
