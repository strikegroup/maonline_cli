// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use crate::api_client::ApiClient;

use crate::news::{
    article::NewsArticle, article_type::ArticleType, raw_response::RawSearchResponse,
};

#[cfg(test)]
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
    let client = ApiClient::new(api_key)?;
    request_with_client(parameters, &client).await
}

async fn request_with_client(
    parameters: &RequestParameters,
    client: &ApiClient,
) -> anyhow::Result<Vec<NewsArticle>> {
    let data = client
        .get_pages::<RawSearchResponse, _>(
            "news/",
            |page| query_parameters(parameters, page),
            |response| (response.data, response.meta.next_page),
        )
        .await?;
    Ok(data.into_iter().map(NewsArticle::from).collect())
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
    #[tokio::test]
    async fn accepts_an_empty_final_page() {
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(wiremock::matchers::path("/news/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [], "meta": {"next_page": null}
            })))
            .expect(1)
            .mount(&server)
            .await;
        let client = crate::api_client::ApiClient::for_test(&format!("{}/", server.uri()));
        let parameters = RequestParameters {
            query: None,
            company_name: None,
            article_type: None,
            date_from: None,
            date_to: None,
            with_raw: false,
        };
        assert!(
            super::request_with_client(&parameters, &client)
                .await
                .unwrap()
                .is_empty()
        );
    }
}
