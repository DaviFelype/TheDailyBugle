//! S13 of docs/test-strategy/001: any text cleans to `Ok` or `Err`, never a
//! panic, and what comes out follows the plain-text rules.

#![no_main]

use libfuzzer_sys::fuzz_target;
use thedailybugle::contract::{PlainText, TITLE_MAX_CHARS};

/// Written out again on purpose: the oracle must not share code with the
/// function under test.
fn is_hidden(character: char) -> bool {
    character.is_control()
        || matches!(
            character,
            '\u{200E}' | '\u{200F}' | '\u{061C}' | '\u{202A}'..='\u{202E}' | '\u{2066}'..='\u{2069}'
        )
}

fuzz_target!(|raw: &str| {
    if let Ok(text) = PlainText::title(raw) {
        let text = text.as_str();
        assert!(!text.is_empty());
        assert!(text.chars().count() <= TITLE_MAX_CHARS);
        assert!(!text.chars().any(is_hidden));
        assert_eq!(text, text.trim());
        assert!(!text.contains("  "));
        assert!(
            !text
                .chars()
                .any(|character| character.is_whitespace() && character != ' ')
        );
    }
});
