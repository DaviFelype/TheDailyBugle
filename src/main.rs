use std::io::Write;
use std::process::ExitCode;

use thedailybugle::cli::{self, Command};

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
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(USAGE_EXIT_CODE)
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
