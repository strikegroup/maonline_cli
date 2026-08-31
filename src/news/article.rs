use serde::Serialize;

use super::raw_response::RawNewsArticle;

#[derive(Debug, Serialize)]
pub(super) struct NewsArticle {
    id: String,
    title: String,
    body_html: String,
    published_at: String,
    related_codes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    body_raw: Option<String>,
}

impl From<RawNewsArticle> for NewsArticle {
    fn from(raw: RawNewsArticle) -> Self {
        Self {
            id: raw.id,
            title: raw.title,
            body_html: raw.body_html,
            published_at: raw.published_at,
            related_codes: raw.related_codes,
            body_raw: raw.body_raw,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{NewsArticle, RawNewsArticle};

    #[test]
    fn converts_raw_news_article() {
        let article = NewsArticle::from(RawNewsArticle {
            id: "news/sample".to_owned(),
            title: "サンプル".to_owned(),
            body_html: "<p>本文</p>".to_owned(),
            published_at: "2024-05-20T10:00:00+09:00".to_owned(),
            related_codes: vec!["1234".to_owned()],
            body_raw: None,
        });

        assert_eq!(article.id, "news/sample");
        assert_eq!(article.related_codes, ["1234"]);
        assert!(article.body_raw.is_none());
    }
}
