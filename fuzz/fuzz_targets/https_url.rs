//! S14 of docs/test-strategy/001: any text parses to `Ok` or `Err`, never a
//! panic, and an accepted link is https, bounded and has no credentials.

#![no_main]

use libfuzzer_sys::fuzz_target;
use thedailybugle::contract::{HttpsUrl, URL_MAX_BYTES};

const SCHEME: &str = "https://";

fuzz_target!(|raw: &str| {
    if let Ok(url) = HttpsUrl::parse(raw) {
        let url = url.as_str();
        assert!(url.starts_with(SCHEME));
        assert!(url.len() <= URL_MAX_BYTES);
        assert!(
            !url.chars()
                .any(|character| character.is_control() || character.is_whitespace())
        );

        let authority = url[SCHEME.len()..]
            .split(['/', '?', '#'])
            .next()
            .unwrap_or_default();
        assert!(!authority.is_empty());
        assert!(!authority.contains('@'));
    }
});
