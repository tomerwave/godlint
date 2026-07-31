fn deploy(branch: &str) {
    Command::new("sh").arg("-c").arg(branch).status();
}
