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

pub struct Discovery {
    pub files: Vec<PathBuf>,
    pub failures: Vec<DiscoveryError>,
}

pub fn discover(paths: &[PathBuf], scope: &Scope<'_>) -> Result<Discovery, DiscoveryError> {
    let mut walk = Walk {
        scope,
        files: BTreeSet::new(),
        failures: Vec::new(),
    };

    for path in paths {
        walk.requested(path)?;
    }

    Ok(walk.finish())
}

enum Kind {
    Skip,
    File,
    Directory,
}

struct Walk<'a> {
    scope: &'a Scope<'a>,
    files: BTreeSet<PathBuf>,
    failures: Vec<DiscoveryError>,
}

impl Walk<'_> {
    fn requested(&mut self, path: &Path) -> Result<(), DiscoveryError> {
        let metadata =
            fs::symlink_metadata(path).map_err(|source| unreadable_metadata(path, source))?;

        match classify(path, self.scope, &metadata, true) {
            Kind::Skip => (),
            Kind::File => self.add_supported_file(path),
            Kind::Directory => self.enter(sorted_entry_paths(path)?),
        }

        Ok(())
    }

    fn enter(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            self.descend(&path);
        }
    }

    fn descend(&mut self, path: &Path) {
        match fs::symlink_metadata(path) {
            Ok(metadata) => self.visit(path, &metadata),
            Err(source) => self.failures.push(unreadable_metadata(path, source)),
        }
    }

    fn visit(&mut self, path: &Path, metadata: &fs::Metadata) {
        match classify(path, self.scope, metadata, false) {
            Kind::Skip => (),
            Kind::File => self.add_supported_file(path),
            Kind::Directory => self.walk_directory(path),
        }
    }

    fn walk_directory(&mut self, directory: &Path) {
        match sorted_entry_paths(directory) {
            Ok(paths) => self.enter(paths),
            Err(failure) => self.failures.push(failure),
        }
    }

    fn add_supported_file(&mut self, path: &Path) {
        if Language::from_path(path).is_some() {
            self.files.insert(path.to_path_buf());
        }
    }

    fn finish(self) -> Discovery {
        Discovery {
            files: self.files.into_iter().collect(),
            failures: self.failures,
        }
    }
}

fn classify(
    path: &Path,
    scope: &Scope<'_>,
    metadata: &fs::Metadata,
    is_requested_root: bool,
) -> Kind {
    if metadata.file_type().is_symlink() || is_excluded(path, scope) {
        return Kind::Skip;
    }

    if metadata.is_file() {
        return Kind::File;
    }

    let descends = metadata.is_dir() && (is_requested_root || !paths::is_repository_root(path));

    if descends {
        return Kind::Directory;
    }

    Kind::Skip
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

fn sorted_entry_paths(directory: &Path) -> Result<Vec<PathBuf>, DiscoveryError> {
    let mut paths = entry_paths(directory)?;

    paths.sort();

    Ok(paths)
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

fn unreadable_metadata(path: &Path, source: std::io::Error) -> DiscoveryError {
    DiscoveryError::ReadMetadata {
        path: path.to_path_buf(),
        source,
    }
}

impl DiscoveryError {
    pub fn path(&self) -> &Path {
        match self {
            Self::ReadDirectory { path, .. } | Self::ReadMetadata { path, .. } => path,
        }
    }

    pub fn reason(&self) -> String {
        match self {
            Self::ReadDirectory { source, .. } | Self::ReadMetadata { source, .. } => {
                source.to_string()
            }
        }
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
