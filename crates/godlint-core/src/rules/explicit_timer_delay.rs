use crate::{
    analyzers::SourceFacts,
    config::{Config, ExplicitTimerDelayRule, Severity},
    facts::CallFact,
    rules::{
        Absence, CallRule, Finding, Languages, Rule, Violation, evaluate_call_rule, when_configured,
    },
    source::{Dialect, Language},
};

const TIMERS: [&str; 2] = ["setTimeout", "setInterval"];
const GO_TIMERS: [&str; 4] = [
    "time.After",
    "time.AfterFunc",
    "time.NewTimer",
    "time.NewTicker",
];

const GLOBALS: [&str; 3] = ["globalThis", "self", "window"];

pub struct ExplicitTimerDelay;

impl Rule for ExplicitTimerDelay {
    const ID: &'static str = "reliability/explicit-timer-delay";

    const LANGUAGES: Languages = Languages::all_but(&[
        (Dialect::Python, Absence::NoSuchConstruct),
        (Dialect::Rust, Absence::NoSuchConstruct),
        (Dialect::Workflow, Absence::NoSuchConstruct),
    ]);

    type Configuration = ExplicitTimerDelayRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl CallRule for ExplicitTimerDelay {
    fn check(call: &CallFact, _configuration: &Self::Configuration) -> Option<Violation> {
        is_timer_without_delay(call).then(|| Violation::TimerWithoutDelay {
            callee: call.callee().to_owned(),
        })
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.explicit_timer_delay.as_ref(), |rule| {
        evaluate_call_rule::<ExplicitTimerDelay>(facts, rule)
    })
}

fn is_timer_without_delay(call: &CallFact) -> bool {
    matches!(
        call.source().language(),
        Language::JavaScript | Language::TypeScript
    ) && TIMERS.contains(&timer_name(call.callee()))
        && call.argument_count() < 2
        || (call.source().language() == Language::Go
            && GO_TIMERS.contains(&call.callee())
            && call.argument_count()
                < if call.callee() == "time.AfterFunc" {
                    2
                } else {
                    1
                })
}

fn timer_name(callee: &str) -> &str {
    match callee.split_once('.') {
        Some((receiver, name)) if GLOBALS.contains(&receiver) => name,
        Some(_) => "",
        None => callee,
    }
}
