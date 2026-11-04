//! The HTTP client's limits: size, redirects, timeouts, URL policy.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Duration;

use thedailybugle::net::{ClientSettings, FetchError, HttpClient, UrlPolicy};
use url::Url;
use wiremock::matchers::{any, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const LIMIT: usize = 8 * 1024 * 1024;

fn client(request_timeout: Duration) -> TestResult<HttpClient> {
    Ok(HttpClient::new(&ClientSettings {
        policy: UrlPolicy::HttpsOrLoopbackHttp,
        connect_timeout: Duration::from_secs(5),
        request_timeout,
    })?)
}

fn url(server: &MockServer, path: &str) -> TestResult<Url> {
    Ok(Url::parse(&format!("{}{path}", server.uri()))?)
}

async fn redirect(server: &MockServer, from: &str, to: &str) {
    Mock::given(path(from))
        .respond_with(ResponseTemplate::new(302).insert_header("location", to))
        .mount(server)
        .await;
}

/// A server that answers once with `head` and then `body_bytes` bytes,
/// written by hand so the framing can lie or be missing.
fn raw_server(head: &'static str, body_bytes: usize) -> TestResult<Url> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let _ = stream.write_all(head.as_bytes());
            let chunk = vec![b'x'; 64 * 1024];
            let mut written = 0;
            // The client hangs up once it has seen enough; stop then.
            while written < body_bytes && stream.write_all(&chunk).is_ok() {
                written += chunk.len();
            }
        }
    });
    Ok(Url::parse(&format!("http://{address}/"))?)
}

#[tokio::test]
async fn body_over_the_limit_is_refused() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![b'x'; 9 * 1024 * 1024]))
        .mount(&server)
        .await;

    let result = client(Duration::from_secs(10))
        .unwrap()
        .get(&url(&server, "/").unwrap(), LIMIT)
        .await;

    assert!(matches!(result, Err(FetchError::TooLarge)));
}

#[tokio::test]
async fn body_without_content_length_is_still_limited() {
    let url = raw_server(
        "HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n",
        9 * 1024 * 1024,
    )
    .unwrap();

    let result = client(Duration::from_secs(10))
        .unwrap()
        .get(&url, LIMIT)
        .await;

    assert!(matches!(result, Err(FetchError::TooLarge)));
}

#[tokio::test]
async fn body_is_never_read_past_a_short_content_length() {
    let url = raw_server(
        "HTTP/1.1 200 OK\r\nContent-Length: 16\r\n\r\n",
        9 * 1024 * 1024,
    )
    .unwrap();

    let response = client(Duration::from_secs(10))
        .unwrap()
        .get(&url, LIMIT)
        .await
        .unwrap();

    assert_eq!(response.body.len(), 16);
}

#[tokio::test]
async fn redirects_leaving_the_policy_are_refused() {
    let server = MockServer::start().await;
    let port = server.address().port();
    let to_another_host = format!("http://localhost:{port}/final");
    redirect(&server, "/to-other-host", &to_another_host).await;
    redirect(&server, "/to-file", "file:///etc/passwd").await;
    Mock::given(path("/final"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let client = client(Duration::from_secs(10)).unwrap();

    for start in ["/to-other-host", "/to-file"] {
        let result = client.get(&url(&server, start).unwrap(), LIMIT).await;
        assert!(
            matches!(result, Err(FetchError::Redirect)),
            "{start}: {result:?}"
        );
    }
}

#[tokio::test]
async fn three_redirects_are_followed_and_four_are_refused() {
    let server = MockServer::start().await;
    for (from, to) in [
        ("/r1", "/r2"),
        ("/r2", "/r3"),
        ("/r3", "/r4"),
        ("/r4", "/final"),
    ] {
        redirect(&server, from, to).await;
    }
    Mock::given(path("/final"))
        .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
        .mount(&server)
        .await;
    let client = client(Duration::from_secs(10)).unwrap();

    let three = client
        .get(&url(&server, "/r2").unwrap(), LIMIT)
        .await
        .unwrap();
    let four = client.get(&url(&server, "/r1").unwrap(), LIMIT).await;

    assert_eq!(three.body, b"ok");
    assert!(matches!(four, Err(FetchError::Redirect)), "{four:?}");
}

#[tokio::test]
async fn slow_response_times_out() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(5)))
        .mount(&server)
        .await;

    let result = client(Duration::from_millis(200))
        .unwrap()
        .get(&url(&server, "/").unwrap(), LIMIT)
        .await;

    assert!(matches!(result, Err(FetchError::Timeout)), "{result:?}");
}

#[tokio::test]
async fn production_client_refuses_http_before_connecting() {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&server)
        .await;
    let client = HttpClient::new(&ClientSettings::production()).unwrap();

    for raw in [
        server.uri(),
        "file:///etc/passwd".to_owned(),
        "ftp://example.com/".to_owned(),
    ] {
        let result = client.get(&Url::parse(&raw).unwrap(), LIMIT).await;
        assert!(
            matches!(result, Err(FetchError::UrlNotAllowed)),
            "{raw}: {result:?}"
        );
    }
}

#[tokio::test]
async fn requests_carry_the_user_agent_and_no_cookie() {
    let server = MockServer::start().await;
    let user_agent = format!(
        "thedailybugle/{} (+https://github.com/DaviFelype/TheDailyBugle)",
        env!("CARGO_PKG_VERSION")
    );
    Mock::given(method("GET"))
        .and(header("user-agent", user_agent.as_str()))
        .respond_with(ResponseTemplate::new(200).insert_header("set-cookie", "session=1"))
        .expect(2)
        .mount(&server)
        .await;
    let client = client(Duration::from_secs(10)).unwrap();

    for _ in 0..2 {
        let response = client
            .get(&url(&server, "/").unwrap(), LIMIT)
            .await
            .unwrap();
        assert_eq!(response.status, 200);
    }
    let requests = server.received_requests().await.unwrap();
    assert!(
        requests
            .iter()
            .all(|request| !request.headers.contains_key("cookie"))
    );
}
