// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

mod api_client;
mod app_config;
mod app_paths;
mod input;
mod logging;
mod ma_data;
mod pagination;

use clap::{Command, CommandFactory, FromArgMatches, Parser};

const HELP_TEMPLATE: &str =
    "{before-help}{about-with-newline}\n使い方: {usage}\n\n{all-args}{after-help}";

#[derive(Parser, Debug)]
#[command(
    name = "maonline",
    version = env!("CARGO_PKG_VERSION"),
    about = "M&A Online のM&A案件を検索するCLIツール",
    arg_required_else_help = true,
    after_help = "使用例:\n  \
                  maonline setup --key YOUR_API_KEY\n  \
                  maonline ma-data --company トヨタ --min-price 1000000000\n\n\
                  各コマンドの詳細は `maonline <COMMAND> --help` で確認できます。"
)]
struct Cli {
    #[arg(short = 'v', long, global = true, action = clap::ArgAction::Count,
        conflicts_with = "quiet", help = "詳細ログを表示します（-vv で最も詳細）")]
    verbose: u8,
    #[arg(short = 'q', long, global = true, action = clap::ArgAction::Count,
        conflicts_with = "verbose", help = "ログを減らします（-qqq で非表示）")]
    quiet: u8,
    #[command(subcommand)]
    subcommand: SubCommand,
}

#[derive(clap::Subcommand, Debug)]
enum SubCommand {
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
        about = "M&A Online API の API キーを登録します",
        after_help = "利用特約（https://maonline.jp/pages/policy_researchkit）を確認し、同意すると API キーを保存します。\n\
                      登録後は、ma-data コマンドで API キーを指定する必要はありません。\n\n\
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
async fn main() -> std::process::ExitCode {
    let _ = dotenvy::dotenv();
    let matches = cli_command().get_matches();
    let cli = Cli::from_arg_matches(&matches).expect("clap validated the arguments");
    logging::init(cli.verbose, cli.quiet);
    match run(cli).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!("{error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    let command = match &cli.subcommand {
        SubCommand::MaData(_) => "ma-data",
        SubCommand::Setup(_) => "setup",
        SubCommand::Status => "status",
    };
    tracing::debug!(command, "command started");
    match cli.subcommand {
        SubCommand::MaData(command) => crate::ma_data::command::run(command).await?,
        SubCommand::Setup(command) => crate::app_config::run_setup(command)?,
        SubCommand::Status => print_status()?,
    }

    tracing::debug!(command, "command completed");
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

        assert!(help.contains("M&A Online のM&A案件を検索するCLIツール"));
        assert!(help.contains("maonline setup --key YOUR_API_KEY"));
        assert!(help.contains("maonline <COMMAND> --help"));
        assert!(help.contains("M&A案件を検索します"));
        assert!(!help.contains("検索条件を1つ以上指定してください"));
        assert!(help.contains("使い方: maonline [OPTIONS] <COMMAND>"));
        assert!(help.contains("コマンド:"));
        assert!(help.contains("オプション:"));
        assert!(!help.contains("Print help"));
    }

    #[test]
    fn every_command_has_a_usage_example() {
        for (path, example) in [
            (
                &["ma-data"][..],
                "maonline ma-data --area 東京都 --inout IN-IN",
            ),
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
        let ma_data_help = render_help(&["ma-data"]);

        assert!(!root_help.contains("複数の検索条件を指定した場合"));
        assert!(ma_data_help.contains("複数の検索条件を指定した場合"));
    }

    #[test]
    fn ma_data_help_explains_api_key_option() {
        let help = render_help(&["ma-data"]);
        assert!(help.contains("API キーを明示的に指定する場合"));
    }

    #[test]
    fn short_and_long_help_flags_have_identical_output() {
        for command in [&[][..], &["ma-data"][..], &["setup"][..], &["status"][..]] {
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
    #[test]
    fn ma_data_accepts_pagination_and_short_options() {
        for prefix in [vec!["maonline", "ma-data"], vec!["maonline", "m"]] {
            for options in [
                vec!["--limit", "20", "--page", "2"],
                vec!["-l", "200", "-p", "3"],
                vec!["--limit", "0"],
            ] {
                let args: Vec<_> = prefix.iter().chain(&options).copied().collect();
                cli_command().try_get_matches_from(args).unwrap();
            }
        }
    }
}
