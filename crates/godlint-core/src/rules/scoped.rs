use crate::glob;

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
