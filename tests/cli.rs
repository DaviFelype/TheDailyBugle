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
fn unknown_argument_prints_usage_to_stderr_and_exits_two() {
    let output = run(&[OsStr::new("--foo")]).unwrap();

    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("usage:"));
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn non_utf8_argument_is_a_usage_error_not_a_crash() {
    let output = run(&[OsStr::from_bytes(b"--vers\xffion")]).unwrap();

    assert!(output.stdout.is_empty());
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn gate_s02_always_fails() {
    assert_eq!(1, 2);
}
