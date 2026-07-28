//! A file that satisfies every configured rule.

/// Sums the values that pass `keep`.
fn total(values: &[u32]) -> u32 {
    values.iter().copied().filter(|value| keep(*value)).sum()
}

/// Reports whether a value contributes to the total.
fn keep(value: u32) -> bool {
    value > 0
}
