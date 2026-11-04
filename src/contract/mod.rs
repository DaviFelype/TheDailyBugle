//! The JSON contract between the binary and the plugin. Its types only hold
//! valid values, so invalid output cannot be built.

mod feed;
mod https_url;
mod item;
mod item_id;
mod plain_text;
mod published_date;

pub use feed::{Feed, MAX_FAILED_SOURCES, MAX_ITEMS, SCHEMA_VERSION};
pub use https_url::{HttpsUrl, URL_MAX_BYTES, UrlError};
pub use item::{Category, Item, ItemError, RawItem};
pub use item_id::{ID_MAX_CHARS, InvalidId, ItemId};
pub use plain_text::{EmptyText, PlainText, SOURCE_MAX_CHARS, TITLE_MAX_CHARS};
pub use published_date::PublishedDate;
