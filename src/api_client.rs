// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use std::fmt;
use std::time::{Duration, SystemTime};

use anyhow::Context;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, RETRY_AFTER};
use reqwest::{StatusCode, Url};
use serde::de::DeserializeOwned;

const API_BASE: &str = "https://maonline.jp/research/api/v1/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ErrorKind {
    Authentication,
    NotFound,
    RateLimited,
    Server,
    Request,
    Timeout,
    Network,
    InvalidResponse,
    InvalidPagination,
}

#[derive(Debug)]
pub(crate) struct ApiError {
    pub(crate) kind: ErrorKind,
    pub(crate) status: Option<StatusCode>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self.kind {
            ErrorKind::Authentication => {
                "API authentication failed; check your API key and access permissions"
            }
            ErrorKind::NotFound => "API resource not found",
            ErrorKind::RateLimited => "API rate limit reached; try again later",
            ErrorKind::Server => "API server error; try again later",
            ErrorKind::Request => "API rejected the request; check the search parameters",
            ErrorKind::Timeout => "API request timed out",
            ErrorKind::Network => "failed to communicate with the API",
            ErrorKind::InvalidResponse => "API returned an invalid JSON response",
            ErrorKind::InvalidPagination => "API returned a non-advancing page number",
        };
        write!(f, "{message}")?;
        if let Some(status) = self.status {
            write!(f, " (HTTP {status})")?;
        }
        Ok(())
    }
}

impl std::error::Error for ApiError {}

#[derive(Clone, Copy)]
struct RequestPolicy {
    connect_timeout: Duration,
    request_timeout: Duration,
    max_attempts: u32,
    backoff: Duration,
    max_delay: Duration,
}

impl Default for RequestPolicy {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(5),
            request_timeout: Duration::from_secs(30),
            max_attempts: 4,
            backoff: Duration::from_millis(500),
            max_delay: Duration::from_secs(30),
        }
    }
}

pub(crate) struct ApiClient {
    http: reqwest::Client,
    base: Url,
    authorization: HeaderValue,
    policy: RequestPolicy,
}

impl ApiClient {
    pub(crate) fn new(api_key: &str) -> anyhow::Result<Self> {
        Self::with_policy(API_BASE, api_key, RequestPolicy::default())
    }

    #[cfg(test)]
    pub(crate) fn for_test(base: &str) -> Self {
        Self::with_policy(
            base,
            "test-key",
            RequestPolicy {
                backoff: Duration::from_millis(1),
                max_delay: Duration::from_millis(10),
                ..RequestPolicy::default()
            },
        )
        .unwrap()
    }

    fn with_policy(base: &str, api_key: &str, policy: RequestPolicy) -> anyhow::Result<Self> {
        let mut authorization =
            HeaderValue::from_str(&format!("Bearer {api_key}")).context("invalid API key")?;
        authorization.set_sensitive(true);
        let http = reqwest::Client::builder()
            .connect_timeout(policy.connect_timeout)
            .timeout(policy.request_timeout)
            .user_agent(concat!("maonline_cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("failed to initialize API client")?;
        Ok(Self {
            http,
            base: Url::parse(base).context("invalid API base URL")?,
            authorization,
            policy,
        })
    }

    pub(crate) fn endpoint(&self, path: &str) -> Url {
        self.base.join(path).expect("static API path must be valid")
    }

    pub(crate) async fn get_json<T: DeserializeOwned>(
        &self,
        url: &Url,
        query: &[(&str, String)],
    ) -> Result<T, ApiError> {
        for attempt in 0..self.policy.max_attempts {
            let response = self
                .http
                .get(url.clone())
                .header(AUTHORIZATION, self.authorization.clone())
                .query(query)
                .send()
                .await;
            let error = match response {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        match response.bytes().await {
                            Ok(body) => {
                                // Response bodies and URLs can contain private data. Do not attach
                                // them (or serde's value excerpts) to errors or diagnostic logs.
                                return serde_json::from_slice(&body).map_err(|_| ApiError {
                                    kind: ErrorKind::InvalidResponse,
                                    status: Some(status),
                                });
                            }
                            Err(error) => transport_error(&error),
                        }
                    } else {
                        let error = status_error(status);
                        if retryable_status(status) && attempt + 1 < self.policy.max_attempts {
                            let delay = retry_delay(
                                response.headers(),
                                attempt,
                                self.policy,
                                SystemTime::now(),
                            );
                            drop(response);
                            tokio::time::sleep(delay).await;
                            continue;
                        }
                        return Err(error);
                    }
                }
                Err(error) => transport_error(&error),
            };
            if attempt + 1 == self.policy.max_attempts {
                return Err(error);
            }
            tokio::time::sleep(retry_delay(
                &HeaderMap::new(),
                attempt,
                self.policy,
                SystemTime::now(),
            ))
            .await;
        }
        unreachable!("request policy always allows at least one attempt")
    }

    pub(crate) async fn get_pages<R: DeserializeOwned, T>(
        &self,
        path: &str,
        query: impl Fn(u32) -> Vec<(&'static str, String)>,
        unpack: impl Fn(R) -> (Vec<T>, Option<u32>),
    ) -> Result<Vec<T>, ApiError> {
        let url = self.endpoint(path);
        let mut items = Vec::new();
        let mut page = 1;
        loop {
            let response = self.get_json(&url, &query(page)).await?;
            let (data, next_page) = unpack(response);
            items.extend(data);
            match next_page {
                None => return Ok(items),
                Some(next) if next > page => page = next,
                Some(_) => {
                    return Err(ApiError {
                        kind: ErrorKind::InvalidPagination,
                        status: None,
                    });
                }
            }
        }
    }
}

fn transport_error(error: &reqwest::Error) -> ApiError {
    ApiError {
        kind: if error.is_timeout() {
            ErrorKind::Timeout
        } else {
            ErrorKind::Network
        },
        status: None,
    }
}

fn status_error(status: StatusCode) -> ApiError {
    let kind = match status.as_u16() {
        401 | 403 => ErrorKind::Authentication,
        404 => ErrorKind::NotFound,
        429 => ErrorKind::RateLimited,
        500..=599 => ErrorKind::Server,
        _ => ErrorKind::Request,
    };
    ApiError {
        kind,
        status: Some(status),
    }
}

fn retryable_status(status: StatusCode) -> bool {
    matches!(status.as_u16(), 429 | 500 | 502 | 503 | 504)
}

fn retry_delay(
    headers: &HeaderMap,
    attempt: u32,
    policy: RequestPolicy,
    now: SystemTime,
) -> Duration {
    let server_delay = headers
        .get(RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| {
            value
                .parse::<u64>()
                .ok()
                .map(Duration::from_secs)
                .or_else(|| {
                    httpdate::parse_http_date(value)
                        .ok()
                        .map(|date| date.duration_since(now).unwrap_or_default())
                })
        });
    server_delay
        .unwrap_or_else(|| policy.backoff.saturating_mul(1 << attempt.min(16)))
        .min(policy.max_delay)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_after_supports_seconds_dates_and_a_bounded_backoff() {
        let policy = RequestPolicy::default();
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let mut headers = HeaderMap::new();
        assert_eq!(
            retry_delay(&headers, 2, policy, now),
            Duration::from_secs(2)
        );
        headers.insert(RETRY_AFTER, HeaderValue::from_static("12"));
        assert_eq!(
            retry_delay(&headers, 0, policy, now),
            Duration::from_secs(12)
        );
        headers.insert(
            RETRY_AFTER,
            HeaderValue::from_str(&httpdate::fmt_http_date(now + Duration::from_secs(8))).unwrap(),
        );
        assert_eq!(
            retry_delay(&headers, 0, policy, now),
            Duration::from_secs(8)
        );
        headers.insert(RETRY_AFTER, HeaderValue::from_static("9999999"));
        assert_eq!(
            retry_delay(&headers, 0, policy, now),
            Duration::from_secs(30)
        );
        headers.insert(RETRY_AFTER, HeaderValue::from_static("invalid"));
        assert_eq!(
            retry_delay(&headers, 1, policy, now),
            Duration::from_secs(1)
        );
    }

    #[test]
    fn permanent_errors_are_classified_without_retrying() {
        for (code, kind) in [
            (400, ErrorKind::Request),
            (401, ErrorKind::Authentication),
            (403, ErrorKind::Authentication),
            (404, ErrorKind::NotFound),
            (501, ErrorKind::Server),
        ] {
            let status = StatusCode::from_u16(code).unwrap();
            assert_eq!(status_error(status).kind, kind);
            assert!(!retryable_status(status));
        }
    }
    use serde_json::{Value, json};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn retries_transient_statuses_and_sends_bearer_auth() {
        for status in [429, 500, 502, 503, 504] {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/news/"))
                .and(header("authorization", "Bearer test-key"))
                .respond_with(ResponseTemplate::new(status).insert_header("Retry-After", "0"))
                .up_to_n_times(1)
                .with_priority(1)
                .expect(1)
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok":true})))
                .expect(1)
                .mount(&server)
                .await;
            let client = ApiClient::for_test(&format!("{}/", server.uri()));
            let result: Value = client
                .get_json(&client.endpoint("news/"), &[])
                .await
                .unwrap();
            assert_eq!(result, json!({"ok":true}));
        }
    }

    #[tokio::test]
    async fn stops_after_three_retries() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(503))
            .expect(4)
            .mount(&server)
            .await;
        let client = ApiClient::for_test(&format!("{}/", server.uri()));
        let error = client
            .get_json::<Value>(&client.endpoint("news/"), &[])
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Server);
    }

    #[tokio::test]
    async fn permanent_status_and_malformed_json_fail_without_retries_or_body_disclosure() {
        for (status, body, kind) in [
            (401, "secret-key", ErrorKind::Authentication),
            (403, "secret-key", ErrorKind::Authentication),
            (404, "secret-key", ErrorKind::NotFound),
            (501, "secret-key", ErrorKind::Server),
            (200, "secret-key invalid json", ErrorKind::InvalidResponse),
        ] {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .respond_with(ResponseTemplate::new(status).set_body_string(body))
                .expect(1)
                .mount(&server)
                .await;
            let client = ApiClient::for_test(&format!("{}/", server.uri()));
            let error = client
                .get_json::<Value>(&client.endpoint("news/"), &[])
                .await
                .unwrap_err();
            assert_eq!(error.kind, kind);
            assert!(!format!("{error:?} {error}").contains("secret-key"));
        }
    }

    #[tokio::test]
    async fn request_timeout_is_bounded_and_classified() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_delay(Duration::from_millis(500))
                    .set_body_json(json!({})),
            )
            .expect(2)
            .mount(&server)
            .await;
        let client = ApiClient::with_policy(
            &format!("{}/", server.uri()),
            "test-key",
            RequestPolicy {
                request_timeout: Duration::from_millis(40),
                max_attempts: 2,
                backoff: Duration::ZERO,
                ..RequestPolicy::default()
            },
        )
        .unwrap();
        let error = client
            .get_json::<Value>(&client.endpoint("news/"), &[])
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Timeout);
    }

    #[derive(serde::Deserialize)]
    struct Page {
        data: Vec<u32>,
        next_page: Option<u32>,
    }

    #[tokio::test]
    async fn rejects_non_advancing_pages_and_propagates_later_page_errors() {
        for next in [0, 1] {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(json!({"data": [1], "next_page": next})),
                )
                .expect(1)
                .mount(&server)
                .await;
            let client = ApiClient::for_test(&format!("{}/", server.uri()));
            let error = client
                .get_pages::<Page, _>(
                    "news/",
                    |page| vec![("page", page.to_string())],
                    |page| (page.data, page.next_page),
                )
                .await
                .unwrap_err();
            assert_eq!(error.kind, ErrorKind::InvalidPagination);
        }
        let server = MockServer::start().await;
        Mock::given(wiremock::matchers::query_param("page", "1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"data":[1],"next_page":2})),
            )
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(wiremock::matchers::query_param("page", "2"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;
        let client = ApiClient::for_test(&format!("{}/", server.uri()));
        let error = client
            .get_pages::<Page, _>(
                "news/",
                |page| vec![("page", page.to_string())],
                |page| (page.data, page.next_page),
            )
            .await
            .unwrap_err();
        assert_eq!(error.kind, ErrorKind::Authentication);
    }
}
