// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use crate::api_client::ApiClient;

use super::{company_info::CompanyInfo, raw_response::RawResponse};

#[cfg(test)]
const COMPANIES_URL: &str = "https://maonline.jp/research/api/v1/companies/";
const PER_PAGE: u32 = 200;

pub(super) struct RequestParameters {
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
            |page| query_parameters(parameters, page),
            |response| (response.data, response.meta.next_page),
        )
        .await?;
    Ok(data.into_iter().map(CompanyInfo::from).collect())
}

fn query_parameters(parameters: &RequestParameters, page: u32) -> Vec<(&'static str, String)> {
    let mut query = vec![
        ("page", page.to_string()),
        ("per_page", PER_PAGE.to_string()),
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
}
