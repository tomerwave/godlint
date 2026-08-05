use crate::source::Dialect;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Absence {
    NoSuchConstruct,
    NotImplemented,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Languages(&'static [(Dialect, Absence)]);

impl Languages {
    pub const EVERY_LANGUAGE: Self = Self(&[
        (Dialect::Workflow, Absence::NoSuchConstruct),
        (Dialect::Repository, Absence::NoSuchConstruct),
    ]);

    pub const WORKFLOWS: Self = Self(&[
        (Dialect::JavaScript, Absence::NoSuchConstruct),
        (Dialect::Python, Absence::NoSuchConstruct),
        (Dialect::Rust, Absence::NoSuchConstruct),
        (Dialect::Repository, Absence::NoSuchConstruct),
    ]);

    pub const REPOSITORY: Self = Self(&[
        (Dialect::JavaScript, Absence::NoSuchConstruct),
        (Dialect::Python, Absence::NoSuchConstruct),
        (Dialect::Rust, Absence::NoSuchConstruct),
        (Dialect::Workflow, Absence::NoSuchConstruct),
    ]);

    pub const fn all_but(absent: &'static [(Dialect, Absence)]) -> Self {
        Self(absent)
    }

    pub fn absence(self, dialect: Dialect) -> Option<Absence> {
        if dialect == Dialect::Repository && self != Self::REPOSITORY {
            return Some(Absence::NoSuchConstruct);
        }

        self.0
            .iter()
            .find(|(entry, _)| *entry == dialect)
            .map(|(_, absence)| *absence)
    }

    pub fn analyses(self, dialect: Dialect) -> bool {
        self.absence(dialect).is_none()
    }
}
