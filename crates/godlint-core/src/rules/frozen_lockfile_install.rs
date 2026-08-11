use crate::{
    analyzers::workflow::WorkflowFacts,
    config::{Config, NoWorkflowCommentsRule, Severity},
    rules::{
        Finding, Languages, Rule, Violation, WorkflowRule, evaluate_workflow_rule, when_configured,
    },
    source::SourceRange,
};

pub struct FrozenLockfileInstall;

impl Rule for FrozenLockfileInstall {
    const ID: &'static str = "ci/frozen-lockfile-install";
    const LANGUAGES: Languages = Languages::WORKFLOWS;
    type Configuration = NoWorkflowCommentsRule;

    fn severity(configuration: &Self::Configuration) -> Severity {
        configuration.severity
    }
}

impl WorkflowRule for FrozenLockfileInstall {
    fn check(
        workflow: &WorkflowFacts,
        _configuration: &Self::Configuration,
    ) -> Vec<(SourceRange, Violation)> {
        workflow
            .steps()
            .iter()
            .filter_map(|step| step.run_range().zip(step.run()))
            .flat_map(|(range, script)| {
                script.lines().filter_map(move |line| {
                    let command = line.trim();
                    let (name, remedy) = missing_lockfile_flag(command)?;
                    Some((
                        range,
                        Violation::FrozenLockfileInstall {
                            command: name.to_owned(),
                            remedy: remedy.to_owned(),
                        },
                    ))
                })
            })
            .collect()
    }
}

fn missing_lockfile_flag(command: &str) -> Option<(&'static str, &'static str)> {
    let command = command.trim_start_matches(['-', ' ']);
    if command.contains("--global") {
        return None;
    }
    const PATTERNS: [(&str, &str, &str); 8] = [
        ("npm install", "", "npm ci"),
        (
            "yarn install",
            "--frozen-lockfile",
            "yarn install --frozen-lockfile",
        ),
        (
            "pnpm install",
            "--frozen-lockfile",
            "pnpm install --frozen-lockfile",
        ),
        (
            "pip install -r",
            "--require-hashes",
            "pip install -r ... --require-hashes",
        ),
        (
            "poetry install",
            "--no-update",
            "poetry install --no-update",
        ),
        ("bundle install", "--frozen", "bundle install --frozen"),
        ("cargo build", "--locked", "add --locked"),
        ("cargo test", "--locked", "add --locked"),
    ];
    PATTERNS
        .iter()
        .find(|(prefix, flag, _)| {
            command.starts_with(prefix)
                && ((*prefix == "npm install" && !command.starts_with("npm ci"))
                    || (!flag.is_empty() && !command.contains(flag)))
        })
        .map(|(prefix, _, remedy)| (*prefix, *remedy))
        .or_else(|| {
            (command.starts_with("uv sync") && !command.contains("--locked"))
                .then_some(("uv sync", "uv sync --locked"))
        })
}

pub fn evaluate(workflows: &[WorkflowFacts], config: &Config) -> Vec<Finding> {
    when_configured(config.rules.frozen_lockfile_install.as_ref(), |rule| {
        evaluate_workflow_rule::<FrozenLockfileInstall>(workflows, rule)
    })
}

#[cfg(test)]
mod tests {
    use super::missing_lockfile_flag;

    #[test]
    fn recognizes_supported_unfrozen_commands() {
        for command in [
            "npm install",
            "yarn install",
            "pnpm install",
            "pip install -r requirements.txt",
            "poetry install",
            "bundle install",
            "cargo build --release",
            "cargo test",
            "uv sync",
        ] {
            assert!(missing_lockfile_flag(command).is_some(), "{command}");
        }
    }

    #[test]
    fn accepts_frozen_commands() {
        for command in [
            "npm ci",
            "yarn install --frozen-lockfile",
            "pnpm install --frozen-lockfile",
            "pip install -r requirements.txt --require-hashes",
            "poetry install --no-update",
            "bundle install --frozen",
            "cargo build --locked",
            "cargo test --locked",
            "uv sync --locked",
            "npm install --global npm@latest",
        ] {
            assert!(missing_lockfile_flag(command).is_none(), "{command}");
        }
    }
}
