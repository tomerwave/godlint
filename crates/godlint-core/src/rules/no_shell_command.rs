use crate::{
    analyzers::SourceFacts,
    config::{Config, NoShellCommandRule, Severity},
    facts::CallFact,
    rules::{
        Finding, Reporting, Rule, Violation,
        catalogue::{Catalogue, Dialect, matches, spelled},
        collect_ranged, when_configured,
    },
};

const SHELLING: Catalogue = Catalogue(&[
    ("os.system", Dialect::Python),
    ("os.popen", Dialect::Python),
    ("commands.getoutput", Dialect::Python),
    ("commands.getstatusoutput", Dialect::Python),
    ("child_process.exec", Dialect::JavaScript),
    ("child_process.execSync", Dialect::JavaScript),
    ("childProcess.exec", Dialect::JavaScript),
    ("childProcess.execSync", Dialect::JavaScript),
]);

const IMPORTED: Catalogue = Catalogue(&[
    ("exec", Dialect::JavaScript),
    ("execSync", Dialect::JavaScript),
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

const PROCESS_MODULES: [&str; 2] = ["child_process", "node:child_process"];

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
            |call, source| check(call, source, rule),
        )
    })
}

fn check(
    call: &CallFact,
    facts: &SourceFacts,
    configuration: &NoShellCommandRule,
) -> Option<Violation> {
    if matches(call.source(), &configuration.allow_in) {
        return None;
    }

    shell_of(call, facts).map(|shell| Violation::ShellCommand { shell })
}

fn shell_of(call: &CallFact, facts: &SourceFacts) -> Option<String> {
    let name = spelled(call);
    let language = call.source().language();

    if asks_for_a_shell(call) {
        return Some("shell=True".to_owned());
    }

    if SHELLING.speaks(language, &name)
        || (IMPORTED.speaks(language, &name) && imports_a_process_module(facts))
    {
        return Some(name);
    }

    launched_shell(call, &name).map(|program| format!("{name}(\"{program}\")"))
}

fn asks_for_a_shell(call: &CallFact) -> bool {
    call.named("shell")
        .and_then(|argument| argument.literal.as_deref())
        == Some("True")
}

fn launched_shell<'call>(call: &'call CallFact, name: &str) -> Option<&'call str> {
    name.ends_with("Command::new")
        .then(|| call.positional_literal(0))
        .flatten()
        .filter(|program| SHELLS.contains(program))
}

fn imports_a_process_module(facts: &SourceFacts) -> bool {
    facts
        .imports()
        .iter()
        .any(|import| PROCESS_MODULES.contains(&import.module()))
        || facts.calls().iter().any(is_process_require)
}

fn is_process_require(call: &CallFact) -> bool {
    call.callee() == "require"
        && call
            .positional_literal(0)
            .is_some_and(|module| PROCESS_MODULES.contains(&module))
}
