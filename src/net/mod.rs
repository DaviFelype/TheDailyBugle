//! HTTPS access shared by every source.

mod client;
mod url_policy;

pub use client::{ClientSettings, FetchError, HttpClient, HttpResponse};
pub use url_policy::UrlPolicy;
