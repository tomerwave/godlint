const SEPARATORS: [char; 3] = [':', '.', '/'];

pub(crate) fn covers(prefix: &str, module: &str) -> bool {
    let Some(rest) = module.strip_prefix(prefix) else {
        return false;
    };

    rest.is_empty() || rest.starts_with(SEPARATORS)
}
