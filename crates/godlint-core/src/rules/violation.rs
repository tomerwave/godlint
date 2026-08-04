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
    WorkflowCommentNotPermitted,
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
    UndeclaredPermissions,
    InheritedPermissions {
        job: String,
    },
    HardcodedContainerCredential {
        key: String,
        job: String,
    },
    MutableActionReference {
        reference: String,
        unversioned: bool,
    },
    UnlabelledActionPin {
        action: String,
    },
    ContradictoryActionLabels {
        action: String,
        sha: String,
        labels: Vec<String>,
    },
    ContradictoryActionPins {
        action: String,
        label: String,
        shas: Vec<String>,
    },
    TemplateInjection {
        expression: String,
        certain: bool,
    },
    AttackerInfluencedBotCondition {
        expression: String,
    },
    InheritedSecrets {
        job: String,
    },
    OverprovisionedSecrets {
        setting: String,
    },
    UnredactedSecret,
    StepContinuesOnError,
    JobContinuesOnError,
    ScriptOrTrue,
    ScriptExitsSuccessfully {
        ending: &'static str,
    },
    TestHelperInProduction {
        module: String,
        segment: String,
    },
    InternalImport {
        module: String,
        marker: String,
        certain: bool,
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
    "Suppression silences nothing; remove it, or restore the rule it names to this path.";

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

const INTERNAL_IMPORT: &str = concat!(
    "reaches past the package's public surface, so nothing promises it will keep working; import ",
    "from the entry point instead."
);

const COMMENT_NOT_PERMITTED: &str = "Comment is not permitted; express the intent in the code.";

const WORKFLOW_COMMENT_NOT_PERMITTED: &str =
    "Workflow comment is not permitted; name the step clearly or move the rationale into docs/.";

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

fn independent(formatter: &mut fmt::Formatter<'_>, set: &str, from: &str, to: &str) -> fmt::Result {
    write!(
        formatter,
        "{from} must not depend on {to}; {set} declares them independent of each other."
    )
}

fn environment(formatter: &mut fmt::Formatter<'_>, target: &str) -> fmt::Result {
    write!(formatter, "{target} {ENVIRONMENT_READ}")
}

fn helper(formatter: &mut fmt::Formatter<'_>, module: &str, segment: &str) -> fmt::Result {
    write!(formatter, "{module} names {segment}, {TEST_HELPER}")
}

fn internal(formatter: &mut fmt::Formatter<'_>, module: &str, marker: &str) -> fmt::Result {
    write!(
        formatter,
        "{module} names {marker}, which {INTERNAL_IMPORT}"
    )
}

fn insecure_random(formatter: &mut fmt::Formatter<'_>, callee: &str, secure: &str) -> fmt::Result {
    write!(
        formatter,
        "{callee} is predictable; use {secure} for a value that must not be guessable."
    )
}

fn template_injection(
    formatter: &mut fmt::Formatter<'_>,
    expression: &str,
    certain: bool,
) -> fmt::Result {
    if certain {
        write!(
            formatter,
            "\"{expression}\" is attacker-influenced, and the runner expands it into the script \
             before the shell runs; bind it to an env variable and reference the variable quoted."
        )
    } else {
        write!(
            formatter,
            "\"{expression}\" comes from whoever triggered the workflow. Reaching it through \
             workflow_dispatch requires write access, but a calling workflow may pass a value it \
             does not control; bind it to an env variable and reference the variable quoted."
        )
    }
}

fn bot(formatter: &mut fmt::Formatter<'_>, expression: &str) -> fmt::Result {
    write!(
        formatter,
        "\"{expression}\" checks an actor field that is attacker-influenced on the trigger, so \
         passing it proves nothing; compare github.event.pull_request.user.login or verify the app \
         instead."
    )
}

impl Violation {
    pub(crate) fn cap(&self) -> Severity {
        match self {
            Self::InternalImport { certain: false, .. } => Severity::Warning,
            Self::UnlabelledActionPin { .. }
            | Self::StepContinuesOnError
            | Self::JobContinuesOnError
            | Self::ScriptOrTrue
            | Self::MissingAssertion
            | Self::UnverifiedHash { .. }
            | Self::TemplateInjection { certain: false, .. } => Severity::Warning,
            _ => Severity::Error,
        }
    }
}

impl fmt::Display for Violation {
    #[rustfmt::skip]
    // godlint-ignore-next-line maintainability/function-size owner=tomer expires=2026-12-31 -- #208 will remove the size pressure without weakening exhaustiveness
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
            Self::WorkflowCommentNotPermitted => formatter.write_str(WORKFLOW_COMMENT_NOT_PERMITTED),
            Self::UnaccountableSuppression { defect } => defect.fmt(formatter),
            Self::UnusedSuppression => formatter.write_str(UNUSED_SUPPRESSION),
            Self::RestrictedCall { callee } => write!(formatter, "{callee} {RESTRICTED_CALL}"),
            Self::DynamicExecution { callee } => write!(formatter, "{callee} {DYNAMIC_EXECUTION}"),
            Self::DirectEnvironmentRead { target } => environment(formatter, target),
            Self::TimerWithoutDelay { callee } => write!(formatter, "{callee} {TIMER_DELAY}"),
            Self::FilenameCase { name, case } => filename_case(formatter, name, case),
            Self::ForbiddenDependency { package } => forbidden(formatter, package),
            Self::CrossedBoundary { from, to } => crossed_boundary(formatter, from, to),
            Self::BrokeIndependence { set, from, to } => independent(formatter, set, from, to),
            Self::RestrictedImport { module } => write!(formatter, "{module} {RESTRICTED_IMPORT}"),
            Self::ProductionLog { callee } => write!(formatter, "{callee} {PRODUCTION_LOG}"),
            Self::WeakHash { weak, strong } => weak_hash(formatter, weak, strong),
            Self::UnverifiedHash { callee } => unverified_hash(formatter, callee),
            Self::FocusedTest => formatter.write_str(FOCUSED_TEST),
            Self::SkippedTest => formatter.write_str(SKIPPED_TEST),
            Self::EmptyTest => formatter.write_str(EMPTY_TEST),
            Self::MissingAssertion => formatter.write_str(MISSING_ASSERTION),
            Self::ShellCommand { shell } => write!(formatter, "{shell} {SHELL_COMMAND}"),
            Self::UndeclaredPermissions => formatter.write_str(UNDECLARED_PERMISSIONS),
            Self::InheritedPermissions { job } => inherited(formatter, job),
            Self::HardcodedContainerCredential { key, job } => credential(formatter, key, job),
            Self::MutableActionReference { reference, unversioned } => mutable_action(formatter, reference, *unversioned),
            Self::UnlabelledActionPin { action } => unlabelled_action_pin(formatter, action),
            Self::ContradictoryActionLabels { action, sha, labels } => contradictory_action_labels(formatter, action, sha, labels),
            Self::ContradictoryActionPins { action, label, shas } => contradictory_action_pins(formatter, action, label, shas),
            Self::TemplateInjection { expression, certain } => template_injection(formatter, expression, *certain),
            Self::AttackerInfluencedBotCondition { expression } => bot(formatter, expression),
            Self::InheritedSecrets { job } => inherited_secrets(formatter, job),
            Self::OverprovisionedSecrets { setting } => overprovisioned_secrets(formatter, setting),
            Self::UnredactedSecret => formatter.write_str(UNREDACTED_SECRET),
            Self::StepContinuesOnError => formatter.write_str(STEP_CONTINUES_ON_ERROR),
            Self::JobContinuesOnError => formatter.write_str(JOB_CONTINUES_ON_ERROR),
            Self::ScriptOrTrue => formatter.write_str(SCRIPT_OR_TRUE),
            Self::ScriptExitsSuccessfully { ending } => script_exits_successfully(formatter, ending),
            Self::TestHelperInProduction { module, segment } => helper(formatter, module, segment),
            Self::InternalImport { module, marker, .. } => internal(formatter, module, marker),
            Self::SleepInTest { callee } => write!(formatter, "{callee} {SLEEP_IN_TEST}"),
            Self::UnseededRandom { callee, remedy } => unseeded(formatter, callee, remedy),
            Self::NetworkInUnitTest { callee } => network(formatter, callee),
            Self::InsecureRandom { callee, secure } => insecure_random(formatter, callee, secure),
        }
    }
}

const UNDECLARED_PERMISSIONS: &str = "No permissions are declared anywhere in this workflow, so \
     every job runs with whatever the repository grants by default; declare what the workflow needs.";

const UNREDACTED_SECRET: &str = "This script writes a secret to GITHUB_ENV or GITHUB_OUTPUT, where \
     GitHub's masking does not follow it; keep the secret in its original context.";

const STEP_CONTINUES_ON_ERROR: &str = "This step can fail while the job stays green, and no later \
     expression reads its outcome; remove continue-on-error or inspect steps.<id>.outcome.";

const JOB_CONTINUES_ON_ERROR: &str = "This job can fail while the workflow stays green; remove \
     continue-on-error or exclude the workflow when the soft failure is deliberate.";

const SCRIPT_OR_TRUE: &str = "This script ends with `|| true`, so the step stays green when the \
     command fails; remove the fallback or check the command's status explicitly.";

fn script_exits_successfully(formatter: &mut fmt::Formatter<'_>, ending: &str) -> fmt::Result {
    write!(
        formatter,
        "This script ends with `{ending}`, so the step cannot report failure; preserve the failing exit status."
    )
}

fn inherited_secrets(formatter: &mut fmt::Formatter<'_>, job: &str) -> fmt::Result {
    write!(
        formatter,
        "The called workflow receives every secret available to {job}, not only the secrets it \
         needs; name each secret explicitly."
    )
}

fn overprovisioned_secrets(formatter: &mut fmt::Formatter<'_>, setting: &str) -> fmt::Result {
    write!(
        formatter,
        "{setting} receives the whole secrets context; pass only the named secret it needs."
    )
}

fn filename_case(formatter: &mut fmt::Formatter<'_>, name: &str, case: &str) -> fmt::Result {
    write!(formatter, "{name} is not {case}; rename the file to match.")
}

fn forbidden(formatter: &mut fmt::Formatter<'_>, package: &str) -> fmt::Result {
    write!(formatter, "{package} {FORBIDDEN_DEPENDENCY}")
}

fn network(formatter: &mut fmt::Formatter<'_>, callee: &str) -> fmt::Result {
    write!(formatter, "{callee} {NETWORK_IN_UNIT_TEST}")
}

fn inherited(formatter: &mut fmt::Formatter<'_>, job: &str) -> fmt::Result {
    write!(
        formatter,
        "{job} declares no permissions and the workflow declares none either, so it runs with \
         whatever the repository grants by default; declare what it needs."
    )
}

fn credential(formatter: &mut fmt::Formatter<'_>, key: &str, job: &str) -> fmt::Result {
    write!(
        formatter,
        "{job} has a literal container credential in {key}; interpolate a secret instead."
    )
}

fn mutable_action(
    formatter: &mut fmt::Formatter<'_>,
    reference: &str,
    unversioned: bool,
) -> fmt::Result {
    if unversioned {
        return write!(
            formatter,
            "{reference} names no version at all, so it runs whatever the default branch holds; \
             pin it to a commit SHA."
        );
    }

    write!(
        formatter,
        "{reference} is a mutable ref, so the action can change under you and it runs with your \
         workflow's token; pin it to a commit SHA."
    )
}

fn unlabelled_action_pin(formatter: &mut fmt::Formatter<'_>, action: &str) -> fmt::Result {
    write!(
        formatter,
        "{action} is pinned to an unreadable commit without a trailing version label; add the tag or version beside it."
    )
}

fn contradictory_action_labels(
    formatter: &mut fmt::Formatter<'_>,
    action: &str,
    sha: &str,
    labels: &[String],
) -> fmt::Result {
    write!(
        formatter,
        "{action}@{sha} carries contradictory version labels {}; make every occurrence state the same version.",
        labels.join(", ")
    )
}

fn contradictory_action_pins(
    formatter: &mut fmt::Formatter<'_>,
    action: &str,
    label: &str,
    shas: &[String],
) -> fmt::Result {
    write!(
        formatter,
        "{action} labels different commits as {label}: {}; update every occurrence to one commit or correct the labels.",
        shas.join(", ")
    )
}
