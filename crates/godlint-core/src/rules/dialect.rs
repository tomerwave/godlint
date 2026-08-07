use crate::{
    rules::Languages,
    source::{Dialect, Language, TextFile, Workflow},
};

pub(super) fn dialect_of(file: &TextFile) -> Option<Dialect> {
    if file.path() == std::path::Path::new("<branch>") {
        return Some(Dialect::Repository);
    }
    if Workflow::names(file.path()) {
        return Some(Dialect::Workflow);
    }
    Language::from_path(file.path()).map(Language::dialect)
}

pub(super) fn supports(languages: Languages, file: &TextFile) -> bool {
    dialect_of(file).is_none_or(|dialect| languages.analyses(dialect))
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::path::PathBuf;

    use super::{dialect_of, supports};
    use crate::{
        rules::{Absence, Languages},
        source::{Dialect, TextFile},
    };

    #[test]
    fn dialect_of_recognises_workflows_and_repository_branches() {
        let workflow = TextFile::new(
            PathBuf::from(".github/workflows/check.yml"),
            "name: check".to_owned(),
        )
        .unwrap();
        let branch = TextFile::new(PathBuf::from("<branch>"), "feature/change".to_owned()).unwrap();
        assert_eq!(dialect_of(&workflow), Some(Dialect::Workflow));
        assert_eq!(dialect_of(&branch), Some(Dialect::Repository));
    }

    #[test]
    fn supports_discards_an_absent_source_dialect() {
        let rust = TextFile::new(PathBuf::from("main.rs"), "fn main() {}".to_owned()).unwrap();
        let languages = Languages::all_but(&[(Dialect::Rust, Absence::NoSuchConstruct)]);
        assert!(!supports(languages, &rust));
    }
}
