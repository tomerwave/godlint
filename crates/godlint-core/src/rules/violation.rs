use std::fmt;

use crate::{
    config::Severity,
    rules::{Metric, SuppressionDefect},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Violation {
    Limit {
        metric: Metric,
        actual: u32,
        max: u32,
    },
    EmptyBody,
    EmptyErrorHandler,
    MissingReference {
        marker: String,
    },
    CommentNotPermitted,
    UnaccountableSuppression {
        defect: SuppressionDefect,
    },
    UnusedSuppression,
    RestrictedCall {
        callee: String,
    },
    DynamicExecution {
        callee: String,
    },
    DirectEnvironmentRead {
        target: String,
    },
    TimerWithoutDelay {
        callee: String,
    },
    ProductionLog {
        callee: String,
    },
    InsecureRandom {
        callee: String,
        secure: String,
    },
    WeakHash {
        weak: String,
        strong: String,
    },
    UnverifiedHash {
        callee: String,
    },
    FocusedTest,
    SkippedTest,
    RestrictedImport {
        module: String,
    },
    CrossedBoundary {
        from: String,
        to: String,
    },
    BrokeIndependence {
        set: String,
        from: String,
        to: String,
    },
    ForbiddenDependency {
        package: String,
    },
    FilenameCase {
        name: String,
        case: String,
    },
}

impl Violation {
    pub const fn limit(metric: Metric, actual: u32, max: u32) -> Self {
        Self::Limit {
            metric,
            actual,
            max,
        }
    }
}

const DYNAMIC_EXECUTION: &str =
    "executes dynamically generated code; use an explicit, reviewed boundary instead.";

const ENVIRONMENT_READ: &str =
    "reads environment directly; read configuration through a config boundary instead.";

const TIMER_DELAY: &str = "needs an explicit delay; pass the intended delay in milliseconds.";

const FORBIDDEN_DEPENDENCY: &str =
    "is a forbidden dependency; the policy that names it decides where it may be used.";

const RESTRICTED_IMPORT: &str =
    "is restricted by project policy; import it through an approved boundary.";

const PRODUCTION_LOG: &str =
    "logs from production code; route it through the project's logger or an approved path.";

const UNUSED_SUPPRESSION: &str =
    "Suppression does not silence an enabled finding; remove it or narrow the rule.";

const UNVERIFIED_HASH: &str = concat!(
    "takes its algorithm from a value Godlint cannot read; name the algorithm inline, ",
    "or confirm it is not a broken one."
);

const EMPTY_ERROR_HANDLER: &str = "Error handler has an empty body; handle or re-raise the error.";

const MISSING_REFERENCE: &str = "comment requires an issue reference.";

const RESTRICTED_CALL: &str = "is restricted by project policy.";

const CROSSED_BOUNDARY: &str = "the dependency runs against the declared layer order.";

const FOCUSED_TEST: &str = concat!(
    "This test is focused, so the rest of the suite does not run; remove the focus before ",
    "merging, because a green run then proves almost nothing."
);

const SKIPPED_TEST: &str = concat!(
    "This test does not run, so it can rot without anything noticing; delete it, fix it, or ",
    "suppress it with an owner and an expiry."
);

const COMMENT_NOT_PERMITTED: &str = "Comment is not permitted; express the intent in the code.";

fn unverified_hash(formatter: &mut fmt::Formatter<'_>, callee: &str) -> fmt::Result {
    write!(formatter, "{callee} {UNVERIFIED_HASH}")
}

fn weak_hash(formatter: &mut fmt::Formatter<'_>, weak: &str, strong: &str) -> fmt::Result {
    write!(
        formatter,
        "{weak} is not collision resistant; use {strong} where collision resistance matters."
    )
}

fn insecure_random(formatter: &mut fmt::Formatter<'_>, callee: &str, secure: &str) -> fmt::Result {
    write!(
        formatter,
        "{callee} is predictable; use {secure} for a value that must not be guessable."
    )
}

impl Violation {
    pub(crate) fn cap(&self) -> Severity {
        match self {
            Self::UnverifiedHash { .. } => Severity::Warning,
            _ => Severity::Error,
        }
    }
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit {
                metric,
                actual,
                max,
            } => metric.describe(formatter, *actual, *max),
            Self::EmptyBody => write!(formatter, "Function has an empty body."),
            Self::EmptyErrorHandler => formatter.write_str(EMPTY_ERROR_HANDLER),
            Self::MissingReference { marker } => write!(formatter, "{marker} {MISSING_REFERENCE}"),
            Self::CommentNotPermitted => formatter.write_str(COMMENT_NOT_PERMITTED),
            Self::UnaccountableSuppression { defect } => defect.fmt(formatter),
            Self::UnusedSuppression => formatter.write_str(UNUSED_SUPPRESSION),
            Self::RestrictedCall { callee } => write!(formatter, "{callee} {RESTRICTED_CALL}"),
            Self::DynamicExecution { callee } => write!(formatter, "{callee} {DYNAMIC_EXECUTION}"),
            Self::DirectEnvironmentRead { target } => {
                write!(formatter, "{target} {ENVIRONMENT_READ}")
            }
            Self::TimerWithoutDelay { callee } => write!(formatter, "{callee} {TIMER_DELAY}"),
            Self::FilenameCase { name, case } => {
                write!(formatter, "{name} is not {case}; rename the file to match.")
            }
            Self::ForbiddenDependency { package } => {
                write!(formatter, "{package} {FORBIDDEN_DEPENDENCY}")
            }
            Self::CrossedBoundary { from, to } => {
                write!(
                    formatter,
                    "{from} must not depend on {to}; {CROSSED_BOUNDARY}"
                )
            }
            Self::BrokeIndependence { set, from, to } => write!(
                formatter,
                "{from} must not depend on {to}; {set} declares them independent of each other."
            ),
            Self::RestrictedImport { module } => write!(formatter, "{module} {RESTRICTED_IMPORT}"),
            Self::ProductionLog { callee } => write!(formatter, "{callee} {PRODUCTION_LOG}"),
            Self::WeakHash { weak, strong } => weak_hash(formatter, weak, strong),
            Self::UnverifiedHash { callee } => unverified_hash(formatter, callee),
            Self::FocusedTest => write!(formatter, "{FOCUSED_TEST}"),
            Self::SkippedTest => write!(formatter, "{SKIPPED_TEST}"),
            Self::InsecureRandom { callee, secure } => insecure_random(formatter, callee, secure),
        }
    }
}
