//! Helpers for the tests that run the real binary. They keep it off the
//! network: a fresh cache answers everything, and a proxy on a closed port
//! catches any request that slips through.

use std::collections::BTreeMap;
use std::error::Error;
use std::path::Path;
use std::process::Command;

use thedailybugle::cache::CacheDir;
use thedailybugle::cache::files::{CachedPoc, KEV_FILE, POC_FILE, encode_kev, encode_poc};
use thedailybugle::collect::Clock;
use thedailybugle::contract::PublishedDate;
use thedailybugle::sources::kev::entry_from_fields;

pub type TestResult<T = ()> = Result<T, Box<dyn Error>>;

/// Writes a cache fetched just now, with one KEV entry added today.
pub fn write_fresh_cache(cache_home: &Path) -> TestResult {
    let clock = Clock::now();
    let today = PublishedDate::from_date(clock.today)
        .ok_or("today is out of range")?
        .to_string();
    let entry = entry_from_fields(
        "CVE-2026-97229",
        "GitLab Community and Enterprise Editions Improper Access Control Vulnerability",
        &today,
        true,
    )
    .ok_or("invalid entry")?;
    let cache = CacheDir::open(&cache_home.join("thedailybugle"))?;
    cache.write_atomic(
        KEV_FILE,
        &encode_kev(clock.unix_seconds, std::slice::from_ref(&entry))?,
    )?;
    let pocs = BTreeMap::from([(
        entry.cve,
        CachedPoc {
            count: 7,
            fetched_at: clock.unix_seconds,
        },
    )]);
    cache.write_atomic(POC_FILE, &encode_poc(&pocs)?)?;
    Ok(())
}

/// `feed --json` with its cache in `cache_home` and no way out.
pub fn feed_command(cache_home: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_thedailybugle"));
    command
        .args(["feed", "--json"])
        .env("XDG_CACHE_HOME", cache_home)
        .env_remove("NO_PROXY")
        .env_remove("no_proxy");
    for name in [
        "HTTPS_PROXY",
        "https_proxy",
        "HTTP_PROXY",
        "http_proxy",
        "ALL_PROXY",
        "all_proxy",
    ] {
        command.env(name, "http://127.0.0.1:9");
    }
    command
}
