//! S12 of docs/test-strategy/002: any bytes parse to a catalog or an error,
//! never a panic, and every entry kept is valid.

#![no_main]

use libfuzzer_sys::fuzz_target;
use thedailybugle::contract::TITLE_MAX_CHARS;
use thedailybugle::sources::kev::parse_catalog;

fuzz_target!(|body: &[u8]| {
    if let Ok(catalog) = parse_catalog(body) {
        for entry in &catalog.entries {
            assert!(entry.cve.as_str().starts_with("CVE-"));
            assert!(!entry.name.as_str().is_empty());
            assert!(entry.name.as_str().chars().count() <= TITLE_MAX_CHARS);
        }
    }
});
