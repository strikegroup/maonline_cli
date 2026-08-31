// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

const QUALIFIER: &str = "jp.co";
const ORGANIZATION: &str = "strike";
const APPLICATION: &str = "maonline_cli";

fn project_dirs() -> anyhow::Result<directories::ProjectDirs> {
    directories::ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION).ok_or_else(|| {
        anyhow::anyhow!("failed to resolve application directory from the operating system")
    })
}

/// アプリのデータ保存先ディレクトリを返す。
pub fn default_data_local_dir() -> anyhow::Result<std::path::PathBuf> {
    Ok(project_dirs()?.data_local_dir().to_path_buf())
}

/// アプリの設定ディレクトリを返す。
pub fn config_dir() -> anyhow::Result<std::path::PathBuf> {
    Ok(project_dirs()?.config_dir().to_path_buf())
}

/// 設定ファイル `config.toml` のパスを返す。
pub fn config_file_path() -> anyhow::Result<std::path::PathBuf> {
    Ok(config_dir()?.join("config.toml"))
}
