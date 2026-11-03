//! The whole document printed by `feed --json`.

use serde::Serialize;

use super::item::Item;

/// Version of the JSON contract. The plugin refuses any other.
pub const SCHEMA_VERSION: u32 = 1;
/// Most items in one feed; the plugin shows no more than this either.
pub const MAX_ITEMS: usize = 200;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    schema_version: u32,
    items: Vec<Item>,
}

impl Feed {
    /// Keeps the first `MAX_ITEMS` items, in the given order.
    pub fn new(_items: Vec<Item>) -> Self {
        todo!()
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }

    /// One line of JSON, the contract read by the plugin.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        todo!()
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
        let feed = Feed::new((0..=MAX_ITEMS).map(item).collect());

        assert_eq!(feed.items().len(), MAX_ITEMS);
        assert_eq!(feed.items()[0].id.as_str(), "test:0");
        assert_eq!(feed.items()[MAX_ITEMS - 1].id.as_str(), "test:199");
    }

    #[test]
    fn json_has_the_version_and_the_items_in_one_line() {
        let json = Feed::new(vec![item(1)]).to_json().unwrap();

        assert_eq!(
            json,
            r#"{"schemaVersion":1,"items":[{"id":"test:1","category":"news","title":"Title","url":"https://example.com/","source":"Example"}]}"#
        );
    }
}
