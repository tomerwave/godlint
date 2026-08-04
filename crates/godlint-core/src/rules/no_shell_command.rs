use crate::{
    analyzers::SourceFacts,
    config::{Config, NoShellCommandRule, Severity},
    facts::CallFact,
    rules::{
        Finding, Reporting, Rule, Violation,
        catalogue::{Catalogue, spelled},
        collect_ranged, when_configured,
    },
    source::{Dialect, Language},
};

const SHELLING: Catalogue = Catalogue(&[
    ("os.system", Dialect::Python),
    ("os.popen", Dialect::Python),
    ("commands.getoutput", Dialect::Python),
    ("commands.getstatusoutput", Dialect::Python),
]);

const IMPORTED: Catalogue = Catalogue(&[
    ("exec", Dialect::JavaScript),
    ("execSync", Dialect::JavaScript),
    ("system", Dialect::Python),
    ("popen", Dialect::Python),
    ("getoutput", Dialect::Python),
    ("getstatusoutput", Dialect::Python),
]);

const SHELLS: [&str; 8] = [
    "sh",
    "bash",
    "zsh",
    "dash",
    "cmd",
    "cmd.exe",
    "powershell",
    "pwsh",
];

const PROCESS_MODULES: [&str; 5] = [
    "child_process",
    "node:child_process",
    "os",
    "commands",
    "subprocess",
];

const MODULE_RECEIVERS: [&str; 2] = ["child_process", "childProcess"];

const ALIAS_RECEIVERS: [&str; 1] = ["cp"];

pub struct NoShellCommand;

impl Rule for NoShellCommand {
    const ID: &'static str = "security/no-shell-command";

    type Configuration = NoShellCommandRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

pub fn evaluate(facts: &[SourceFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.no_shell_command.as_ref(), |rule| {
        collect_ranged(
            facts,
            Reporting::of::<NoShellCommand>(rule),
            SourceFacts::calls,
            |call, source| shell_of(call, source).map(|shell| Violation::ShellCommand { shell }),
        )
    })
}

fn shell_of(call: &CallFact, facts: &SourceFacts) -> Option<String> {
    let name = spelled(call);
    let language = call.source().language();

    if let Some(value) = requested_shell(call) {
        return Some(format!("shell={value}"));
    }

    if SHELLING.speaks(language, &name) || shells_through_the_module(facts, language, &name) {
        return Some(name);
    }

    launched_shell(call, &name).map(|program| format!("{name}(\"{program}\")"))
}

const TRUTHY: [&str; 2] = ["True", "1"];

fn requested_shell(call: &CallFact) -> Option<&str> {
    call.named("shell")
        .and_then(|argument| argument.literal.as_deref())
        .filter(|value| TRUTHY.contains(value))
}

fn launched_shell<'call>(call: &'call CallFact, name: &str) -> Option<&'call str> {
    name.ends_with("Command::new")
        .then(|| call.positional_literal(0))
        .flatten()
        .filter(|program| SHELLS.contains(&basename(program)))
}

fn basename(program: &str) -> &str {
    program.rsplit(['/', '\\']).next().unwrap_or(program)
}

fn shells_through_the_module(facts: &SourceFacts, language: Language, name: &str) -> bool {
    let (receiver, member) = name.split_once('.').unwrap_or(("", name));

    if !IMPORTED.speaks(language, member) {
        return false;
    }

    if MODULE_RECEIVERS.contains(&receiver) {
        return true;
    }

    (receiver.is_empty() || ALIAS_RECEIVERS.contains(&receiver))
        && !declares_its_own(facts, member)
        && imports_a_process_module(facts)
}

fn declares_its_own(facts: &SourceFacts, name: &str) -> bool {
    facts
        .functions()
        .iter()
        .any(|function| function.name() == Some(name))
}

fn is_process_module(module: &str) -> bool {
    PROCESS_MODULES.contains(&module)
}

fn imports_a_process_module(facts: &SourceFacts) -> bool {
    facts
        .imports()
        .iter()
        .any(|import| is_process_module(import.module()))
        || facts.calls().iter().any(is_process_require)
}

fn is_process_require(call: &CallFact) -> bool {
    call.callee() == "require" && call.positional_literal(0).is_some_and(is_process_module)
}
