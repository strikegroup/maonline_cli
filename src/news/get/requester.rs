use anyhow::Context;

use crate::news::{article::NewsArticle, raw_response::RawNewsArticle};

const NEWS_URL: &str = "https://maonline.jp/research/api/v1/news/";

pub(super) async fn request_news_article(
    article_id: &str,
    api_key: &str,
) -> anyhow::Result<NewsArticle> {
    let url = article_url(article_id)?;
    let client = reqwest::Client::new();
    let response = client
        .get(url.clone())
        .bearer_auth(api_key)
        .send()
        .await
        .with_context(|| format!("failed to send request to {url}"))?;
    let status = response.status();

    if !status.is_success() {
        return Err(anyhow::anyhow!(
            "request to {} failed with status {}",
            url,
            status
        ));
    }

    let body = response
        .bytes()
        .await
        .with_context(|| format!("failed to read response body from {url}"))?;

    let raw_article: RawNewsArticle = serde_json::from_slice(&body)
        .with_context(|| format!("failed to parse response from {url}"))?;

    Ok(NewsArticle::from(raw_article))
}

fn article_url(article_id: &str) -> anyhow::Result<reqwest::Url> {
    let article_id = article_id.trim();
    if article_id.is_empty() {
        return Err(anyhow::anyhow!("article id must not be empty"));
    }

    let mut url = reqwest::Url::parse(NEWS_URL).context("invalid news API URL")?;
    url.path_segments_mut()
        .map_err(|_| anyhow::anyhow!("news API URL cannot be a base URL"))?
        .pop_if_empty()
        .push(article_id);

    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::article_url;

    #[test]
    fn url_encodes_url_style_article_id_as_one_path_parameter() {
        let url = article_url("news/20240520_sample").expect("URL must be buildable");

        assert_eq!(
            url.as_str(),
            "https://maonline.jp/research/api/v1/news/news%2F20240520_sample"
        );
    }

    #[test]
    fn rejects_empty_article_id() {
        assert!(article_url("  ").is_err());
    }
}
