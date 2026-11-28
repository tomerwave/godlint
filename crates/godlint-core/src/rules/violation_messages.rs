pub(super) const DYNAMIC_EXECUTION: &str =
    "executes dynamically generated code; use an explicit, reviewed boundary instead.";

pub(super) const ENVIRONMENT_READ: &str =
    "reads environment directly; read configuration through a config boundary instead.";

pub(super) const TIMER_DELAY: &str =
    "needs an explicit delay; pass the intended delay in milliseconds.";

pub(super) const FORBIDDEN_DEPENDENCY: &str =
    "is a forbidden dependency; the policy that names it decides where it may be used.";

pub(super) const RESTRICTED_IMPORT: &str =
    "is restricted by project policy; import it through an approved boundary.";

pub(super) const PRODUCTION_LOG: &str =
    "logs from production code; route it through the project's logger or an approved path.";

pub(super) const UNPROTECTED_STATEFUL_RESOURCE: &str = "holds data but is not created with `protect: true`; set it in the resource options so a `pulumi up` or `pulumi destroy` cannot delete it.";

pub(super) const UNCHECKED_CONFIG: &str = "casts configuration to its type argument without checking it; drop the type argument and validate the value with a schema, such as `schema.parse(config.requireObject(\"key\"))`.";

pub(super) const NETWORK_TIMEOUT: &str =
    "has no explicit timeout; bound network calls so they cannot wait forever.";
pub(super) const NO_CONTROL_FLOW_IN_FINALLY: &str = "A finally block contains escaping control flow; return, break, and continue can suppress errors or skip cleanup.";
pub(super) const REDUNDANT_CATCH_RETHROW: &str =
    "This handler only rethrows the caught error; remove it or add useful handling or context.";
pub(super) const COMMITTED_SECRET_FILE: &str = "This filename commonly contains credentials or private keys; keep it out of version control and provide a safe example instead.";
pub(super) const COMMENTED_CODE: &str =
    "This comment appears to be disabled source code; delete it or restore it as executable code.";
pub(super) const DUPLICATE_STRING: &str =
    "is repeated in this file; give the shared value a named constant.";

pub(super) const UNUSED_SUPPRESSION: &str =
    "Suppression silences nothing; remove it, or restore the rule it names to this path.";

pub(super) const UNVERIFIED_HASH: &str = concat!(
    "takes its algorithm from a value Godlint cannot read; name the algorithm inline, ",
    "or confirm it is not a broken one."
);

pub(super) const EMPTY_ERROR_HANDLER: &str =
    "Error handler has an empty body; handle or re-raise the error.";

pub(super) const MISSING_REFERENCE: &str = "comment requires an issue reference.";

pub(super) const RESTRICTED_CALL: &str = "is restricted by project policy.";

pub(super) const CROSSED_BOUNDARY: &str = "the dependency runs against the declared layer order.";

pub(super) const EMPTY_TEST: &str = "This test has an empty body, so it cannot fail; write the assertion or \
                          delete the test.";

pub(super) const FOCUSED_TEST: &str = concat!(
    "This test is focused, so the rest of the suite does not run; remove the focus before ",
    "merging, because a green run then proves almost nothing."
);

pub(super) const SKIPPED_TEST: &str = concat!(
    "This test does not run, so it can rot without anything noticing; delete it, fix it, or ",
    "suppress it with an owner and an expiry."
);

pub(super) const SLEEP_IN_TEST: &str = concat!(
    "makes this test wait on the clock, which is the usual cause of a flaky suite; wait for the ",
    "condition instead."
);

pub(super) const UNSEEDED_RANDOM: &str = "is unseeded, so a failure here cannot be reproduced;";

pub(super) const NETWORK_IN_UNIT_TEST: &str = concat!(
    "reaches the network from a unit test, which makes it slow, dependent on a service being up, ",
    "and unable to run offline; inject the client and fake it."
);

pub(super) const MISSING_ASSERTION: &str = concat!(
    "This test asserts nothing, so it passes unless the code raises; assert what the code should ",
    "do, or name the helper that asserts for it in extra-assertions."
);

pub(super) const SHELL_COMMAND: &str = concat!(
    "runs its argument through a shell, so any value interpolated into it becomes executable; ",
    "pass the program and its arguments as an array instead."
);

pub(super) const TEST_HELPER: &str = concat!(
    "which is test scaffolding, so production now depends on the test tree and ships it to users; ",
    "keep the fake in the tests and take an interface here."
);

pub(super) const INTERNAL_IMPORT: &str = concat!(
    "reaches past the package's public surface, so nothing promises it will keep working; import ",
    "from the entry point instead."
);

pub(super) const COMMENT_NOT_PERMITTED: &str =
    "Comment is not permitted; express the intent in the code.";

pub(super) const WORKFLOW_COMMENT_NOT_PERMITTED: &str =
    "Workflow comment is not permitted; name the step clearly or move the rationale into docs/.";
