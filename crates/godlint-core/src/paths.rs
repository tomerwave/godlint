//! The single place path safety is decided.
//!
//! Deciding "does this path escape the repository" or "is a symlink involved" in several
//! modules invites them to disagree, and this is the boundary that keeps analysis inside
//! the tree the operator pointed at.

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

/// Resolves `.` and `..` lexically without touching the filesystem.
///
/// Returns `None` when the path climbs past its own root, which no caller can honour.
pub fn normalize(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
        }
    }

    Some(normalized)
}

/// Reports whether the path is a symbolic link, treating unreadable paths as not links.
pub fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

/// Reports whether any component of `path` below `root` is a symbolic link.
pub fn contains_symlink(root: &Path, path: &Path) -> bool {
    let Ok(relative) = path.strip_prefix(root) else {
        return false;
    };
    let mut current = root.to_path_buf();

    for component in relative.components() {
        if let Component::Normal(name) = component {
            current.push(name);

            if is_symlink(&current) {
                return true;
            }
        }
    }

    false
}

/// Reports whether the path names a parent directory anywhere in its components.
pub fn climbs(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

/// Reports whether `path` is safe to record as a repository-relative source path.
pub fn is_repository_relative(path: &Path) -> bool {
    path.is_relative() && !climbs(path)
}

/// Finds the directory owning `marker`, searching upward but never leaving the repository.
///
/// The search stops at a directory holding `boundary` so that a stray configuration file
/// in a parent directory, or in a home directory, cannot silently govern this run.
pub fn find_upward(start: &Path, marker: &str, boundary: &str) -> Option<PathBuf> {
    for directory in start.ancestors() {
        if directory.join(marker).is_file() {
            return Some(directory.to_path_buf());
        }

        if directory.join(boundary).exists() {
            return None;
        }
    }

    None
}
