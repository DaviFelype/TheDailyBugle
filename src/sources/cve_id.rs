//! CVE identifiers. URLs and cache keys are built from them, so only the
//! exact `CVE-YYYY-NNNN` shape gets through.

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CveId(String);

impl CveId {
    /// `CVE-` + four-digit year + `-` + 4 to 19 digits, nothing else.
    pub fn parse(_raw: &str) -> Option<Self> {
        todo!()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn year(&self) -> &str {
        // Safe to slice: `parse` only lets ASCII through, and "CVE-" is 4 bytes.
        &self.0[4..8]
    }
}

impl fmt::Display for CveId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_ids_are_accepted() {
        let id = CveId::parse("CVE-2026-119977").unwrap();
        assert_eq!(id.as_str(), "CVE-2026-119977");
        assert_eq!(id.year(), "2026");
        assert_eq!(id.to_string(), "CVE-2026-119977");
        assert!(CveId::parse("CVE-2015-3306").is_some());
        assert!(CveId::parse(&format!("CVE-2026-{}", "9".repeat(19))).is_some());
    }

    #[test]
    fn anything_that_could_change_a_path_is_refused() {
        for raw in [
            "",
            "CVE-2026-1/../../x",
            "CVE-2026-1234/../x",
            "CVE-2026-1?x",
            "CVE-2026-1234#x",
            "cve-2026-1234",
            "CVE-26-1234",
            "CVE-2026-123",
            "CVE-2026-12345678901234567890",
            "CVE-2026-1234 ",
            " CVE-2026-1234",
            "CVE-2026-１２３４",
            "CVE-2026-1234-1",
            "CVE-2026",
        ] {
            assert_eq!(CveId::parse(raw), None, "input: {raw:?}");
        }
    }
}
