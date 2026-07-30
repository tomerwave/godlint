use std::collections::BTreeSet;

const DEPENDENCY_BOUNDARY: &str = "architecture/dependency-boundary layer";
const MODULE_INDEPENDENCE: &str = "architecture/module-independence member";

use crate::config::{
    Config, ConfigError, ForbiddenDependency, IndependentSet, Layer, RestrictedCall,
    RestrictedImport,
};

impl Config {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        [
            Self::validate_version,
            Self::validate_exclude,
            Self::validate_complexity_rule,
            Self::validate_todo_rule,
            Self::validate_restricted_call_rule,
            Self::validate_restricted_import_rule,
            Self::validate_dependency_boundary_rule,
            Self::validate_module_independence_rule,
            Self::validate_forbidden_dependency_rule,
            Self::validate_filename_case_rule,
            Self::validate_no_production_log_rule,
            Self::validate_empty_function_rule,
            Self::validate_direct_environment_read_rule,
        ]
        .iter()
        .try_for_each(|check| check(self))
    }

    fn validate_version(&self) -> Result<(), ConfigError> {
        match self.version {
            1 => Ok(()),
            version => Err(ConfigError::UnsupportedVersion { version }),
        }
    }

    fn validate_exclude(&self) -> Result<(), ConfigError> {
        match self
            .exclude
            .iter()
            .find(|pattern| pattern.trim().is_empty())
        {
            Some(pattern) => Err(ConfigError::InvalidExclude {
                pattern: pattern.clone(),
            }),
            None => Ok(()),
        }
    }

    fn validate_complexity_rule(&self) -> Result<(), ConfigError> {
        if self
            .rules
            .decision_complexity
            .as_ref()
            .is_some_and(|rule| rule.limit() == 0)
        {
            return Err(ConfigError::InvalidComplexityLimit);
        }

        Ok(())
    }

    fn validate_todo_rule(&self) -> Result<(), ConfigError> {
        let Some(rule) = &self.rules.todo_requires_reference else {
            return Ok(());
        };

        if rule.markers.is_empty() || any_blank(&rule.markers) {
            return Err(ConfigError::InvalidTodoMarkers);
        }

        if rule.reference_prefixes.is_empty()
            || rule
                .reference_prefixes
                .iter()
                .any(|prefix| prefix_is_unusable(prefix))
        {
            return Err(ConfigError::InvalidTodoReferencePrefixes);
        }

        Ok(())
    }

    fn validate_restricted_call_rule(&self) -> Result<(), ConfigError> {
        let Some(rule) = &self.rules.restricted_call else {
            return Ok(());
        };

        let mut seen = BTreeSet::new();

        for call in &rule.calls {
            validate_restricted_call(call)?;

            if !seen.insert(call.name.as_str()) {
                return Err(ConfigError::DuplicateRestrictedCallName {
                    name: call.name.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_restricted_import_rule(&self) -> Result<(), ConfigError> {
        let Some(rule) = &self.rules.restricted_import else {
            return Ok(());
        };

        let mut seen = BTreeSet::new();

        for module in &rule.modules {
            validate_restricted_import(module)?;

            if !seen.insert(module.name.as_str()) {
                return Err(ConfigError::DuplicateRestrictedImportName {
                    name: module.name.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_module_independence_rule(&self) -> Result<(), ConfigError> {
        let Some(rule) = &self.rules.module_independence else {
            return Ok(());
        };

        rule.sets.iter().try_for_each(validate_independent_set)
    }

    fn validate_dependency_boundary_rule(&self) -> Result<(), ConfigError> {
        let Some(rule) = &self.rules.dependency_boundary else {
            return Ok(());
        };

        let mut seen = BTreeSet::new();

        for layer in &rule.layers {
            validate_layer(DEPENDENCY_BOUNDARY, layer)?;

            if !seen.insert(layer.name.as_str()) {
                return Err(ConfigError::DuplicateLayerName {
                    rule: DEPENDENCY_BOUNDARY,
                    name: layer.name.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_forbidden_dependency_rule(&self) -> Result<(), ConfigError> {
        let Some(rule) = &self.rules.forbidden_dependency else {
            return Ok(());
        };

        let mut seen = BTreeSet::new();

        for package in &rule.packages {
            validate_forbidden_dependency(package)?;

            if !seen.insert(package.name.as_str()) {
                return Err(ConfigError::DuplicatePackageName {
                    name: package.name.clone(),
                });
            }
        }

        Ok(())
    }

    fn validate_filename_case_rule(&self) -> Result<(), ConfigError> {
        let Some(rule) = &self.rules.filename_case else {
            return Ok(());
        };

        if any_blank(&rule.allow) || rule.scopes.iter().any(|scope| any_blank(&scope.paths)) {
            return Err(ConfigError::BlankAllowIn {
                rule: "architecture/filename-case",
            });
        }

        match rule.scopes.iter().find(|scope| scope.paths.is_empty()) {
            Some(scope) => Err(ConfigError::EmptyNamingScope {
                case: scope.case.describe().to_owned(),
            }),
            None => Ok(()),
        }
    }

    fn validate_no_production_log_rule(&self) -> Result<(), ConfigError> {
        if self
            .rules
            .no_production_log
            .as_ref()
            .is_some_and(|rule| any_blank(&rule.allow_in))
        {
            return Err(ConfigError::BlankAllowIn {
                rule: "logging/no-production-log",
            });
        }

        Ok(())
    }

    fn validate_empty_function_rule(&self) -> Result<(), ConfigError> {
        if self
            .rules
            .empty_function
            .as_ref()
            .is_some_and(|rule| any_blank(&rule.allow_names))
        {
            return Err(ConfigError::BlankAllowIn {
                rule: "maintainability/empty-function",
            });
        }

        Ok(())
    }

    fn validate_direct_environment_read_rule(&self) -> Result<(), ConfigError> {
        if self
            .rules
            .direct_environment_read
            .as_ref()
            .is_some_and(|rule| any_blank(&rule.allow_in))
        {
            return Err(ConfigError::BlankAllowIn {
                rule: "security/direct-environment-read",
            });
        }

        Ok(())
    }
}

fn validate_restricted_call(call: &RestrictedCall) -> Result<(), ConfigError> {
    if call.name.trim().is_empty() {
        return Err(ConfigError::InvalidRestrictedCallName);
    }

    if any_blank(&call.allow_in) {
        return Err(ConfigError::BlankAllowIn {
            rule: "architecture/restricted-call",
        });
    }

    Ok(())
}

fn validate_restricted_import(module: &RestrictedImport) -> Result<(), ConfigError> {
    if module.name.trim().is_empty() {
        return Err(ConfigError::InvalidRestrictedImportName);
    }

    if any_blank(&module.allow_in) {
        return Err(ConfigError::BlankAllowIn {
            rule: "architecture/restricted-import",
        });
    }

    Ok(())
}

fn validate_forbidden_dependency(package: &ForbiddenDependency) -> Result<(), ConfigError> {
    if package.name.trim().is_empty() {
        return Err(ConfigError::InvalidPackageName);
    }

    if any_blank(&package.allow_in) {
        return Err(ConfigError::BlankAllowIn {
            rule: "security/forbidden-dependency",
        });
    }

    Ok(())
}

fn validate_independent_set(set: &IndependentSet) -> Result<(), ConfigError> {
    if set.name.trim().is_empty() {
        return Err(ConfigError::InvalidLayerName);
    }

    let mut seen = BTreeSet::new();

    for member in &set.members {
        validate_layer(MODULE_INDEPENDENCE, member)?;

        if !seen.insert(member.name.as_str()) {
            return Err(ConfigError::DuplicateLayerName {
                rule: MODULE_INDEPENDENCE,
                name: member.name.clone(),
            });
        }
    }

    Ok(())
}

fn validate_layer(rule: &'static str, layer: &Layer) -> Result<(), ConfigError> {
    if layer.name.trim().is_empty() {
        return Err(ConfigError::InvalidLayerName);
    }

    if layer.paths.is_empty() || layer.modules.is_empty() {
        return Err(ConfigError::EmptyLayer {
            rule,
            name: layer.name.clone(),
        });
    }

    if any_blank(&layer.paths) || any_blank(&layer.modules) {
        return Err(ConfigError::BlankAllowIn {
            rule: "architecture/dependency-boundary",
        });
    }

    Ok(())
}

fn any_blank(values: &[String]) -> bool {
    values.iter().any(|value| value.trim().is_empty())
}

fn prefix_is_unusable(prefix: &str) -> bool {
    let trimmed = prefix.trim();

    trimmed.is_empty() || trimmed.chars().all(|character| character.is_ascii_digit())
}
