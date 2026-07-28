use std::{error::Error, fmt, path::PathBuf};

use crate::{
    config::{Config, Severity},
    facts::FunctionFact,
    source::SourceFileError,
};

pub trait Rule {
    type Input;
    type Configuration;
    type Violation;

    const ID: &'static str;

    fn evaluate(
        input: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation>;
}

pub mod function_size;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finding {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub rule_id: &'static str,
    pub message: String,
}

#[derive(Debug)]
pub enum RuleError {
    LocatesSource { source: SourceFileError },
}

/// Evaluates every configured rule against the language-neutral facts.
pub fn evaluate(functions: &[FunctionFact], config: &Config) -> Result<Vec<Finding>, RuleError> {
    let Some(configuration) = &config.rules.function_size else {
        return Ok(Vec::new());
    };

    let mut findings = function_size::evaluate(functions, configuration)?;

    findings.sort_by(|left, right| {
        (
            &left.path,
            left.line,
            left.column,
            left.rule_id,
            &left.message,
        )
            .cmp(&(
                &right.path,
                right.line,
                right.column,
                right.rule_id,
                &right.message,
            ))
    });

    Ok(findings)
}

impl fmt::Display for RuleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LocatesSource { source } => write!(formatter, "invalid source file: {source}"),
        }
    }
}

impl Error for RuleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LocatesSource { source } => Some(source),
        }
    }
}
