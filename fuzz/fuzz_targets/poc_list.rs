//! S12 of docs/test-strategy/002: any bytes count to a number within the cap
//! or an error, never a panic.

#![no_main]

use libfuzzer_sys::fuzz_target;
use thedailybugle::sources::poc::{MAX_POC_COUNT, count_pocs};

fuzz_target!(|body: &[u8]| {
    if let Ok(count) = count_pocs(body) {
        assert!(count <= MAX_POC_COUNT);
    }
});
