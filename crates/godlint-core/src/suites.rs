use std::num::NonZeroU32;

use crate::config::{
    AccountableSuppressionRule, DecisionComplexityRule, DirectEnvironmentReadRule,
    EmptyFunctionRule, FunctionNestingRule, FunctionStatementsRule, LineLimitRule, NoCommentsRule,
    NoDynamicExecutionRule, ParameterCountRule, RestrictedCallRule, ReturnCountRule, Rules,
    Severity, TodoRequiresReferenceRule, UnusedSuppressionRule,
};

pub const RECOMMENDED: &str = "recommended@1";

pub const NAMES: [&str; 1] = [RECOMMENDED];

pub fn apply(name: &str, rules: &mut Rules) {
    if name == RECOMMENDED {
        recommended(rules);
    }
}

fn recommended(rules: &mut Rules) {
    let error = Severity::Error;

    rules.function_size.get_or_insert(LineLimitRule {
        severity: error,
        max_lines: lines(50),
        skip_blank_lines: true,
        skip_comments: true,
    });
    rules.file_size.get_or_insert(LineLimitRule {
        severity: error,
        max_lines: lines(500),
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
    rules
        .todo_requires_reference
        .get_or_insert(TodoRequiresReferenceRule {
            severity: error,
            markers: default_markers(),
            reference_prefixes: default_prefixes(),
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
    rules.restricted_call.get_or_insert(RestrictedCallRule {
        severity: error,
        calls: Vec::new(),
    });
    rules
        .no_dynamic_execution
        .get_or_insert(NoDynamicExecutionRule { severity: error });
    rules
        .direct_environment_read
        .get_or_insert(DirectEnvironmentReadRule {
            severity: error,
            allow_in: default_configuration_paths(),
        });
}

fn lines(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).unwrap_or(NonZeroU32::MIN)
}

fn default_markers() -> Vec<String> {
    vec!["TODO".into(), "FIXME".into(), "HACK".into(), "XXX".into()]
}

fn default_prefixes() -> Vec<String> {
    vec!["#".into()]
}

fn default_configuration_paths() -> Vec<String> {
    vec!["**/config.*".into(), "**/config/**".into()]
}
