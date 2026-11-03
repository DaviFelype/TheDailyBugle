use std::io::Write;
use std::process::ExitCode;

use thedailybugle::cli::{self, Command};
use thedailybugle::sample;

/// Exit code for wrong arguments, as in most Unix tools.
const USAGE_EXIT_CODE: u8 = 2;

fn main() -> ExitCode {
    // Lossy conversion: a non-UTF-8 argument must become a usage error, not a panic.
    let args: Vec<String> = std::env::args_os()
        .skip(1)
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();

    match cli::parse_command(&args) {
        Ok(Command::Version) => print_line(&cli::version_line()),
        Ok(Command::Feed) => print_feed(),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(USAGE_EXIT_CODE)
        }
    }
}

/// Prints the JSON contract. Until the real sources exist, the feed is the
/// built-in sample.
fn print_feed() -> ExitCode {
    match sample::feed().to_json() {
        Ok(json) => print_line(&json),
        Err(error) => {
            eprintln!("error: could not encode the feed: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Writes to stdout without `println!`, which panics when the reader has gone away.
fn print_line(line: &str) -> ExitCode {
    match writeln!(std::io::stdout().lock(), "{line}") {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
