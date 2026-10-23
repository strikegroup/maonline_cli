// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use crate::api_client::ApiClient;

use super::{data::MaData, raw_response::RawResponse};

#[cfg(test)]
const MA_DATA_URL: &str = "https://maonline.jp/research/api/v1/ma_data/";
const PER_PAGE: u32 = 200;

pub(super) struct RequestParameters {
    pub(super) company_name: Option<String>,
    pub(super) min_price: Option<u64>,
    pub(super) max_price: Option<u64>,
    pub(super) scheme: Option<String>,
    pub(super) inout: Option<String>,
    pub(super) area: Option<String>,
}

pub(super) async fn request_ma_data(
    parameters: &RequestParameters,
    api_key: &str,
) -> anyhow::Result<Vec<MaData>> {
    let client = ApiClient::new(api_key)?;
    request_with_client(parameters, &client).await
}

async fn request_with_client(
    parameters: &RequestParameters,
    client: &ApiClient,
) -> anyhow::Result<Vec<MaData>> {
    let data = client
        .get_pages::<RawResponse, _>(
            "ma_data/",
            |page| query_parameters(parameters, page),
            |response| (response.data, response.meta.next_page),
        )
        .await?;
    Ok(data.into_iter().map(MaData::from).collect())
}

fn query_parameters(parameters: &RequestParameters, page: u32) -> Vec<(&'static str, String)> {
    let mut query = vec![
        ("page", page.to_string()),
        ("per_page", PER_PAGE.to_string()),
    ];

    if let Some(value) = &parameters.company_name {
        query.push(("company_name", value.clone()));
    }
    if let Some(value) = parameters.min_price {
        query.push(("min_price", value.to_string()));
    }
    if let Some(value) = parameters.max_price {
        query.push(("max_price", value.to_string()));
    }
    if let Some(value) = &parameters.scheme {
        query.push(("scheme", value.clone()));
    }
    if let Some(value) = &parameters.inout {
        query.push(("inout", value.clone()));
    }
    if let Some(value) = &parameters.area {
        query.push(("area", value.clone()));
    }

    query
}

#[cfg(test)]
mod tests {
    use super::{MA_DATA_URL, RequestParameters, query_parameters};

    #[test]
    fn builds_encoded_ma_data_request_url() {
        let parameters = RequestParameters {
            company_name: Some("サンプル 株式会社".to_owned()),
            min_price: Some(10_000_000_000),
            max_price: Some(20_000_000_000),
            scheme: Some("4".to_owned()),
            inout: Some("IN-OUT".to_owned()),
            area: Some("東京都".to_owned()),
        };
        let request = reqwest::Client::new()
            .get(MA_DATA_URL)
            .query(&query_parameters(&parameters, 3))
            .build()
            .expect("request must be buildable");
        let url = request.url();

        let pairs: Vec<_> = url.query_pairs().collect();
        assert!(pairs.contains(&("page".into(), "3".into())));
        assert!(pairs.contains(&("company_name".into(), "サンプル 株式会社".into())));
        assert!(pairs.contains(&("min_price".into(), "10000000000".into())));
        assert!(pairs.contains(&("max_price".into(), "20000000000".into())));
        assert!(pairs.contains(&("scheme".into(), "4".into())));
        assert!(pairs.contains(&("inout".into(), "IN-OUT".into())));
        assert!(pairs.contains(&("area".into(), "東京都".into())));
        assert!(!pairs.iter().any(|(key, _)| key == "schema"));
        assert!(!url.as_str().contains(' '));
    }
}
