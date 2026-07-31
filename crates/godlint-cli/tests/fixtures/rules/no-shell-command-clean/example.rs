fn deploy(branch: &str) {
    Command::new("git").arg("checkout").arg(branch).status();
}
