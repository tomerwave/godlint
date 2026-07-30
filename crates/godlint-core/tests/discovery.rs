#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use godlint_core::{
    config::DEFAULT_EXCLUDES,
    discovery::{Discovery, DiscoveryError, Scope, discover},
};

#[path = "support/temporary.rs"]
mod temporary;

use temporary::TemporaryDirectory;

struct Repository {
    directory: TemporaryDirectory,
}

impl Repository {
    fn new() -> Self {
        Self {
            directory: TemporaryDirectory::new("discovery"),
        }
    }

    fn path(&self) -> &Path {
        self.directory.path()
    }

    fn create_file(&self, relative_path: &str) {
        self.directory.write(relative_path, "source");
    }

    fn discover(&self) -> Result<Vec<PathBuf>, DiscoveryError> {
        self.discover_excluding(&defaults())
    }

    fn discover_excluding(&self, excludes: &[String]) -> Result<Vec<PathBuf>, DiscoveryError> {
        self.discover_paths(&[self.path().to_path_buf()], excludes)
    }

    fn discover_paths(
        &self,
        paths: &[PathBuf],
        excludes: &[String],
    ) -> Result<Vec<PathBuf>, DiscoveryError> {
        self.walk(paths, excludes)
            .map(|discovered| discovered.files)
    }

    fn walk(&self, paths: &[PathBuf], excludes: &[String]) -> Result<Discovery, DiscoveryError> {
        discover(
            paths,
            &Scope {
                root: self.path(),
                excludes,
            },
        )
    }
}

fn relative_paths(repository: &Repository, paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .map(|path| {
            path.strip_prefix(repository.path())
                .unwrap_or_else(|error| panic!("makes path relative: {error}"))
                .to_path_buf()
        })
        .collect()
}

#[test]
fn discovers_supported_files_in_sorted_order() {
    let repository = Repository::new();

    repository.create_file("z.py");
    repository.create_file("nested/a.tsx");
    repository.create_file("nested/b.rs");
    repository.create_file("root.js");
    repository.create_file("types.pyi");

    let discovered = repository
        .discover()
        .unwrap_or_else(|error| panic!("discovers source files: {error}"));
    let paths = relative_paths(&repository, discovered);

    assert_eq!(
        paths,
        vec![
            Path::new("nested/a.tsx").to_path_buf(),
            Path::new("nested/b.rs").to_path_buf(),
            Path::new("root.js").to_path_buf(),
            Path::new("types.pyi").to_path_buf(),
            Path::new("z.py").to_path_buf(),
        ]
    );
}

#[test]
fn ignores_unsupported_and_generated_files() {
    let repository = Repository::new();

    repository.create_file("source.rs");
    repository.create_file("README.md");
    repository.create_file(".git/hooks/pre-commit.rs");
    repository.create_file("node_modules/package/index.ts");
    repository.create_file("target/debug/generated.py");

    let discovered = repository
        .discover()
        .unwrap_or_else(|error| panic!("discovers source files: {error}"));
    let paths = relative_paths(&repository, discovered);

    assert_eq!(paths, vec![Path::new("source.rs").to_path_buf()]);
}

#[test]
fn reports_missing_input_paths() {
    let path = std::env::temp_dir().join("godlint-discovery-missing-path");
    let result = discover(
        std::slice::from_ref(&path),
        &Scope {
            root: &path,
            excludes: &defaults(),
        },
    );

    assert!(matches!(result, Err(DiscoveryError::ReadMetadata { .. })));
}

fn defaults() -> Vec<String> {
    DEFAULT_EXCLUDES.iter().map(|name| (*name).into()).collect()
}

#[test]
fn honours_configured_exclusions() {
    let repository = Repository::new();

    repository.create_file("source.rs");
    repository.create_file("generated/api.rs");
    repository.create_file("src/legacy.py");

    let discovered = repository
        .discover_excluding(&["generated".to_owned(), "*.py".to_owned()])
        .unwrap_or_else(|error| panic!("discovers source files: {error}"));

    assert_eq!(
        relative_paths(&repository, discovered),
        vec![Path::new("source.rs").to_path_buf()]
    );
}

#[test]
fn scans_everything_when_nothing_is_excluded() {
    let repository = Repository::new();

    repository.create_file("source.rs");
    repository.create_file("node_modules/package/index.ts");

    let discovered = repository
        .discover_excluding(&[])
        .unwrap_or_else(|error| panic!("discovers source files: {error}"));

    assert_eq!(relative_paths(&repository, discovered).len(), 2);
}

#[test]
fn skips_a_nested_repository_unless_it_is_explicitly_requested() {
    let repository = Repository::new();

    repository.create_file("outer.rs");
    repository.create_file("nested/.git");
    repository.create_file("nested/inner.rs");

    let discovered = repository
        .discover()
        .unwrap_or_else(|error| panic!("discovers parent repository: {error}"));

    assert_eq!(
        relative_paths(&repository, discovered),
        vec![Path::new("outer.rs").to_path_buf()]
    );

    let nested = repository.path().join("nested");
    let discovered = repository
        .discover_paths(std::slice::from_ref(&nested), &defaults())
        .unwrap_or_else(|error| panic!("discovers requested nested repository: {error}"));

    assert_eq!(
        relative_paths(&repository, discovered),
        vec![Path::new("nested/inner.rs").to_path_buf()]
    );
}

#[test]
fn skips_a_repository_nested_several_levels_deep() {
    let repository = Repository::new();

    repository.create_file("outer.rs");
    repository.create_file("a/b/.git");
    repository.create_file("a/b/inner.rs");
    repository.create_file("a/kept.rs");

    let discovered = repository
        .discover()
        .unwrap_or_else(|error| panic!("discovers parent repository: {error}"));

    assert_eq!(
        relative_paths(&repository, discovered),
        vec![
            Path::new("a/kept.rs").to_path_buf(),
            Path::new("outer.rs").to_path_buf()
        ]
    );
}

#[test]
fn treats_a_git_directory_and_a_git_file_alike() {
    let repository = Repository::new();

    repository.create_file("as_file/.git");
    repository.create_file("as_file/inner.rs");
    repository.create_file("as_directory/.git/HEAD");
    repository.create_file("as_directory/inner.rs");

    let discovered = repository
        .discover()
        .unwrap_or_else(|error| panic!("discovers parent repository: {error}"));

    assert!(
        relative_paths(&repository, discovered).is_empty(),
        "a worktree or submodule .git file marks a boundary exactly as a directory does"
    );
}

#[cfg(unix)]
fn deny_access(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o000))
        .unwrap_or_else(|error| panic!("removes permissions: {error}"));

    fs::read_dir(path).is_err()
}

#[cfg(unix)]
fn restore_access(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
        .unwrap_or_else(|error| panic!("restores permissions: {error}"));
}

#[cfg(unix)]
#[test]
fn an_unreadable_directory_does_not_discard_the_rest_of_the_walk() {
    let repository = Repository::new();

    repository.create_file("readable.rs");
    repository.create_file("denied/hidden.rs");

    let denied = repository.path.join("denied");

    assert!(
        deny_access(&denied),
        "the test cannot prove degradation while the directory is still readable"
    );

    let discovered = repository
        .walk(std::slice::from_ref(&repository.path), &defaults())
        .unwrap_or_else(|error| panic!("keeps walking past an unreadable directory: {error}"));

    restore_access(&denied);

    assert_eq!(
        relative_paths(&repository, discovered.files),
        vec![Path::new("readable.rs").to_path_buf()],
        "a sibling of an unreadable directory must survive"
    );
    assert_eq!(discovered.failures.len(), 1);
    assert_eq!(discovered.failures[0].path(), denied);
}

#[cfg(unix)]
#[test]
fn an_unreadable_requested_root_is_still_fatal() {
    let repository = Repository::new();

    repository.create_file("denied/hidden.rs");

    let denied = repository.path.join("denied");

    assert!(deny_access(&denied), "the root must be unreadable");

    let result = repository.walk(std::slice::from_ref(&denied), &defaults());

    restore_access(&denied);

    assert!(
        matches!(result, Err(DiscoveryError::ReadDirectory { .. })),
        "a path the user named by hand is not a partial result"
    );
}
