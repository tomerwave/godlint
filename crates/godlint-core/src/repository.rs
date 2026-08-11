use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::source::{SourceRange, TextFile};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionDriftFact {
    file: TextFile,
    range: SourceRange,
    package: String,
    declared: String,
    locked: String,
    lockfile: PathBuf,
}

struct ManifestData {
    file: TextFile,
    range: SourceRange,
    package: String,
    declared: String,
}

impl VersionDriftFact {
    pub fn file(&self) -> &TextFile {
        &self.file
    }
    pub fn range(&self) -> SourceRange {
        self.range
    }
    pub fn package(&self) -> &str {
        &self.package
    }
    pub fn declared(&self) -> &str {
        &self.declared
    }
    pub fn locked(&self) -> &str {
        &self.locked
    }
    pub fn lockfile(&self) -> &Path {
        &self.lockfile
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RepositoryFacts {
    branch: Option<TextFile>,
    version_drifts: Vec<VersionDriftFact>,
}

impl RepositoryFacts {
    pub fn new(branch: Option<TextFile>) -> Self {
        Self {
            branch,
            version_drifts: Vec::new(),
        }
    }

    pub fn branch(&self) -> Option<&TextFile> {
        self.branch.as_ref()
    }

    pub fn with_version_drifts(mut self, version_drifts: Vec<VersionDriftFact>) -> Self {
        self.version_drifts = version_drifts;
        self
    }

    pub fn version_drifts(&self) -> &[VersionDriftFact] {
        &self.version_drifts
    }

    pub fn read_version_drifts(root: &Path) -> Vec<VersionDriftFact> {
        let mut facts = Vec::new();
        collect(root, root, &mut facts);
        facts
    }
}

fn collect(root: &Path, directory: &Path, facts: &mut Vec<VersionDriftFact>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        collect_entry(root, &entry.path(), facts);
    }
}

fn collect_entry(root: &Path, path: &Path, facts: &mut Vec<VersionDriftFact>) {
    if path.is_dir() {
        if ignored_directory(path) {
            return;
        }
        collect(root, path, facts);
        return;
    }
    if let Some(found) = read_manifest(root, path) {
        facts.extend(found);
    }
}

fn ignored_directory(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "target" | "node_modules" | "dist" | ".venv")
    )
}

fn read_manifest(root: &Path, path: &Path) -> Option<Vec<VersionDriftFact>> {
    let name = path.file_name()?.to_str()?;
    if name == "pyproject.toml" {
        return read_python(root, path);
    }
    let lock_name = standard_lock_name(name)?;
    read_standard(root, path, name, lock_name)
}

fn standard_lock_name(name: &str) -> Option<&'static str> {
    match name {
        "Cargo.toml" => Some("Cargo.lock"),
        "package.json" => Some("package-lock.json"),
        _ => None,
    }
}

fn read_standard(
    root: &Path,
    path: &Path,
    name: &str,
    lock_name: &str,
) -> Option<Vec<VersionDriftFact>> {
    let lock = path.parent()?.join(lock_name);
    let data = manifest_data(root, path, name)?;
    let locked = fs::read_to_string(&lock).ok()?;
    let locked_version = lock_version(name, &locked, &data.package)?;
    drift(data, locked_version, &lock, root)
}

fn read_text_file(root: &Path, path: &Path) -> Option<TextFile> {
    let relative = path.strip_prefix(root).ok()?.to_path_buf();
    let text = fs::read_to_string(path).ok()?;
    TextFile::new(relative, text).ok()
}

fn manifest_data(root: &Path, path: &Path, name: &str) -> Option<ManifestData> {
    let file = read_text_file(root, path)?;
    let (package, declared, range) = manifest_fields(name, file.text())?;
    Some(ManifestData {
        file,
        range,
        package,
        declared,
    })
}

fn drift(
    data: ManifestData,
    locked: String,
    lock: &Path,
    root: &Path,
) -> Option<Vec<VersionDriftFact>> {
    let lockfile = lock.strip_prefix(root).ok()?.to_path_buf();
    (data.declared != locked).then_some(vec![VersionDriftFact {
        file: data.file,
        range: data.range,
        package: data.package,
        declared: data.declared,
        locked,
        lockfile,
    }])
}

fn manifest_fields(name: &str, text: &str) -> Option<(String, String, SourceRange)> {
    if name == "package.json" {
        json_fields(text)
    } else {
        toml_fields(text)
    }
}

fn lock_version(name: &str, text: &str, package: &str) -> Option<String> {
    if name == "package.json" {
        json_lock_version(text)
    } else {
        cargo_lock_version(text, package)
    }
}

fn read_python(root: &Path, path: &Path) -> Option<Vec<VersionDriftFact>> {
    let data = manifest_data(root, path, "Cargo.toml")?;
    for lock_name in ["uv.lock", "poetry.lock"] {
        if let Some(result) = python_lock(root, path, &data, lock_name) {
            return result;
        }
    }
    None
}

fn python_lock(
    root: &Path,
    path: &Path,
    data: &ManifestData,
    name: &str,
) -> Option<Option<Vec<VersionDriftFact>>> {
    let lock = path.parent()?.join(name);
    let text = fs::read_to_string(&lock).ok()?;
    let locked = named_lock_version(&text, &data.package)?;
    Some(drift(
        ManifestData {
            file: data.file.clone(),
            range: data.range,
            package: data.package.clone(),
            declared: data.declared.clone(),
        },
        locked,
        &lock,
        root,
    ))
}

fn toml_fields(text: &str) -> Option<(String, String, SourceRange)> {
    let name = field(text, "name")?;
    let version = field_with_range(text, "version")?;
    Some((name.0, version.0, version.1))
}

fn field(text: &str, key: &str) -> Option<(String, SourceRange)> {
    field_with_range(text, key)
}

fn field_with_range(text: &str, key: &str) -> Option<(String, SourceRange)> {
    let (offset, line) = find_field(text, key)?;
    let value = line_value(line)?;
    let start = text
        .lines()
        .take(offset)
        .map(|line| line.len() + 1)
        .sum::<usize>();
    let value_start = start + line.find(value)?;
    Some((
        value.to_owned(),
        SourceRange::new(value_start, value_start + value.len()),
    ))
}

fn find_field<'a>(text: &'a str, key: &str) -> Option<(usize, &'a str)> {
    text.lines().enumerate().find_map(|(line, value)| {
        let trimmed = value.trim_start();
        (trimmed.starts_with(key) && trimmed[key.len()..].trim_start().starts_with('='))
            .then_some((line, value))
    })
}

fn line_value(line: &str) -> Option<&str> {
    line.split_once('=')?
        .1
        .trim()
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .or_else(|| {
            line.split_once('=')?
                .1
                .trim()
                .strip_prefix('\'')
                .and_then(|value| value.strip_suffix('\''))
        })
}

fn json_fields(text: &str) -> Option<(String, String, SourceRange)> {
    let (package, _) = json_value(text, "name")?;
    let (version, range) = json_value(text, "version")?;
    Some((package, version, range))
}

fn json_value(text: &str, key: &str) -> Option<(String, SourceRange)> {
    let needle = format!("\"{key}\"");
    let start = text.find(&needle)?;
    let after = &text[start + needle.len()..];
    let value = json_string_value(after)?;
    let value_start = start + needle.len() + after.find(value)?;
    Some((
        value.to_owned(),
        SourceRange::new(value_start, value_start + value.len()),
    ))
}

fn json_string_value(text: &str) -> Option<&str> {
    text.split_once(':')?
        .1
        .trim_start()
        .strip_prefix('"')?
        .split_once('"')
        .map(|pair| pair.0)
}

fn cargo_lock_version(text: &str, package: &str) -> Option<String> {
    named_lock_version(text, package)
}

fn json_lock_version(text: &str) -> Option<String> {
    let root = text.split("\"packages\"").next()?;
    json_value(root, "version").map(|value| value.0)
}

fn named_lock_version(text: &str, package: &str) -> Option<String> {
    let mut matching = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if matching && trimmed.starts_with("version") {
            return line_version(trimmed);
        }
        matching = trimmed.starts_with("name") && trimmed.contains(package);
        if trimmed.starts_with("[[") && !trimmed.contains("package") {
            matching = false;
        }
    }
    None
}

fn line_version(line: &str) -> Option<String> {
    Some(line.split_once('=')?.1.trim().trim_matches('"').to_owned())
}
