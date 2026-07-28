use std::{
    fs,
    path::{Component, Path, PathBuf},
};

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

pub fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

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

pub fn climbs(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

pub fn is_repository_relative(path: &Path) -> bool {
    path.is_relative() && !climbs(path)
}

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
