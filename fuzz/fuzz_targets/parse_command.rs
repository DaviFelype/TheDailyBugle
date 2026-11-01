//! T08: any argument list parses to `Ok` or `Err`, never a panic.

#![no_main]

use libfuzzer_sys::fuzz_target;
use thedailybugle::cli::parse_command;

fuzz_target!(|args: Vec<String>| {
    let _ = parse_command(&args);
});
