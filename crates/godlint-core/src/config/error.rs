use std::{error::Error, fmt, path::PathBuf};

use crate::suites;

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: yaml_serde::Error,
    },
    UnsupportedVersion {
        version: u8,
    },
    InvalidComplexityLimit,
    InvalidTodoMarkers,
    InvalidTodoReferencePrefixes,
    UnknownSuite {
        name: String,
    },
    InvalidRestrictedCallName,
    InvalidRestrictedImportName,
    InvalidLayerName,
    InvalidPackageName,
    EmptyNamingScope {
        case: String,
    },
    DuplicatePackageName {
        name: String,
    },
    EmptyLayer {
        rule: &'static str,
        name: String,
    },
    DuplicateLayerName {
        rule: &'static str,
        name: String,
    },
    DuplicateRestrictedImportName {
        name: String,
    },
    DuplicateRestrictedCallName {
        name: String,
    },
    BlankAllowIn {
        rule: &'static str,
    },
    InvalidExclude {
        pattern: String,
    },
}

const COMPLEXITY_AT_LEAST_ONE: &str =
    "maintainability/decision-complexity max-complexity must be at least 1";

const TODO_MARKERS_REQUIRED: &str = "policy/todo-requires-reference markers must not be empty";

const TODO_PREFIXES_REQUIRED: &str =
    "policy/todo-requires-reference reference-prefixes must not be empty or numeric";

const CALL_NAME_REQUIRED: &str = "architecture/restricted-call call names must not be blank";

const LAYER_NAME_REQUIRED: &str = "architecture/dependency-boundary layer names must not be blank";

const PACKAGE_NAME_REQUIRED: &str = "security/forbidden-dependency package names must not be blank";

const LAYER_NEEDS_BOTH: &str =
    "must declare both the paths it contains and the modules that name it";

const LAYER_POSITION: &str = "more than once; one entry decides its position";

const SCOPE_NAMES_NOTHING: &str = "names no paths, so it applies to nothing";

const IMPORT_NAME_REQUIRED: &str = "architecture/restricted-import module names must not be blank";

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Parse { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::UnsupportedVersion { version } => {
                write!(formatter, "unsupported configuration version: {version}")
            }
            Self::UnknownSuite { name } => write!(
                formatter,
                "unknown suite {name}; available suites are {}",
                suites::names().collect::<Vec<_>>().join(", ")
            ),
            Self::DuplicateRestrictedCallName { name } => {
                duplicate(formatter, "architecture/restricted-call", name)
            }
            Self::BlankAllowIn { rule } => {
                write!(formatter, "{rule} path patterns must not be blank")
            }
            Self::InvalidExclude { pattern } => {
                write!(formatter, "exclude pattern must not be blank: {pattern:?}")
            }
            Self::InvalidComplexityLimit => formatter.write_str(COMPLEXITY_AT_LEAST_ONE),
            Self::InvalidTodoMarkers => formatter.write_str(TODO_MARKERS_REQUIRED),
            Self::InvalidTodoReferencePrefixes => formatter.write_str(TODO_PREFIXES_REQUIRED),
            Self::InvalidRestrictedCallName => formatter.write_str(CALL_NAME_REQUIRED),
            Self::InvalidRestrictedImportName => formatter.write_str(IMPORT_NAME_REQUIRED),
            Self::InvalidLayerName => formatter.write_str(LAYER_NAME_REQUIRED),
            Self::InvalidPackageName => formatter.write_str(PACKAGE_NAME_REQUIRED),
            Self::EmptyNamingScope { case } => write!(
                formatter,
                "architecture/filename-case scope for {case} {SCOPE_NAMES_NOTHING}"
            ),
            Self::DuplicatePackageName { name } => {
                duplicate(formatter, "security/forbidden-dependency", name)
            }
            Self::EmptyLayer { rule, name } => {
                write!(formatter, "{rule} {name} {LAYER_NEEDS_BOTH}")
            }
            Self::DuplicateLayerName { rule, name } => {
                write!(formatter, "{rule} lists {name} {LAYER_POSITION}")
            }
            Self::DuplicateRestrictedImportName { name } => {
                duplicate(formatter, "architecture/restricted-import", name)
            }
        }
    }
}

fn duplicate(formatter: &mut fmt::Formatter<'_>, rule: &str, name: &str) -> fmt::Result {
    write!(
        formatter,
        "{rule} lists {name} more than once; one entry decides its allow-in boundary"
    )
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Read { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::UnsupportedVersion { .. }
            | Self::InvalidComplexityLimit
            | Self::InvalidTodoMarkers
            | Self::InvalidTodoReferencePrefixes
            | Self::UnknownSuite { .. }
            | Self::InvalidRestrictedCallName
            | Self::DuplicateRestrictedCallName { .. }
            | Self::InvalidRestrictedImportName
            | Self::DuplicateRestrictedImportName { .. }
            | Self::InvalidLayerName
            | Self::InvalidPackageName
            | Self::EmptyNamingScope { .. }
            | Self::DuplicatePackageName { .. }
            | Self::EmptyLayer { .. }
            | Self::DuplicateLayerName { .. }
            | Self::BlankAllowIn { .. }
            | Self::InvalidExclude { .. } => None,
        }
    }
}
