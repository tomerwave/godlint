use crate::{
    config::{Config, Severity},
    rules::{
        Rule, accountable_suppression::AccountableSuppression,
        decision_complexity::DecisionComplexity, dependency_boundary::DependencyBoundary,
        direct_environment_read::DirectEnvironmentRead, empty_function::EmptyFunction,
        explicit_timer_delay::ExplicitTimerDelay, file_size::FileSize, filename_case::FilenameCase,
        forbidden_dependency::ForbiddenDependency, function_nesting::FunctionNesting,
        function_size::FunctionSize, function_statements::FunctionStatements,
        no_comments::NoComments, no_dynamic_execution::NoDynamicExecution,
        no_production_log::NoProductionLog, parameter_count::ParameterCount,
        restricted_call::RestrictedCall, restricted_import::RestrictedImport,
        return_count::ReturnCount, todo_requires_reference::TodoRequiresReference,
        unused_suppression::UnusedSuppression,
    },
};

struct Registration {
    id: &'static str,
    severity: fn(&Config) -> Severity,
    suppressible: bool,
}

macro_rules! severity {
    ($name:ident, $rule:ty, $field:ident) => {
        fn $name(config: &Config) -> Severity {
            config
                .rules
                .$field
                .as_ref()
                .map_or(Severity::Off, <$rule as Rule>::severity)
        }
    };
}

severity!(function_size_severity, FunctionSize, function_size);
severity!(function_nesting_severity, FunctionNesting, function_nesting);
severity!(file_size_severity, FileSize, file_size);
severity!(empty_function_severity, EmptyFunction, empty_function);
severity!(
    todo_requires_reference_severity,
    TodoRequiresReference,
    todo_requires_reference
);
severity!(parameter_count_severity, ParameterCount, parameter_count);
severity!(
    decision_complexity_severity,
    DecisionComplexity,
    decision_complexity
);
severity!(return_count_severity, ReturnCount, return_count);
severity!(
    function_statements_severity,
    FunctionStatements,
    function_statements
);
severity!(no_comments_severity, NoComments, no_comments);
severity!(
    accountable_suppression_severity,
    AccountableSuppression,
    accountable_suppression
);
severity!(
    unused_suppression_severity,
    UnusedSuppression,
    unused_suppression
);

severity!(restricted_call_severity, RestrictedCall, restricted_call);
severity!(
    no_dynamic_execution_severity,
    NoDynamicExecution,
    no_dynamic_execution
);
severity!(
    direct_environment_read_severity,
    DirectEnvironmentRead,
    direct_environment_read
);
severity!(
    explicit_timer_delay_severity,
    ExplicitTimerDelay,
    explicit_timer_delay
);
severity!(
    no_production_log_severity,
    NoProductionLog,
    no_production_log
);
severity!(
    restricted_import_severity,
    RestrictedImport,
    restricted_import
);
severity!(
    dependency_boundary_severity,
    DependencyBoundary,
    dependency_boundary
);
severity!(
    forbidden_dependency_severity,
    ForbiddenDependency,
    forbidden_dependency
);
severity!(filename_case_severity, FilenameCase, filename_case);

const REGISTRATIONS: &[Registration] = &[
    Registration {
        id: FunctionSize::ID,
        severity: function_size_severity,
        suppressible: true,
    },
    Registration {
        id: FunctionNesting::ID,
        severity: function_nesting_severity,
        suppressible: true,
    },
    Registration {
        id: FileSize::ID,
        severity: file_size_severity,
        suppressible: true,
    },
    Registration {
        id: EmptyFunction::ID,
        severity: empty_function_severity,
        suppressible: true,
    },
    Registration {
        id: TodoRequiresReference::ID,
        severity: todo_requires_reference_severity,
        suppressible: true,
    },
    Registration {
        id: ParameterCount::ID,
        severity: parameter_count_severity,
        suppressible: true,
    },
    Registration {
        id: DecisionComplexity::ID,
        severity: decision_complexity_severity,
        suppressible: true,
    },
    Registration {
        id: ReturnCount::ID,
        severity: return_count_severity,
        suppressible: true,
    },
    Registration {
        id: FunctionStatements::ID,
        severity: function_statements_severity,
        suppressible: true,
    },
    Registration {
        id: NoComments::ID,
        severity: no_comments_severity,
        suppressible: true,
    },
    Registration {
        id: AccountableSuppression::ID,
        severity: accountable_suppression_severity,
        suppressible: false,
    },
    Registration {
        id: UnusedSuppression::ID,
        severity: unused_suppression_severity,
        suppressible: false,
    },
    Registration {
        id: RestrictedCall::ID,
        severity: restricted_call_severity,
        suppressible: true,
    },
    Registration {
        id: NoDynamicExecution::ID,
        severity: no_dynamic_execution_severity,
        suppressible: true,
    },
    Registration {
        id: DirectEnvironmentRead::ID,
        severity: direct_environment_read_severity,
        suppressible: true,
    },
    Registration {
        id: ExplicitTimerDelay::ID,
        severity: explicit_timer_delay_severity,
        suppressible: true,
    },
    Registration {
        id: NoProductionLog::ID,
        severity: no_production_log_severity,
        suppressible: true,
    },
    Registration {
        id: RestrictedImport::ID,
        severity: restricted_import_severity,
        suppressible: true,
    },
    Registration {
        id: DependencyBoundary::ID,
        severity: dependency_boundary_severity,
        suppressible: true,
    },
    Registration {
        id: ForbiddenDependency::ID,
        severity: forbidden_dependency_severity,
        suppressible: true,
    },
    Registration {
        id: FilenameCase::ID,
        severity: filename_case_severity,
        suppressible: false,
    },
];

pub fn rule_ids() -> impl Iterator<Item = &'static str> {
    REGISTRATIONS.iter().map(|registration| registration.id)
}

pub fn is_known_rule(rule_id: &str) -> bool {
    registration(rule_id).is_some()
}

fn registration(rule_id: &str) -> Option<&'static Registration> {
    REGISTRATIONS
        .iter()
        .find(|registration| registration.id == rule_id)
}

pub fn configured_severity(config: &Config, rule_id: &str) -> Severity {
    registration(rule_id).map_or(Severity::Off, |registration| {
        (registration.severity)(config)
    })
}

pub fn is_suppressible_rule(rule_id: &str) -> bool {
    registration(rule_id).is_some_and(|registration| registration.suppressible)
}
