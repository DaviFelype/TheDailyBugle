use std::io::Write;
use std::process::ExitCode;

use thedailybugle::cache::{CacheDir, default_cache_path};
use thedailybugle::cli::{self, Command};
use thedailybugle::collect::{self, Clock, SourceSettings};
use thedailybugle::contract::{Feed, PlainText};
use thedailybugle::net::{ClientSettings, HttpClient};
use thedailybugle::sample;
use thedailybugle::sources::{kev::KEV_SOURCE, poc::POC_SOURCE};

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

/// Prints the JSON contract. A source that cannot be reached is listed in
/// `failedSources`; the feed is printed anyway.
fn print_feed() -> ExitCode {
    match build_feed().to_json() {
        Ok(json) => print_line(&json),
        Err(error) => {
            eprintln!("error: could not encode the feed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn build_feed() -> Feed {
    let cache = open_cache();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build();
    let client = HttpClient::new(&ClientSettings::production());
    let settings = SourceSettings::production();
    match (runtime, client, settings) {
        (Ok(runtime), Ok(client), Ok(settings)) => runtime.block_on(collect::build_feed(
            &client,
            cache.as_ref(),
            &settings,
            Clock::now(),
        )),
        _ => {
            eprintln!("error: could not set up the network client");
            let failed = [KEV_SOURCE, POC_SOURCE]
                .iter()
                .filter_map(|name| PlainText::source(name).ok())
                .collect();
            Feed::new(sample::items(), failed)
        }
    }
}

/// Without a cache the feed still works, it just fetches every time.
fn open_cache() -> Option<CacheDir> {
    let xdg_cache_home = std::env::var_os("XDG_CACHE_HOME");
    let home = std::env::var_os("HOME");
    let path = default_cache_path(xdg_cache_home.as_deref(), home.as_deref())?;
    CacheDir::open(&path)
        .inspect_err(|error| eprintln!("warning: cache disabled: {error}"))
        .ok()
}

/// Writes to stdout without `println!`, which panics when the reader has gone away.
fn print_line(line: &str) -> ExitCode {
    match writeln!(std::io::stdout().lock(), "{line}") {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::FAILURE,
    }
}
