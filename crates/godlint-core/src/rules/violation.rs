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
    EmptyTest,
    MissingAssertion,
    ShellCommand {
        shell: String,
    },
    TestHelperInProduction {
        module: String,
        segment: String,
    },
    SleepInTest {
        callee: String,
    },
    UnseededRandom {
        callee: String,
        remedy: String,
    },
    NetworkInUnitTest {
        callee: String,
    },
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

const EMPTY_TEST: &str = "This test has an empty body, so it cannot fail; write the assertion or \
                          delete the test.";

const FOCUSED_TEST: &str = concat!(
    "This test is focused, so the rest of the suite does not run; remove the focus before ",
    "merging, because a green run then proves almost nothing."
);

const SKIPPED_TEST: &str = concat!(
    "This test does not run, so it can rot without anything noticing; delete it, fix it, or ",
    "suppress it with an owner and an expiry."
);

const SLEEP_IN_TEST: &str = concat!(
    "makes this test wait on the clock, which is the usual cause of a flaky suite; wait for the ",
    "condition instead."
);

const UNSEEDED_RANDOM: &str = "is unseeded, so a failure here cannot be reproduced;";

const NETWORK_IN_UNIT_TEST: &str = concat!(
    "reaches the network from a unit test, which makes it slow, dependent on a service being up, ",
    "and unable to run offline; inject the client and fake it."
);

const MISSING_ASSERTION: &str = concat!(
    "This test asserts nothing, so it passes unless the code raises; assert what the code should ",
    "do, or name the helper that asserts for it in extra-assertions."
);

const SHELL_COMMAND: &str = concat!(
    "runs its argument through a shell, so any value interpolated into it becomes executable; ",
    "pass the program and its arguments as an array instead."
);

const TEST_HELPER: &str = concat!(
    "which is test scaffolding, so production now depends on the test tree and ships it to users; ",
    "keep the fake in the tests and take an interface here."
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

fn unseeded(formatter: &mut fmt::Formatter<'_>, callee: &str, remedy: &str) -> fmt::Result {
    write!(
        formatter,
        "{callee} {UNSEEDED_RANDOM} {remedy}, or use a fixed fixture."
    )
}

fn crossed_boundary(formatter: &mut fmt::Formatter<'_>, from: &str, to: &str) -> fmt::Result {
    write!(
        formatter,
        "{from} must not depend on {to}; {CROSSED_BOUNDARY}"
    )
}

fn broke_independence(
    formatter: &mut fmt::Formatter<'_>,
    set: &str,
    from: &str,
    to: &str,
) -> fmt::Result {
    write!(
        formatter,
        "{from} must not depend on {to}; {set} declares them independent of each other."
    )
}

fn environment(formatter: &mut fmt::Formatter<'_>, target: &str) -> fmt::Result {
    write!(formatter, "{target} {ENVIRONMENT_READ}")
}

fn test_helper(formatter: &mut fmt::Formatter<'_>, module: &str, segment: &str) -> fmt::Result {
    write!(formatter, "{module} names {segment}, {TEST_HELPER}")
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
            Self::MissingAssertion | Self::UnverifiedHash { .. } => Severity::Warning,
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
            Self::DirectEnvironmentRead { target } => environment(formatter, target),
            Self::TimerWithoutDelay { callee } => write!(formatter, "{callee} {TIMER_DELAY}"),
            Self::FilenameCase { name, case } => {
                write!(formatter, "{name} is not {case}; rename the file to match.")
            }
            Self::ForbiddenDependency { package } => {
                write!(formatter, "{package} {FORBIDDEN_DEPENDENCY}")
            }
            Self::CrossedBoundary { from, to } => crossed_boundary(formatter, from, to),
            Self::BrokeIndependence { set, from, to } => {
                broke_independence(formatter, set, from, to)
            }
            Self::RestrictedImport { module } => write!(formatter, "{module} {RESTRICTED_IMPORT}"),
            Self::ProductionLog { callee } => write!(formatter, "{callee} {PRODUCTION_LOG}"),
            Self::WeakHash { weak, strong } => weak_hash(formatter, weak, strong),
            Self::UnverifiedHash { callee } => unverified_hash(formatter, callee),
            Self::FocusedTest => write!(formatter, "{FOCUSED_TEST}"),
            Self::SkippedTest => write!(formatter, "{SKIPPED_TEST}"),
            Self::EmptyTest => formatter.write_str(EMPTY_TEST),
            Self::MissingAssertion => formatter.write_str(MISSING_ASSERTION),
            Self::ShellCommand { shell } => write!(formatter, "{shell} {SHELL_COMMAND}"),
            Self::TestHelperInProduction { module, segment } => {
                test_helper(formatter, module, segment)
            }
            Self::SleepInTest { callee } => write!(formatter, "{callee} {SLEEP_IN_TEST}"),
            Self::UnseededRandom { callee, remedy } => unseeded(formatter, callee, remedy),
            Self::NetworkInUnitTest { callee } => {
                write!(formatter, "{callee} {NETWORK_IN_UNIT_TEST}")
            }
            Self::InsecureRandom { callee, secure } => insecure_random(formatter, callee, secure),
        }
    }
}
