//! The JSON contract: what `feed --json` prints, checked against the schema
//! the plugin is written for.

use std::path::Path;
use std::time::Duration;

use serde_json::{Value, json};
use thedailybugle::collect::{self, Clock, SourceSettings};
use thedailybugle::net::{ClientSettings, HttpClient, UrlPolicy};
use url::Url;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod common;
use common::TestResult;

/// 2026-11-03T12:00:00Z, the day the fixtures were captured.
const FIXTURE_DAY_NOON: u64 = 1_793_707_200;

fn read_json(relative_path: &str) -> TestResult<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn validator() -> TestResult<jsonschema::Validator> {
    Ok(jsonschema::validator_for(&read_json(
        "schema/feed-v1.schema.json",
    )?)?)
}

fn schema_errors(document: &Value) -> TestResult<Vec<String>> {
    Ok(validator()?
        .iter_errors(document)
        .map(|error| error.to_string())
        .collect())
}

/// The binary with a fresh cache, so it never reaches the network.
fn feed_stdout() -> TestResult<String> {
    let cache_home = tempfile::tempdir()?;
    common::write_fresh_cache(cache_home.path())?;
    let output = common::feed_command(cache_home.path()).output()?;
    if !output.status.success() {
        return Err(format!("feed --json failed: {:?}", output.status).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

/// The library against a fake KEV and PoC-in-GitHub serving the real
/// fixtures, on the day they were captured: the same output every run.
async fn fixture_feed_json() -> TestResult<String> {
    let server = MockServer::start().await;
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    Mock::given(method("GET"))
        .and(path("/kev.json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(std::fs::read(fixtures.join("kev/catalog.json"))?),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/poc/2026/CVE-2026-97229.json"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(std::fs::read(fixtures.join("poc/CVE-2026-97229.json"))?),
        )
        .with_priority(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex("^/poc/"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let client = HttpClient::new(&ClientSettings {
        policy: UrlPolicy::HttpsOrLoopbackHttp,
        ..ClientSettings::production()
    })?;
    let settings = SourceSettings {
        kev_url: Url::parse(&format!("{}/kev.json", server.uri()))?,
        poc_base_url: Url::parse(&format!("{}/poc/", server.uri()))?,
        budget: Duration::from_secs(10),
    };
    let feed = collect::build_feed(&client, None, &settings, Clock::at(FIXTURE_DAY_NOON)).await;
    Ok(feed.to_json()?)
}

#[test]
fn binary_output_with_a_fresh_cache_follows_the_schema() {
    let output: Value = serde_json::from_str(&feed_stdout().unwrap()).unwrap();

    assert_eq!(schema_errors(&output).unwrap(), Vec::<String>::new());
    assert_eq!(output["items"][0]["id"], "kev:CVE-2026-97229");
    assert_eq!(output["items"][0]["pocCount"], 7);
    assert!(output.get("failedSources").is_none());
}

#[tokio::test]
async fn fixture_feed_follows_the_schema() {
    let output: Value = serde_json::from_str(&fixture_feed_json().await.unwrap()).unwrap();

    assert_eq!(schema_errors(&output).unwrap(), Vec::<String>::new());
}

#[tokio::test]
async fn feed_output_matches_the_snapshot() {
    insta::assert_snapshot!(fixture_feed_json().await.unwrap());
}

#[test]
fn schema_accepts_the_optional_fields() {
    let document = json!({
        "schemaVersion": 1,
        "items": [{"id": "kev:CVE-2026-1234", "category": "vulnerability",
                   "title": "CVE-2026-1234 Example", "url": "https://nvd.nist.gov/vuln/detail/CVE-2026-1234",
                   "source": "CISA KEV", "publishedAt": "2026-11-03", "pocCount": 0,
                   "knownRansomwareUse": false}],
        "failedSources": ["PoC-in-GitHub"]
    });
    assert_eq!(schema_errors(&document).unwrap(), Vec::<String>::new());
}

#[test]
fn schema_rejects_invalid_optional_fields() {
    let validator = validator().unwrap();
    let item = json!({"id": "kev:CVE-2026-1234", "category": "vulnerability",
                      "title": "CVE-2026-1234 Example", "url": "https://example.com/",
                      "source": "CISA KEV"});
    for (field, value) in [
        ("pocCount", json!(-1)),
        ("pocCount", json!(10_001)),
        ("pocCount", json!(1.5)),
        ("pocCount", json!("3")),
        ("publishedAt", json!("2026-13-01")),
        ("publishedAt", json!("2026-11-3")),
        ("publishedAt", json!("<b>")),
        ("knownRansomwareUse", json!("true")),
    ] {
        let mut bad_item = item.clone();
        bad_item[field] = value.clone();
        let document = json!({"schemaVersion": 1, "items": [bad_item]});
        assert!(!validator.is_valid(&document), "accepted {field}: {value}");
    }

    let names: Vec<String> = (0..11).map(|index| format!("Source {index}")).collect();
    for failed in [
        json!(names),
        json!(["a\u{202e}b"]),
        json!([""]),
        json!("CISA KEV"),
    ] {
        let document = json!({"schemaVersion": 1, "items": [], "failedSources": failed});
        assert!(
            !validator.is_valid(&document),
            "accepted failedSources: {failed}"
        );
    }
}

#[test]
fn schema_accepts_the_valid_fixture() {
    assert!(
        validator()
            .unwrap()
            .is_valid(&read_json("tests/fixtures/feed-v1/valid.json").unwrap())
    );
}

#[test]
fn schema_rejects_output_that_breaks_the_contract() {
    let validator = validator().unwrap();
    for fixture in [
        "http-url.json",
        "credentials-url.json",
        "missing-title.json",
        "schema-version-2.json",
        "bidi-title.json",
        "extra-field.json",
    ] {
        let document = read_json(&format!("tests/fixtures/feed-v1/{fixture}")).unwrap();
        assert!(!validator.is_valid(&document), "accepted: {fixture}");
    }
}

#[test]
fn schema_rejects_titles_with_surrounding_spaces_or_line_breaks() {
    let validator = validator().unwrap();
    for title in [" a", "a ", "a\nb", ""] {
        let document = serde_json::json!({
            "schemaVersion": 1,
            "items": [{"id": "x:1", "category": "news", "title": title,
                       "url": "https://example.com/", "source": "Example"}]
        });
        assert!(!validator.is_valid(&document), "accepted title: {title:?}");
    }
}
