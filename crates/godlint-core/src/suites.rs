use std::num::NonZeroU32;

use crate::config::{
    AccountableSuppressionRule, CognitiveComplexityRule, ConditionComplexityRule,
    DecisionComplexityRule, DependencyBoundaryRule, DirectEnvironmentReadRule,
    EmptyErrorHandlerRule, EmptyFunctionRule, ExplicitTimerDelayRule, FilenameCaseRule,
    ForbiddenDependencyRule, FunctionNestingRule, FunctionStatementsRule, LineLimitRule,
    ModuleIndependenceRule, NoCommentsRule, NoDynamicExecutionRule, NoProductionLogRule,
    ParameterCountRule, RestrictedCallRule, RestrictedImportRule, ReturnCountRule, Rules, Severity,
    TodoRequiresReferenceRule, UnusedSuppressionRule, default_configuration_paths, default_markers,
    default_reference_prefixes,
};

pub const RECOMMENDED: &str = "recommended@1";

type Expand = fn(&mut Rules);

const SUITES: &[(&str, Expand)] = &[(RECOMMENDED, recommended)];

const FUNCTION_LINES: NonZeroU32 = NonZeroU32::new(50).expect("50 is not zero");

const FILE_LINES: NonZeroU32 = NonZeroU32::new(500).expect("500 is not zero");

pub fn names() -> impl Iterator<Item = &'static str> {
    SUITES.iter().map(|(name, _)| *name)
}

pub fn lookup(name: &str) -> Option<Expand> {
    SUITES
        .iter()
        .find(|(known, _)| *known == name)
        .map(|(_, expand)| *expand)
}

fn recommended(rules: &mut Rules) {
    maintainability(rules);
    policy(rules);
    security(rules);
    reliability(rules);
    logging(rules);
}

fn maintainability(rules: &mut Rules) {
    size(rules);
    complexity(rules);
}

fn size(rules: &mut Rules) {
    let error = Severity::Error;

    rules.function_size.get_or_insert(LineLimitRule {
        severity: error,
        max_lines: FUNCTION_LINES,
        skip_blank_lines: true,
        skip_comments: true,
    });
    rules.file_size.get_or_insert(LineLimitRule {
        severity: error,
        max_lines: FILE_LINES,
        skip_blank_lines: true,
        skip_comments: true,
    });
    rules.function_nesting.get_or_insert(FunctionNestingRule {
        severity: error,
        max_depth: 2,
    });
    rules.parameter_count.get_or_insert(ParameterCountRule {
        severity: error,
        max_parameters: 4,
    });
}

fn complexity(rules: &mut Rules) {
    let error = Severity::Error;

    rules
        .decision_complexity
        .get_or_insert(DecisionComplexityRule {
            severity: error,
            max_complexity: 5,
        });
    rules
        .condition_complexity
        .get_or_insert(ConditionComplexityRule {
            severity: error,
            max_operators: 3,
        });
    rules
        .cognitive_complexity
        .get_or_insert(CognitiveComplexityRule {
            severity: error,
            max_score: 15,
        });
    rules.return_count.get_or_insert(ReturnCountRule {
        severity: error,
        max_returns: 6,
    });
    rules
        .function_statements
        .get_or_insert(FunctionStatementsRule {
            severity: error,
            max_statements: 14,
        });
    rules.empty_function.get_or_insert(EmptyFunctionRule {
        severity: error,
        allow_names: Vec::new(),
    });
}

fn policy(rules: &mut Rules) {
    let error = Severity::Error;

    rules
        .todo_requires_reference
        .get_or_insert_with(|| TodoRequiresReferenceRule {
            severity: error,
            markers: default_markers(),
            reference_prefixes: default_reference_prefixes(),
        });
    rules.no_comments.get_or_insert(NoCommentsRule {
        severity: error,
        allow_doc_comments: false,
    });
    rules
        .accountable_suppression
        .get_or_insert(AccountableSuppressionRule {
            severity: error,
            require_owner: true,
            require_expiry: true,
        });
    rules
        .unused_suppression
        .get_or_insert(UnusedSuppressionRule { severity: error });
}

fn security(rules: &mut Rules) {
    let error = Severity::Error;

    rules.restricted_call.get_or_insert(RestrictedCallRule {
        severity: error,
        calls: Vec::new(),
    });
    rules.filename_case.get_or_insert(FilenameCaseRule {
        severity: error,
        scopes: Vec::new(),
        allow: Vec::new(),
    });
    rules.restricted_import.get_or_insert(RestrictedImportRule {
        severity: error,
        modules: Vec::new(),
    });
    rules
        .dependency_boundary
        .get_or_insert(DependencyBoundaryRule {
            severity: error,
            layers: Vec::new(),
        });
    rules
        .forbidden_dependency
        .get_or_insert(ForbiddenDependencyRule {
            severity: error,
            packages: Vec::new(),
        });
    rules
        .module_independence
        .get_or_insert(ModuleIndependenceRule {
            severity: error,
            sets: Vec::new(),
        });
    rules
        .no_dynamic_execution
        .get_or_insert(NoDynamicExecutionRule { severity: error });
    rules
        .direct_environment_read
        .get_or_insert_with(|| DirectEnvironmentReadRule {
            severity: error,
            allow_in: default_configuration_paths(),
        });
}

fn reliability(rules: &mut Rules) {
    rules
        .explicit_timer_delay
        .get_or_insert(ExplicitTimerDelayRule {
            severity: Severity::Error,
        });
    rules
        .empty_error_handler
        .get_or_insert(EmptyErrorHandlerRule {
            severity: Severity::Error,
        });
}

fn logging(rules: &mut Rules) {
    rules.no_production_log.get_or_insert(NoProductionLogRule {
        severity: Severity::Error,
        allow_in: Vec::new(),
    });
}
