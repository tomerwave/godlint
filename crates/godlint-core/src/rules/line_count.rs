use crate::{
    analyzers::SourceFacts,
    config::LineLimitRule,
    facts::CommentFact,
    source::{SourceRange, TextFile},
};

#[derive(Clone, Copy)]
pub(crate) struct Skipped {
    pub blank_lines: bool,
    pub comments: bool,
}

impl From<&LineLimitRule> for Skipped {
    fn from(configuration: &LineLimitRule) -> Self {
        Self {
            blank_lines: configuration.skip_blank_lines,
            comments: configuration.skip_comments,
        }
    }
}

pub(crate) fn effective_line_count(
    facts: &SourceFacts,
    range: SourceRange,
    skipped: Skipped,
) -> u32 {
    let comments = facts.comments();
    let mut cursor = comments.partition_point(|comment| comment.range().end() <= range.start());

    count_lines(facts.source().source(), range, skipped, |line, start| {
        while cursor < comments.len() && comments[cursor].range().end() <= start {
            cursor += 1;
        }

        line_is_commentary(line, start, &comments[cursor..])
    })
}

pub(crate) fn effective_script_line_count(
    file: &TextFile,
    range: SourceRange,
    skipped: Skipped,
) -> u32 {
    count_lines(file.text(), range, skipped, |line, _| {
        line.trim_start().starts_with('#')
    })
}

fn count_lines(
    source: &str,
    range: SourceRange,
    skipped: Skipped,
    mut is_commentary: impl FnMut(&str, usize) -> bool,
) -> u32 {
    let text = &source[range.start()..range.end()];
    let mut offset = range.start();
    let mut counted = 0_u32;

    for raw_line in text.split_inclusive('\n') {
        let start = offset;
        let line = raw_line.trim_end_matches(['\n', '\r']);

        offset += raw_line.len();

        if !(skipped.blank_lines && line.trim().is_empty())
            && !(skipped.comments && is_commentary(line, start))
        {
            counted += 1;
        }
    }

    counted
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
