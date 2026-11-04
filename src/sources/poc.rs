//! nomi-sec/PoC-in-GitHub (CC0 1.0): one JSON file per CVE listing the
//! GitHub repositories that claim a proof of concept.

use std::fmt;

use serde_json::Value;
use url::Url;

use super::cve_id::CveId;

pub const POC_SOURCE: &str = "PoC-in-GitHub";
pub const POC_BASE_URL: &str = "https://raw.githubusercontent.com/nomi-sec/PoC-in-GitHub/master/";

/// The largest file today (Log4Shell) is about 0.6 MiB.
pub const MAX_POC_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_POC_COUNT: u32 = 10_000;

#[derive(Debug, PartialEq, Eq)]
pub struct MalformedPocList;

impl fmt::Display for MalformedPocList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PoC list is not the expected JSON")
    }
}

impl std::error::Error for MalformedPocList {}

/// `<base><year>/<CVE>.json`. The CVE is already validated, so nothing in it
/// can add a path segment or a query.
pub fn poc_url(base: &Url, cve: &CveId) -> Result<Url, url::ParseError> {
    base.join(&format!("{}/{}.json", cve.year(), cve))
}

/// Counts the listed repositories whose link is a plain GitHub repository.
pub fn count_pocs(body: &[u8]) -> Result<u32, MalformedPocList> {
    let list: Vec<Value> = serde_json::from_slice(body).map_err(|_| MalformedPocList)?;
    let count = list
        .iter()
        .filter(|repository| {
            repository
                .get("html_url")
                .and_then(Value::as_str)
                .is_some_and(is_github_repository)
        })
        .count();
    Ok(u32::try_from(count)
        .unwrap_or(MAX_POC_COUNT)
        .min(MAX_POC_COUNT))
}

/// `https://github.com/<owner>/<repo>` and nothing more.
fn is_github_repository(raw: &str) -> bool {
    let Some(path) = raw.strip_prefix("https://github.com/") else {
        return false;
    };
    let valid_segment = |segment: &str| {
        !segment.is_empty()
            && segment
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    };
    let mut segments = path.split('/');
    match (segments.next(), segments.next(), segments.next()) {
        (Some(owner), Some(repository), None) => valid_segment(owner) && valid_segment(repository),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_has_the_year_folder_and_the_cve_file() {
        let base = Url::parse(POC_BASE_URL).unwrap();
        let cve = CveId::parse("CVE-2026-97229").unwrap();

        assert_eq!(
            poc_url(&base, &cve).unwrap().as_str(),
            "https://raw.githubusercontent.com/nomi-sec/PoC-in-GitHub/master/2026/CVE-2026-97229.json"
        );
    }

    #[test]
    fn real_fixture_counts_every_repository() {
        let body = std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/poc/CVE-2026-97229.json"
        ))
        .unwrap();
        assert_eq!(count_pocs(&body), Ok(7));
    }

    #[test]
    fn entries_without_a_plain_github_link_are_not_counted() {
        let body = br#"[
            {"html_url": "https://github.com/a/b"},
            {"html_url": "javascript:alert(1)"},
            {"html_url": "http://github.com/a/b"},
            {"html_url": "https://github.com.evil.example/a/b"},
            {"html_url": "https://github.com/a"},
            {"html_url": "https://github.com/a/b/c"},
            {"html_url": "https://github.com/a/b?x=1"},
            {"html_url": "https://github.com//b"},
            {"html_url": 5},
            {},
            "text",
            null
        ]"#;
        assert_eq!(count_pocs(body), Ok(1));
    }

    #[test]
    fn empty_list_is_zero() {
        assert_eq!(count_pocs(b"[]"), Ok(0));
    }

    #[test]
    fn count_is_capped() {
        let one = r#"{"html_url":"https://github.com/a/b"}"#;
        let body = format!("[{}]", vec![one; 10_001].join(","));
        assert_eq!(count_pocs(body.as_bytes()), Ok(MAX_POC_COUNT));
    }

    #[test]
    fn not_a_list_is_an_error() {
        for body in [
            &b"not json"[..],
            br#"{"message":"Not Found","documentation_url":"x","status":"404"}"#,
            b"null",
        ] {
            assert_eq!(count_pocs(body), Err(MalformedPocList));
        }
        assert_eq!(
            MalformedPocList.to_string(),
            "PoC list is not the expected JSON"
        );
    }
}
