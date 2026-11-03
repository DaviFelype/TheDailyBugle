//! Text from external sources, cleaned so it can only be shown as one line of
//! plain text.

use std::fmt;

use serde::Serialize;

/// Longest title kept, in characters.
pub const TITLE_MAX_CHARS: usize = 300;
/// Longest source name kept, in characters.
pub const SOURCE_MAX_CHARS: usize = 64;

/// One line of text without control or bidi characters, without repeated or
/// surrounding spaces, and within a length limit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct PlainText(String);

/// Nothing visible was left after cleaning.
#[derive(Debug, PartialEq, Eq)]
pub struct EmptyText;

impl fmt::Display for EmptyText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("text is empty after cleaning")
    }
}

impl std::error::Error for EmptyText {}

impl PlainText {
    /// Cleans an item title.
    pub fn title(_raw: &str) -> Result<Self, EmptyText> {
        todo!()
    }

    /// Cleans the name of the source an item came from.
    pub fn source(_raw: &str) -> Result<Self, EmptyText> {
        todo!()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn title(raw: &str) -> Result<String, EmptyText> {
        PlainText::title(raw).map(|text| text.as_str().to_owned())
    }

    #[test]
    fn plain_text_passes_unchanged() {
        assert_eq!(title("Ransomware hits X").unwrap(), "Ransomware hits X");
    }

    #[test]
    fn line_breaks_tabs_and_repeated_spaces_become_one_space() {
        assert_eq!(title("  a\n\tb  ").unwrap(), "a b");
        assert_eq!(title("a\r\n\r\nb").unwrap(), "a b");
    }

    #[test]
    fn title_at_the_limit_is_kept_whole() {
        let raw = "a".repeat(TITLE_MAX_CHARS);
        assert_eq!(title(&raw).unwrap(), raw);
    }

    #[test]
    fn title_over_the_limit_is_cut_with_an_ellipsis_without_splitting_a_character() {
        let raw = format!("{}😀😀😀", "a".repeat(TITLE_MAX_CHARS - 2));
        let cleaned = title(&raw).unwrap();

        assert_eq!(cleaned.chars().count(), TITLE_MAX_CHARS);
        assert_eq!(cleaned, format!("{}😀…", "a".repeat(TITLE_MAX_CHARS - 2)));
    }

    #[test]
    fn cut_does_not_leave_a_space_before_the_ellipsis() {
        let raw = format!("{} bcd", "a".repeat(TITLE_MAX_CHARS - 2));
        assert_eq!(
            title(&raw).unwrap(),
            format!("{}…", "a".repeat(TITLE_MAX_CHARS - 2))
        );
    }

    #[test]
    fn source_has_its_own_limit() {
        let raw = "s".repeat(SOURCE_MAX_CHARS + 1);
        let cleaned = PlainText::source(&raw).unwrap();
        assert_eq!(cleaned.as_str().chars().count(), SOURCE_MAX_CHARS);
    }

    #[test]
    fn text_with_nothing_visible_is_empty() {
        for raw in ["", "   ", "\u{202E}", "\n\t\u{0}"] {
            assert_eq!(title(raw), Err(EmptyText), "input: {raw:?}");
        }
    }

    #[test]
    fn bidi_overrides_are_removed() {
        assert_eq!(title("abc\u{202E}gpj.exe").unwrap(), "abcgpj.exe");
        assert_eq!(
            title("\u{2066}a\u{2069}\u{200E}b\u{200F}c\u{061C}").unwrap(),
            "abc"
        );
    }

    #[test]
    fn control_characters_and_terminal_escapes_are_removed() {
        assert_eq!(title("\u{1b}[31mx\u{0}\u{85}").unwrap(), "[31mx");
        assert_eq!(title("a\u{7f}b\u{9b}c").unwrap(), "abc");
    }

    #[test]
    fn markup_is_kept_as_literal_text() {
        assert_eq!(title("<img src=x>").unwrap(), "<img src=x>");
    }

    #[test]
    fn empty_text_error_explains_itself() {
        assert_eq!(EmptyText.to_string(), "text is empty after cleaning");
    }
}
