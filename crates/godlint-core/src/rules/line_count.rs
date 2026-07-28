use crate::{analyzers::SourceFacts, source::SourceRange};

pub(crate) fn effective_line_count(
    facts: &SourceFacts,
    range: SourceRange,
    skip_blank_lines: bool,
    skip_comments: bool,
) -> u32 {
    let source = facts.source();
    let commentary = commentary_within(facts, range);
    let text = &source.source()[range.start()..range.end()];
    let mut offset = range.start();
    let mut counted = 0_u32;

    for line in text.split_inclusive('\n') {
        let start = offset;

        offset += line.len();

        if line_is_counted(
            line.trim_end_matches(['\n', '\r']),
            start,
            &commentary,
            skip_blank_lines,
            skip_comments,
        ) {
            counted += 1;
        }
    }

    counted
}

fn commentary_within(facts: &SourceFacts, range: SourceRange) -> Vec<SourceRange> {
    facts
        .comments()
        .iter()
        .map(|comment| comment.range())
        .filter(|comment| comment.start() < range.end() && comment.end() > range.start())
        .collect()
}

fn line_is_counted(
    line: &str,
    start: usize,
    commentary: &[SourceRange],
    skip_blank_lines: bool,
    skip_comments: bool,
) -> bool {
    if skip_blank_lines && line.trim().is_empty() {
        return false;
    }

    !(skip_comments && line_is_commentary(line, start, commentary))
}

fn line_is_commentary(line: &str, start: usize, commentary: &[SourceRange]) -> bool {
    let mut has_content = false;

    for (index, character) in line.char_indices() {
        if character.is_whitespace() {
            continue;
        }

        has_content = true;

        if !is_inside(start + index, commentary) {
            return false;
        }
    }

    has_content
}

fn is_inside(offset: usize, commentary: &[SourceRange]) -> bool {
    commentary
        .iter()
        .any(|comment| comment.start() <= offset && offset < comment.end())
}
