use crate::source::TextFile;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RepositoryFacts {
    branch: Option<TextFile>,
}

impl RepositoryFacts {
    pub fn new(branch: Option<TextFile>) -> Self {
        Self { branch }
    }

    pub fn branch(&self) -> Option<&TextFile> {
        self.branch.as_ref()
    }
}
