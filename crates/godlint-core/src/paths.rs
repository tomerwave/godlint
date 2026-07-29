use std::{
    borrow::Cow,
    ffi::OsStr,
    fs,
    path::{Component, Path, PathBuf},
};

const REPOSITORY_MARKER: &str = ".git";

pub fn normalize(path: &Path) -> Option<PathBuf> {
    path.components().try_fold(PathBuf::new(), absorb)
}

fn absorb(mut normalized: PathBuf, component: Component<'_>) -> Option<PathBuf> {
    match component {
        Component::CurDir => {}
        Component::Normal(part) => normalized.push(part),
        Component::ParentDir => normalized.pop().then_some(())?,
        Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
        Component::RootDir => normalized.push(component.as_os_str()),
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

    for name in relative.components().filter_map(normal_name) {
        current.push(name);

        if is_symlink(&current) {
            return true;
        }
    }

    false
}

fn normal_name(component: Component<'_>) -> Option<&OsStr> {
    match component {
        Component::Normal(name) => Some(name),
        _ => None,
    }
}

pub fn climbs(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::ParentDir))
}

pub fn is_repository_root(path: &Path) -> bool {
    path.join(REPOSITORY_MARKER).exists()
}

pub fn find_upward(start: &Path, marker: &str) -> Option<PathBuf> {
    for directory in start.ancestors() {
        if directory.join(marker).is_file() {
            return Some(directory.to_path_buf());
        }

        if is_repository_root(directory) {
            return None;
        }
    }

    None
}

pub fn slashed(text: &str) -> Cow<'_, str> {
    if text.contains('\\') {
        Cow::Owned(text.replace('\\', "/"))
    } else {
        Cow::Borrowed(text)
    }
}
