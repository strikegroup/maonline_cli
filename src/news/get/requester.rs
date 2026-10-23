// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use crate::api_client::ApiClient;

use crate::news::{article::NewsArticle, raw_response::RawNewsArticle};

#[cfg(test)]
const NEWS_URL: &str = "https://maonline.jp/research/api/v1/news/";

pub(super) async fn request_news_article(
    article_id: &str,
    api_key: &str,
) -> anyhow::Result<NewsArticle> {
    let client = ApiClient::new(api_key)?;
    request_with_client(article_id, &client).await
}

async fn request_with_client(article_id: &str, client: &ApiClient) -> anyhow::Result<NewsArticle> {
    let url = article_url(client.endpoint("news/"), article_id)?;
    let raw_article: RawNewsArticle = client.get_json(&url, &[]).await?;
    Ok(NewsArticle::from(raw_article))
}

fn article_url(mut url: reqwest::Url, article_id: &str) -> anyhow::Result<reqwest::Url> {
    let article_id = article_id.trim();
    if article_id.is_empty() {
        return Err(anyhow::anyhow!("article id must not be empty"));
    }

    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("news API URL cannot be a base URL"))?
        .pop_if_empty()
        .push(article_id);

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::{NEWS_URL, article_url};

    #[test]
    fn url_encodes_url_style_article_id_as_one_path_parameter() {
        let url = article_url(NEWS_URL.parse().unwrap(), "news/20240520_sample")
            .expect("URL must be buildable");

        assert_eq!(
            url.as_str(),
            "https://maonline.jp/research/api/v1/news/news%2F20240520_sample"
        );
    }

    #[test]
    fn rejects_empty_article_id() {
        assert!(article_url(NEWS_URL.parse().unwrap(), "  ").is_err());
    }
}
