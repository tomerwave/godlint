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
        discover(
            std::slice::from_ref(&self.path),
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

/// Exclusions are policy, so a repository can name its own generated directories.
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

/// Without exclusions every file is in scope, including dependency trees.
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
