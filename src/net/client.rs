//! The one HTTP client every source uses: the URL policy on every hop,
//! timeouts, and a body size limit that does not trust `Content-Length`.

use std::fmt;
use std::time::Duration;

use reqwest::redirect;
use url::Url;

use super::UrlPolicy;

const MAX_REDIRECTS: usize = 3;
const USER_AGENT: &str = concat!(
    "thedailybugle/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/DaviFelype/TheDailyBugle)"
);

#[derive(Debug, Clone)]
pub struct ClientSettings {
    pub policy: UrlPolicy,
    pub connect_timeout: Duration,
    pub request_timeout: Duration,
}

impl ClientSettings {
    pub fn production() -> Self {
        Self {
            policy: UrlPolicy::HttpsOnly,
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(10),
        }
    }
}

/// Cheap to clone: clones share one connection pool.
#[derive(Debug, Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
    policy: UrlPolicy,
}

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

/// Why a request failed. Messages never carry the response body.
#[derive(Debug, PartialEq, Eq)]
pub enum FetchError {
    Setup,
    UrlNotAllowed,
    Timeout,
    /// A redirect to a URL the policy refuses, or too many of them.
    Redirect,
    TooLarge,
    Transport,
}

impl fmt::Display for FetchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Setup => "could not set up the HTTPS client",
            Self::UrlNotAllowed => "URL not allowed (https only)",
            Self::Timeout => "timed out",
            Self::Redirect => "redirect refused",
            Self::TooLarge => "response too large",
            Self::Transport => "connection failed",
        })
    }
}

impl std::error::Error for FetchError {}

impl FetchError {
    fn from_reqwest(error: &reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::Timeout
        } else if error.is_redirect() {
            Self::Redirect
        } else {
            Self::Transport
        }
    }
}

impl HttpClient {
    pub fn new(settings: &ClientSettings) -> Result<Self, FetchError> {
        // Only the first install in a process counts; later calls (one per
        // client in tests) fail harmlessly.
        let _ = rustls::crypto::ring::default_provider().install_default();

        let policy = settings.policy;
        let redirects = redirect::Policy::custom(move |attempt| {
            if attempt.previous().len() > MAX_REDIRECTS {
                attempt.error("too many redirects")
            } else if !policy.allows(attempt.url()) {
                attempt.error("redirect to a URL the policy refuses")
            } else {
                attempt.follow()
            }
        });

        let mut builder = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(settings.connect_timeout)
            .timeout(settings.request_timeout)
            .redirect(redirects);
        if policy == UrlPolicy::HttpsOnly {
            // A second, independent check of the same rule.
            builder = builder.https_only(true);
        }
        let inner = builder.build().map_err(|_| FetchError::Setup)?;
        Ok(Self { inner, policy })
    }

    /// GETs `url` and reads at most `max_bytes` of body, whatever the status.
    pub async fn get(&self, url: &Url, max_bytes: usize) -> Result<HttpResponse, FetchError> {
        if !self.policy.allows(url) {
            return Err(FetchError::UrlNotAllowed);
        }
        let mut response = self
            .inner
            .get(url.as_str())
            .send()
            .await
            .map_err(|error| FetchError::from_reqwest(&error))?;
        // reqwest hands back a redirect it will not follow (to `file:`, for
        // one) as a plain response; it is still a redirect we refused.
        if response.status().is_redirection() {
            return Err(FetchError::Redirect);
        }
        let status = response.status().as_u16();

        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| FetchError::from_reqwest(&error))?
        {
            if body.len() + chunk.len() > max_bytes {
                return Err(FetchError::TooLarge);
            }
            body.extend_from_slice(&chunk);
        }
        Ok(HttpResponse { status, body })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fetch_errors_explain_themselves_without_details() {
        for (error, text) in [
            (FetchError::Setup, "could not set up the HTTPS client"),
            (FetchError::UrlNotAllowed, "URL not allowed (https only)"),
            (FetchError::Timeout, "timed out"),
            (FetchError::Redirect, "redirect refused"),
            (FetchError::TooLarge, "response too large"),
            (FetchError::Transport, "connection failed"),
        ] {
            assert_eq!(error.to_string(), text);
        }
    }

    #[test]
    fn user_agent_names_the_project_and_version() {
        assert!(USER_AGENT.starts_with(&format!("thedailybugle/{}", env!("CARGO_PKG_VERSION"))));
    }
}
