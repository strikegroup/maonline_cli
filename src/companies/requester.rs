use anyhow::Context;

use super::{company_info::CompanyInfo, raw_response::RawResponse};

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
    let client = reqwest::Client::new();
    let response = request_raw_company_info(&client, parameters, api_key, 1).await?;

    Ok(response.data.into_iter().map(CompanyInfo::from).collect())
}

/// /companies から企業情報を取得する。ページネーションは再帰的に解決する。
#[async_recursion::async_recursion]
async fn request_raw_company_info(
    client: &reqwest::Client,
    parameters: &RequestParameters,
    api_key: &str,
    page: u32,
) -> anyhow::Result<RawResponse> {
    let request = client
        .get(COMPANIES_URL)
        .query(&query_parameters(parameters, page))
        .bearer_auth(api_key)
        .build()
        .context("failed to build companies API request")?;
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
    let mut raw_response: RawResponse = serde_json::from_slice(&body)
        .with_context(|| format!("failed to parse response from {url}"))?;

    if let Some(next_page) = raw_response.meta.next_page {
        let next_response =
            request_raw_company_info(client, parameters, api_key, next_page).await?;
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
