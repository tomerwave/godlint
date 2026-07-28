use std::{
    collections::BTreeSet,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::{glob, source::Language};

#[derive(Debug)]
pub enum DiscoveryError {
    ReadDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    ReadMetadata {
        path: PathBuf,
        source: std::io::Error,
    },
}

/// Where discovery is rooted and what it must leave alone.
///
/// Exclusions are configured policy rather than a constant, so a repository can keep a
/// virtual environment or a build directory out of its results.
pub struct Scope<'a> {
    pub root: &'a Path,
    pub excludes: &'a [String],
}

pub fn discover(paths: &[PathBuf], scope: &Scope<'_>) -> Result<Vec<PathBuf>, DiscoveryError> {
    let mut files = BTreeSet::new();

    for path in paths {
        discover_path(path, scope, &mut files)?;
    }

    Ok(files.into_iter().collect())
}

fn discover_path(
    path: &Path,
    scope: &Scope<'_>,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), DiscoveryError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| DiscoveryError::ReadMetadata {
        path: path.to_path_buf(),
        source,
    })?;

    if metadata.file_type().is_symlink() || is_excluded(path, scope) {
        return Ok(());
    }

    if metadata.is_file() {
        add_supported_file(path, files);
    } else if metadata.is_dir() {
        discover_directory(path, scope, files)?;
    }

    Ok(())
}

/// Reports whether configured policy excludes this path from scanning.
fn is_excluded(path: &Path, scope: &Scope<'_>) -> bool {
    let relative = path.strip_prefix(scope.root).unwrap_or(path);
    let Some(candidate) = relative.to_str() else {
        return false;
    };

    scope
        .excludes
        .iter()
        .any(|pattern| glob::matches(pattern, candidate))
}

fn discover_directory(
    directory: &Path,
    scope: &Scope<'_>,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), DiscoveryError> {
    let entries = fs::read_dir(directory).map_err(|source| DiscoveryError::ReadDirectory {
        path: directory.to_path_buf(),
        source,
    })?;
    let mut paths = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|source| DiscoveryError::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })?;

        paths.push(entry.path());
    }

    paths.sort();

    for path in paths {
        discover_path(&path, scope, files)?;
    }

    Ok(())
}

fn add_supported_file(path: &Path, files: &mut BTreeSet<PathBuf>) {
    if Language::from_path(path).is_some() {
        files.insert(path.to_path_buf());
    }
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDirectory { path, source } | Self::ReadMetadata { path, source } => {
                write!(formatter, "{}: {source}", path.display())
            }
        }
    }
}

impl Error for DiscoveryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ReadDirectory { source, .. } | Self::ReadMetadata { source, .. } => Some(source),
        }
    }
}
