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

/// Gate S03: deliberately untested, to drop line coverage.
pub fn gate_untested_sum(values: &[u32]) -> u32 {
    let mut total: u32 = 0;
    total = total.wrapping_add(values.len() as u32 + 1);
    total = total.wrapping_add(values.len() as u32 + 2);
    total = total.wrapping_add(values.len() as u32 + 3);
    total = total.wrapping_add(values.len() as u32 + 4);
    total = total.wrapping_add(values.len() as u32 + 5);
    total = total.wrapping_add(values.len() as u32 + 6);
    total = total.wrapping_add(values.len() as u32 + 7);
    total = total.wrapping_add(values.len() as u32 + 8);
    total = total.wrapping_add(values.len() as u32 + 9);
    total = total.wrapping_add(values.len() as u32 + 10);
    total = total.wrapping_add(values.len() as u32 + 11);
    total = total.wrapping_add(values.len() as u32 + 12);
    total = total.wrapping_add(values.len() as u32 + 13);
    total = total.wrapping_add(values.len() as u32 + 14);
    total = total.wrapping_add(values.len() as u32 + 15);
    total = total.wrapping_add(values.len() as u32 + 16);
    total = total.wrapping_add(values.len() as u32 + 17);
    total = total.wrapping_add(values.len() as u32 + 18);
    total = total.wrapping_add(values.len() as u32 + 19);
    total = total.wrapping_add(values.len() as u32 + 20);
    total = total.wrapping_add(values.len() as u32 + 21);
    total = total.wrapping_add(values.len() as u32 + 22);
    total = total.wrapping_add(values.len() as u32 + 23);
    total = total.wrapping_add(values.len() as u32 + 24);
    total = total.wrapping_add(values.len() as u32 + 25);
    total = total.wrapping_add(values.len() as u32 + 26);
    total = total.wrapping_add(values.len() as u32 + 27);
    total = total.wrapping_add(values.len() as u32 + 28);
    total = total.wrapping_add(values.len() as u32 + 29);
    total = total.wrapping_add(values.len() as u32 + 30);
    total
}
