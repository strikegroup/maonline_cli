// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use crate::api_client::ApiClient;

use super::{company_info::CompanyInfo, raw_response::RawResponse};

#[cfg(test)]
const COMPANIES_URL: &str = "https://maonline.jp/research/api/v1/companies/";

pub(super) struct RequestParameters {
    pub(super) pagination: crate::pagination::Pagination,
    /// 証券コード
    pub(super) sec_code: Option<String>,
    /// EDINET コード
    pub(super) edinet_code: Option<String>,
    /// 会社名
    pub(super) company_name: Option<String>,
    /// 東証市場区分
    pub(super) market: Option<String>,
}

pub(super) async fn request_company_info(
    parameters: &RequestParameters,
    api_key: &str,
) -> anyhow::Result<Vec<CompanyInfo>> {
    let client = ApiClient::new(api_key)?;
    request_with_client(parameters, &client).await
}

async fn request_with_client(
    parameters: &RequestParameters,
    client: &ApiClient,
) -> anyhow::Result<Vec<CompanyInfo>> {
    let data = client
        .get_pages::<RawResponse, _>(
            "companies/",
            parameters.pagination,
            |page| query_parameters(parameters, page),
            |response| (response.data, response.meta.next_page),
        )
        .await?;
    Ok(data.into_iter().map(CompanyInfo::from).collect())
}

fn query_parameters(parameters: &RequestParameters, page: u32) -> Vec<(&'static str, String)> {
    let mut query = vec![
        ("page", page.to_string()),
        ("per_page", parameters.pagination.per_page().to_string()),
    ];

    if let Some(value) = &parameters.sec_code {
        query.push(("syokencode", value.clone()));
    }
    if let Some(value) = &parameters.edinet_code {
        query.push(("edcode", value.clone()));
    }
    if let Some(value) = &parameters.company_name {
        query.push(("name", value.clone()));
    }
    if let Some(value) = &parameters.market {
        query.push(("market", value.clone()));
    }

    query
}

#[cfg(test)]
mod tests {
    use super::{COMPANIES_URL, RequestParameters, query_parameters};

    #[test]
    fn builds_encoded_company_request_url() {
        let parameters = RequestParameters {
            pagination: Default::default(),
            sec_code: Some("1234".to_owned()),
            edinet_code: None,
            company_name: Some("サンプル 株式会社".to_owned()),
            market: None,
        };
        let request = reqwest::Client::new()
            .get(COMPANIES_URL)
            .query(&query_parameters(&parameters, 2))
            .build()
            .expect("request must be buildable");
        let url = request.url();

        let pairs: Vec<_> = url.query_pairs().collect();
        assert!(pairs.contains(&("page".into(), "2".into())));
        assert!(pairs.contains(&("per_page".into(), "200".into())));
        assert!(pairs.contains(&("name".into(), "サンプル 株式会社".into())));
        assert!(!url.as_str().contains(' '));
    }
    #[tokio::test]
    async fn fetches_all_pages_with_filters_and_converts_company_models() {
        use serde_json::json;
        use wiremock::matchers::{header, method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        for (page, next, name) in [(1, Some(3), "企業A"), (3, None, "企業B")] {
            Mock::given(method("GET")).and(path("/companies/"))
                .and(header("authorization", "Bearer test-key"))
                .and(query_param("page", page.to_string()))
                .and(query_param("per_page", "200"))
                .and(query_param("name", "企業 & 検索"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "data": [{"edsyokencode":"1234","edcode":"E01234","edname":name,
                        "market":"東証プライム","address":"東京都","homepage":"https://example.com"}],
                    "meta":{"next_page":next}
                }))).expect(1).mount(&server).await;
        }
        let client = crate::api_client::ApiClient::for_test(&format!("{}/", server.uri()));
        let parameters = RequestParameters {
            pagination: Default::default(),
            sec_code: None,
            edinet_code: None,
            company_name: Some("企業 & 検索".into()),
            market: None,
        };
        let result = super::request_with_client(&parameters, &client)
            .await
            .unwrap();
        let output = serde_json::to_value(result).unwrap();
        assert_eq!(output[0]["company_name"], "企業A");
        assert_eq!(output[1]["company_name"], "企業B");
    }
    #[tokio::test]
    async fn fetches_only_the_selected_page_including_partial_and_empty_pages() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        for (page, count) in [(1, 2), (3, 1), (9, 0)] {
            let server = MockServer::start().await;
            let sample = serde_json::json!({
                "edsyokencode": "1234",
                "edcode": "E01234",
                "edname": "企業",
                "market": "東証",
                "address": "東京",
                "homepage": ""
            });
            let data = vec![sample; count];
            Mock::given(method("GET"))
                .and(path("/companies/"))
                .and(query_param("page", page.to_string()))
                .and(query_param("per_page", "2"))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": data, "meta": {"next_page": page + 1}
                })))
                .expect(1)
                .mount(&server)
                .await;
            let client = crate::api_client::ApiClient::for_test(&format!("{}/", server.uri()));
            let parameters = RequestParameters {
                pagination: crate::pagination::Pagination { limit: 2, page },
                sec_code: None,
                edinet_code: None,
                company_name: Some("企業".into()),
                market: None,
            };
            let result = super::request_with_client(&parameters, &client)
                .await
                .unwrap();
            assert_eq!(result.len(), count);
            assert_eq!(server.received_requests().await.unwrap().len(), 1);
        }
    }
}
