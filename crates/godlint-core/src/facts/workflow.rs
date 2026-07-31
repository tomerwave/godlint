use crate::source::{SourceRange, TextFile};

const QUOTES: [char; 2] = ['"', '\''];
const COMMIT_LENGTH: usize = 40;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionFact {
    file: TextFile,
    range: SourceRange,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobFact {
    file: TextFile,
    range: SourceRange,
    name: String,
    declares_permissions: bool,
}

impl ActionFact {
    pub fn new(file: TextFile, range: SourceRange) -> Self {
        Self { file, range }
    }

    pub fn file(&self) -> &TextFile {
        &self.file
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn reference(&self) -> &str {
        self.file.text()[self.range.start()..self.range.end()].trim_matches(QUOTES)
    }

    pub fn name(&self) -> &str {
        match self.reference().split_once('@') {
            Some((name, _)) => name,
            None => self.reference(),
        }
    }

    pub fn version(&self) -> Option<&str> {
        self.reference().split_once('@').map(|(_, version)| version)
    }

    pub fn owner(&self) -> Option<&str> {
        (!self.is_local() && !self.is_container())
            .then(|| self.name().split('/').next())
            .flatten()
            .filter(|owner| !owner.is_empty())
    }

    pub fn is_commit(&self) -> bool {
        self.version().is_some_and(is_commit)
    }

    pub fn is_local(&self) -> bool {
        self.reference().starts_with("./") || self.reference().starts_with(".\\")
    }

    pub fn is_container(&self) -> bool {
        self.reference().starts_with("docker://")
    }
}

impl JobFact {
    pub fn new(
        file: TextFile,
        range: SourceRange,
        name: String,
        declares_permissions: bool,
    ) -> Self {
        Self {
            file,
            range,
            name,
            declares_permissions,
        }
    }

    pub fn file(&self) -> &TextFile {
        &self.file
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn declares_permissions(&self) -> bool {
        self.declares_permissions
    }
}

fn is_commit(version: &str) -> bool {
    version.len() == COMMIT_LENGTH
        && version
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}
