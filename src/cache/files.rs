//! What the cache files hold. Only validated, reduced data is written, and
//! everything read back is validated again: the files are as untrusted as the
//! network.

// Stubs until the implementation commit.
#![allow(dead_code, unused_imports)]

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::sources::cve_id::CveId;
use crate::sources::kev::{KevEntry, entry_from_fields};
use crate::sources::poc::MAX_POC_COUNT;

pub const KEV_FILE: &str = "kev.json";
pub const POC_FILE: &str = "poc.json";
/// Both files stay far below this; anything larger is not ours.
pub const MAX_CACHE_FILE_BYTES: usize = 4 * 1024 * 1024;

const FORMAT_VERSION: u32 = 1;
const MAX_POC_ENTRIES: usize = 1000;

pub const HOUR: u64 = 60 * 60;
pub const KEV_TTL_SECONDS: u64 = 6 * HOUR;
pub const POC_TTL_SECONDS: u64 = 6 * HOUR;
/// How old data may be and still be shown when a fetch fails.
pub const MAX_STALE_SECONDS: u64 = 7 * 24 * HOUR;

/// How long ago `fetched_at` was. A time in the future (clock moved back, or
/// a tampered file) has no age, so it is never fresh.
pub fn age_seconds(_fetched_at: u64, _now: u64) -> Option<u64> {
    todo!()
}

pub fn is_fresh(_fetched_at: u64, _now: u64, _ttl_seconds: u64) -> bool {
    todo!()
}

pub fn is_usable_when_stale(_fetched_at: u64, _now: u64) -> bool {
    todo!()
}

#[derive(Debug, PartialEq, Eq)]
pub struct CachedKev {
    pub fetched_at: u64,
    pub entries: Vec<KevEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CachedPoc {
    pub count: u32,
    pub fetched_at: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KevFile {
    version: u32,
    fetched_at: u64,
    entries: Vec<KevFileEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct KevFileEntry {
    cve: String,
    name: String,
    date_added: String,
    known_ransomware: bool,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PocFile {
    version: u32,
    entries: BTreeMap<String, PocFileEntry>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PocFileEntry {
    count: u32,
    fetched_at: u64,
}

pub fn encode_kev(_fetched_at: u64, _entries: &[KevEntry]) -> Result<Vec<u8>, serde_json::Error> {
    todo!()
}

/// `None` for anything that is not a version 1 file; bad entries are dropped.
pub fn decode_kev(_bytes: &[u8]) -> Option<CachedKev> {
    todo!()
}

pub fn encode_poc(_entries: &BTreeMap<CveId, CachedPoc>) -> Result<Vec<u8>, serde_json::Error> {
    todo!()
}

pub fn decode_poc(_bytes: &[u8]) -> Option<BTreeMap<CveId, CachedPoc>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(cve: &str) -> KevEntry {
        entry_from_fields(cve, "Name", "2026-11-01", true).unwrap()
    }

    #[test]
    fn kev_round_trips() {
        let entries = vec![entry("CVE-2026-1234"), entry("CVE-2026-5678")];
        let decoded = decode_kev(&encode_kev(100, &entries).unwrap()).unwrap();

        assert_eq!(
            decoded,
            CachedKev {
                fetched_at: 100,
                entries
            }
        );
    }

    #[test]
    fn tampered_kev_entries_are_dropped() {
        let bytes = br#"{"version":1,"fetchedAt":5,"entries":[
            {"cve":"CVE-2026-1/../x","name":"A","dateAdded":"2026-11-01","knownRansomware":false},
            {"cve":"CVE-2026-1234","name":"\u202e","dateAdded":"2026-11-01","knownRansomware":false},
            {"cve":"CVE-2026-1235","name":"B","dateAdded":"2026-02-30","knownRansomware":false},
            {"cve":"CVE-2026-1236","name":"a\u202eb","dateAdded":"2026-11-01","knownRansomware":false}
        ]}"#;
        let decoded = decode_kev(bytes).unwrap();

        assert_eq!(decoded.entries.len(), 1);
        assert_eq!(decoded.entries[0].name.as_str(), "ab");
    }

    #[test]
    fn garbage_or_other_versions_are_not_a_cache() {
        for bytes in [
            &b"garbage"[..],
            b"{}",
            br#"{"version":2,"fetchedAt":5,"entries":[]}"#,
            br#"{"version":1,"fetchedAt":-5,"entries":[]}"#,
        ] {
            assert_eq!(decode_kev(bytes), None);
            assert_eq!(decode_poc(bytes), None);
        }
    }

    #[test]
    fn poc_round_trips_and_drops_tampered_entries() {
        let mut entries = BTreeMap::new();
        entries.insert(
            CveId::parse("CVE-2026-1234").unwrap(),
            CachedPoc {
                count: 3,
                fetched_at: 9,
            },
        );
        assert_eq!(decode_poc(&encode_poc(&entries).unwrap()).unwrap(), entries);

        let tampered = br#"{"version":1,"entries":{
            "CVE-2026-1234":{"count":3,"fetchedAt":9},
            "../x":{"count":1,"fetchedAt":9},
            "CVE-2026-1235":{"count":10001,"fetchedAt":9}
        }}"#;
        assert_eq!(decode_poc(tampered).unwrap(), entries);
    }

    #[test]
    fn poc_file_with_too_many_entries_is_not_a_cache() {
        let many: Vec<String> = (0..=MAX_POC_ENTRIES)
            .map(|index| {
                format!(
                    r#""CVE-2026-{:05}":{{"count":1,"fetchedAt":1}}"#,
                    index + 10_000
                )
            })
            .collect();
        let bytes = format!(r#"{{"version":1,"entries":{{{}}}}}"#, many.join(","));
        assert_eq!(decode_poc(bytes.as_bytes()), None);
    }

    #[test]
    fn freshness_follows_the_ttl_and_the_future_is_never_fresh() {
        assert!(is_fresh(0, KEV_TTL_SECONDS - 1, KEV_TTL_SECONDS));
        assert!(!is_fresh(0, KEV_TTL_SECONDS, KEV_TTL_SECONDS));
        assert!(!is_fresh(100, 50, KEV_TTL_SECONDS));
        assert!(is_usable_when_stale(0, MAX_STALE_SECONDS));
        assert!(!is_usable_when_stale(0, MAX_STALE_SECONDS + 1));
        assert!(!is_usable_when_stale(100, 50));
    }
}
