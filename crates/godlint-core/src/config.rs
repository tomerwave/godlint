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

        Ok(())
    }

    fn function_size_limit_is_invalid(&self) -> bool {
        self.rules
            .function_size
            .as_ref()
            .is_some_and(|rule| rule.max_lines == 0)
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
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::UnsupportedVersion { .. } | Self::InvalidFunctionSizeLimit => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{Config, ConfigError};

    static NEXT_CONFIG_ID: AtomicU64 = AtomicU64::new(0);

    fn config_file(contents: &str) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let id = NEXT_CONFIG_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("godlint-config-{timestamp}-{id}.yaml"));

        fs::write(&path, contents).unwrap_or_else(|error| panic!("writes config: {error}"));

        path
    }

    fn load(contents: &str) -> Result<Config, ConfigError> {
        let path = config_file(contents);
        let result = Config::load(&path);

        fs::remove_file(path).unwrap_or_else(|error| panic!("removes config: {error}"));

        result
    }

    #[test]
    fn accepts_the_function_size_rule() {
        let result = load(
            "version: 1\nrules:\n  maintainability/function-size:\n    severity: error\n    max-lines: 30\n    skip-blank-lines: true\n    skip-comments: true\n",
        );

        assert!(result.is_ok());
    }

    #[test]
    fn rejects_an_unknown_rule() {
        let result = load("version: 1\nrules:\n  maintainability/unknown: {}\n");

        assert!(matches!(result, Err(ConfigError::Parse { .. })));
    }

    #[test]
    fn rejects_an_unknown_top_level_field() {
        let result = load("version: 1\nunknown: true\n");

        assert!(matches!(result, Err(ConfigError::Parse { .. })));
    }

    #[test]
    fn rejects_an_unsupported_version() {
        let result = load("version: 2\n");

        assert!(matches!(
            result,
            Err(ConfigError::UnsupportedVersion { version: 2 })
        ));
    }

    #[test]
    fn rejects_a_zero_function_size_limit() {
        let result = load(
            "version: 1\nrules:\n  maintainability/function-size:\n    severity: error\n    max-lines: 0\n    skip-blank-lines: true\n    skip-comments: true\n",
        );

        assert!(matches!(result, Err(ConfigError::InvalidFunctionSizeLimit)));
    }
}
