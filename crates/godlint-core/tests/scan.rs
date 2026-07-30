#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::Path;

use godlint_core::scan::{MAX_SOURCE_BYTES, ScanReport, scan};

#[path = "support/temporary.rs"]
mod temporary;

use temporary::TemporaryDirectory;

struct Repository {
    directory: TemporaryDirectory,
}

impl Repository {
    fn new() -> Self {
        Self {
            directory: TemporaryDirectory::new("scan"),
        }
    }

    fn write(&self, relative_path: &str, contents: &str) {
        self.directory.write(relative_path, contents);
    }

    fn scan(&self) -> ScanReport {
        let root = self.directory.path();

        scan(root, &[root.to_path_buf()], &[])
            .unwrap_or_else(|error| panic!("scans repository: {error}"))
    }
}

fn statements(bytes: u64) -> String {
    let statement = "const value = 1;\n";
    let repeats = bytes as usize / statement.len();
    let mut source = statement.repeat(repeats);

    while (source.len() as u64) < bytes {
        source.push('\n');
    }

    source
}

#[test]
fn an_oversized_file_becomes_an_issue_instead_of_being_loaded() {
    let repository = Repository::new();

    repository.write("small.ts", "function reported() {}\n");
    repository.write("huge.ts", &statements(MAX_SOURCE_BYTES + 1));

    let report = repository.scan();
    let scanned: Vec<&Path> = report
        .facts
        .iter()
        .map(|facts| facts.source().path())
        .collect();

    assert_eq!(
        scanned,
        vec![Path::new("small.ts")],
        "the file above the limit must not be parsed"
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].path, Path::new("huge.ts"));
    assert!(
        report.issues[0].message.contains("scan limit"),
        "the issue must say why the file was skipped: {}",
        report.issues[0].message
    );
}

#[test]
fn a_file_at_the_limit_is_still_scanned() {
    let repository = Repository::new();

    repository.write("edge.ts", &statements(MAX_SOURCE_BYTES));

    let report = repository.scan();

    assert!(
        report.issues.is_empty(),
        "the limit is inclusive: {:?}",
        report.issues
    );
    assert_eq!(report.facts.len(), 1);
}
