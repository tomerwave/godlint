use std::{
    collections::BTreeSet,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use crate::{glob, paths, source::Language};

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

pub struct Scope<'a> {
    pub root: &'a Path,
    pub excludes: &'a [String],
}

pub fn discover(paths: &[PathBuf], scope: &Scope<'_>) -> Result<Vec<PathBuf>, DiscoveryError> {
    let mut files = BTreeSet::new();

    for path in paths {
        discover_path(path, scope, &mut files, true)?;
    }

    Ok(files.into_iter().collect())
}

enum Walk {
    Skip,
    File,
    Directory,
}

fn discover_path(
    path: &Path,
    scope: &Scope<'_>,
    files: &mut BTreeSet<PathBuf>,
    is_requested_root: bool,
) -> Result<(), DiscoveryError> {
    let metadata = fs::symlink_metadata(path).map_err(|source| DiscoveryError::ReadMetadata {
        path: path.to_path_buf(),
        source,
    })?;

    match classify(path, scope, &metadata, is_requested_root) {
        Walk::Skip => Ok(()),
        Walk::File => {
            add_supported_file(path, files);

            Ok(())
        }
        Walk::Directory => discover_directory(path, scope, files),
    }
}

fn classify(
    path: &Path,
    scope: &Scope<'_>,
    metadata: &fs::Metadata,
    is_requested_root: bool,
) -> Walk {
    if metadata.file_type().is_symlink() || is_excluded(path, scope) {
        return Walk::Skip;
    }

    if metadata.is_file() {
        return Walk::File;
    }

    let descends = metadata.is_dir() && (is_requested_root || !paths::is_repository_root(path));

    if descends {
        return Walk::Directory;
    }

    Walk::Skip
}

fn is_excluded(path: &Path, scope: &Scope<'_>) -> bool {
    let relative = path.strip_prefix(scope.root).unwrap_or(path);
    let Some(candidate) = relative.to_str() else {
        return false;
    };

    let candidate = paths::slashed(candidate);

    scope
        .excludes
        .iter()
        .any(|pattern| glob::matches(pattern, &candidate))
}

fn discover_directory(
    directory: &Path,
    scope: &Scope<'_>,
    files: &mut BTreeSet<PathBuf>,
) -> Result<(), DiscoveryError> {
    let mut paths = entry_paths(directory)?;

    paths.sort();

    paths
        .into_iter()
        .try_for_each(|path| discover_path(&path, scope, files, false))
}

fn entry_paths(directory: &Path) -> Result<Vec<PathBuf>, DiscoveryError> {
    fs::read_dir(directory)
        .map_err(|source| unreadable(directory, source))?
        .map(|entry| {
            entry
                .map(|entry| entry.path())
                .map_err(|source| unreadable(directory, source))
        })
        .collect()
}

fn unreadable(directory: &Path, source: std::io::Error) -> DiscoveryError {
    DiscoveryError::ReadDirectory {
        path: directory.to_path_buf(),
        source,
    }
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
