//! Runs the real binary, as the bar widget does.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::process::{Command, Output};

fn run(args: &[&OsStr]) -> std::io::Result<Output> {
    Command::new(env!("CARGO_BIN_EXE_thedailybugle"))
        .args(args)
        .output()
}

#[test]
fn version_flag_prints_the_version_line_and_exits_zero() {
    let output = run(&[OsStr::new("--version")]).unwrap();

    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("thedailybugle {}\n", env!("CARGO_PKG_VERSION"))
    );
    assert!(output.stderr.is_empty());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn feed_json_prints_one_json_line_and_exits_zero() {
    let output = run(&[OsStr::new("feed"), OsStr::new("--json")]).unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.ends_with('\n'));
    assert!(serde_json::from_str::<serde_json::Value>(&stdout).is_ok());
    assert!(output.stderr.is_empty());
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn wrong_arguments_print_usage_to_stderr_and_exit_two() {
    for args in [&["--foo"][..], &["feed"], &["feed", "--xml"]] {
        let args: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
        let output = run(&args).unwrap();

        assert!(output.stdout.is_empty(), "args: {args:?}");
        assert!(
            String::from_utf8(output.stderr).unwrap().contains("usage:"),
            "args: {args:?}"
        );
        assert_eq!(output.status.code(), Some(2), "args: {args:?}");
    }
}

#[test]
fn non_utf8_argument_is_a_usage_error_not_a_crash() {
    let output = run(&[OsStr::from_bytes(b"--vers\xffion")]).unwrap();

    assert!(output.stdout.is_empty());
    assert_eq!(output.status.code(), Some(2));
}
