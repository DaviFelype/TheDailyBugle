//! Built-in sample items for the categories that have no real source yet
//! (roadmap phases 3.2 to 3.4), which replace them.

use crate::contract::{Category, Item, RawItem};

const SAMPLE_SOURCE: &str = "TheDailyBugle sample";

const SAMPLE_ITEMS: [RawItem<'static>; 2] = [
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

/// The sample items. They go through the same checks as real ones; an
/// invalid one is left out.
pub fn items() -> Vec<Item> {
    SAMPLE_ITEMS
        .iter()
        .filter_map(|raw| Item::from_raw(raw).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_cover_the_categories_without_a_real_source() {
        let categories: Vec<Category> = items().iter().map(|item| item.category).collect();

        assert_eq!(categories, [Category::Ransomware, Category::News]);
    }

    #[test]
    fn sample_items_are_marked_as_samples() {
        for item in items() {
            assert!(item.title.as_str().starts_with("[Sample] "));
            assert_eq!(item.source.as_str(), SAMPLE_SOURCE);
        }
    }
}
