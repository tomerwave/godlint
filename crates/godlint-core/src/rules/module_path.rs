use crate::source::Language;

pub(crate) fn covers(prefix: &str, module: &str, language: Language) -> bool {
    let Some(rest) = module.strip_prefix(prefix) else {
        return false;
    };

    rest.is_empty() || rest.starts_with(separator(language))
}

fn separator(language: Language) -> &'static str {
    match language {
        Language::JavaScript | Language::TypeScript => "/",
        Language::Python => ".",
        Language::Rust => "::",
    }
}
