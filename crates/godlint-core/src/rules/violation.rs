use std::fmt;

use crate::rules::{Metric, SuppressionDefect};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Violation {
    Limit {
        metric: Metric,
        actual: u32,
        max: u32,
    },
    EmptyBody,
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
    RestrictedImport {
        module: String,
    },
    CrossedBoundary {
        from: String,
        to: String,
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

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Limit {
                metric,
                actual,
                max,
            } => metric.describe(formatter, *actual, *max),
            Self::EmptyBody => write!(formatter, "Function has an empty body."),
            Self::MissingReference { marker } => {
                write!(formatter, "{marker} comment requires an issue reference.")
            }
            Self::CommentNotPermitted => write!(
                formatter,
                "Comment is not permitted; express the intent in the code."
            ),
            Self::UnaccountableSuppression { defect } => defect.fmt(formatter),
            Self::UnusedSuppression => write!(
                formatter,
                "Suppression does not silence an enabled finding; remove it or narrow the rule."
            ),
            Self::RestrictedCall { callee } => {
                write!(formatter, "{callee} is restricted by project policy.")
            }
            Self::DynamicExecution { callee } => write!(
                formatter,
                "{callee} executes dynamically generated code; use an explicit, reviewed boundary instead."
            ),
            Self::DirectEnvironmentRead { target } => write!(
                formatter,
                "{target} reads environment directly; read configuration through a config boundary instead."
            ),
            Self::TimerWithoutDelay { callee } => write!(
                formatter,
                "{callee} needs an explicit delay; pass the intended delay in milliseconds."
            ),
            Self::CrossedBoundary { from, to } => write!(
                formatter,
                "{from} must not depend on {to}; the dependency runs against the declared layer order."
            ),
            Self::RestrictedImport { module } => write!(
                formatter,
                "{module} is restricted by project policy; import it through an approved boundary."
            ),
            Self::ProductionLog { callee } => write!(
                formatter,
                "{callee} logs from production code; route it through the project's logger or an approved path."
            ),
        }
    }
}
