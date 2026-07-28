#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use godlint_core::{
    config::DEFAULT_EXCLUDES,
    discovery::{DiscoveryError, Scope, discover},
};

static NEXT_REPOSITORY_ID: AtomicU64 = AtomicU64::new(0);

struct Repository {
    path: PathBuf,
}

impl Repository {
    fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        let id = NEXT_REPOSITORY_ID.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!("godlint-discovery-{timestamp}-{id}"));

        fs::create_dir(&path).unwrap_or_else(|error| panic!("creates repository: {error}"));

        Self { path }
    }

    fn create_file(&self, relative_path: &str) {
        let path = self.path.join(relative_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| panic!("creates parent: {error}"));
        }

        fs::write(path, "source").unwrap_or_else(|error| panic!("writes source file: {error}"));
    }

    fn discover(&self) -> Result<Vec<PathBuf>, DiscoveryError> {
        self.discover_excluding(&defaults())
    }

    fn discover_excluding(&self, excludes: &[String]) -> Result<Vec<PathBuf>, DiscoveryError> {
        self.discover_paths(std::slice::from_ref(&self.path), excludes)
    }

    fn discover_paths(
        &self,
        paths: &[PathBuf],
        excludes: &[String],
    ) -> Result<Vec<PathBuf>, DiscoveryError> {
        discover(
            paths,
            &Scope {
                root: &self.path,
                excludes,
            },
        )
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.path)
            .unwrap_or_else(|error| panic!("removes repository: {error}"));
    }
}

fn relative_paths(repository: &Repository, paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .map(|path| {
            path.strip_prefix(&repository.path)
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

    let nested = repository.path.join("nested");
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
