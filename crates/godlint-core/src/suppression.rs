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

pub fn is_directive_only(text: &str) -> bool {
    let mut directives = 0;

    for (_, line) in lines(text) {
        if directive(line).is_some() {
            directives += 1;
        } else if !is_furniture_only(line) {
            return false;
        }
    }

    directives > 0
}

fn is_furniture_only(line: &str) -> bool {
    line.trim_start_matches(is_furniture).is_empty()
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
    let numbered: Vec<(usize, &str)> = lines(text).collect();

    numbered
        .iter()
        .enumerate()
        .filter_map(|(index, (offset, line))| {
            let found = directive(line)?;
            let start = comment_range.start() + offset + found.offset;

            Some(suppression(
                found.scope,
                found.arguments,
                SourceRange::new(start, start + found.length).ok()?,
                closing_lines(&numbered[index + 1..]),
                facts,
            ))
        })
        .collect()
}

fn closing_lines(rest: &[(usize, &str)]) -> usize {
    rest.iter()
        .take_while(|(_, line)| is_furniture_only(line))
        .count()
}

fn lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;

    text.split('\n').map(move |line| {
        let start = offset;

        offset += line.len() + 1;

        (start, line.strip_suffix('\r').unwrap_or(line))
    })
}

struct Directive<'a> {
    scope: Scope,
    arguments: &'a str,
    offset: usize,
    length: usize,
}

fn directive(line: &str) -> Option<Directive<'_>> {
    let opened = line.trim_start_matches(is_furniture);
    let offset = line.len() - opened.len();
    let body = opened.trim_end_matches(is_closing);

    DIRECTIVES.into_iter().find_map(|(keyword, scope)| {
        let arguments = body.strip_prefix(keyword)?;

        (arguments.is_empty() || arguments.starts_with(char::is_whitespace)).then_some(Directive {
            scope,
            arguments,
            offset,
            length: body.len(),
        })
    })
}

fn is_closing(character: char) -> bool {
    character.is_whitespace() || "*/\"'".contains(character)
}

fn is_furniture(character: char) -> bool {
    character.is_whitespace() || "/#*\"'!".contains(character)
}

fn suppression(
    scope: Scope,
    arguments: &str,
    range: SourceRange,
    closing: usize,
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
        covers: coverage(scope, line + closing, range, facts),
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
