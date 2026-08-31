use anyhow::Context;

use crate::companies::requester::{RequestParameters, request_company_info};

#[derive(Debug, clap::Args)]
pub struct CompaniesArgs {
    #[arg(
        long,
        short,
        value_name = "SEC_CODE",
        help = "証券コードで完全一致検索します"
    )]
    sec_code: Option<String>,
    #[arg(
        long = "name",
        short = 'n',
        value_name = "COMPANY_NAME",
        help = "企業名またはカナで曖昧検索します"
    )]
    company_name: Option<String>,
    #[arg(
        long,
        short,
        value_name = "EDINET_CODE",
        help = "EDINETコードで完全一致検索します"
    )]
    edinet_code: Option<String>,
    #[arg(
        long,
        short,
        value_name = "MARKET",
        help = "市場区分で絞り込みます（例: 東証プライム）"
    )]
    market: Option<String>,
    #[arg(
        long,
        short,
        value_name = "API_KEY",
        help = "API キーを明示的に指定する場合"
    )]
    key: Option<String>,
}

pub async fn run(args: CompaniesArgs) -> anyhow::Result<()> {
    let parameters: RequestParameters = RequestParameters {
        sec_code: args.sec_code,
        company_name: args.company_name,
        edinet_code: args.edinet_code,
        market: args.market,
    };

    if parameters.sec_code.is_none()
        && parameters.company_name.is_none()
        && parameters.edinet_code.is_none()
        && parameters.market.is_none()
    {
        return Err(anyhow::anyhow!("at least one argument must be specified"));
    }

    let api_key = crate::app_config::resolve_api_key(args.key.as_deref())
        .context("failed to resolve M&A Online API key")?;
    let company_infos = request_company_info(&parameters, &api_key)
        .await
        .context("failed to request company info")?;

    println!(
        "{}",
        serde_json::to_string_pretty(&company_infos)
            .context("failed to serialize company info to JSON")?
    );

    Ok(())
}
