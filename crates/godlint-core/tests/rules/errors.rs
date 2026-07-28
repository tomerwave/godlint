use std::error::Error;

use godlint_core::{rules::RuleError, source::SourceFileError};

fn error() -> RuleError {
    RuleError::LocatesSource {
        source: SourceFileError::InvalidUtf8Boundary { offset: 7 },
    }
}

#[test]
fn describes_a_source_location_failure() {
    let message = error().to_string();

    assert!(message.contains("invalid source file"), "{message}");
    assert!(message.contains("UTF-8 boundary"), "{message}");
}

#[test]
fn exposes_the_underlying_source_error() {
    let reported = error();
    let source = reported.source().map(ToString::to_string);

    assert_eq!(
        source.as_deref(),
        Some("source offset is not on a UTF-8 boundary: 7")
    );
}
