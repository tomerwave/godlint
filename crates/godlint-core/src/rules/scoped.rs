use crate::{config::Layer, facts::ImportFact, glob, rules::module_path};

pub(crate) fn most_specific<T>(items: &[T], reach: impl Fn(&T) -> Option<usize>) -> Option<usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| reach(item).map(|length| (length, index)))
        .max()
        .map(|(_, index)| index)
}

pub(crate) fn longest_match(patterns: &[String], path: &str) -> Option<usize> {
    patterns
        .iter()
        .filter(|pattern| glob::matches_any(std::iter::once(pattern.as_str()), path))
        .map(String::len)
        .max()
}

pub(crate) fn endpoints(members: &[Layer], import: &ImportFact) -> Option<(usize, usize)> {
    let from = most_specific(members, |member| holds(member, import))?;
    let to = most_specific(members, |member| names(member, import))?;

    Some((from, to))
}

fn holds(member: &Layer, import: &ImportFact) -> Option<usize> {
    longest_match(&member.paths, import.source().path_text())
}

fn names(member: &Layer, import: &ImportFact) -> Option<usize> {
    let module = import.module();
    let language = import.source().language();

    member
        .modules
        .iter()
        .filter(|spelling| module_path::covers(spelling, module, language))
        .map(String::len)
        .max()
}
