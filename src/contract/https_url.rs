//! Links the plugin may open: HTTPS only, no credentials, bounded length.

use std::fmt;

use serde::Serialize;

/// Longest link kept, in bytes, before and after normalization.
pub const URL_MAX_BYTES: usize = 2048;

/// A normalized `https://` URL with a host and no user name or password.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct HttpsUrl(String);

/// Why a link was refused.
#[derive(Debug, PartialEq, Eq)]
pub enum UrlError {
    TooLong,
    Malformed,
    NotHttps,
    /// `https://bank.example@evil.example/` reads as the bank but goes to evil.
    HasCredentials,
}

impl fmt::Display for UrlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::TooLong => "link is too long",
            Self::Malformed => "link is malformed",
            Self::NotHttps => "link is not https",
            Self::HasCredentials => "link carries a user name or password",
        })
    }
}

impl std::error::Error for UrlError {}

impl HttpsUrl {
    pub fn parse(_raw: &str) -> Result<Self, UrlError> {
        todo!()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(raw: &str) -> Result<String, UrlError> {
        HttpsUrl::parse(raw).map(|url| url.as_str().to_owned())
    }

    #[test]
    fn https_url_is_accepted() {
        assert_eq!(
            parse("https://example.com/a?b=c").unwrap(),
            "https://example.com/a?b=c"
        );
    }

    #[test]
    fn uppercase_scheme_and_host_are_normalized() {
        assert_eq!(
            parse("HTTPS://EXAMPLE.com").unwrap(),
            "https://example.com/"
        );
    }

    #[test]
    fn other_schemes_are_refused() {
        for raw in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "data:text/html,<script>alert(1)</script>",
            "http://example.com/",
        ] {
            assert_eq!(parse(raw), Err(UrlError::NotHttps), "input: {raw}");
        }
    }

    #[test]
    fn credentials_are_refused() {
        assert_eq!(
            parse("https://bank.example@evil.example/"),
            Err(UrlError::HasCredentials)
        );
        assert_eq!(
            parse("https://user:pass@example.com/"),
            Err(UrlError::HasCredentials)
        );
    }

    #[test]
    fn url_over_the_limit_is_refused() {
        let raw = format!("https://example.com/{}", "a".repeat(URL_MAX_BYTES));
        assert_eq!(parse(&raw[..URL_MAX_BYTES + 1]), Err(UrlError::TooLong));
    }

    #[test]
    fn url_at_the_limit_is_accepted() {
        let raw = format!("https://example.com/{}", "a".repeat(URL_MAX_BYTES));
        assert!(parse(&raw[..URL_MAX_BYTES]).is_ok());
    }

    #[test]
    fn url_that_grows_past_the_limit_when_encoded_is_refused() {
        let raw = format!("https://example.com/{}", "é".repeat(900));
        assert!(raw.len() <= URL_MAX_BYTES);
        assert_eq!(parse(&raw), Err(UrlError::TooLong));
    }

    #[test]
    fn line_breaks_inside_the_url_are_removed() {
        assert_eq!(parse("https://a.com/\nx").unwrap(), "https://a.com/x");
    }

    #[test]
    fn space_in_the_host_is_refused() {
        assert_eq!(parse("https://a .com"), Err(UrlError::Malformed));
    }

    #[test]
    fn missing_host_is_refused() {
        assert_eq!(parse("https://"), Err(UrlError::Malformed));
        assert_eq!(parse("not a url"), Err(UrlError::Malformed));
    }

    #[test]
    fn url_errors_explain_themselves() {
        assert_eq!(UrlError::TooLong.to_string(), "link is too long");
        assert_eq!(UrlError::Malformed.to_string(), "link is malformed");
        assert_eq!(UrlError::NotHttps.to_string(), "link is not https");
        assert_eq!(
            UrlError::HasCredentials.to_string(),
            "link carries a user name or password"
        );
    }
}
