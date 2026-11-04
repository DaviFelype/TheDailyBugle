//! On-disk cache with a time to live per source.

pub mod files;
mod store;

pub use store::{CacheDir, CacheError, default_cache_path};
