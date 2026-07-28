use crate::{analyzers::SourceFacts, facts::CommentFact, source::SourceRange};

#[derive(Clone, Copy)]
pub(crate) struct Skipped {
    pub blank_lines: bool,
    pub comments: bool,
}

pub(crate) fn effective_line_count(
    facts: &SourceFacts,
    range: SourceRange,
    skipped: Skipped,
) -> u32 {
    let comments = facts.comments();
    let text = &facts.source().source()[range.start()..range.end()];
    let mut cursor = comments.partition_point(|comment| comment.range().end() <= range.start());
    let mut offset = range.start();
    let mut counted = 0_u32;

    for line in text.split_inclusive('\n') {
        let start = offset;

        offset += line.len();

        while cursor < comments.len() && comments[cursor].range().end() <= start {
            cursor += 1;
        }

        if line_is_counted(
            line.trim_end_matches(['\n', '\r']),
            start,
            &comments[cursor..],
            skipped,
        ) {
            counted += 1;
        }
    }

    counted
}

fn line_is_counted(line: &str, start: usize, comments: &[CommentFact], skipped: Skipped) -> bool {
    if skipped.blank_lines && line.trim().is_empty() {
        return false;
    }

    !(skipped.comments && line_is_commentary(line, start, comments))
}

fn line_is_commentary(line: &str, start: usize, comments: &[CommentFact]) -> bool {
    let mut has_content = false;

    for (index, character) in line.char_indices() {
        if character.is_whitespace() {
            continue;
        }

        has_content = true;

        if !is_inside(start + index, comments) {
            return false;
        }
    }

    has_content
}

fn is_inside(offset: usize, comments: &[CommentFact]) -> bool {
    comments
        .iter()
        .take_while(|comment| comment.range().start() <= offset)
        .any(|comment| offset < comment.range().end())
}
