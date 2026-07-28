use crate::{
    analyzers::SourceFacts,
    config::{Config, Severity, TodoRequiresReferenceRule},
    facts::CommentFact,
    rules::{
        CommentRule, Finding, Rule, RuleError, Violation, evaluate_comment_rule, when_configured,
    },
    source::SourceRange,
};

pub struct TodoRequiresReference;

/// One marker found in a comment, with the text that belongs to it.
struct Marker<'a> {
    name: &'a str,
    start: usize,
    end: usize,
}

impl Rule for TodoRequiresReference {
    const ID: &'static str = "policy/todo-requires-reference";

    type Configuration = TodoRequiresReferenceRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CommentRule for TodoRequiresReference {
    /// Reports each unreferenced marker at its own position.
    ///
    /// Markers are matched on word boundaries, so `AUTODOWNLOAD` is not a marker, and
    /// each marker owns only the text up to the next one, so one issue reference cannot
    /// excuse every other marker in the same comment.
    fn check(
        comment: &CommentFact,
        configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        let text = comment.text();
        let markers = markers(text, &configuration.markers);

        markers
            .iter()
            .enumerate()
            .filter(|(index, marker)| {
                let until = markers.get(index + 1).map_or(text.len(), |next| next.start);

                !has_reference(&text[marker.end..until], &configuration.reference_prefixes)
            })
            .filter_map(|(_, marker)| {
                let start = comment.range().start() + marker.start;
                let range = SourceRange::new(start, start + marker.name.len()).ok()?;

                Some((
                    range,
                    Violation::MissingReference {
                        marker: marker.name.to_owned(),
                    },
                ))
            })
            .collect()
    }
}

/// Finds configured markers appearing as whole words, in source order.
fn markers<'a>(text: &str, names: &'a [String]) -> Vec<Marker<'a>> {
    let mut found: Vec<Marker<'a>> = names
        .iter()
        .flat_map(|name| {
            word_positions(text, name).map(move |start| Marker {
                name,
                start,
                end: start + name.len(),
            })
        })
        .collect();

    found.sort_by_key(|marker| marker.start);

    found
}

fn word_positions<'a>(text: &'a str, needle: &'a str) -> impl Iterator<Item = usize> + 'a {
    text.match_indices(needle)
        .filter(move |(index, _)| is_word_bounded(text, *index, needle.len()))
        .map(|(index, _)| index)
}

/// Reports whether the span is delimited by non-word characters on both sides.
fn is_word_bounded(text: &str, start: usize, length: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[start + length..].chars().next();

    !before.is_some_and(is_word_character) && !after.is_some_and(is_word_character)
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

/// Reports whether `text` carries an issue reference for one of `prefixes`.
fn has_reference(text: &str, prefixes: &[String]) -> bool {
    prefixes
        .iter()
        .any(|prefix| references_with_prefix(text, prefix))
}

fn references_with_prefix(text: &str, prefix: &str) -> bool {
    text.match_indices(prefix).any(|(index, _)| {
        prefix_is_delimited(text, index) && digits_follow(&text[index + prefix.len()..])
    })
}

/// Requires the prefix to start a word so that `NOTJIRA-42` is not a `JIRA-` reference.
fn prefix_is_delimited(text: &str, index: usize) -> bool {
    !text[..index]
        .chars()
        .next_back()
        .is_some_and(char::is_alphanumeric)
}

/// Requires digits that end the token.
///
/// An issue number is digits and nothing else, which keeps a colour such as `#3366ff`
/// from reading as accountability.
fn digits_follow(text: &str) -> bool {
    let digits = text.chars().take_while(char::is_ascii_digit).count();

    if digits == 0 {
        return false;
    }

    !text[digits..]
        .chars()
        .next()
        .is_some_and(char::is_alphanumeric)
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Result<Vec<Finding>, RuleError> {
    when_configured(
        config.rules.todo_requires_reference.as_ref(),
        |configuration| evaluate_comment_rule::<TodoRequiresReference>(facts, configuration),
    )
}
