//! One feed entry, built only from validated parts.

use std::fmt;

use serde::Serialize;

use super::https_url::{HttpsUrl, UrlError};
use super::item_id::{InvalidId, ItemId};
use super::plain_text::{EmptyText, PlainText};

/// What kind of news an item is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Vulnerability,
    Ransomware,
    News,
}

/// An item as a source hands it over, before any check.
#[derive(Debug, Clone, Copy)]
pub struct RawItem<'a> {
    pub id: &'a str,
    pub category: Category,
    pub title: &'a str,
    pub url: &'a str,
    pub source: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    pub id: ItemId,
    pub category: Category,
    pub title: PlainText,
    pub url: HttpsUrl,
    pub source: PlainText,
}

/// Which field made an item invalid.
#[derive(Debug, PartialEq, Eq)]
pub enum ItemError {
    Id(InvalidId),
    Title(EmptyText),
    Url(UrlError),
    Source(EmptyText),
}

impl fmt::Display for ItemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Id(error) => write!(formatter, "id: {error}"),
            Self::Title(error) => write!(formatter, "title: {error}"),
            Self::Url(error) => write!(formatter, "url: {error}"),
            Self::Source(error) => write!(formatter, "source: {error}"),
        }
    }
}

impl std::error::Error for ItemError {}

impl Item {
    pub fn from_raw(raw: &RawItem<'_>) -> Result<Self, ItemError> {
        Ok(Self {
            id: ItemId::new(raw.id).map_err(ItemError::Id)?,
            category: raw.category,
            title: PlainText::title(raw.title).map_err(ItemError::Title)?,
            url: HttpsUrl::parse(raw.url).map_err(ItemError::Url)?,
            source: PlainText::source(raw.source).map_err(ItemError::Source)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID: RawItem<'static> = RawItem {
        id: "test:1",
        category: Category::News,
        title: "  A\ntitle ",
        url: "https://example.com/1",
        source: "Example",
    };

    #[test]
    fn valid_raw_item_becomes_a_clean_item() {
        let item = Item::from_raw(&VALID).unwrap();

        assert_eq!(item.id.as_str(), "test:1");
        assert_eq!(item.category, Category::News);
        assert_eq!(item.title.as_str(), "A title");
        assert_eq!(item.url.as_str(), "https://example.com/1");
        assert_eq!(item.source.as_str(), "Example");
    }

    #[test]
    fn each_invalid_field_is_reported() {
        let cases = [
            (RawItem { id: "a b", ..VALID }, ItemError::Id(InvalidId)),
            (
                RawItem {
                    title: " ",
                    ..VALID
                },
                ItemError::Title(EmptyText),
            ),
            (
                RawItem {
                    url: "http://example.com/",
                    ..VALID
                },
                ItemError::Url(UrlError::NotHttps),
            ),
            (
                RawItem {
                    source: "\u{202E}",
                    ..VALID
                },
                ItemError::Source(EmptyText),
            ),
        ];
        for (raw, expected) in cases {
            assert_eq!(Item::from_raw(&raw), Err(expected));
        }
    }

    #[test]
    fn item_errors_name_the_field() {
        assert!(ItemError::Id(InvalidId).to_string().starts_with("id: "));
        assert!(
            ItemError::Title(EmptyText)
                .to_string()
                .starts_with("title: ")
        );
        assert!(
            ItemError::Url(UrlError::NotHttps)
                .to_string()
                .starts_with("url: ")
        );
        assert!(
            ItemError::Source(EmptyText)
                .to_string()
                .starts_with("source: ")
        );
    }
}
