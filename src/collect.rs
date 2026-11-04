//! Builds the feed: KEV entries from the cache or the network, PoC counts for
//! the recent ones, then the sample items. A failing source is reported in
//! `failedSources`; it never stops the rest.

// Stubs until the implementation commit.
#![allow(dead_code, unused_imports)]

use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use time::{Date, OffsetDateTime};
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{Instant, timeout_at};
use url::Url;

use crate::cache::CacheDir;
use crate::cache::files::{
    self, CachedPoc, KEV_FILE, KEV_TTL_SECONDS, MAX_CACHE_FILE_BYTES, POC_FILE, POC_TTL_SECONDS,
};
use crate::contract::{Category, Feed, HttpsUrl, Item, ItemId, PlainText};
use crate::net::{FetchError, HttpClient};
use crate::sample;
use crate::sources::cve_id::CveId;
use crate::sources::kev::{
    self, CACHE_DAYS, KEV_SOURCE, KevEntry, MAX_CATALOG_BYTES, MAX_RECENT, WINDOW_DAYS,
};
use crate::sources::poc::{self, MAX_POC_BYTES, POC_SOURCE};

const MAX_PARALLEL_POC_REQUESTS: usize = 6;
const HTTP_OK: u16 = 200;
const HTTP_NOT_FOUND: u16 = 404;

/// Where the sources live and how long one run may spend on the network.
#[derive(Debug, Clone)]
pub struct SourceSettings {
    pub kev_url: Url,
    pub poc_base_url: Url,
    pub budget: Duration,
}

impl SourceSettings {
    pub fn production() -> Result<Self, url::ParseError> {
        todo!()
    }
}

/// "Now", passed in so tests can pin it.
#[derive(Debug, Clone, Copy)]
pub struct Clock {
    pub unix_seconds: u64,
    pub today: Date,
}

impl Clock {
    pub fn now() -> Self {
        todo!()
    }

    pub fn at(_unix_seconds: u64) -> Self {
        todo!()
    }
}

/// The whole feed: vulnerabilities from KEV, then the sample items.
pub async fn build_feed(
    _client: &HttpClient,
    _cache: Option<&CacheDir>,
    _settings: &SourceSettings,
    _clock: Clock,
) -> Feed {
    todo!()
}

#[derive(Debug)]
pub struct Collected {
    pub items: Vec<Item>,
    pub failed_sources: Vec<&'static str>,
}

pub async fn collect_vulnerabilities(
    _client: &HttpClient,
    _cache: Option<&CacheDir>,
    _settings: &SourceSettings,
    _clock: Clock,
) -> Collected {
    todo!()
}

/// The KEV entries to use, and whether updating them failed.
async fn kev_entries(
    client: &HttpClient,
    cache: Option<&CacheDir>,
    settings: &SourceSettings,
    clock: Clock,
    deadline: Instant,
) -> (Vec<KevEntry>, bool) {
    let cached = cache
        .and_then(|cache| cache.read(KEV_FILE, MAX_CACHE_FILE_BYTES))
        .and_then(|bytes| files::decode_kev(&bytes));
    if let Some(cached) = &cached
        && files::is_fresh(cached.fetched_at, clock.unix_seconds, KEV_TTL_SECONDS)
    {
        return (cached.entries.clone(), false);
    }

    match fetch_kev(client, &settings.kev_url, deadline).await {
        Ok(catalog) => {
            if catalog.skipped > 0 {
                eprintln!(
                    "warning: {KEV_SOURCE}: skipped {} invalid entries",
                    catalog.skipped
                );
            }
            let kept = kev::added_within(&catalog.entries, clock.today, CACHE_DAYS, usize::MAX);
            if let Some(cache) = cache {
                save(
                    cache,
                    KEV_FILE,
                    files::encode_kev(clock.unix_seconds, &kept),
                );
            }
            (kept, false)
        }
        Err(error) => {
            eprintln!("warning: {KEV_SOURCE}: {error}");
            let stale = cached
                .filter(|cached| files::is_usable_when_stale(cached.fetched_at, clock.unix_seconds))
                .map(|cached| cached.entries)
                .unwrap_or_default();
            (stale, true)
        }
    }
}

#[derive(Debug)]
enum SourceError {
    Fetch(FetchError),
    Status(u16),
    Parse,
    OutOfTime,
}

impl std::fmt::Display for SourceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fetch(error) => write!(formatter, "{error}"),
            Self::Status(status) => write!(formatter, "HTTP {status}"),
            Self::Parse => formatter.write_str("response is not the expected JSON"),
            Self::OutOfTime => formatter.write_str("out of time for this run"),
        }
    }
}

async fn fetch_kev(
    client: &HttpClient,
    url: &Url,
    deadline: Instant,
) -> Result<kev::Catalog, SourceError> {
    let response = timeout_at(deadline, client.get(url, MAX_CATALOG_BYTES))
        .await
        .map_err(|_| SourceError::OutOfTime)?
        .map_err(SourceError::Fetch)?;
    if response.status != HTTP_OK {
        return Err(SourceError::Status(response.status));
    }
    kev::parse_catalog(&response.body).map_err(|_| SourceError::Parse)
}

/// PoC counts for `recent`, from the cache when fresh, from the network
/// otherwise; and whether any lookup failed.
async fn poc_counts(
    client: &HttpClient,
    cache: Option<&CacheDir>,
    settings: &SourceSettings,
    clock: Clock,
    deadline: Instant,
    recent: &[KevEntry],
) -> (BTreeMap<CveId, u32>, bool) {
    let cached = cache
        .and_then(|cache| cache.read(POC_FILE, MAX_CACHE_FILE_BYTES))
        .and_then(|bytes| files::decode_poc(&bytes))
        .unwrap_or_default();

    let mut counts = BTreeMap::new();
    let mut to_cache = BTreeMap::new();
    let mut lookups = JoinSet::new();
    let permits = Arc::new(Semaphore::new(MAX_PARALLEL_POC_REQUESTS));

    for entry in recent {
        let cve = entry.cve.clone();
        match cached.get(&cve) {
            Some(poc) if files::is_fresh(poc.fetched_at, clock.unix_seconds, POC_TTL_SECONDS) => {
                counts.insert(cve.clone(), poc.count);
                to_cache.insert(cve, *poc);
            }
            _ => {
                let client = client.clone();
                let base = settings.poc_base_url.clone();
                let permits = Arc::clone(&permits);
                lookups.spawn(async move {
                    let count = timeout_at(deadline, async {
                        // A closed semaphore cannot happen: nothing closes it.
                        let _permit = permits
                            .acquire()
                            .await
                            .map_err(|_| SourceError::OutOfTime)?;
                        fetch_poc_count(&client, &base, &cve).await
                    })
                    .await
                    .unwrap_or(Err(SourceError::OutOfTime));
                    (cve, count)
                });
            }
        }
    }

    let mut failed = false;
    while let Some(joined) = lookups.join_next().await {
        let Ok((cve, result)) = joined else {
            failed = true;
            continue;
        };
        match result {
            Ok(count) => {
                counts.insert(cve.clone(), count);
                to_cache.insert(
                    cve,
                    CachedPoc {
                        count,
                        fetched_at: clock.unix_seconds,
                    },
                );
            }
            Err(error) => {
                failed = true;
                eprintln!("warning: {POC_SOURCE}: {cve}: {error}");
                if let Some(stale) = cached
                    .get(&cve)
                    .filter(|poc| files::is_usable_when_stale(poc.fetched_at, clock.unix_seconds))
                {
                    counts.insert(cve.clone(), stale.count);
                    to_cache.insert(cve, *stale);
                }
            }
        }
    }

    if let Some(cache) = cache
        && to_cache != cached
    {
        save(cache, POC_FILE, files::encode_poc(&to_cache));
    }
    (counts, failed)
}

async fn fetch_poc_count(client: &HttpClient, base: &Url, cve: &CveId) -> Result<u32, SourceError> {
    let url = poc::poc_url(base, cve).map_err(|_| SourceError::Parse)?;
    let response = client
        .get(&url, MAX_POC_BYTES)
        .await
        .map_err(SourceError::Fetch)?;
    match response.status {
        HTTP_OK => poc::count_pocs(&response.body).map_err(|_| SourceError::Parse),
        // No file yet means no PoC listed for this CVE.
        HTTP_NOT_FOUND => Ok(0),
        status => Err(SourceError::Status(status)),
    }
}

/// Saving is best effort: a cache that cannot be written only costs a fetch
/// next time.
fn save(cache: &CacheDir, name: &str, encoded: Result<Vec<u8>, serde_json::Error>) {
    let result = encoded
        .map_err(|error| error.to_string())
        .and_then(|bytes| {
            cache
                .write_atomic(name, &bytes)
                .map_err(|error| error.to_string())
        });
    if let Err(error) = result {
        eprintln!("warning: could not save {name}: {error}");
    }
}

fn vulnerability_item(_entry: &KevEntry, _poc_count: Option<u32>) -> Option<Item> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sources::kev::entry_from_fields;

    #[test]
    fn vulnerability_item_links_to_nvd_and_carries_the_details() {
        let entry = entry_from_fields("CVE-2026-97229", "GitLab RCE", "2026-10-29", true).unwrap();
        let item = vulnerability_item(&entry, Some(7)).unwrap();

        assert_eq!(item.id.as_str(), "kev:CVE-2026-97229");
        assert_eq!(item.category, Category::Vulnerability);
        assert_eq!(item.title.as_str(), "CVE-2026-97229 GitLab RCE");
        assert_eq!(
            item.url.as_str(),
            "https://nvd.nist.gov/vuln/detail/CVE-2026-97229"
        );
        assert_eq!(item.source.as_str(), "CISA KEV");
        assert_eq!(item.published_at.unwrap().to_string(), "2026-10-29");
        assert_eq!(item.poc_count, Some(7));
        assert_eq!(item.known_ransomware_use, Some(true));
    }

    #[test]
    fn clock_derives_the_utc_date() {
        // 2026-11-03T23:59:59Z
        let clock = Clock::at(1_793_750_399);
        assert_eq!(
            crate::contract::PublishedDate::from_date(clock.today)
                .unwrap()
                .to_string(),
            "2026-11-03"
        );
        assert_eq!(Clock::at(1_793_750_400).today.day(), 4);
    }

    #[test]
    fn source_errors_explain_themselves() {
        assert_eq!(
            SourceError::Fetch(FetchError::Timeout).to_string(),
            "timed out"
        );
        assert_eq!(SourceError::Status(503).to_string(), "HTTP 503");
        assert_eq!(
            SourceError::Parse.to_string(),
            "response is not the expected JSON"
        );
        assert_eq!(
            SourceError::OutOfTime.to_string(),
            "out of time for this run"
        );
    }
}
