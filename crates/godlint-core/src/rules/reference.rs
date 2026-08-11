use crate::{
    analyzers::{SourceFacts, workflow::WorkflowFacts},
    facts::{
        AccessFact, ActionFact, CallFact, ConditionFact, ErrorHandlerFact, FinallyFact, ImportFact,
        TestFact,
    },
    rules::{
        Finding, Ranged, Reporting, Rule, Violation, collect_ranged, enclosing::in_test, report,
    },
    source::SourceRange,
};

pub trait ActionRule: Rule {
    fn check(action: &ActionFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub fn evaluate_action_rule<R: ActionRule>(
    workflows: &[WorkflowFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    let reporting = Reporting::of::<R>(configuration);

    report(
        reporting,
        workflows.iter().flat_map(|workflow| {
            workflow.actions().iter().filter_map(|action| {
                R::check(action, configuration)
                    .map(|violation| (action.file(), action.range(), violation))
            })
        }),
    )
}

pub trait WorkflowRule: Rule {
    fn check(
        workflow: &WorkflowFacts,
        configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)>;
}

pub fn evaluate_workflow_rule<R: WorkflowRule>(
    workflows: &[WorkflowFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    let reporting = Reporting::of::<R>(configuration);

    report(
        reporting,
        workflows.iter().flat_map(|workflow| {
            R::check(workflow, configuration)
                .into_iter()
                .map(|(range, violation)| (workflow.file(), range, violation))
        }),
    )
}

pub trait CallRule: Rule {
    fn check(call: &CallFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub trait CallInTestRule: Rule {
    fn check(
        call: &CallFact,
        facts: &SourceFacts,
        configuration: &Self::Configuration,
    ) -> Option<Violation>;
}

pub trait AccessRule: Rule {
    fn check(access: &AccessFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub trait ImportRule: Rule {
    fn check(import: &ImportFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub trait ErrorHandlerRule: Rule {
    fn check(
        error_handler: &ErrorHandlerFact,
        configuration: &Self::Configuration,
    ) -> Option<Violation>;
}

pub trait FinallyRule: Rule {
    fn check(finally_block: &FinallyFact, configuration: &Self::Configuration)
    -> Option<Violation>;
}

pub trait TestRule: Rule {
    fn check(test: &TestFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub trait ConditionRule: Rule {
    fn check(condition: &ConditionFact, configuration: &Self::Configuration) -> Option<Violation>;
}

pub fn evaluate_call_rule<R: CallRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::calls,
        |call, _| R::check(call, configuration),
    )
}

pub fn evaluate_test_rule<R: TestRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::tests,
        |test, _| R::check(test, configuration),
    )
}

pub fn evaluate_call_in_test_rule<R: CallInTestRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::calls,
        |call, source| {
            in_test(source, call.range())
                .then(|| R::check(call, source, configuration))
                .flatten()
        },
    )
}

pub fn evaluate_access_rule<R: AccessRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::accesses,
        |access, _| R::check(access, configuration),
    )
}

impl Ranged for TestFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

impl Ranged for CallFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

impl Ranged for ImportFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

impl Ranged for AccessFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

pub fn evaluate_import_rule<R: ImportRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::imports,
        |import, _| R::check(import, configuration),
    )
}

impl Ranged for ErrorHandlerFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

pub fn evaluate_error_handler_rule<R: ErrorHandlerRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::error_handlers,
        |error_handler, _| R::check(error_handler, configuration),
    )
}

impl Ranged for FinallyFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

pub fn evaluate_finally_rule<R: FinallyRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::finally_blocks,
        |finally_block, _| R::check(finally_block, configuration),
    )
}

impl Ranged for ConditionFact {
    fn source_range(&self) -> SourceRange {
        self.range()
    }
}

pub fn evaluate_condition_rule<R: ConditionRule>(
    facts: &[SourceFacts],
    configuration: &R::Configuration,
) -> Vec<Finding> {
    collect_ranged(
        facts,
        Reporting::of::<R>(configuration),
        SourceFacts::conditions,
        |condition, _| R::check(condition, configuration),
    )
}

pub(crate) fn when_configured<C>(
    configuration: Option<&C>,
    evaluate: impl FnOnce(&C) -> Vec<Finding>,
) -> Vec<Finding> {
    configuration.map_or_else(Vec::new, evaluate)
}
