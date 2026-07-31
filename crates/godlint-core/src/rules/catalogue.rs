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

pub(super) const GENERATORS: Catalogue = Catalogue(&[
    ("Math.random", Dialect::JavaScript),
    ("crypto.pseudoRandomBytes", Dialect::JavaScript),
    ("random.random", Dialect::Python),
    ("random.randint", Dialect::Python),
    ("random.randrange", Dialect::Python),
    ("random.choice", Dialect::Python),
    ("random.choices", Dialect::Python),
    ("random.sample", Dialect::Python),
    ("random.shuffle", Dialect::Python),
    ("random.uniform", Dialect::Python),
    ("np.random.rand", Dialect::Python),
    ("np.random.randn", Dialect::Python),
    ("np.random.random", Dialect::Python),
    ("np.random.randint", Dialect::Python),
    ("np.random.choice", Dialect::Python),
    ("np.random.shuffle", Dialect::Python),
    ("np.random.permutation", Dialect::Python),
    ("numpy.random.rand", Dialect::Python),
    ("numpy.random.randn", Dialect::Python),
    ("numpy.random.random", Dialect::Python),
    ("numpy.random.randint", Dialect::Python),
    ("numpy.random.choice", Dialect::Python),
    ("numpy.random.shuffle", Dialect::Python),
    ("numpy.random.permutation", Dialect::Python),
    ("rand::random", Dialect::Rust),
    ("rand::thread_rng", Dialect::Rust),
    ("rand::rng", Dialect::Rust),
    ("rand::random_range", Dialect::Rust),
]);

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
