use anyhow::Context;

use crate::news::{get, search};

#[derive(clap::Args, Debug)]
pub struct NewsArgs {
    #[command(subcommand)]
    subcommand: NewsSubCommand,
    #[arg(
        long,
        short,
        value_name = "API_KEY",
        help = "API キーを明示的に指定する場合"
    )]
    key: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum NewsSubCommand {
    #[command(
        about = "キーワード、関連企業、記事種別、公開日の範囲でニュース記事を検索し、結果を JSON 配列で出力します。\n\
                 条件を省略した場合は全記事が対象です。複数指定した場合は、すべての条件で絞り込みます。",
        after_help = "使用例:\n  \
                      maonline news search \"M&A\"\n  \
                      maonline news search トヨタ --article-type tob_news\n  \
                      maonline news search --company ストライク --date-from 2025-01-01\n  \
                      maonline news search --date-from 2025-01-01 --date-to 2025-12-31",
        visible_alias = "s"
    )]
    Search(search::command::SearchNewsArgs),
    #[command(
        about = "検索結果に含まれる記事IDを指定し、ニュース記事を1件取得して JSON で出力します。",
        after_help = "使用例:\n  maonline news get summary20251011",
        visible_alias = "g"
    )]
    Get(get::command::GetNewsArgs),
}

pub async fn run(args: NewsArgs) -> anyhow::Result<()> {
    let api_key = crate::app_config::resolve_api_key(args.key.as_deref())
        .context("failed to resolve M&A Online API key")?;

    match args.subcommand {
        NewsSubCommand::Search(command) => search::command::run(command, &api_key).await,
        NewsSubCommand::Get(command) => get::command::run(command, &api_key).await,
    }
}
