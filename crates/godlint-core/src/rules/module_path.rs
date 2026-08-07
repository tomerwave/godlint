use crate::source::Language;

pub(crate) fn covers(prefix: &str, module: &str, language: Language) -> bool {
    let Some(rest) = module.strip_prefix(prefix) else {
        return false;
    };

    rest.is_empty() || rest.starts_with(separator(language))
}

pub(crate) fn separator(language: Language) -> &'static str {
    match language {
        Language::JavaScript | Language::TypeScript => "/",
        Language::Python => ".",
        Language::Rust => "::",
    }
}

pub(crate) fn segments(module: &str, language: Language) -> impl Iterator<Item = &str> {
    module.split(separator(language))
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

    if first.starts_with('@') {
        return scoped_package(module, first);
    }

    non_empty(first)
}

fn scoped_package<'a>(module: &'a str, scope: &str) -> Option<&'a str> {
    if scope.len() == 1 {
        return None;
    }

    let name = non_empty(first_segment(module.get(scope.len() + 1..)?, "/"))?;

    non_empty(&module[..scope.len() + 1 + name.len()])
}

fn python_package(module: &str) -> Option<&str> {
    if module.starts_with('.') {
        return None;
    }

    non_empty(first_segment(module, "."))
}

fn rust_package(module: &str) -> Option<&str> {
    let path = module.strip_prefix("::").unwrap_or(module);

    match first_segment(path, "::") {
        "crate" | "self" | "super" => None,
        root => non_empty(root),
    }
}

pub(crate) fn first_segment<'a>(text: &'a str, separator: &str) -> &'a str {
    match text.find(separator) {
        Some(index) => &text[..index],
        None => text,
    }
}

fn non_empty(name: &str) -> Option<&str> {
    (!name.is_empty()).then_some(name)
}

pub(crate) fn last_segment(text: &str, separator: char) -> &str {
    match text.rfind(separator) {
        Some(index) => &text[index + separator.len_utf8()..],
        None => text,
    }
}
