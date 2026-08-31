// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;

use crate::news::{
    article::NewsArticle, article_type::ArticleType, raw_response::RawSearchResponse,
};

const NEWS_URL: &str = "https://maonline.jp/research/api/v1/news/";
const PER_PAGE: u32 = 200;

pub(super) struct RequestParameters {
    pub(super) query: Option<String>,
    pub(super) company_name: Option<String>,
    pub(super) article_type: Option<ArticleType>,
    pub(super) date_from: Option<String>,
    pub(super) date_to: Option<String>,
    pub(super) with_raw: bool,
}

pub(super) async fn request_news(
    parameters: &RequestParameters,
    api_key: &str,
) -> anyhow::Result<Vec<NewsArticle>> {
    let client = reqwest::Client::new();
    let response = request_raw_news(&client, parameters, api_key, 1).await?;

    Ok(response.data.into_iter().map(NewsArticle::from).collect())
}

#[async_recursion::async_recursion]
async fn request_raw_news(
    client: &reqwest::Client,
    parameters: &RequestParameters,
    api_key: &str,
    page: u32,
) -> anyhow::Result<RawSearchResponse> {
    let request = client
        .get(NEWS_URL)
        .query(&query_parameters(parameters, page))
        .bearer_auth(api_key)
        .build()
        .context("failed to build news API request")?;
    let url = request.url().clone();
    let response = client
        .execute(request)
        .await
        .with_context(|| format!("failed to send request to {url}"))?;
    let status = response.status();

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "request to {url} failed with status {status}"
        ));
    }

    let body = response
        .bytes()
        .await
        .with_context(|| format!("failed to read response body from {url}"))?;
    let mut raw_response: RawSearchResponse = serde_json::from_slice(&body)
        .with_context(|| format!("failed to parse response from {url}"))?;

    if let Some(next_page) = raw_response.meta.next_page {
        let next_response = request_raw_news(client, parameters, api_key, next_page).await?;
        raw_response.data.extend(next_response.data);
        raw_response.meta = next_response.meta;
    }

    Ok(raw_response)
}

fn query_parameters(parameters: &RequestParameters, page: u32) -> Vec<(&'static str, String)> {
    let mut query = vec![
        ("page", page.to_string()),
        ("per_page", PER_PAGE.to_string()),
    ];

    if let Some(value) = &parameters.query {
        query.push(("q", value.clone()));
    }
    if let Some(value) = &parameters.company_name {
        query.push(("company_name", value.clone()));
    }
    if let Some(value) = parameters.article_type {
        query.push(("article_type", value.as_str().to_owned()));
    }
    if let Some(value) = &parameters.date_from {
        query.push(("date_from", value.clone()));
    }
    if let Some(value) = &parameters.date_to {
        query.push(("date_to", value.clone()));
    }
    if parameters.with_raw {
        query.push(("with_raw", "true".to_owned()));
    }

    query
}

#[cfg(test)]
mod tests {
    use crate::news::article_type::ArticleType;

    use super::{NEWS_URL, RequestParameters, query_parameters};

    #[test]
    fn builds_encoded_news_request_url() {
        let parameters = RequestParameters {
            query: Some("買収 案件".to_owned()),
            company_name: Some("サンプル株式会社".to_owned()),
            article_type: Some(ArticleType::TobNews),
            date_from: Some("2024-01-01".to_owned()),
            date_to: None,
            with_raw: true,
        };
        let request = reqwest::Client::new()
            .get(NEWS_URL)
            .query(&query_parameters(&parameters, 4))
            .build()
            .expect("request must be buildable");
        let url = request.url();

        let pairs: Vec<_> = url.query_pairs().collect();
        assert!(pairs.contains(&("page".into(), "4".into())));
        assert!(pairs.contains(&("q".into(), "買収 案件".into())));
        assert!(pairs.contains(&("article_type".into(), "tob_news".into())));
        assert!(pairs.contains(&("with_raw".into(), "true".into())));
        assert!(!url.as_str().contains(' '));
    }
}
