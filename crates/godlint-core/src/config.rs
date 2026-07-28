use std::{
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u8,
    #[serde(default)]
    pub rules: Rules,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rules {
    #[serde(rename = "maintainability/function-size")]
    pub function_size: Option<FunctionSizeRule>,
    #[serde(rename = "maintainability/function-nesting")]
    pub function_nesting: Option<FunctionNestingRule>,
    #[serde(rename = "maintainability/file-size")]
    pub file_size: Option<FileSizeRule>,
    #[serde(rename = "maintainability/empty-function")]
    pub empty_function: Option<EmptyFunctionRule>,
    #[serde(rename = "policy/todo-requires-reference")]
    pub todo_requires_reference: Option<TodoRequiresReferenceRule>,
    #[serde(rename = "maintainability/parameter-count")]
    pub parameter_count: Option<ParameterCountRule>,
    #[serde(rename = "maintainability/cyclomatic-complexity")]
    pub cyclomatic_complexity: Option<CyclomaticComplexityRule>,
    #[serde(rename = "maintainability/return-count")]
    pub return_count: Option<ReturnCountRule>,
    #[serde(rename = "maintainability/function-statements")]
    pub function_statements: Option<FunctionStatementsRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionSizeRule {
    pub severity: Severity,
    #[serde(rename = "max-lines")]
    pub max_lines: u32,
    #[serde(rename = "skip-blank-lines")]
    pub skip_blank_lines: bool,
    #[serde(rename = "skip-comments")]
    pub skip_comments: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionNestingRule {
    pub severity: Severity,
    #[serde(rename = "max-depth")]
    pub max_depth: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSizeRule {
    pub severity: Severity,
    #[serde(rename = "max-lines")]
    pub max_lines: u32,
    #[serde(rename = "skip-blank-lines")]
    pub skip_blank_lines: bool,
    #[serde(rename = "skip-comments")]
    pub skip_comments: bool,
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
pub struct TodoRequiresReferenceRule {
    pub severity: Severity,
    #[serde(default = "default_reference_prefixes", rename = "reference-prefixes")]
    pub reference_prefixes: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParameterCountRule {
    pub severity: Severity,
    #[serde(rename = "max-parameters")]
    pub max_parameters: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CyclomaticComplexityRule {
    pub severity: Severity,
    #[serde(rename = "max-complexity")]
    pub max_complexity: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReturnCountRule {
    pub severity: Severity,
    #[serde(rename = "max-returns")]
    pub max_returns: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FunctionStatementsRule {
    pub severity: Severity,
    #[serde(rename = "max-statements")]
    pub max_statements: u32,
}

fn default_reference_prefixes() -> Vec<String> {
    vec!["#".into()]
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
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
    InvalidFunctionSizeLimit,
    InvalidFileSizeLimit,
    InvalidTodoReferencePrefixes,
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

    fn validate(&self) -> Result<(), ConfigError> {
        if self.version != 1 {
            return Err(ConfigError::UnsupportedVersion {
                version: self.version,
            });
        }

        if self.function_size_limit_is_invalid() {
            return Err(ConfigError::InvalidFunctionSizeLimit);
        }

        if self.file_size_limit_is_invalid() {
            return Err(ConfigError::InvalidFileSizeLimit);
        }

        if self.todo_reference_prefixes_are_invalid() {
            return Err(ConfigError::InvalidTodoReferencePrefixes);
        }

        Ok(())
    }

    fn function_size_limit_is_invalid(&self) -> bool {
        self.rules
            .function_size
            .as_ref()
            .is_some_and(|rule| rule.max_lines == 0)
    }

    fn file_size_limit_is_invalid(&self) -> bool {
        self.rules
            .file_size
            .as_ref()
            .is_some_and(|rule| rule.max_lines == 0)
    }

    fn todo_reference_prefixes_are_invalid(&self) -> bool {
        self.rules
            .todo_requires_reference
            .as_ref()
            .is_some_and(|rule| {
                rule.reference_prefixes.is_empty()
                    || rule
                        .reference_prefixes
                        .iter()
                        .any(|prefix| prefix.trim().is_empty())
            })
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Parse { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::UnsupportedVersion { version } => {
                write!(formatter, "unsupported configuration version: {version}")
            }
            Self::InvalidFunctionSizeLimit => {
                write!(
                    formatter,
                    "maintainability/function-size max-lines must be at least 1"
                )
            }
            Self::InvalidFileSizeLimit => {
                write!(
                    formatter,
                    "maintainability/file-size max-lines must be at least 1"
                )
            }
            Self::InvalidTodoReferencePrefixes => {
                write!(
                    formatter,
                    "policy/todo-requires-reference reference-prefixes must not be empty"
                )
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
            | Self::InvalidFunctionSizeLimit
            | Self::InvalidFileSizeLimit
            | Self::InvalidTodoReferencePrefixes => None,
        }
    }
}
