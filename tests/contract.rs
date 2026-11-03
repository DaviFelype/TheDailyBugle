//! The JSON contract: what `feed --json` prints, checked against the schema
//! the plugin is written for.

use std::error::Error;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

type TestResult<T> = Result<T, Box<dyn Error>>;

fn read_json(relative_path: &str) -> TestResult<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn validator() -> TestResult<jsonschema::Validator> {
    Ok(jsonschema::validator_for(&read_json(
        "schema/feed-v1.schema.json",
    )?)?)
}

fn feed_stdout() -> TestResult<String> {
    let output = Command::new(env!("CARGO_BIN_EXE_thedailybugle"))
        .args(["feed", "--json"])
        .output()?;
    if !output.status.success() {
        return Err(format!("feed --json failed: {:?}", output.status).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

#[test]
fn feed_output_follows_the_schema() {
    let output: Value = serde_json::from_str(&feed_stdout().unwrap()).unwrap();

    let errors: Vec<String> = validator()
        .unwrap()
        .iter_errors(&output)
        .map(|error| error.to_string())
        .collect();
    assert!(errors.is_empty(), "schema errors: {errors:?}");
}

#[test]
fn feed_output_matches_the_snapshot() {
    insta::assert_snapshot!(feed_stdout().unwrap());
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
