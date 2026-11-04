//! CISA's Known Exploited Vulnerabilities catalog (CC0 1.0).

// Stubs until the implementation commit.
#![allow(dead_code, unused_imports)]

use std::collections::HashSet;
use std::fmt;

use serde::Deserialize;
use serde_json::Value;
use time::Date;

use super::cve_id::CveId;
use crate::contract::{PlainText, PublishedDate};

pub const KEV_SOURCE: &str = "CISA KEV";
pub const KEV_URL: &str =
    "https://www.cisa.gov/sites/default/files/feeds/known_exploited_vulnerabilities.json";

/// The real catalog is about 1.8 MiB.
pub const MAX_CATALOG_BYTES: usize = 8 * 1024 * 1024;
const MAX_ENTRIES: usize = 20_000;

/// Entries added this many days ago or less are shown.
pub const WINDOW_DAYS: i64 = 30;
/// The cache keeps a longer stretch, so the window can move without a fetch.
pub const CACHE_DAYS: i64 = 60;
pub const MAX_RECENT: usize = 100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KevEntry {
    pub cve: CveId,
    pub name: PlainText,
    pub date_added: PublishedDate,
    pub known_ransomware: bool,
}

/// The entries that passed every check, and how many did not.
#[derive(Debug, PartialEq, Eq)]
pub struct Catalog {
    pub entries: Vec<KevEntry>,
    pub skipped: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub enum KevError {
    /// Not JSON, or no `vulnerabilities` array.
    Malformed,
    TooManyEntries,
}

impl fmt::Display for KevError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Malformed => "catalog is not the expected JSON",
            Self::TooManyEntries => "catalog has too many entries",
        })
    }
}

impl std::error::Error for KevError {}

#[derive(Deserialize)]
struct RawCatalog {
    vulnerabilities: Vec<Value>,
}

/// Reads the catalog. A bad entry is skipped and counted; only a catalog that
/// is not the expected shape at all is an error.
pub fn parse_catalog(_body: &[u8]) -> Result<Catalog, KevError> {
    todo!()
}

/// Builds an entry from the fields the cache and the feed both use.
pub fn entry_from_fields(
    _cve: &str,
    _name: &str,
    _date_added: &str,
    _known_ransomware: bool,
) -> Option<KevEntry> {
    todo!()
}

fn entry_from(value: &Value) -> Option<KevEntry> {
    let field = |name: &str| value.get(name).and_then(Value::as_str);
    entry_from_fields(
        field("cveID")?,
        field("vulnerabilityName")?,
        field("dateAdded")?,
        field("knownRansomwareCampaignUse") == Some("Known"),
    )
}

/// Entries added between `days` days ago and today, newest first, at most
/// `max`. Dates in the future are dropped.
pub fn added_within(_entries: &[KevEntry], _today: Date, _days: i64, _max: usize) -> Vec<KevEntry> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Month;

    fn day(raw: &str) -> Date {
        PublishedDate::parse(raw).unwrap().date()
    }

    fn entry(cve: &str, date_added: &str) -> KevEntry {
        entry_from_fields(cve, "Some vulnerability", date_added, false).unwrap()
    }

    fn catalog(entries: &str) -> Vec<u8> {
        format!(r#"{{"catalogVersion":"x","vulnerabilities":[{entries}]}}"#).into_bytes()
    }

    const VALID: &str = r#"{"cveID":"CVE-2026-1234","vulnerabilityName":"Name","dateAdded":"2026-11-01","knownRansomwareCampaignUse":"Unknown"}"#;

    #[test]
    fn real_catalog_fixture_parses_every_entry() {
        let body = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/kev/catalog.json"
        ))
        .unwrap();
        let catalog = parse_catalog(&body).unwrap();

        assert_eq!(catalog.skipped, 0);
        assert_eq!(catalog.entries.len(), 7);
        assert_eq!(catalog.entries[0].cve.as_str(), "CVE-2026-119977");
        assert!(
            catalog.entries[0]
                .name
                .as_str()
                .starts_with("WordPress WordPress Core")
        );
        assert_eq!(catalog.entries[0].date_added.to_string(), "2026-11-03");
    }

    #[test]
    fn window_keeps_today_and_the_last_30_days() {
        let today = Date::from_calendar_date(2026, Month::November, 3).unwrap();
        // The id's last digits are the age in days; 9999 is tomorrow.
        let entries: Vec<KevEntry> = [0, 29, 30, 31, -1]
            .iter()
            .map(|days_ago: &i64| {
                let date =
                    PublishedDate::from_date(today - time::Duration::days(*days_ago)).unwrap();
                let number = if *days_ago < 0 { 9999 } else { 1000 + days_ago };
                entry(&format!("CVE-2026-{number}"), &date.to_string())
            })
            .collect();

        let kept: Vec<String> = added_within(&entries, today, WINDOW_DAYS, MAX_RECENT)
            .iter()
            .map(|entry| entry.cve.to_string())
            .collect();
        assert_eq!(kept, ["CVE-2026-1000", "CVE-2026-1029", "CVE-2026-1030"]);
    }

    #[test]
    fn window_is_newest_first_and_capped() {
        let today = day("2026-11-03");
        let entries: Vec<KevEntry> = (0..150)
            .map(|index| {
                let date = if index < 75 {
                    "2026-10-05"
                } else {
                    "2026-11-02"
                };
                entry(&format!("CVE-2026-{:04}", 1000 + index), date)
            })
            .collect();

        let recent = added_within(&entries, today, WINDOW_DAYS, MAX_RECENT);

        assert_eq!(recent.len(), MAX_RECENT);
        assert_eq!(recent[0].cve.as_str(), "CVE-2026-1075");
        assert_eq!(recent[0].date_added.to_string(), "2026-11-02");
        assert_eq!(recent[MAX_RECENT - 1].date_added.to_string(), "2026-10-05");
    }

    #[test]
    fn repeated_cve_keeps_the_first_entry() {
        let second = VALID.replace("Name", "Other name");
        let catalog = parse_catalog(&catalog(&format!("{VALID},{second}"))).unwrap();

        assert_eq!(catalog.entries.len(), 1);
        assert_eq!(catalog.entries[0].name.as_str(), "Name");
        assert_eq!(catalog.skipped, 1);
    }

    #[test]
    fn only_known_counts_as_ransomware() {
        for (value, expected) in [
            (r#""Known""#, true),
            (r#""Unknown""#, false),
            (r#""known""#, false),
            ("true", false),
        ] {
            let raw = VALID.replace(r#""Unknown""#, value);
            let catalog = parse_catalog(&catalog(&raw)).unwrap();
            assert_eq!(
                catalog.entries[0].known_ransomware, expected,
                "value: {value}"
            );
        }
        let without =
            r#"{"cveID":"CVE-2026-1234","vulnerabilityName":"Name","dateAdded":"2026-11-01"}"#;
        assert!(!parse_catalog(&catalog(without)).unwrap().entries[0].known_ransomware);
    }

    #[test]
    fn hostile_entries_are_skipped_and_counted() {
        let hostile = [
            VALID.replace("CVE-2026-1234", "CVE-2026-1/../../x"),
            VALID.replace("CVE-2026-1234", "cve-2026-1234"),
            VALID.replace(r#""2026-11-01""#, r#""2026-13-01""#),
            VALID.replace(r#""2026-11-01""#, "20261101"),
            VALID.replace(r#""Name""#, r#""\u202e""#),
            VALID.replace(r#""Name""#, "{}"),
            "null".to_owned(),
            r#""text""#.to_owned(),
        ];
        let catalog = parse_catalog(&catalog(&hostile.join(","))).unwrap();

        assert!(catalog.entries.is_empty());
        assert_eq!(catalog.skipped, hostile.len());
    }

    #[test]
    fn hostile_text_in_the_name_is_cleaned() {
        let raw = VALID.replace(
            r#""Name""#,
            &format!(r#""a\u202e\u001b[31m<img src=x> {}""#, "z".repeat(10_000)),
        );
        let entry = &parse_catalog(&catalog(&raw)).unwrap().entries[0];

        assert!(entry.name.as_str().starts_with("a[31m<img src=x> zz"));
        assert!(entry.name.as_str().chars().count() <= crate::contract::TITLE_MAX_CHARS);
    }

    #[test]
    fn catalog_of_the_wrong_shape_is_an_error() {
        for body in [
            &b"not json"[..],
            br#"{"vulnerabilities":{}}"#,
            br#"{"vulnerabilities":null}"#,
            br#"{"other":[]}"#,
            b"[]",
        ] {
            assert_eq!(parse_catalog(body), Err(KevError::Malformed));
        }
    }

    #[test]
    fn deeply_nested_json_is_an_error_not_a_stack_overflow() {
        let body = format!(r#"{{"vulnerabilities":[{}]}}"#, "[".repeat(100_000));
        assert_eq!(parse_catalog(body.as_bytes()), Err(KevError::Malformed));
    }

    #[test]
    fn too_many_entries_is_an_error() {
        let body = catalog(&vec!["0"; MAX_ENTRIES + 1].join(","));
        assert_eq!(parse_catalog(&body), Err(KevError::TooManyEntries));
    }

    #[test]
    fn kev_errors_explain_themselves() {
        assert_eq!(
            KevError::Malformed.to_string(),
            "catalog is not the expected JSON"
        );
        assert_eq!(
            KevError::TooManyEntries.to_string(),
            "catalog has too many entries"
        );
    }
}
