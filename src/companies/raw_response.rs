use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct RawResponse {
    pub(super) data: Vec<RawCompanyInfo>,
    pub(super) meta: RawMeta,
}

#[derive(Debug, Deserialize)]
pub(super) struct RawCompanyInfo {
    pub(super) edsyokencode: String,
    pub(super) edcode: String,
    pub(super) edname: String,
    pub(super) market: String,
    pub(super) address: String,
    pub(super) homepage: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct RawMeta {
    pub(super) next_page: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::RawResponse;

    #[test]
    fn deserializes_company_response() {
        let response: RawResponse = serde_json::from_str(
            r#"{
                "data": [{
                    "edsyokencode": "1234",
                    "edcode": "E01234",
                    "edname": "株式会社サンプル",
                    "market": "東証プライム",
                    "address": "東京都",
                    "homepage": "https://example.com"
                }],
                "meta": {"current_page": 1, "next_page": null, "total_count": 1}
            }"#,
        )
        .expect("response must be deserializable");

        assert_eq!(response.data.len(), 1);
        assert!(response.meta.next_page.is_none());
    }
}
