use std::{path::Path, process::Command};

pub fn resolve(root: &Path) -> Option<String> {
    if let Some(branch) = std::env::var("GITHUB_HEAD_REF")
        .ok()
        .filter(|branch| !branch.is_empty())
    {
        return Some(branch.to_owned());
    }

    let branch = git(root, ["branch", "--show-current"])?;
    let branch = branch.trim().to_owned();

    if branch.is_empty() || default_branch(root).is_some_and(|default| default == branch) {
        None
    } else {
        Some(branch)
    }
}

fn default_branch(root: &Path) -> Option<String> {
    git(
        root,
        [
            "symbolic-ref",
            "--quiet",
            "--short",
            "refs/remotes/origin/HEAD",
        ],
    )
    .and_then(|branch| branch.trim().strip_prefix("origin/").map(str::to_owned))
}

fn git<const N: usize>(root: &Path, arguments: [&str; N]) -> Option<String> {
    let output = Command::new("git")
        .args(["-C"])
        .arg(root)
        .args(arguments)
        .output()
        .ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8(output.stdout).ok())
        .flatten()
}
