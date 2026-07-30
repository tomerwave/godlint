#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process,
    sync::atomic::{AtomicU64, Ordering},
};

use godlint_core::scan::{MAX_SOURCE_BYTES, ScanReport, scan};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

struct Repository {
    path: PathBuf,
}

impl Repository {
    fn new() -> Self {
        loop {
            let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("godlint-scan-{}-{id}", process::id()));

            match fs::create_dir(&path) {
                Ok(()) => return Self { path },
                Err(error) if error.kind() == ErrorKind::AlreadyExists => (),
                Err(error) => panic!("creates repository: {error}"),
            }
        }
    }

    fn write(&self, relative_path: &str, contents: &str) {
        fs::write(self.path.join(relative_path), contents)
            .unwrap_or_else(|error| panic!("writes {relative_path}: {error}"));
    }

    fn scan(&self) -> ScanReport {
        scan(&self.path, std::slice::from_ref(&self.path), &[])
            .unwrap_or_else(|error| panic!("scans repository: {error}"))
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
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
