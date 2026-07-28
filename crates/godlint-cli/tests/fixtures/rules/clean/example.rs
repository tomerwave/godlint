//! A file that satisfies every configured rule.

/// Sums the values that pass `keep`.
fn total(values: &[u32]) -> u32 {
    values.iter().copied().filter(|value| keep(*value)).sum()
}

fn keep(value: u32) -> bool {
    value > 0
}
