// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use anyhow::Context;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};

const RESEARCH_KIT_POLICY_URL: &str = "https://maonline.jp/pages/policy_researchkit";

#[derive(clap::Args, Debug)]
pub struct SetupArgs {
    #[arg(long, value_name = "API_KEY", help = "M&A Online API キー")]
    pub key: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    maonline_api_key: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ApiKeyRegistrationStatus {
    Registered,
    Missing,
    Invalid,
}

impl ApiKeyRegistrationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Registered => "registered",
            Self::Missing => "missing",
            Self::Invalid => "invalid",
        }
    }
}

/// M&A Online API キーを `config.toml` に保存する。
pub fn save_api_key(api_key: &str) -> anyhow::Result<std::path::PathBuf> {
    let api_key = api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow::anyhow!("api key must not be empty"));
    }

    let config_dir = crate::app_paths::config_dir()?;
    std::fs::create_dir_all(&config_dir).with_context(|| {
        format!(
            "failed to create configuration directory {}",
            config_dir.display()
        )
    })?;

    let config_path = crate::app_paths::config_file_path()?;
    let content = toml::to_string(&ConfigFile {
        maonline_api_key: api_key.to_owned(),
    })
    .context("failed to serialize config file")?;
    std::fs::write(&config_path, content)
        .with_context(|| format!("failed to write config file {}", config_path.display()))?;

    Ok(config_path)
}

/// 保存済み M&A Online API キーの登録状態を返す。
pub fn api_key_registration_status() -> anyhow::Result<ApiKeyRegistrationStatus> {
    let config_path = crate::app_paths::config_file_path()?;
    let content = match std::fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ApiKeyRegistrationStatus::Missing);
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("failed to read config file {}", config_path.display()));
        }
    };

    let Ok(config) = toml::from_str::<ConfigFile>(&content) else {
        return Ok(ApiKeyRegistrationStatus::Invalid);
    };
    if config.maonline_api_key.trim().is_empty() {
        return Ok(ApiKeyRegistrationStatus::Invalid);
    }

    Ok(ApiKeyRegistrationStatus::Registered)
}

/// `config.toml` から M&A Online API キーを読み込む。
///
/// 設定ファイルが存在しない場合や値が空の場合はエラーを返す。
pub fn load_api_key() -> anyhow::Result<String> {
    let config_path = crate::app_paths::config_file_path()?;
    let content = std::fs::read_to_string(&config_path).with_context(|| {
        format!(
            "failed to read config file {}. run `cargo run -- setup --key <MAONLINE_API_KEY>` first or pass `--key`",
            config_path.display()
        )
    })?;
    let config: ConfigFile = toml::from_str(&content)
        .with_context(|| format!("failed to parse config file {}", config_path.display()))?;
    let api_key = config.maonline_api_key.trim();
    if api_key.is_empty() {
        return Err(anyhow::anyhow!(
            "maonline_api_key is empty in {}",
            config_path.display()
        ));
    }

    Ok(api_key.to_owned())
}

/// 実行時 API キーを解決する。
///
/// - `cli_api_key` が指定されていればそれを優先
/// - 未指定なら `config.toml` の保存済みキーを利用
pub fn resolve_api_key(cli_api_key: Option<&str>) -> anyhow::Result<String> {
    if let Some(api_key) = cli_api_key {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            return Err(anyhow::anyhow!("api key must not be empty"));
        }
        return Ok(api_key.to_owned());
    }

    load_api_key()
}

pub fn run_setup(args: SetupArgs) -> anyhow::Result<()> {
    confirm_research_kit_policy(&mut std::io::stdin().lock(), &mut std::io::stderr().lock())?;
    let path = save_api_key(&args.key)?;
    println!("Saved M&A Online API key to {}", path.display());
    Ok(())
}

fn confirm_research_kit_policy(
    input: &mut impl BufRead,
    output: &mut impl Write,
) -> anyhow::Result<()> {
    writeln!(
        output,
        "You must agree to the M&A Online Research Kit Terms of Use to use M&A Online CLI."
    )?;
    let url = RESEARCH_KIT_POLICY_URL.cyan().underline();
    writeln!(
        output,
        "Please review the terms at the following URL:\n{url}\n"
    )?;

    loop {
        write!(output, "Do you agree to the terms? [y/N]: ")?;
        output.flush()?;

        let mut answer = String::new();
        input
            .read_line(&mut answer)
            .context("failed to read policy confirmation")?;
        match answer.trim().to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(()),
            "" | "n" | "no" => {
                anyhow::bail!("Setup cancelled because you did not agree to the terms.")
            }
            _ => writeln!(output, "Please enter y (agree) or n (decline).")?,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn policy_confirmation_requires_explicit_consent() {
        for answer in ["y\n", "yes\n", "Y\n", "YES\n", " yes \r\n"] {
            let mut output = Vec::new();
            confirm_research_kit_policy(&mut answer.as_bytes(), &mut output).unwrap();
            let output = String::from_utf8(output).unwrap();
            assert!(output.contains(&RESEARCH_KIT_POLICY_URL.cyan().underline().to_string()));
            assert!(output.contains("Do you agree to the terms? [y/N]: "));
        }
    }

    #[test]
    fn policy_confirmation_rejects_decline_empty_input_and_eof() {
        for answer in ["n\n", "no\n", "N\n", "NO\n", "\n", ""] {
            let error =
                confirm_research_kit_policy(&mut answer.as_bytes(), &mut Vec::new()).unwrap_err();
            assert!(error.to_string().contains("Setup cancelled"));
        }
    }

    #[test]
    fn policy_confirmation_reprompts_after_invalid_input() {
        let mut output = Vec::new();
        confirm_research_kit_policy(&mut &b"maybe\ny\n"[..], &mut output).unwrap();
        let output = String::from_utf8(output).unwrap();
        assert_eq!(
            output.matches("Do you agree to the terms? [y/N]: ").count(),
            2
        );
    }
}
