use std::ops::RangeInclusive;

use crate::{
    analyzers::SourceFacts,
    facts::FunctionFact,
    rules::Finding,
    source::{SourceFile, SourceRange},
};

pub const NEXT_LINE: &str = "godlint-ignore-next-line";

pub const ENCLOSING: &str = "godlint-ignore-enclosing";

const SEPARATOR: &str = "--";

const OWNER: &str = "owner";

const EXPIRES: &str = "expires";

const DIRECTIVES: [(&str, Scope); 2] =
    [(NEXT_LINE, Scope::NextLine), (ENCLOSING, Scope::Enclosing)];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Scope {
    NextLine,
    Enclosing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Suppression {
    source: SourceFile,
    range: SourceRange,
    line: usize,
    scope: Scope,
    rules: Vec<String>,
    justification: Option<String>,
    owner: Option<String>,
    expires: Option<String>,
    unknown_options: Vec<String>,
    covers: Option<RangeInclusive<usize>>,
}

impl Scope {
    pub const fn directive(self) -> &'static str {
        match self {
            Self::NextLine => NEXT_LINE,
            Self::Enclosing => ENCLOSING,
        }
    }
}

impl Suppression {
    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn scope(&self) -> Scope {
        self.scope
    }

    pub fn rules(&self) -> &[String] {
        &self.rules
    }

    pub fn justification(&self) -> Option<&str> {
        self.justification.as_deref()
    }

    pub fn owner(&self) -> Option<&str> {
        self.owner.as_deref()
    }

    pub fn expires(&self) -> Option<&str> {
        self.expires.as_deref()
    }

    pub fn unknown_options(&self) -> &[String] {
        &self.unknown_options
    }

    pub fn resolves(&self) -> bool {
        self.covers.is_some()
    }

    pub fn covers_line(&self, line: usize) -> bool {
        self.covers
            .as_ref()
            .is_some_and(|lines| lines.contains(&line))
    }

    fn covers(&self, finding: &Finding) -> bool {
        self.source.path() == finding.path
            && self.covers_line(finding.line)
            && self.rules.iter().any(|rule| rule == finding.rule_id)
    }
}

pub fn collect(facts: &[SourceFacts]) -> Vec<Suppression> {
    let mut suppressions: Vec<Suppression> = facts
        .iter()
        .flat_map(|source_facts| {
            source_facts
                .comments()
                .iter()
                .flat_map(|comment| in_comment(comment.range(), comment.text(), source_facts))
        })
        .collect();

    suppressions.sort_by(|left, right| {
        (left.source.path(), left.line).cmp(&(right.source.path(), right.line))
    });

    suppressions
}

pub fn is_directive(text: &str) -> bool {
    lines(text).any(|(_, line)| directive(line).is_some())
}

pub fn apply(findings: Vec<Finding>, suppressions: &[Suppression]) -> Vec<Finding> {
    findings
        .into_iter()
        .filter(|finding| {
            !suppressions
                .iter()
                .any(|suppression| suppression.covers(finding))
        })
        .collect()
}

fn in_comment(comment_range: SourceRange, text: &str, facts: &SourceFacts) -> Vec<Suppression> {
    lines(text)
        .filter_map(|(offset, line)| {
            let (scope, arguments, keyword_offset) = directive(line)?;
            let start = comment_range.start() + offset + keyword_offset;

            Some(suppression(
                scope,
                arguments,
                SourceRange::new(start, comment_range.start() + offset + line.len()).ok()?,
                facts,
            ))
        })
        .collect()
}

fn lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;

    text.split('\n').map(move |line| {
        let start = offset;

        offset += line.len() + 1;

        (start, line.strip_suffix('\r').unwrap_or(line))
    })
}

fn directive(line: &str) -> Option<(Scope, &str, usize)> {
    let trimmed = line.trim_start_matches(is_furniture);
    let keyword_offset = line.len() - trimmed.len();

    DIRECTIVES.into_iter().find_map(|(keyword, scope)| {
        let rest = trimmed.strip_prefix(keyword)?;

        (rest.is_empty() || rest.starts_with(char::is_whitespace)).then_some((
            scope,
            rest,
            keyword_offset,
        ))
    })
}

fn is_furniture(character: char) -> bool {
    character.is_whitespace() || "/#*\"'!".contains(character)
}

fn suppression(
    scope: Scope,
    arguments: &str,
    range: SourceRange,
    facts: &SourceFacts,
) -> Suppression {
    let source = facts.source();
    let line = source.line(range.start());
    let (head, justification) = split_justification(arguments);
    let mut parsed = Arguments::default();

    for token in head.split_whitespace() {
        parsed.absorb(token);
    }

    Suppression {
        source: source.clone(),
        range,
        line,
        scope,
        rules: parsed.rules,
        justification: justification.map(str::to_owned),
        owner: parsed.owner,
        expires: parsed.expires,
        unknown_options: parsed.unknown_options,
        covers: coverage(scope, line, range, facts),
    }
}

#[derive(Default)]
struct Arguments {
    rules: Vec<String>,
    owner: Option<String>,
    expires: Option<String>,
    unknown_options: Vec<String>,
}

impl Arguments {
    fn absorb(&mut self, token: &str) {
        let Some((key, value)) = token.split_once('=') else {
            if self.rules.is_empty() {
                self.rules = rule_list(token);
            } else {
                self.unknown_options.push(token.to_owned());
            }

            return;
        };

        match key {
            OWNER => self.owner = Some(value.to_owned()),
            EXPIRES => self.expires = Some(value.to_owned()),
            _ => self.unknown_options.push(key.to_owned()),
        }
    }
}

fn rule_list(token: &str) -> Vec<String> {
    token
        .split(',')
        .map(str::trim)
        .filter(|rule| !rule.is_empty())
        .map(str::to_owned)
        .collect()
}

fn split_justification(arguments: &str) -> (&str, Option<&str>) {
    let Some(index) = separator(arguments) else {
        return (arguments, None);
    };
    let justification = arguments[index + SEPARATOR.len()..].trim();

    (
        &arguments[..index],
        (!justification.is_empty()).then_some(justification),
    )
}

fn separator(text: &str) -> Option<usize> {
    text.match_indices(SEPARATOR)
        .find(|(index, _)| is_standalone(text, *index))
        .map(|(index, _)| index)
}

fn is_standalone(text: &str, index: usize) -> bool {
    let before = text[..index].chars().next_back();
    let after = text[index + SEPARATOR.len()..].chars().next();

    before.is_none_or(char::is_whitespace) && after.is_none_or(char::is_whitespace)
}

fn coverage(
    scope: Scope,
    line: usize,
    range: SourceRange,
    facts: &SourceFacts,
) -> Option<RangeInclusive<usize>> {
    match scope {
        Scope::NextLine => Some(line + 1..=line + 1),
        Scope::Enclosing => enclosing(range, facts).map(|function| {
            let source = facts.source();

            source.line(function.range().start())..=source.line(function.range().end())
        }),
    }
}

fn enclosing(range: SourceRange, facts: &SourceFacts) -> Option<&FunctionFact> {
    facts
        .functions()
        .iter()
        .filter(|function| {
            function.range().start() <= range.start() && range.end() <= function.range().end()
        })
        .min_by_key(|function| function.range().end() - function.range().start())
}
