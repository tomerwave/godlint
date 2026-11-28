use crate::source::{SourceRange, TextFile};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DependencySpecFact {
    file: TextFile,
    section: &'static str,
    package: SourceRange,
    spec: SourceRange,
}

impl DependencySpecFact {
    pub fn new(
        file: TextFile,
        section: &'static str,
        package: SourceRange,
        spec: SourceRange,
    ) -> Self {
        Self {
            file,
            section,
            package,
            spec,
        }
    }

    pub fn file(&self) -> &TextFile {
        &self.file
    }

    pub fn section(&self) -> &'static str {
        self.section
    }

    pub fn package(&self) -> &str {
        unquoted(self.file.slice(self.package))
    }

    pub fn spec(&self) -> &str {
        unquoted(self.file.slice(self.spec))
    }

    pub fn range(&self) -> SourceRange {
        self.spec
    }
}

fn unquoted(token: &str) -> &str {
    token
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(token)
}
