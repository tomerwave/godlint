use std::collections::BTreeSet;

use crate::{
    analyzers::SourceFacts,
    facts::{CommentKind, FunctionFact},
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

struct Comment<'a> {
    range: SourceRange,
    text: &'a str,
    kind: CommentKind,
}

struct Scan<'a> {
    occupied: &'a BTreeSet<usize>,
    facts: &'a SourceFacts,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Covers {
    Line(usize),
    Declaration {
        range: SourceRange,
        nested: Vec<SourceRange>,
    },
}

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
    repeated_options: Vec<String>,
    covers: Option<Covers>,
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

    pub fn repeated_options(&self) -> &[String] {
        &self.repeated_options
    }

    pub fn resolves(&self) -> bool {
        self.covers.is_some()
    }

    pub fn covers(&self, finding: &Finding) -> bool {
        self.source.path() == finding.path
            && self.rules.iter().any(|rule| rule == finding.rule_id)
            && self.reaches(finding)
    }

    fn reaches(&self, finding: &Finding) -> bool {
        match &self.covers {
            Some(Covers::Line(line)) => *line == finding.line,
            Some(Covers::Declaration { range, nested }) => {
                encloses(*range, finding.range)
                    && !nested.iter().any(|inner| encloses(*inner, finding.range))
            }
            None => false,
        }
    }
}

pub fn collect(facts: &[SourceFacts]) -> Vec<Suppression> {
    let mut suppressions: Vec<Suppression> = facts
        .iter()
        .flat_map(|source_facts| {
            let occupied = directive_lines(source_facts);

            source_facts.comments().iter().flat_map(move |comment| {
                in_comment(
                    Comment {
                        range: comment.range(),
                        text: comment.text(),
                        kind: comment.kind(),
                    },
                    &Scan {
                        occupied: &occupied,
                        facts: source_facts,
                    },
                )
            })
        })
        .collect();

    suppressions.sort_by(|left, right| {
        (left.source.path(), left.line).cmp(&(right.source.path(), right.line))
    });

    suppressions
}

pub fn is_directive_only(text: &str, kind: CommentKind) -> bool {
    let mut directives = 0;

    for (index, (_, line)) in lines(text).enumerate() {
        if directive(line, kind, index == 0).is_some() {
            directives += 1;
        } else if !is_furniture_only(line, kind) {
            return false;
        }
    }

    directives > 0
}

fn is_furniture_only(line: &str, kind: CommentKind) -> bool {
    line.trim_start_matches(|character| is_furniture(character, quotes_delimit(kind)))
        .is_empty()
}

fn quotes_delimit(kind: CommentKind) -> bool {
    kind == CommentKind::Docstring
}

fn quotes_open(kind: CommentKind, opens: bool) -> bool {
    opens && quotes_delimit(kind)
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

fn directive_lines(facts: &SourceFacts) -> BTreeSet<usize> {
    let source = facts.source();

    facts
        .comments()
        .iter()
        .flat_map(|comment| {
            lines(comment.text())
                .enumerate()
                .filter(|(index, (_, line))| directive(line, comment.kind(), *index == 0).is_some())
                .map(|(_, (offset, _))| source.line(comment.range().start() + offset))
                .collect::<Vec<_>>()
        })
        .collect()
}

fn in_comment(comment: Comment<'_>, scan: &Scan<'_>) -> Vec<Suppression> {
    let numbered: Vec<(usize, &str)> = lines(comment.text).collect();

    numbered
        .iter()
        .enumerate()
        .filter_map(|(index, (offset, line))| {
            let found = directive(line, comment.kind, index == 0)?;
            let start = comment.range.start() + offset + found.offset;

            Some(suppression(
                found,
                SourceRange::new(start, start + found.length).ok()?,
                spent_lines(&numbered[index + 1..], comment.kind),
                scan,
            ))
        })
        .collect()
}

fn spent_lines(rest: &[(usize, &str)], kind: CommentKind) -> usize {
    rest.iter()
        .take_while(|(_, line)| {
            is_furniture_only(line, kind) || directive(line, kind, false).is_some()
        })
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

#[derive(Clone, Copy)]
struct Directive<'a> {
    scope: Scope,
    arguments: &'a str,
    offset: usize,
    length: usize,
}

fn directive(line: &str, kind: CommentKind, opens: bool) -> Option<Directive<'_>> {
    let quotes = quotes_open(kind, opens);
    let opened = line.trim_start_matches(|character| is_furniture(character, quotes));
    let offset = line.len() - opened.len();
    let body = opened.trim_end_matches(|character| is_closing(character, quotes));

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

fn is_closing(character: char, quotes: bool) -> bool {
    character.is_whitespace() || "*/".contains(character) || (quotes && "\"'".contains(character))
}

fn is_furniture(character: char, quotes: bool) -> bool {
    character.is_whitespace() || "/#*!".contains(character) || (quotes && "\"'".contains(character))
}

fn suppression(
    found: Directive<'_>,
    range: SourceRange,
    spent: usize,
    scan: &Scan<'_>,
) -> Suppression {
    let source = scan.facts.source();
    let line = source.line(range.start());
    let (head, justification) = split_justification(found.arguments);
    let mut parsed = Arguments::default();

    for token in head.split_whitespace() {
        parsed.absorb(token);
    }

    Suppression {
        source: source.clone(),
        range,
        line,
        scope: found.scope,
        rules: parsed.rules,
        justification: justification.map(str::to_owned),
        owner: parsed.owner,
        expires: parsed.expires,
        unknown_options: parsed.unknown_options,
        repeated_options: parsed.repeated_options,
        covers: coverage(found.scope, line + spent, range, scan),
    }
}

#[derive(Default)]
struct Arguments {
    rules: Vec<String>,
    owner: Option<String>,
    expires: Option<String>,
    unknown_options: Vec<String>,
    repeated_options: Vec<String>,
}

impl Arguments {
    fn absorb(&mut self, token: &str) {
        let Some((key, value)) = token.split_once('=') else {
            self.absorb_bare(token);

            return;
        };

        self.absorb_option(key, value);
    }

    fn absorb_bare(&mut self, token: &str) {
        if self.rules.is_empty() {
            self.rules = rule_list(token);
        } else {
            self.unknown_options.push(token.to_owned());
        }
    }

    fn absorb_option(&mut self, key: &str, value: &str) {
        let slot = match key {
            OWNER => &mut self.owner,
            EXPIRES => &mut self.expires,
            _ => {
                self.unknown_options.push(key.to_owned());

                return;
            }
        };

        if slot.is_some() {
            self.repeated_options.push(key.to_owned());

            return;
        }

        if !value.is_empty() {
            *slot = Some(value.to_owned());
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

fn coverage(scope: Scope, line: usize, range: SourceRange, scan: &Scan<'_>) -> Option<Covers> {
    match scope {
        Scope::NextLine => {
            let mut target = line + 1;

            while scan.occupied.contains(&target) {
                target += 1;
            }

            Some(Covers::Line(target))
        }
        Scope::Enclosing => enclosing(range, scan.facts).map(|function| Covers::Declaration {
            range: function.range(),
            nested: nested_functions(function.range(), scan.facts),
        }),
    }
}

fn nested_functions(declaration: SourceRange, facts: &SourceFacts) -> Vec<SourceRange> {
    facts
        .functions()
        .iter()
        .map(FunctionFact::range)
        .filter(|inner| inner != &declaration && encloses(declaration, *inner))
        .collect()
}

fn encloses(outer: SourceRange, inner: SourceRange) -> bool {
    outer.start() <= inner.start() && inner.end() <= outer.end()
}

fn enclosing(range: SourceRange, facts: &SourceFacts) -> Option<&FunctionFact> {
    facts
        .functions()
        .iter()
        .filter(|function| encloses(function.range(), range))
        .min_by_key(|function| function.range().end() - function.range().start())
}
