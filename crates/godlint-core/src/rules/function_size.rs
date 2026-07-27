use crate::{
    config::{FunctionSizeRule, Severity},
    facts::FunctionFact,
    rules::Rule,
    source::Language,
};

pub struct FunctionSize;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionSizeViolation {
    pub effective_line_count: usize,
}

impl Rule for FunctionSize {
    type Input = FunctionFact;
    type Configuration = FunctionSizeRule;
    type Violation = FunctionSizeViolation;

    const ID: &'static str = "maintainability/function-size";

    fn evaluate(
        function: &Self::Input,
        configuration: &Self::Configuration,
    ) -> Option<Self::Violation> {
        if configuration.severity == Severity::Off {
            return None;
        }

        let effective_line_count = effective_line_count(function, configuration);

        (effective_line_count > configuration.max_lines as usize).then_some(FunctionSizeViolation {
            effective_line_count,
        })
    }
}

fn effective_line_count(function: &FunctionFact, configuration: &FunctionSizeRule) -> usize {
    let range = function.range();
    let source = &function.source().source()[range.start()..range.end()];
    let mut block_comment = false;

    source
        .lines()
        .filter(|line| {
            line_is_effective(
                line,
                function.source().language(),
                configuration,
                &mut block_comment,
            )
        })
        .count()
}

fn line_is_effective(
    line: &str,
    language: Language,
    configuration: &FunctionSizeRule,
    block_comment: &mut bool,
) -> bool {
    if configuration.skip_blank_lines && line.trim().is_empty() {
        return false;
    }

    !configuration.skip_comments || !is_comment_only(line, language, block_comment)
}

fn is_comment_only(line: &str, language: Language, block_comment: &mut bool) -> bool {
    let mut remaining = line.trim_start();

    loop {
        if *block_comment {
            let Some(end) = remaining.find("*/") else {
                return true;
            };

            remaining = remaining[end + 2..].trim_start();
            *block_comment = false;

            if remaining.is_empty() {
                return true;
            }

            continue;
        }

        if remaining.is_empty() {
            return false;
        }

        if line_comment_marker(language).is_some_and(|marker| remaining.starts_with(marker)) {
            return true;
        }

        if supports_block_comments(language) && remaining.starts_with("/*") {
            remaining = &remaining[2..];
            *block_comment = true;

            continue;
        }

        return false;
    }
}

fn line_comment_marker(language: Language) -> Option<&'static str> {
    match language {
        Language::JavaScript | Language::Rust | Language::TypeScript => Some("//"),
        Language::Python => Some("#"),
    }
}

fn supports_block_comments(language: Language) -> bool {
    matches!(
        language,
        Language::JavaScript | Language::Rust | Language::TypeScript
    )
}
