pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn exposes_a_non_empty_version() {
        assert!(!VERSION.is_empty());
    }
}
