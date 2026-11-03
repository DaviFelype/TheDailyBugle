//! Command-line parsing. Pure functions only, so they can be tested without
//! running the binary.

use std::fmt;

/// What the user asked the binary to do.
#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Version,
    Feed,
}

/// The arguments did not match any command.
#[derive(Debug, PartialEq, Eq)]
pub struct UsageError;

impl fmt::Display for UsageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("usage: thedailybugle --version | thedailybugle feed --json")
    }
}

impl std::error::Error for UsageError {}

/// Parses the arguments that follow the program name.
pub fn parse_command(_args: &[String]) -> Result<Command, UsageError> {
    todo!()
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
    fn feed_json_selects_the_feed_command() {
        assert_eq!(parse_command(&args(&["feed", "--json"])), Ok(Command::Feed));
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
    fn feed_without_json_is_a_usage_error() {
        assert_eq!(parse_command(&args(&["feed"])), Err(UsageError));
        assert_eq!(parse_command(&args(&["feed", "--xml"])), Err(UsageError));
        assert_eq!(
            parse_command(&args(&["feed", "--json", "x"])),
            Err(UsageError)
        );
    }

    #[test]
    fn help_and_version_together_with_feed_are_usage_errors() {
        assert_eq!(parse_command(&args(&["--help"])), Err(UsageError));
        assert_eq!(parse_command(&args(&["help"])), Err(UsageError));
        assert_eq!(
            parse_command(&args(&["--version", "feed", "--json"])),
            Err(UsageError)
        );
    }

    #[test]
    fn version_line_has_the_name_and_the_package_version() {
        assert_eq!(
            version_line(),
            format!("thedailybugle {}", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn usage_names_both_commands() {
        assert_eq!(
            UsageError.to_string(),
            "usage: thedailybugle --version | thedailybugle feed --json"
        );
    }
}
