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

pub(crate) fn package(module: &str, language: Language) -> Option<&str> {
    match language {
        Language::JavaScript | Language::TypeScript => ecmascript_package(module),
        Language::Python => python_package(module),
        Language::Rust => rust_package(module),
    }
}

fn ecmascript_package(module: &str) -> Option<&str> {
    if module.starts_with('.') || module.contains(':') {
        return None;
    }

    let first = first_segment(module, "/");

    if !first.starts_with('@') {
        return non_empty(first);
    }

    let scoped = first_segment(module.get(first.len() + 1..)?, "/");

    non_empty(&module[..first.len() + 1 + scoped.len()])
}

fn python_package(module: &str) -> Option<&str> {
    if module.starts_with('.') {
        return None;
    }

    non_empty(first_segment(module, "."))
}

fn rust_package(module: &str) -> Option<&str> {
    match first_segment(module, "::") {
        "crate" | "self" | "super" => None,
        root => non_empty(root),
    }
}

fn first_segment<'a>(text: &'a str, separator: &str) -> &'a str {
    match text.find(separator) {
        Some(index) => &text[..index],
        None => text,
    }
}

fn non_empty(name: &str) -> Option<&str> {
    (!name.is_empty()).then_some(name)
}
