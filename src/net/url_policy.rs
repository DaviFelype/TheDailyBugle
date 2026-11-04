//! Which URLs the client may connect to. Checked before every request and
//! every redirect.

use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlPolicy {
    /// What the binary uses: `https://` only.
    HttpsOnly,
    /// For tests against a local fake server, which speaks plain HTTP: also
    /// `http://127.0.0.1`.
    HttpsOrLoopbackHttp,
}

impl UrlPolicy {
    pub fn allows(self, _url: &Url) -> bool {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn url(raw: &str) -> Url {
        Url::parse(raw).unwrap()
    }

    #[test]
    fn production_allows_only_https() {
        assert!(UrlPolicy::HttpsOnly.allows(&url("https://www.cisa.gov/feed.json")));
        for raw in [
            "http://www.cisa.gov/feed.json",
            "http://127.0.0.1:8080/",
            "file:///etc/passwd",
            "ftp://example.com/",
            "data:text/plain,x",
            "javascript:alert(1)",
        ] {
            assert!(!UrlPolicy::HttpsOnly.allows(&url(raw)), "allowed: {raw}");
        }
    }

    #[test]
    fn test_policy_adds_only_loopback_http() {
        let policy = UrlPolicy::HttpsOrLoopbackHttp;
        assert!(policy.allows(&url("http://127.0.0.1:8080/kev.json")));
        assert!(policy.allows(&url("https://example.com/")));
        assert!(!policy.allows(&url("http://example.com/")));
        assert!(!policy.allows(&url("http://localhost:8080/")));
        assert!(!policy.allows(&url("file:///etc/passwd")));
    }
}
