use crate::{
    facts::CallFact,
    glob,
    source::{Language, SourceFile},
};

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Dialect {
    JavaScript,
    Python,
    Rust,
}

pub(super) struct Catalogue(pub(super) &'static [(&'static str, Dialect)]);

impl Catalogue {
    pub(super) fn lists(&self, name: &str) -> bool {
        self.0.iter().any(|(entry, _)| *entry == name)
    }

    pub(super) fn speaks(&self, language: Language, name: &str) -> bool {
        let spoken = dialect(language);

        self.0
            .iter()
            .any(|(entry, dialect)| *entry == name && *dialect == spoken)
    }
}

pub(super) fn spelled(call: &CallFact) -> String {
    let callee = call.callee();

    if call.is_macro() {
        format!("{callee}!")
    } else {
        callee.to_owned()
    }
}

pub(super) fn is_allowed(source: &SourceFile, paths: &[String]) -> bool {
    glob::matches_any(paths.iter().map(String::as_str), source.path_text())
}

fn dialect(language: Language) -> Dialect {
    match language {
        Language::JavaScript | Language::TypeScript => Dialect::JavaScript,
        Language::Python => Dialect::Python,
        Language::Rust => Dialect::Rust,
    }
}
