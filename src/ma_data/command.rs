use anyhow::Context;

#[derive(clap::Args, Debug)]
pub struct MaDataArgs {
    #[arg(
        long = "company",
        short = 'c',
        value_name = "COMPANY_NAME",
        help = "買い手・対象・売り手の企業名で曖昧検索します"
    )]
    company_name: Option<String>,
    #[arg(long, value_name = "YEN", help = "買収金額の下限を円単位で指定します")]
    min_price: Option<u64>,
    #[arg(long, value_name = "YEN", help = "買収金額の上限を円単位で指定します")]
    max_price: Option<u64>,
    #[arg(
        long,
        short,
        value_name = "CODE",
        help = "M&Aスキームコードで絞り込みます"
    )]
    scheme: Option<String>,
    #[arg(
        long,
        short,
        value_name = "CATEGORY",
        help = "国内外区分で絞り込みます（例: IN-IN、IN-OUT）"
    )]
    inout: Option<String>,
    #[arg(
        long,
        short,
        value_name = "AREA",
        help = "対象エリア名で絞り込みます（例: 東京都）"
    )]
    area: Option<String>,
    #[arg(
        long,
        short,
        value_name = "API_KEY",
        help = "API キーを明示的に指定する場合"
    )]
    key: Option<String>,
}

pub async fn run(args: MaDataArgs) -> anyhow::Result<()> {
    let api_key = crate::app_config::resolve_api_key(args.key.as_deref())
        .context("failed to resolve M&A Online API key")?;

    let parameters = crate::ma_data::requester::RequestParameters {
        company_name: args.company_name,
        min_price: args.min_price,
        max_price: args.max_price,
        scheme: args.scheme,
        inout: args.inout,
        area: args.area,
    };

    let ma_data = crate::ma_data::requester::request_ma_data(&parameters, &api_key)
        .await
        .context("failed to request M&A data")?;

    println!(
        "{}",
        serde_json::to_string_pretty(&ma_data).context("failed to serialize M&A data to JSON")?
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::MaDataArgs;

    #[derive(Debug, Parser)]
    struct TestCli {
        #[command(flatten)]
        args: MaDataArgs,
    }

    #[test]
    fn accepts_prices_over_u32_max() {
        let cli = TestCli::try_parse_from([
            "maonline",
            "--min-price",
            "10000000000",
            "--max-price",
            "20000000000",
        ])
        .expect("prices over u32::MAX must be accepted");

        assert_eq!(cli.args.min_price, Some(10_000_000_000));
        assert_eq!(cli.args.max_price, Some(20_000_000_000));
    }

    #[test]
    fn accepts_scheme_and_string_inout() {
        let cli = TestCli::try_parse_from(["maonline", "--scheme", "4", "--inout", "IN-OUT"])
            .expect("scheme and string inout must be accepted");

        assert_eq!(cli.args.scheme.as_deref(), Some("4"));
        assert_eq!(cli.args.inout.as_deref(), Some("IN-OUT"));
    }
}
