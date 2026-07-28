fn guarded(value: Option<u32>) -> u32 {
    match value {
        Some(n) if n > 100 => 1,
        Some(n) if n > 50 => 2,
        Some(n) if n > 10 => 3,
        Some(_) => 4,
        None => 0,
    }
}
