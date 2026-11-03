//! Command-line parsing. Pure functions only, so they can be tested without
//! running the binary.

use std::fmt;

use clap::{Arg, ArgAction};

const PROGRAM_NAME: &str = "thedailybugle";

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

/// clap's own help and version flags print to stdout, which is reserved for
/// the JSON contract, so they are off and errors map to `UsageError`.
fn command_line() -> clap::Command {
    clap::Command::new(PROGRAM_NAME)
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .args_conflicts_with_subcommands(true)
        .arg(
            Arg::new("version")
                .long("version")
                .action(ArgAction::SetTrue),
        )
        .subcommand(
            clap::Command::new("feed").disable_help_flag(true).arg(
                Arg::new("json")
                    .long("json")
                    .action(ArgAction::SetTrue)
                    .required(true),
            ),
        )
}

/// Parses the arguments that follow the program name.
pub fn parse_command(args: &[String]) -> Result<Command, UsageError> {
    let all_args = std::iter::once(PROGRAM_NAME).chain(args.iter().map(String::as_str));
    let matches = command_line()
        .try_get_matches_from(all_args)
        .map_err(|_| UsageError)?;
    // `try_get_one` instead of `get_flag`, which panics on a programming error.
    let version = matches!(matches.try_get_one::<bool>("version"), Ok(Some(true)));

    match (matches.subcommand_name(), version) {
        (Some("feed"), _) => Ok(Command::Feed),
        (None, true) => Ok(Command::Version),
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
