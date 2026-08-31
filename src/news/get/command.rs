use anyhow::Context;

use crate::news::get::requester::request_news_article;

#[derive(clap::Args, Debug)]
pub(in crate::news) struct GetNewsArgs {
    #[arg(value_name = "ARTICLE_ID", help = "取得する記事の記事ID")]
    article_id: String,
}

pub(in crate::news) async fn run(args: GetNewsArgs, api_key: &str) -> anyhow::Result<()> {
    let article = request_news_article(&args.article_id, api_key)
        .await
        .context("failed to get news article")?;

    println!(
        "{}",
        serde_json::to_string_pretty(&article)
            .context("failed to serialize news article to JSON")?
    );

    Ok(())
}
