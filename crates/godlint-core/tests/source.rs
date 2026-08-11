#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use godlint_core::source::{Language, SourceFile, SourceFileError, SourcePosition, TextFile};

#[test]
fn reports_the_line_an_offset_falls_on() {
    let source = SourceFile::new(PathBuf::from("src/example.rs"), "one\ntwo\n\nfour".into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));

    assert_eq!(source.line(0), 1);
    assert_eq!(source.line(3), 1);
    assert_eq!(source.line(4), 2);
    assert_eq!(source.line(8), 3);
    assert_eq!(source.line(9), 4);
    assert_eq!(
        source.line(source.source().len()),
        4,
        "the end of the file is on its last line"
    );
}

#[test]
fn detects_the_supported_languages() {
    assert_eq!(
        Language::from_path(Path::new("src/main.rs")),
        Some(Language::Rust)
    );
    assert_eq!(
        Language::from_path(Path::new("web/component.tsx")),
        Some(Language::TypeScript)
    );
    assert_eq!(
        Language::from_path(Path::new("web/component.jsx")),
        Some(Language::JavaScript)
    );
    assert_eq!(
        Language::from_path(Path::new("types.pyi")),
        Some(Language::Python)
    );
    assert_eq!(
        Language::from_path(Path::new("cmd/main.go")),
        Some(Language::Go)
    );
}

#[test]
fn keeps_repository_relative_source_identity() {
    let file = SourceFile::new(
        PathBuf::from("crates/core/src/lib.rs"),
        "fn main() {}".into(),
    )
    .unwrap_or_else(|error| panic!("creates source file: {error}"));

    assert_eq!(file.path(), Path::new("crates/core/src/lib.rs"));
    assert_eq!(file.language(), Language::Rust);
    assert_eq!(file.source(), "fn main() {}");
}

#[test]
fn rejects_paths_outside_the_repository() {
    let absolute = SourceFile::new(PathBuf::from("/tmp/example.rs"), String::new());
    let escaped = SourceFile::new(PathBuf::from("../example.rs"), String::new());

    assert!(matches!(
        absolute,
        Err(SourceFileError::AbsolutePath { .. })
    ));
    assert!(matches!(
        escaped,
        Err(SourceFileError::PathOutsideRepository { .. })
    ));
}

#[test]
fn rejects_an_unsupported_source_language() {
    let result = SourceFile::new(PathBuf::from("README.md"), String::new());

    assert!(matches!(
        result,
        Err(SourceFileError::UnsupportedLanguage { .. })
    ));
}

#[test]
fn converts_byte_ranges_to_one_based_source_locations() {
    let file = SourceFile::new(PathBuf::from("example.py"), "alpha\nβeta\n".into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));
    let range = file
        .range(6, 9)
        .unwrap_or_else(|error| panic!("creates range: {error}"));
    let location = file.location(range);

    assert_eq!(location.start, SourcePosition { line: 2, column: 1 });
    assert_eq!(location.end, SourcePosition { line: 2, column: 3 });
}

#[test]
fn rejects_reversed_or_invalid_source_ranges() {
    let file = SourceFile::new(PathBuf::from("example.rs"), "é".into())
        .unwrap_or_else(|error| panic!("creates source file: {error}"));

    assert!(matches!(
        file.range(3, 2),
        Err(SourceFileError::ReversedRange { .. })
    ));
    assert!(matches!(
        file.range(1, 1),
        Err(SourceFileError::InvalidUtf8Boundary { offset: 1 })
    ));
    assert!(matches!(
        file.range(3, 3),
        Err(SourceFileError::InvalidRange { .. })
    ));
}

#[test]
fn a_path_reads_with_forward_slashes_whatever_the_platform_uses() {
    let file = SourceFile::new(
        PathBuf::from("src").join("ui").join("widget.ts"),
        "export const value = 1;\n".into(),
    )
    .unwrap_or_else(|error| panic!("creates source file: {error}"));

    assert_eq!(
        file.path_text(),
        "src/ui/widget.ts",
        "a policy matches a glob written with forward slashes, so the path it sees uses them too"
    );
    assert!(
        !file.path_text().contains('\\'),
        "a native separator must not reach a policy"
    );
}

#[test]
fn slice_returns_the_text_a_range_covers() {
    let file = TextFile::new(PathBuf::from("src/a.ts"), "const value = 1;\n".into())
        .unwrap_or_else(|error| panic!("creates file: {error}"));
    let range = file
        .range(6, 11)
        .unwrap_or_else(|error| panic!("makes range: {error}"));

    assert_eq!(file.slice(range), "value");
    assert_eq!(file.slice(file.full_range()), "const value = 1;\n");
}

#[test]
#[should_panic(expected = "out of bounds")]
fn slice_refuses_a_range_from_another_file_loudly() {
    let short = TextFile::new(PathBuf::from("src/short.ts"), "a\n".into())
        .unwrap_or_else(|error| panic!("creates file: {error}"));
    let long = TextFile::new(PathBuf::from("src/long.ts"), "a longer file\n".into())
        .unwrap_or_else(|error| panic!("creates file: {error}"));

    let _ = short.slice(long.full_range());
}
