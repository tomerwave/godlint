#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::Path;

use godlint_core::scan::{MAX_SOURCE_BYTES, ScanError, ScanReport, scan};

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

    fn scanned_paths(&self) -> Vec<String> {
        let report = self.scan();

        assert!(report.issues.is_empty(), "{:?}", report.issues);

        let mut paths: Vec<String> = report
            .facts
            .iter()
            .map(|facts| facts.source().path().display().to_string())
            .chain(
                report
                    .workflows
                    .iter()
                    .map(|workflow| workflow.file().path().display().to_string()),
            )
            .collect();

        paths.sort();
        paths
            .into_iter()
            .map(|path| path.replace('\\', "/"))
            .collect()
    }

    fn scanned_beside(&self, files: &[&str]) -> Vec<String> {
        for file in files {
            self.write(file, "export const value = 1;\n");
        }

        self.scanned_paths()
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

#[test]
fn a_file_well_below_the_limit_is_not_rejected() {
    let repository = Repository::new();

    repository.write("mid.ts", &statements(2_000_000));

    let report = repository.scan();

    assert!(report.issues.is_empty(), "{:?}", report.issues);
    assert_eq!(report.facts.len(), 1);
}

#[test]
fn a_scan_collects_workflows_beside_the_source_it_finds() {
    let repository = Repository::new();

    repository.write("source.rs", "fn main() {}\n");
    repository.write(
        ".github/workflows/ci.yml",
        "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n",
    );
    repository.write(".github/dependabot.yml", "version: 2\n");

    let report = repository.scan();
    let workflows: Vec<&Path> = report
        .workflows
        .iter()
        .map(|workflow| workflow.file().path())
        .collect();

    assert!(report.issues.is_empty(), "{:?}", report.issues);
    assert_eq!(report.facts.len(), 1, "the source file is still scanned");
    assert_eq!(
        workflows,
        vec![Path::new(".github/workflows/ci.yml")],
        "only what GitHub will run is a workflow"
    );
    assert_eq!(report.workflows[0].actions().len(), 1);
}

#[test]
fn an_excluded_workflow_is_not_read() {
    let repository = Repository::new();

    repository.write(
        ".github/workflows/ci.yml",
        "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n",
    );

    let root = repository.directory.path();
    let report = scan(root, &[root.to_path_buf()], &[".github".to_owned()])
        .unwrap_or_else(|error| panic!("scans repository: {error}"));

    assert!(
        report.workflows.is_empty(),
        "a workflow is subject to the same exclusions as source"
    );
}

#[test]
fn a_workflow_it_cannot_read_becomes_an_issue_rather_than_silence() {
    let repository = Repository::new();

    repository.write(
        ".github/workflows/torn.yml",
        "jobs:\n  build:\n    steps:\n      - run: echo \"a: b\"\n",
    );

    let report = repository.scan();

    assert_eq!(report.workflows.len(), 1, "what parsed is still available");
    assert_eq!(report.issues.len(), 1, "{:?}", report.issues);
    assert!(
        report.issues[0].message.contains("syntax not recognised"),
        "a workflow that contributes nothing must leave a trace: {:?}",
        report.issues[0]
    );
    assert!(!report.issues[0].message.contains("more place"));
}

#[test]
fn a_missing_input_reports_the_discovery_error() {
    let repository = Repository::new();
    let missing = repository.directory.path().join("missing");

    let error = scan(
        repository.directory.path(),
        std::slice::from_ref(&missing),
        &[],
    )
    .expect_err("a missing input path must fail the scan");

    assert!(matches!(error, ScanError::DiscoversFiles { .. }));
    assert!(error.to_string().contains("unable to discover files"));
    assert!(error.to_string().contains("missing"));
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn a_file_outside_the_root_names_the_source_path_error() {
    let repository = Repository::new();
    let outside_directory = TemporaryDirectory::new("scan-outside");
    let outside = outside_directory.write("outside.rs", "fn outside() {}\n");

    let error = scan(
        repository.directory.path(),
        std::slice::from_ref(&outside),
        &[],
    )
    .expect_err("a source outside the root must fail the scan");

    assert!(matches!(error, ScanError::SourcePath { .. }));
    assert!(
        error
            .to_string()
            .contains("source file is outside scan root")
    );
    assert!(error.to_string().contains("outside.rs"));
    assert!(std::error::Error::source(&error).is_none());
}

#[cfg(unix)]
fn set_access(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))
        .unwrap_or_else(|error| panic!("sets directory permissions: {error}"));
}

#[cfg(unix)]
#[test]
fn an_unreadable_nested_directory_becomes_a_relative_scan_issue() {
    let repository = Repository::new();
    repository.write("readable.ts", "const value = 1;\n");
    repository.write("denied/hidden.ts", "const value = 2;\n");

    let denied = repository.directory.path().join("denied");
    set_access(&denied, 0o000);

    if std::fs::read_dir(&denied).is_ok() {
        set_access(&denied, 0o700);
        panic!("the test cannot prove degradation while the directory is readable");
    }

    let report = repository.scan();

    set_access(&denied, 0o700);

    assert_eq!(report.facts.len(), 1);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].path, Path::new("denied"));
    assert!(report.issues[0].message.contains("unable to discover"));
}

fn wide_repository(count: usize) -> Repository {
    let repository = Repository::new();

    for index in 0..count {
        repository.write(
            &format!("src/file{index:03}.ts"),
            &format!("export const value{index} = {index};\n"),
        );
    }
    repository.write("src/aaa-torn.ts", "export const value = ;\n");
    repository.write("src/zzz-torn.ts", "export const value = ;\n");
    repository.write(
        ".github/workflows/one.yml",
        "name: One\non: [push]\njobs: {}\n",
    );

    repository
}

#[test]
fn a_tree_wide_enough_to_split_across_workers_keeps_its_order() {
    let count = 200;
    let report = wide_repository(count).scan();
    let scanned: Vec<String> = report
        .facts
        .iter()
        .map(|facts| facts.source().path_text().to_owned())
        .collect();
    let mut expected = scanned.clone();
    expected.sort();

    assert_eq!(report.facts.len(), count + 2);
    assert_eq!(report.workflows.len(), 1);
    assert_eq!(
        scanned, expected,
        "facts must come back in path order however the work was divided"
    );
    let torn: Vec<String> = report
        .issues
        .iter()
        .map(|issue| issue.path.display().to_string())
        .collect();

    assert_eq!(
        torn,
        vec!["src/aaa-torn.ts".to_owned(), "src/zzz-torn.ts".to_owned()],
        "issues from files in different chunks keep their order"
    );
}

#[test]
fn a_file_whose_header_declares_it_generated_is_not_analysed() {
    let repository = Repository::new();

    repository.write("authored.rs", "fn authored() {}\n");
    repository.write(
        "models.go",
        "// Code generated by sqlc. DO NOT EDIT.\n\npackage models\n",
    );
    repository.write(
        "index.ts",
        "// *** WARNING: this file was generated by pulumi-language-nodejs. ***\n\
         // *** Do not edit by hand unless you're certain you know what you are doing! ***\n\
         export const value = 1;\n",
    );
    repository.write(
        "provider.py",
        "# coding=utf-8\n# *** Warning: This File Was Generated By pulumi. ***\nvalue = 1\n",
    );
    repository.write(
        "schema.rs",
        "/*\n * This file is @generated by a build script.\n */\nfn schema() {}\n",
    );
    repository.write(
        "messages_pb2.py",
        "\u{feff}# Generated by the protocol buffer compiler.  DO NOT EDIT!\nvalue = 1\n",
    );
    repository.write(
        "client.js",
        "  /* DO NOT EDIT */\nexport const value = 1;\n",
    );
    repository.write(
        ".github/workflows/generated.yml",
        "# This file was automatically generated. DO NOT EDIT.\njobs: {}\n",
    );

    assert_eq!(repository.scanned_paths(), vec!["authored.rs"]);
}

#[test]
fn a_generated_marker_only_counts_in_a_header_comment() {
    let repository = Repository::new();
    let header = "fn line() {}\n".repeat(10);

    repository.write("late.rs", &format!("{header}// DO NOT EDIT\n"));
    repository.write(
        "literal.ts",
        "export const banner = \"// Code generated by hand. DO NOT EDIT.\";\n",
    );
    repository.write("lowercase.rs", "// do not edit\nfn authored() {}\n");
    repository.write("capital.rs", "// @Generated\nfn authored() {}\n");
    repository.write(
        "pulumi.py",
        "value = '*** WARNING: this file was generated by pulumi'\n",
    );
    repository.write(
        "tenth.rs",
        &format!("{}// @generated\n", "fn line() {}\n".repeat(9)),
    );

    assert_eq!(
        repository.scanned_paths(),
        vec![
            "capital.rs",
            "late.rs",
            "literal.ts",
            "lowercase.rs",
            "pulumi.py"
        ],
        "the tenth line is still the header; the eleventh is not"
    );
}

#[test]
fn a_root_gitattributes_pattern_marks_files_generated() {
    let repository = Repository::new();

    repository.write(
        ".gitattributes",
        "# Pulumi SDKs are generated.\n\
         infra/*/sdks/** linguist-generated=true\n\
         /top.ts linguist-generated\n\
         gen/*.ts linguist-generated\n\
         *.pb.ts linguist-generated\n\
         *.pb.ts text eol=lf\n\
         .github/workflows/lock.yml linguist-generated\n",
    );
    repository.write(
        ".github/workflows/lock.yml",
        "jobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n",
    );

    assert_eq!(
        repository.scanned_beside(&[
            "infra/db/sdks/db/index.ts",
            "infra/db/main.ts",
            "top.ts",
            "nested/top.ts",
            "gen/api.ts",
            "gen/deep/api.ts",
            "other/gen/api.ts",
            "nested/messages.pb.ts",
        ]),
        vec![
            "gen/deep/api.ts",
            "infra/db/main.ts",
            "nested/top.ts",
            "other/gen/api.ts",
        ]
    );
}

#[test]
fn a_nested_gitattributes_applies_below_its_directory() {
    let repository = Repository::new();

    repository.write("sdks/db/.gitattributes", "* linguist-generated\n");
    repository.write(
        "vendored/.gitattributes",
        "** linguist-generated\nauthored.ts linguist-generated=false\n",
    );
    repository.write(".gitattributes", "vendored/kept.ts -linguist-generated\n");

    assert_eq!(
        repository.scanned_beside(&[
            "sdks/db/scripts/postinstall.js",
            "sdks/db/index.ts",
            "sdks/other.ts",
            "vendored/authored.ts",
            "vendored/kept.ts",
        ]),
        vec!["sdks/other.ts", "vendored/authored.ts"],
        "a deeper file overrides the root one, and a later line an earlier one"
    );
}

#[test]
fn a_gitattributes_line_only_marks_generated_when_it_sets_the_attribute() {
    let repository = Repository::new();

    repository.write(
        ".gitattributes",
        "*.ts linguist-generated\n\
         unset.ts -linguist-generated\n\
         unspecified.ts !linguist-generated\n\
         false.ts linguist-generated=false\n\
         yes.ts linguist-generated=yes\n\
         both.ts linguist-generated -linguist-generated\n\
         *.js linguist-vendored\n\
         dir/ linguist-generated\n\
         \"quoted.mjs\" linguist-generated\n\
         !negated.mjs linguist-generated\n\
         [attr]macro.mjs linguist-generated\n\
         #comment.mjs linguist-generated\n",
    );

    assert_eq!(
        repository.scanned_beside(&[
            "marked.ts",
            "unset.ts",
            "unspecified.ts",
            "false.ts",
            "yes.ts",
            "both.ts",
            "vendored.js",
            "dir/file.cjs",
            "quoted.mjs",
            "negated.mjs",
            "[attr]macro.mjs",
            "#comment.mjs",
        ]),
        vec![
            "#comment.mjs",
            "[attr]macro.mjs",
            "both.ts",
            "dir/file.cjs",
            "false.ts",
            "negated.mjs",
            "quoted.mjs",
            "unset.ts",
            "unspecified.ts",
            "vendored.js",
            "yes.ts",
        ]
    );
}

#[cfg(unix)]
#[test]
fn a_symbolically_linked_gitattributes_is_not_read() {
    let repository = Repository::new();
    let target = repository
        .directory
        .write("attributes.txt", "* linguist-generated\n");

    std::os::unix::fs::symlink(&target, repository.directory.path().join(".gitattributes"))
        .unwrap_or_else(|error| panic!("links .gitattributes: {error}"));

    assert_eq!(repository.scanned_beside(&["index.ts"]), vec!["index.ts"]);
}
