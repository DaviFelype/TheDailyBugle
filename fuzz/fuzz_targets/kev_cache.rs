//! S12 of docs/test-strategy/002: any cache file decodes or is ignored, never
//! a panic, and what decodes can be written back and read again unchanged.

#![no_main]

use libfuzzer_sys::fuzz_target;
use thedailybugle::cache::files::{decode_kev, decode_poc, encode_kev, encode_poc};

fuzz_target!(|bytes: &[u8]| {
    if let Some(kev) = decode_kev(bytes) {
        let encoded = encode_kev(kev.fetched_at, &kev.entries).ok();
        assert_eq!(encoded.as_deref().and_then(decode_kev), Some(kev));
    }
    if let Some(poc) = decode_poc(bytes) {
        let encoded = encode_poc(&poc).ok();
        assert_eq!(encoded.as_deref().and_then(decode_poc), Some(poc));
    }
});
