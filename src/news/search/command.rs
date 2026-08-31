// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;

use crate::news::{
    article_type::ArticleType,
    search::requester::{RequestParameters, request_news},
};

#[derive(clap::Args, Debug)]
pub(in crate::news) struct SearchNewsArgs {
    #[arg(
        value_name = "QUERY",
        help = "タイトル、本文、タグを対象にキーワード検索します"
    )]
    query: Option<String>,
    #[arg(
        long = "company",
        short = 'c',
        value_name = "COMPANY_NAME",
        help = "関連企業名で絞り込みます"
    )]
    company_name: Option<String>,
    #[arg(
        long,
        short,
        value_name = "TYPE",
        help = "記事種別で絞り込みます（news、shareholdings、tob_news）",
        hide_possible_values = true
    )]
    article_type: Option<CliArticleType>,
    #[arg(
        long,
        short = 'f',
        value_name = "YYYY-MM-DD",
        help = "この日以降に公開された記事へ絞り込みます"
    )]
    date_from: Option<String>,
    #[arg(
        long,
        short = 't',
        value_name = "YYYY-MM-DD",
        help = "この日以前に公開された記事へ絞り込みます"
    )]
    date_to: Option<String>,
    #[arg(long, short, help = "解析前の本文を `body_raw` として出力します")]
    with_raw: bool,
}

#[derive(clap::ValueEnum, Debug, Clone)]
enum CliArticleType {
    /// ニュース
    #[value(name = "news")]
    News,
    /// 大量保有速報
    #[value(name = "shareholdings")]
    ShareHoldings,
    /// TOB速報
    #[value(name = "tob_news")]
    TobNews,
}

impl From<CliArticleType> for ArticleType {
    fn from(value: CliArticleType) -> Self {
        match value {
            CliArticleType::News => Self::News,
            CliArticleType::ShareHoldings => Self::ShareHoldings,
            CliArticleType::TobNews => Self::TobNews,
        }
    }
}

pub(in crate::news) async fn run(args: SearchNewsArgs, api_key: &str) -> anyhow::Result<()> {
    let parameters = RequestParameters {
        query: args.query,
        company_name: args.company_name,
        article_type: args.article_type.map(ArticleType::from),
        date_from: args.date_from,
        date_to: args.date_to,
        with_raw: args.with_raw,
    };
    let articles = request_news(&parameters, api_key)
        .await
        .context("failed to search news")?;

    println!(
        "{}",
        serde_json::to_string_pretty(&articles).context("failed to serialize news to JSON")?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::news::article_type::ArticleType;

    use super::CliArticleType;

    #[test]
    fn converts_cli_article_types_to_internal_types() {
        assert_eq!(ArticleType::from(CliArticleType::News), ArticleType::News);
        assert_eq!(
            ArticleType::from(CliArticleType::ShareHoldings),
            ArticleType::ShareHoldings
        );
        assert_eq!(
            ArticleType::from(CliArticleType::TobNews),
            ArticleType::TobNews
        );
    }
}
