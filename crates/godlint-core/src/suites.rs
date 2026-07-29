use std::num::NonZeroU32;

use crate::config::{
    AccountableSuppressionRule, DecisionComplexityRule, DirectEnvironmentReadRule,
    EmptyFunctionRule, FunctionNestingRule, FunctionStatementsRule, LineLimitRule, NoCommentsRule,
    NoDynamicExecutionRule, ParameterCountRule, RestrictedCallRule, ReturnCountRule, Rules,
    Severity, TodoRequiresReferenceRule, UnusedSuppressionRule, default_configuration_paths,
    default_markers, default_reference_prefixes,
};

pub const RECOMMENDED: &str = "recommended@1";

const FUNCTION_LINES: NonZeroU32 = NonZeroU32::new(50).expect("50 is not zero");

const FILE_LINES: NonZeroU32 = NonZeroU32::new(500).expect("500 is not zero");

pub const NAMES: [&str; 1] = [RECOMMENDED];

pub fn apply(name: &str, rules: &mut Rules) {
    if name == RECOMMENDED {
        recommended(rules);
    }
}

fn recommended(rules: &mut Rules) {
    maintainability(rules);
    policy(rules);
    security(rules);
}

fn maintainability(rules: &mut Rules) {
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
    rules
        .decision_complexity
        .get_or_insert(DecisionComplexityRule {
            severity: error,
            max_complexity: 5,
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
