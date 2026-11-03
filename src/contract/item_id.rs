//! Stable item identifiers, used by the plugin to tell items apart.

use std::fmt;

use serde::Serialize;

/// Longest identifier accepted, in characters.
pub const ID_MAX_CHARS: usize = 128;

/// 1 to 128 characters from `[A-Za-z0-9:._-]`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct ItemId(String);

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidId;

impl fmt::Display for InvalidId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("item id is empty, too long or has a character outside [A-Za-z0-9:._-]")
    }
}

impl std::error::Error for InvalidId {}

impl ItemId {
    pub fn new(_raw: &str) -> Result<Self, InvalidId> {
        todo!()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_with_allowed_characters_is_accepted() {
        assert_eq!(ItemId::new("sample:news").unwrap().as_str(), "sample:news");
        assert!(ItemId::new("CVE-2026_0001.v2").is_ok());
    }

    #[test]
    fn id_at_the_limit_is_accepted() {
        assert!(ItemId::new(&"a".repeat(ID_MAX_CHARS)).is_ok());
    }

    #[test]
    fn empty_long_or_odd_ids_are_refused() {
        let too_long = "a".repeat(ID_MAX_CHARS + 1);
        for raw in ["", "a b", "../x", "é", "a\u{0}", too_long.as_str()] {
            assert_eq!(ItemId::new(raw), Err(InvalidId), "input: {raw:?}");
        }
    }

    #[test]
    fn invalid_id_error_explains_itself() {
        assert!(InvalidId.to_string().starts_with("item id is empty"));
    }
}
