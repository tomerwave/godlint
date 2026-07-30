use godlint_core::rules::{Violation, no_shell_command};

use super::support::rule_violations;

const ENABLED: &str = "version: 1\nrules:\n  security/no-shell-command:\n    severity: error\n";

fn violations(path: &str, source: &str, configuration: &str) -> Vec<Violation> {
    rule_violations(no_shell_command::evaluate, path, source, configuration)
}

fn reported(path: &str, source: &str) -> Vec<Violation> {
    violations(path, source, ENABLED)
}

fn message(path: &str, source: &str) -> String {
    reported(path, source)
        .first()
        .unwrap_or_else(|| panic!("reports {path}: {source}"))
        .to_string()
}

#[test]
fn reports_a_python_shell_keyword_on_every_launcher() {
    for launcher in [
        "subprocess.run",
        "subprocess.call",
        "subprocess.check_call",
        "subprocess.check_output",
        "subprocess.Popen",
    ] {
        let source = format!("def deploy(b):\n    {launcher}(f'git checkout {{b}}', shell=True)\n");

        assert_eq!(
            reported("src/deploy.py", &source).len(),
            1,
            "the finding is shell=True being written, not the callee: {launcher}"
        );
    }
}

#[test]
fn names_the_keyword_rather_than_the_callee() {
    assert!(
        message("src/deploy.py", "subprocess.run(command, shell=True)").starts_with("shell=True "),
        "the argument is what made the string executable, so the message names it"
    );
}

#[test]
fn keeps_a_python_launcher_given_an_argument_array() {
    assert!(reported("src/deploy.py", "subprocess.run(['git', 'checkout', b])").is_empty());
    assert!(
        reported("src/deploy.py", "subprocess.run(command, shell=False)").is_empty(),
        "the keyword is read, not merely looked for"
    );
    assert!(reported("src/deploy.py", "subprocess.run(command, check=True)").is_empty());
}

#[test]
fn reports_a_python_call_that_is_a_shell_by_itself() {
    for callee in [
        "os.system",
        "os.popen",
        "commands.getoutput",
        "commands.getstatusoutput",
    ] {
        let source = format!("def deploy(b):\n    {callee}(f'git push {{b}}')\n");

        assert_eq!(reported("src/deploy.py", &source).len(), 1, "{callee}");
    }
}

#[test]
fn reports_a_javascript_shell_launcher() {
    assert_eq!(
        reported("src/deploy.js", "child_process.exec(`git checkout ${b}`);").len(),
        1
    );
    assert_eq!(
        reported(
            "src/deploy.ts",
            "childProcess.execSync(`git checkout ${b}`);"
        )
        .len(),
        1
    );
}

#[test]
fn reports_a_destructured_launcher_only_where_the_module_is_imported() {
    let imported = "import { exec } from \"node:child_process\";\nexec(`git checkout ${b}`);\n";
    let required = "const { execSync } = require(\"child_process\");\nexecSync(`git ${b}`);\n";

    assert_eq!(reported("src/deploy.js", imported).len(), 1);
    assert_eq!(reported("src/deploy.js", required).len(), 1);
}

#[test]
fn reads_both_halves_of_a_require() {
    assert!(
        reported(
            "src/deploy.js",
            "const fs = require(\"fs\");\nexec(`git ${b}`);\n"
        )
        .is_empty(),
        "requiring another module says nothing about a shell"
    );
    assert!(
        reported(
            "src/deploy.js",
            "load(\"child_process\");\nexec(`git ${b}`);\n"
        )
        .is_empty(),
        "naming the module in some other call is not importing it"
    );
}

#[test]
fn keeps_a_regular_expression_exec() {
    assert!(
        reported("src/parse.js", "const m = pattern.exec(reference);").is_empty(),
        "a member exec is a regular expression, and a bare one is only a shell where the module is \
         imported"
    );
    assert!(
        reported("src/parse.js", "const m = exec(reference);").is_empty(),
        "without the import this name says nothing about a shell"
    );
}

#[test]
fn keeps_a_javascript_launcher_that_takes_an_argument_array() {
    assert!(
        reported(
            "src/deploy.js",
            "import { execFile } from \"node:child_process\";\nexecFile('git', ['checkout', b]);\n"
        )
        .is_empty()
    );
    assert!(
        reported(
            "src/deploy.js",
            "import { spawn } from \"node:child_process\";\nspawn('git', ['checkout', b]);\n"
        )
        .is_empty()
    );
}

#[test]
fn reports_a_rust_command_that_launches_a_shell() {
    for shell in ["sh", "bash", "zsh", "dash", "cmd", "powershell", "pwsh"] {
        let source = format!("fn deploy(b: &str) {{\n    Command::new(\"{shell}\").arg(b);\n}}\n");

        assert_eq!(reported("src/deploy.rs", &source).len(), 1, "{shell}");
    }
}

#[test]
fn names_the_shell_it_found() {
    assert!(
        message(
            "src/deploy.rs",
            "fn a() {\n    Command::new(\"bash\").arg(b);\n}\n"
        )
        .starts_with("Command::new(\"bash\") "),
        "naming the program is what makes the finding readable at a glance"
    );
}

#[test]
fn keeps_a_rust_command_that_runs_a_program_directly() {
    assert!(
        reported(
            "src/deploy.rs",
            "fn a() {\n    Command::new(\"git\").arg(\"checkout\").arg(b);\n}\n"
        )
        .is_empty()
    );
    assert!(
        reported(
            "src/deploy.rs",
            "fn a() {\n    Command::new(program).arg(b);\n}\n"
        )
        .is_empty(),
        "a program Godlint cannot read is not reported as a shell"
    );
}

#[test]
fn accepts_the_fully_qualified_rust_spelling() {
    assert_eq!(
        reported(
            "src/deploy.rs",
            "fn a() {\n    std::process::Command::new(\"sh\").arg(b);\n}\n"
        )
        .len(),
        1
    );
}

#[test]
fn binds_a_launcher_to_the_language_that_spells_it() {
    assert!(
        reported("src/deploy.py", "child_process.exec('git')").is_empty(),
        "Python has no child_process module"
    );
    assert!(reported("src/deploy.js", "os.system('git')").is_empty());
}

#[test]
fn permits_a_shell_inside_an_approved_path() {
    let configuration = concat!(
        "version: 1\n",
        "rules:\n",
        "  security/no-shell-command:\n",
        "    severity: error\n",
        "    allow-in:\n",
        "      - scripts/**\n"
    );

    assert!(violations("scripts/release.py", "os.system('git push')", configuration).is_empty());
    assert_eq!(
        violations("src/deploy.py", "os.system('git push')", configuration).len(),
        1
    );
}

#[test]
fn can_disable_the_rule() {
    let configuration = "version: 1\nrules:\n  security/no-shell-command:\n    severity: off\n";

    assert!(
        violations(
            "src/deploy.py",
            "subprocess.run(command, shell=True)",
            configuration
        )
        .is_empty()
    );
}
