use std::process::Command;

fn examples(url: &str) {
    // ruleid: rust-no-shell
    Command::new("sh");
    // ruleid: rust-no-shell
    std::process::Command::new("/bin/bash");
    // ruleid: rust-no-shell-c-flag
    Command::new("python3").arg("-c");
    // ok: rust-no-shell
    Command::new("xdg-open").arg(url);
}
