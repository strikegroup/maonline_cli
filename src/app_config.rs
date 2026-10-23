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
            "failed to read config file {}. run `maonline setup --key <MAONLINE_API_KEY>`, set `MAONLINE_API_KEY`, or pass `--key`",
            config_path.display()
        )
    })?;
    // TOML parser errors include source excerpts, which may contain the API key.
    let config = parse_config(&content)
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

fn parse_config(content: &str) -> anyhow::Result<ConfigFile> {
    toml::from_str(content).map_err(|_| anyhow::anyhow!("invalid configuration TOML"))
}

/// `--key` → `MAONLINE_API_KEY` → `config.toml` の順でAPIキーを解決する。
pub fn resolve_api_key(cli_api_key: Option<&str>) -> anyhow::Result<String> {
    let env_api_key = std::env::var("MAONLINE_API_KEY").ok();
    resolve_api_key_from_sources(cli_api_key, env_api_key.as_deref(), load_api_key)
}

fn resolve_api_key_from_sources(
    cli_api_key: Option<&str>,
    env_api_key: Option<&str>,
    load_saved_key: impl FnOnce() -> anyhow::Result<String>,
) -> anyhow::Result<String> {
    if let Some(api_key) = cli_api_key {
        let api_key = api_key.trim();
        if api_key.is_empty() {
            return Err(anyhow::anyhow!("api key must not be empty"));
        }
        return Ok(api_key.to_owned());
    }
    if let Some(api_key) = env_api_key.map(str::trim).filter(|key| !key.is_empty()) {
        return Ok(api_key.to_owned());
    }
    load_saved_key()
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
    #[test]
    fn api_key_precedence_is_cli_then_environment_then_saved_config() {
        let unused = || -> anyhow::Result<String> { panic!("config must not be read") };
        assert_eq!(
            resolve_api_key_from_sources(Some(" cli "), Some("env"), unused).unwrap(),
            "cli"
        );
        assert_eq!(
            resolve_api_key_from_sources(None, Some(" env "), unused).unwrap(),
            "env"
        );
        assert!(resolve_api_key_from_sources(Some(" "), Some("env"), unused).is_err());
        for env in [None, Some(""), Some(" ")] {
            assert_eq!(
                resolve_api_key_from_sources(None, env, || Ok("saved".into())).unwrap(),
                "saved"
            );
        }
        assert!(
            resolve_api_key_from_sources(None, None, || anyhow::bail!("missing config")).is_err()
        );
    }
    #[test]
    fn malformed_config_diagnostics_do_not_disclose_key_values() {
        let error = parse_config("maonline_api_key = secret-key").unwrap_err();
        assert!(!format!("{error:#?} {error:#}").contains("secret-key"));
    }
}
