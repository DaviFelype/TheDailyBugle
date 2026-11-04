//! KEV, PoC-in-GitHub and the cache together, against a fake server.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tempfile::TempDir;
use thedailybugle::cache::CacheDir;
use thedailybugle::cache::files::{
    self, CachedPoc, KEV_FILE, MAX_CACHE_FILE_BYTES, POC_FILE, encode_kev, encode_poc,
};
use thedailybugle::collect::{Clock, Collected, SourceSettings, collect_vulnerabilities};
use thedailybugle::net::{ClientSettings, HttpClient, UrlPolicy};
use thedailybugle::sources::cve_id::CveId;
use thedailybugle::sources::kev::{self, KevEntry, entry_from_fields};
use url::Url;
use wiremock::matchers::{any, method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// 2026-11-03T12:00:00Z, the day the fixtures were captured.
const NOW: u64 = 1_793_707_200;
const HOUR: u64 = 60 * 60;
/// The fixture entries inside the 30-day window on 2026-11-03, newest first.
const IN_WINDOW: [&str; 5] = [
    "CVE-2026-119977",
    "CVE-2026-118527",
    "CVE-2026-97229",
    "CVE-2015-3306",
    "CVE-2026-88779",
];

fn fixture(relative_path: &str) -> TestResult<Vec<u8>> {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(relative_path);
    Ok(std::fs::read(path)?)
}

fn client() -> TestResult<HttpClient> {
    Ok(HttpClient::new(&ClientSettings {
        policy: UrlPolicy::HttpsOrLoopbackHttp,
        ..ClientSettings::production()
    })?)
}

fn settings(server: &MockServer, budget: Duration) -> TestResult<SourceSettings> {
    Ok(SourceSettings {
        kev_url: Url::parse(&format!("{}/kev.json", server.uri()))?,
        poc_base_url: Url::parse(&format!("{}/poc/", server.uri()))?,
        budget,
    })
}

fn cache() -> TestResult<(TempDir, CacheDir)> {
    let root = tempfile::tempdir()?;
    let cache = CacheDir::open(&root.path().join("thedailybugle"))?;
    Ok((root, cache))
}

fn fixture_entries() -> TestResult<Vec<KevEntry>> {
    Ok(kev::parse_catalog(&fixture("kev/catalog.json")?)?.entries)
}

fn entry(cve: &str, date_added: &str) -> TestResult<KevEntry> {
    Ok(
        entry_from_fields(cve, "Example Vulnerability", date_added, false)
            .ok_or("invalid entry")?,
    )
}

fn cve(raw: &str) -> TestResult<CveId> {
    Ok(CveId::parse(raw).ok_or("invalid CVE")?)
}

fn write_kev_cache(cache: &CacheDir, fetched_at: u64, entries: &[KevEntry]) -> TestResult {
    Ok(cache.write_atomic(KEV_FILE, &encode_kev(fetched_at, entries)?)?)
}

fn write_poc_cache(cache: &CacheDir, counts: &[(&str, u32, u64)]) -> TestResult {
    let mut entries = BTreeMap::new();
    for (raw, count, fetched_at) in counts {
        entries.insert(
            cve(raw)?,
            CachedPoc {
                count: *count,
                fetched_at: *fetched_at,
            },
        );
    }
    Ok(cache.write_atomic(POC_FILE, &encode_poc(&entries)?)?)
}

async fn serve_kev(server: &MockServer, response: ResponseTemplate, expected_requests: u64) {
    Mock::given(method("GET"))
        .and(path("/kev.json"))
        .respond_with(response)
        .expect(expected_requests)
        .mount(server)
        .await;
}

async fn serve_kev_fixture(server: &MockServer) -> TestResult {
    let body = fixture("kev/catalog.json")?;
    serve_kev(server, ResponseTemplate::new(200).set_body_bytes(body), 1).await;
    Ok(())
}

/// The real PoC file for CVE-2026-97229, and `fallback` for every other CVE.
async fn serve_pocs(server: &MockServer, fallback: ResponseTemplate) -> TestResult {
    let body = fixture("poc/CVE-2026-97229.json")?;
    Mock::given(method("GET"))
        .and(path("/poc/2026/CVE-2026-97229.json"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(body))
        .with_priority(1)
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex("^/poc/"))
        .respond_with(fallback)
        .mount(server)
        .await;
    Ok(())
}

async fn collect(server: &MockServer, cache: Option<&CacheDir>) -> TestResult<Collected> {
    Ok(collect_vulnerabilities(
        &client()?,
        cache,
        &settings(server, Duration::from_secs(10))?,
        Clock::at(NOW),
    )
    .await)
}

fn ids(collected: &Collected) -> Vec<String> {
    collected
        .items
        .iter()
        .map(|item| item.id.as_str().trim_start_matches("kev:").to_owned())
        .collect()
}

fn poc_counts(collected: &Collected) -> Vec<Option<u32>> {
    collected.items.iter().map(|item| item.poc_count).collect()
}

#[tokio::test]
async fn kev_window_with_poc_counts_and_404_as_zero() {
    let server = MockServer::start().await;
    serve_kev_fixture(&server).await.unwrap();
    serve_pocs(&server, ResponseTemplate::new(404))
        .await
        .unwrap();

    let collected = collect(&server, None).await.unwrap();

    assert_eq!(ids(&collected), IN_WINDOW);
    assert_eq!(
        poc_counts(&collected),
        [Some(0), Some(0), Some(7), Some(0), Some(0)]
    );
    assert!(collected.failed_sources.is_empty());
    let first = &collected.items[0];
    assert_eq!(first.published_at.unwrap().to_string(), "2026-11-03");
    assert_eq!(first.known_ransomware_use, Some(false));
    assert!(
        first
            .title
            .as_str()
            .starts_with("CVE-2026-119977 WordPress")
    );
}

#[tokio::test]
async fn poc_lookup_that_fails_leaves_the_count_out_and_reports_the_source() {
    let server = MockServer::start().await;
    serve_kev_fixture(&server).await.unwrap();
    for (cve_path, response) in [
        ("/poc/2026/CVE-2026-119977.json", ResponseTemplate::new(500)),
        (
            "/poc/2026/CVE-2026-118527.json",
            ResponseTemplate::new(200).set_body_string("not json"),
        ),
    ] {
        Mock::given(path(cve_path))
            .respond_with(response)
            .with_priority(1)
            .mount(&server)
            .await;
    }
    serve_pocs(&server, ResponseTemplate::new(404))
        .await
        .unwrap();

    let collected = collect(&server, None).await.unwrap();

    assert_eq!(
        poc_counts(&collected),
        [None, None, Some(7), Some(0), Some(0)]
    );
    assert_eq!(collected.failed_sources, ["PoC-in-GitHub"]);
}

#[tokio::test]
async fn fresh_cache_makes_no_request() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    let (_root, cache) = cache().unwrap();
    write_kev_cache(&cache, NOW - HOUR, &fixture_entries().unwrap()).unwrap();
    write_poc_cache(&cache, &IN_WINDOW.map(|raw| (raw, 2, NOW - HOUR))).unwrap();

    let collected = collect(&server, Some(&cache)).await.unwrap();

    assert_eq!(ids(&collected), IN_WINDOW);
    assert_eq!(poc_counts(&collected), [Some(2); 5]);
    assert!(collected.failed_sources.is_empty());
}

#[tokio::test]
async fn stale_cache_is_fetched_again_and_rewritten() {
    let server = MockServer::start().await;
    serve_kev_fixture(&server).await.unwrap();
    serve_pocs(&server, ResponseTemplate::new(404))
        .await
        .unwrap();
    let (_root, cache) = cache().unwrap();
    write_kev_cache(
        &cache,
        NOW - 7 * HOUR,
        &[entry("CVE-2026-1234", "2026-11-01").unwrap()],
    )
    .unwrap();

    let collected = collect(&server, Some(&cache)).await.unwrap();

    assert_eq!(ids(&collected), IN_WINDOW);
    let rewritten =
        files::decode_kev(&cache.read(KEV_FILE, MAX_CACHE_FILE_BYTES).unwrap()).unwrap();
    assert_eq!(rewritten.fetched_at, NOW);
    // The cache keeps 60 days: the 32-day-old entry stays, the 77-day-old one goes.
    assert_eq!(rewritten.entries.len(), 6);
    let pocs = files::decode_poc(&cache.read(POC_FILE, MAX_CACHE_FILE_BYTES).unwrap()).unwrap();
    assert_eq!(pocs.len(), 5);
    assert!(pocs.values().all(|poc| poc.fetched_at == NOW));
}

#[tokio::test]
async fn stale_cache_is_used_when_kev_is_down() {
    let server = MockServer::start().await;
    serve_kev(&server, ResponseTemplate::new(503), 1).await;
    let (_root, cache) = cache().unwrap();
    write_kev_cache(
        &cache,
        NOW - 7 * HOUR,
        &[entry("CVE-2026-1234", "2026-11-01").unwrap()],
    )
    .unwrap();
    write_poc_cache(&cache, &[("CVE-2026-1234", 4, NOW - HOUR)]).unwrap();

    let collected = collect(&server, Some(&cache)).await.unwrap();

    assert_eq!(ids(&collected), ["CVE-2026-1234"]);
    assert_eq!(poc_counts(&collected), [Some(4)]);
    assert_eq!(collected.failed_sources, ["CISA KEV"]);
}

#[tokio::test]
async fn cache_older_than_seven_days_is_not_used_when_kev_is_down() {
    let server = MockServer::start().await;
    serve_kev(&server, ResponseTemplate::new(503), 1).await;
    let (_root, cache) = cache().unwrap();
    write_kev_cache(
        &cache,
        NOW - 8 * 24 * HOUR,
        &[entry("CVE-2026-1234", "2026-11-01").unwrap()],
    )
    .unwrap();

    let collected = collect(&server, Some(&cache)).await.unwrap();

    assert!(collected.items.is_empty());
    assert_eq!(collected.failed_sources, ["CISA KEV"]);
}

#[tokio::test]
async fn no_cache_and_kev_down_gives_no_vulnerabilities() {
    let server = MockServer::start().await;
    serve_kev(&server, ResponseTemplate::new(503), 1).await;
    let (_root, cache) = cache().unwrap();

    let collected = collect(&server, Some(&cache)).await.unwrap();

    assert!(collected.items.is_empty());
    assert_eq!(collected.failed_sources, ["CISA KEV"]);
    assert_eq!(cache.read(KEV_FILE, MAX_CACHE_FILE_BYTES), None);
}

#[tokio::test]
async fn corrupted_cache_is_treated_as_missing() {
    let server = MockServer::start().await;
    serve_kev_fixture(&server).await.unwrap();
    serve_pocs(&server, ResponseTemplate::new(404))
        .await
        .unwrap();
    let (_root, cache) = cache().unwrap();
    cache.write_atomic(KEV_FILE, b"\x00garbage{").unwrap();
    cache.write_atomic(POC_FILE, b"[").unwrap();

    let collected = collect(&server, Some(&cache)).await.unwrap();

    assert_eq!(ids(&collected), IN_WINDOW);
    assert!(collected.failed_sources.is_empty());
}

#[tokio::test]
async fn stale_poc_count_is_used_when_the_lookup_fails() {
    let server = MockServer::start().await;
    serve_kev_fixture(&server).await.unwrap();
    serve_pocs(&server, ResponseTemplate::new(500))
        .await
        .unwrap();
    let (_root, cache) = cache().unwrap();
    write_poc_cache(&cache, &[("CVE-2026-119977", 5, NOW - 7 * HOUR)]).unwrap();

    let collected = collect(&server, Some(&cache)).await.unwrap();

    assert_eq!(poc_counts(&collected), [Some(5), None, Some(7), None, None]);
    assert_eq!(collected.failed_sources, ["PoC-in-GitHub"]);
}

#[tokio::test]
async fn hostile_cache_entries_are_validated_like_the_network() {
    let server = MockServer::start().await;
    serve_kev(&server, ResponseTemplate::new(503), 0).await;
    let (_root, cache) = cache().unwrap();
    let tampered = format!(
        r#"{{"version":1,"fetchedAt":{},"entries":[
            {{"cve":"CVE-2026-1/../x","name":"A","dateAdded":"2026-11-01","knownRansomware":false}},
            {{"cve":"CVE-2026-1234","name":"a\u202eb","dateAdded":"2026-11-01","knownRansomware":true}}
        ]}}"#,
        NOW - HOUR
    );
    cache.write_atomic(KEV_FILE, tampered.as_bytes()).unwrap();

    let collected = collect(&server, Some(&cache)).await.unwrap();

    assert_eq!(ids(&collected), ["CVE-2026-1234"]);
    assert_eq!(collected.items[0].title.as_str(), "CVE-2026-1234 ab");
}

#[tokio::test]
async fn slow_poc_lookups_stop_at_the_run_budget() {
    let server = MockServer::start().await;
    let entries: Vec<String> = (0..40)
        .map(|index| {
            format!(
                r#"{{"cveID":"CVE-2026-{}","vulnerabilityName":"Slow","dateAdded":"2026-11-01"}}"#,
                10_000 + index
            )
        })
        .collect();
    let catalog = format!(r#"{{"vulnerabilities":[{}]}}"#, entries.join(","));
    serve_kev(
        &server,
        ResponseTemplate::new(200).set_body_string(catalog),
        1,
    )
    .await;
    Mock::given(path_regex("^/poc/"))
        .respond_with(ResponseTemplate::new(404).set_delay(Duration::from_secs(5)))
        .mount(&server)
        .await;

    let started = std::time::Instant::now();
    let collected = collect_vulnerabilities(
        &client().unwrap(),
        None,
        &settings(&server, Duration::from_secs(1)).unwrap(),
        Clock::at(NOW),
    )
    .await;

    assert!(
        started.elapsed() < Duration::from_secs(3),
        "took {:?}",
        started.elapsed()
    );
    assert_eq!(collected.items.len(), 40);
    assert!(collected.items.iter().all(|item| item.poc_count.is_none()));
    assert_eq!(collected.failed_sources, ["PoC-in-GitHub"]);
}

#[tokio::test]
async fn oversized_kev_catalog_fails_the_source() {
    let server = MockServer::start().await;
    let body = vec![b' '; kev::MAX_CATALOG_BYTES + 1];
    serve_kev(&server, ResponseTemplate::new(200).set_body_bytes(body), 1).await;

    let collected = collect(&server, None).await.unwrap();

    assert!(collected.items.is_empty());
    assert_eq!(collected.failed_sources, ["CISA KEV"]);
}

#[tokio::test]
async fn production_client_refuses_http_sources_before_connecting() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;

    let collected = collect_vulnerabilities(
        &HttpClient::new(&ClientSettings::production()).unwrap(),
        None,
        &settings(&server, Duration::from_secs(5)).unwrap(),
        Clock::at(NOW),
    )
    .await;

    assert!(collected.items.is_empty());
    assert_eq!(collected.failed_sources, ["CISA KEV"]);
}
