//! Built-in sample items. They let the plugin be tried end to end before the
//! real sources exist (roadmap phase 3), which replace them.

use crate::contract::{Category, Feed, Item, RawItem};

const SAMPLE_SOURCE: &str = "TheDailyBugle sample";

const SAMPLE_ITEMS: [RawItem<'static>; 3] = [
    RawItem {
        id: "sample:vulnerability",
        category: Category::Vulnerability,
        title: "[Sample] Exploited vulnerability with a public PoC",
        url: "https://example.com/thedailybugle/sample/vulnerability",
        source: SAMPLE_SOURCE,
    },
    RawItem {
        id: "sample:ransomware",
        category: Category::Ransomware,
        title: "[Sample] Ransomware group claims a new victim",
        url: "https://example.com/thedailybugle/sample/ransomware",
        source: SAMPLE_SOURCE,
    },
    RawItem {
        id: "sample:news",
        category: Category::News,
        title: "[Sample] Security news of the day",
        url: "https://example.com/thedailybugle/sample/news",
        source: SAMPLE_SOURCE,
    },
];

/// The sample feed. Items go through the same checks as real ones; an
/// invalid one is left out.
pub fn feed() -> Feed {
    Feed::new(
        SAMPLE_ITEMS
            .iter()
            .filter_map(|raw| Item::from_raw(raw).ok())
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_feed_has_one_item_of_each_category() {
        let categories: Vec<Category> = feed().items().iter().map(|item| item.category).collect();

        assert_eq!(
            categories,
            [
                Category::Vulnerability,
                Category::Ransomware,
                Category::News
            ]
        );
    }

    #[test]
    fn sample_items_are_marked_as_samples() {
        for item in feed().items() {
            assert!(item.title.as_str().starts_with("[Sample] "));
            assert_eq!(item.source.as_str(), SAMPLE_SOURCE);
        }
    }
}
