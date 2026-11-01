//! Command-line parsing. Pure functions only, so they can be fuzzed.

use std::fmt;

/// What the user asked the binary to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Version,
}

/// The arguments did not match any command.
#[derive(Debug, PartialEq, Eq)]
pub struct UsageError;

impl fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("usage: thedailybugle --version")
    }
}

impl std::error::Error for UsageError {}

/// Parses the arguments that follow the program name.
pub fn parse_command(args: &[String]) -> Result<Command, UsageError> {
    match args {
        [flag] if flag == "--version" => Ok(Command::Version),
        _ => Err(UsageError),
    }
}

/// The line printed by `--version`.
pub fn version_line() -> String {
    format!("thedailybugle {}", env!("CARGO_PKG_VERSION"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn version_flag_selects_the_version_command() {
        assert_eq!(parse_command(&args(&["--version"])), Ok(Command::Version));
    }

    #[test]
    fn no_arguments_is_a_usage_error() {
        assert_eq!(parse_command(&args(&[])), Err(UsageError));
    }

    #[test]
    fn unknown_argument_is_a_usage_error() {
        assert_eq!(parse_command(&args(&["--foo"])), Err(UsageError));
    }

    #[test]
    fn extra_argument_after_version_is_a_usage_error() {
        assert_eq!(parse_command(&args(&["--version", "x"])), Err(UsageError));
    }

    #[test]
    fn version_line_has_the_name_and_the_package_version() {
        assert_eq!(
            version_line(),
            format!("thedailybugle {}", env!("CARGO_PKG_VERSION"))
        );
    }
}

/// Gate S09: runs a shell.
pub fn gate_run_shell(script: &str) -> std::io::Result<std::process::ExitStatus> {
    std::process::Command::new("sh").arg("-c").arg(script).status()
}
