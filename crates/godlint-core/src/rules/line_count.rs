use crate::source::{Language, SourceFile, SourceRange};

pub(crate) fn effective_line_count(
    source_file: &SourceFile,
    range: SourceRange,
    skip_blank_lines: bool,
    skip_comments: bool,
) -> usize {
    let source = &source_file.source()[range.start()..range.end()];
    let mut block_comment = false;

    source
        .lines()
        .filter(|line| {
            line_is_effective(
                line,
                source_file.language(),
                skip_blank_lines,
                skip_comments,
                &mut block_comment,
            )
        })
        .count()
}

fn line_is_effective(
    line: &str,
    language: Language,
    skip_blank_lines: bool,
    skip_comments: bool,
    block_comment: &mut bool,
) -> bool {
    if skip_blank_lines && line.trim().is_empty() {
        return false;
    }

    if !skip_comments {
        return true;
    }

    !is_comment_only(line, language, block_comment)
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
