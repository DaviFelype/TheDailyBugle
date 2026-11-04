//! The whole document printed by `feed --json`.

use serde::Serialize;

use super::item::Item;
use super::plain_text::PlainText;

/// Version of the JSON contract. The plugin refuses any other.
pub const SCHEMA_VERSION: u32 = 1;
/// Most items in one feed; the plugin shows no more than this either.
pub const MAX_ITEMS: usize = 200;
pub const MAX_FAILED_SOURCES: usize = 10;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    schema_version: u32,
    items: Vec<Item>,
    /// Sources that could not be updated in this run. Left out when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    failed_sources: Vec<PlainText>,
}

impl Feed {
    /// Keeps the first `MAX_ITEMS` items and `MAX_FAILED_SOURCES` source
    /// names, in the given order.
    pub fn new(mut items: Vec<Item>, mut failed_sources: Vec<PlainText>) -> Self {
        items.truncate(MAX_ITEMS);
        failed_sources.truncate(MAX_FAILED_SOURCES);
        Self {
            schema_version: SCHEMA_VERSION,
            items,
            failed_sources,
        }
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    pub fn failed_sources(&self) -> &[PlainText] {
        &self.failed_sources
    }

    /// One line of JSON, the contract read by the plugin.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{Category, RawItem};

    fn item(number: usize) -> Item {
        let id = format!("test:{number}");
        Item::from_raw(&RawItem {
            id: &id,
            category: Category::News,
            title: "Title",
            url: "https://example.com/",
            source: "Example",
        })
        .unwrap()
    }

    #[test]
    fn feed_keeps_at_most_the_first_200_items() {
        let feed = Feed::new((0..=MAX_ITEMS).map(item).collect(), Vec::new());

        assert_eq!(feed.items().len(), MAX_ITEMS);
        assert_eq!(feed.items()[0].id.as_str(), "test:0");
        assert_eq!(feed.items()[MAX_ITEMS - 1].id.as_str(), "test:199");
    }

    #[test]
    fn json_has_the_version_and_the_items_in_one_line() {
        let json = Feed::new(vec![item(1)], Vec::new()).to_json().unwrap();

        assert_eq!(
            json,
            r#"{"schemaVersion":1,"items":[{"id":"test:1","category":"news","title":"Title","url":"https://example.com/","source":"Example"}]}"#
        );
    }

    #[test]
    fn optional_item_fields_and_failed_sources_appear_when_set() {
        let mut detailed = item(1);
        detailed.published_at = crate::contract::PublishedDate::parse("2026-11-03");
        detailed.poc_count = Some(3);
        detailed.known_ransomware_use = Some(false);
        let failed = vec![PlainText::source("CISA KEV").unwrap()];

        let json = Feed::new(vec![detailed], failed).to_json().unwrap();

        assert_eq!(
            json,
            r#"{"schemaVersion":1,"items":[{"id":"test:1","category":"news","title":"Title","url":"https://example.com/","source":"Example","publishedAt":"2026-11-03","pocCount":3,"knownRansomwareUse":false}],"failedSources":["CISA KEV"]}"#
        );
    }

    #[test]
    fn feed_keeps_at_most_10_failed_sources() {
        let failed = (0..=MAX_FAILED_SOURCES)
            .map(|number| PlainText::source(&format!("Source {number}")).unwrap())
            .collect();
        assert_eq!(
            Feed::new(Vec::new(), failed).failed_sources().len(),
            MAX_FAILED_SOURCES
        );
    }
}
