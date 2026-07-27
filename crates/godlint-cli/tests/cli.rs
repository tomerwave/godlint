use std::process::Command;

fn godlint() -> Command {
    Command::new(env!("CARGO_BIN_EXE_godlint"))
}

fn run(command: &mut Command) -> std::process::Output {
    match command.output() {
        Ok(output) => output,
        Err(error) => panic!("runs godlint: {error}"),
    }
}

#[test]
fn prints_its_version() {
    let output = run(godlint().arg("--version"));

    assert!(output.status.success());
    assert_eq!(String::from_utf8_lossy(&output.stdout), "godlint 0.1.0\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn rejects_unknown_arguments() {
    let output = run(godlint().arg("check"));

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Unknown argument: check"));
}
