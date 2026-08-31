// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

mod app_config;
mod app_paths;
mod companies;
mod ma_data;
mod news;

use clap::{Command, CommandFactory, FromArgMatches, Parser};

const HELP_TEMPLATE: &str =
    "{before-help}{about-with-newline}\n使い方: {usage}\n\n{all-args}{after-help}";

#[derive(Parser, Debug)]
#[command(
    name = "maonline",
    version = env!("CARGO_PKG_VERSION"),
    about = "M&A Online の企業情報、M&A案件、ニュース・速報を検索するCLIツール",
    arg_required_else_help = true,
    after_help = "使用例:\n  \
                  maonline setup --key YOUR_API_KEY\n  \
                  maonline companies --name ストライク\n  \
                  maonline ma-data --company トヨタ --min-price 1000000000\n  \
                  maonline news search TOB --article-type tob_news\n\n\
                  各コマンドの詳細は `maonline <COMMAND> --help` で確認できます。"
)]
struct Cli {
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(clap::Subcommand, Debug)]
enum SubCommand {
    #[command(
        about = "企業情報を検索します",
        after_help = "複数の検索条件を指定した場合は、すべての条件で絞り込みます。\n\n\
                      使用例:\n  \
                      maonline companies --name ストライク\n  \
                      maonline companies --sec-code 7203\n  \
                      maonline companies --market 東証プライム --name トヨタ",
        visible_alias = "c"
    )]
    Companies(crate::companies::command::CompaniesArgs),
    #[command(
        about = "M&A案件を検索します",
        after_help = "複数の検索条件を指定した場合は、すべての条件で絞り込みます。\n\n\
                      使用例:\n  \
                      maonline ma-data --company 楽天\n  \
                      maonline ma-data --min-price 1000000000000\n  \
                      maonline ma-data --area 東京都 --inout IN-IN",
        visible_alias = "m"
    )]
    MaData(crate::ma_data::command::MaDataArgs),
    #[command(
        about = "ニュース・速報を検索・取得します",
        after_help = "`search` で一覧を検索し、`get` で記事IDを指定して1件取得します。\n\n\
                      使用例:\n  \
                      maonline news search ストライク\n  \
                      maonline news search --article-type tob_news\n  \
                      maonline news get summary20251011",
        visible_alias = "n"
    )]
    News(crate::news::command::NewsArgs),
    #[command(
        about = "M&A Online API の API キーを登録します",
        after_help = "登録後は、各検索コマンドで API キーを指定する必要はありません。\n\n\
                      使用例:\n  maonline setup --key YOUR_API_KEY"
    )]
    Setup(crate::app_config::SetupArgs),
    #[command(
        about = "各種状態を表示します",
        after_help = "API キーそのものは表示しません。\n\n\
                      使用例:\n  maonline status",
        visible_alias = "s"
    )]
    Status,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    let matches = cli_command().get_matches();
    let cli = Cli::from_arg_matches(&matches)?;

    match cli.subcommand {
        SubCommand::Companies(command) => crate::companies::command::run(command).await?,
        SubCommand::MaData(command) => crate::ma_data::command::run(command).await?,
        SubCommand::News(command) => crate::news::command::run(command).await?,
        SubCommand::Setup(command) => crate::app_config::run_setup(command)?,
        SubCommand::Status => print_status()?,
    }

    Ok(())
}

fn cli_command() -> Command {
    let mut command = Cli::command();
    command.build();
    localize_help(command)
}

fn localize_help(command: Command) -> Command {
    command
        .help_template(HELP_TEMPLATE)
        .subcommand_help_heading("コマンド")
        .mut_args(|arg| {
            let heading = if arg.get_index().is_some() {
                "引数"
            } else {
                "オプション"
            };
            let arg = arg.help_heading(heading);

            match arg.get_id().as_str() {
                "help" => arg.help("ヘルプを表示します"),
                "version" => arg.help("バージョンを表示します"),
                _ => arg,
            }
        })
        .mut_subcommands(|subcommand| {
            let subcommand = if subcommand.get_name() == "help" {
                subcommand.about("指定したコマンドのヘルプを表示します")
            } else {
                subcommand
            };
            localize_help(subcommand)
        })
}

fn print_status() -> anyhow::Result<()> {
    let data_dir = crate::app_paths::default_data_local_dir()?;
    let config_path = crate::app_paths::config_file_path()?;
    let api_key_status = crate::app_config::api_key_registration_status()?;

    println!("Data directory: {}", data_dir.display());
    println!("Config file: {}", config_path.display());
    println!("API key: {}", api_key_status.as_str());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::cli_command;

    fn render_help(path: &[&str]) -> String {
        let mut command = cli_command();
        let mut current = &mut command;

        for name in path {
            current = current
                .find_subcommand_mut(name)
                .unwrap_or_else(|| panic!("subcommand `{name}` must exist"));
        }

        let mut output = Vec::new();
        current
            .write_help(&mut output)
            .expect("help must be renderable");
        String::from_utf8(output).expect("help must be UTF-8")
    }

    fn help_output(args: &[&str]) -> String {
        cli_command()
            .try_get_matches_from(args)
            .expect_err("help flag must stop argument parsing")
            .to_string()
    }

    #[test]
    fn root_help_is_concise_and_guides_command_discovery() {
        let help = render_help(&[]);

        assert!(help.contains("M&A Online の企業情報、M&A案件、ニュース・速報を検索するCLIツール"));
        assert!(help.contains("maonline setup --key YOUR_API_KEY"));
        assert!(help.contains("maonline <COMMAND> --help"));
        assert!(help.contains("企業情報を検索します"));
        assert!(help.contains("M&A案件を検索します"));
        assert!(help.contains("ニュース・速報を検索・取得します"));
        assert!(!help.contains("検索条件を1つ以上指定してください"));
        assert!(help.contains("使い方: maonline <COMMAND>"));
        assert!(help.contains("コマンド:"));
        assert!(help.contains("オプション:"));
        assert!(!help.contains("Print help"));
    }

    #[test]
    fn every_command_has_a_usage_example() {
        for (path, example) in [
            (&["companies"][..], "maonline companies --name ストライク"),
            (
                &["ma-data"][..],
                "maonline ma-data --area 東京都 --inout IN-IN",
            ),
            (&["news"][..], "maonline news search ストライク"),
            (&["news", "search"][..], "maonline news search \"M&A\""),
            (&["news", "get"][..], "maonline news get summary20251011"),
            (&["setup"][..], "maonline setup --key YOUR_API_KEY"),
            (&["status"][..], "maonline status"),
        ] {
            let help = render_help(path);
            assert!(
                help.contains(example),
                "help for `{}` must contain `{example}`",
                path.join(" ")
            );
        }
    }

    #[test]
    fn subcommand_help_keeps_operational_details_out_of_root_help() {
        let root_help = render_help(&[]);
        let companies_help = render_help(&["companies"]);

        assert!(!root_help.contains("複数の検索条件を指定した場合"));
        assert!(companies_help.contains("複数の検索条件を指定した場合"));
    }

    #[test]
    fn api_key_options_use_consistent_help() {
        for path in [&["companies"][..], &["ma-data"][..], &["news"][..]] {
            let help = render_help(path);
            assert!(help.contains("API キーを明示的に指定する場合"));
        }
    }

    #[test]
    fn short_and_long_help_flags_have_identical_output() {
        for command in [
            &[][..],
            &["companies"][..],
            &["ma-data"][..],
            &["news"][..],
            &["news", "search"][..],
            &["news", "get"][..],
            &["setup"][..],
            &["status"][..],
        ] {
            let mut short_args = vec!["maonline"];
            short_args.extend_from_slice(command);
            short_args.push("-h");

            let mut long_args = vec!["maonline"];
            long_args.extend_from_slice(command);
            long_args.push("--help");

            assert_eq!(
                help_output(&short_args),
                help_output(&long_args),
                "help output differs for `{}`",
                command.join(" ")
            );
        }
    }
}
