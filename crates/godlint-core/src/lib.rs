//! Core contracts shared by the Godlint command-line interface and future analyzers.
//!
//! This crate intentionally contains only the project version during the workspace
//! foundation milestone. Scan orchestration and policy contracts are introduced in the
//! next vertical-slice steps.

/// The version of the Godlint workspace package.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn exposes_a_non_empty_version() {
        assert!(!VERSION.is_empty());
    }
}
