// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct RawSearchResponse {
    pub(super) data: Vec<RawNewsArticle>,
    pub(super) meta: RawMeta,
}

#[derive(Debug, Deserialize)]
pub(super) struct RawNewsArticle {
    pub(super) id: String,
    pub(super) title: String,
    pub(super) body_html: String,
    pub(super) published_at: String,
    pub(super) related_codes: Vec<String>,
    #[serde(default)]
    pub(super) body_raw: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct RawMeta {
    pub(super) next_page: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::{RawNewsArticle, RawSearchResponse};

    #[test]
    fn deserializes_search_response_without_optional_fields() {
        let response: RawSearchResponse = serde_json::from_str(
            r#"{
                "data": [{
                    "id": "news/20240520_sample",
                    "title": "sample",
                    "body_html": "<p>body</p>",
                    "published_at": "2024-05-20T10:00:00+09:00",
                    "related_codes": ["1234"]
                }],
                "meta": {"current_page": 1, "total_count": 1}
            }"#,
        )
        .expect("response must be deserializable");

        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].body_raw.is_none());
        assert!(response.meta.next_page.is_none());
    }

    #[test]
    fn deserializes_detail_response() {
        let article: RawNewsArticle = serde_json::from_str(
            r#"{
                "id": "news/20240520_sample",
                "title": "sample",
                "body_html": "<p>body</p>",
                "published_at": "2024-05-20T10:00:00+09:00",
                "related_codes": ["1234"],
                "body_raw": "raw"
            }"#,
        )
        .expect("article must be deserializable");

        assert_eq!(article.body_raw.as_deref(), Some("raw"));
    }
}
