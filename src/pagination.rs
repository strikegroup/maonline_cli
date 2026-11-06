// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

#[derive(Debug, Clone, Copy, clap::Args)]
pub(crate) struct Pagination {
    #[arg(long, short = 'l', default_value_t = 0,
        value_parser = clap::value_parser!(u32).range(0..=200),
        help = "1ページの取得件数を指定します（1〜200、0は全件取得）")]
    pub(crate) limit: u32,
    #[arg(long, short = 'p', default_value_t = 1,
        value_parser = clap::value_parser!(u32).range(1..),
        help = "取得するページ番号を指定します（--limit が1以上の場合のみ）")]
    pub(crate) page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self { limit: 0, page: 1 }
    }
}

impl Pagination {
    pub(crate) fn validate(&self) -> anyhow::Result<()> {
        anyhow::ensure!(
            self.limit > 0 || self.page == 1,
            "--page greater than 1 requires --limit between 1 and 200"
        );
        Ok(())
    }

    pub(crate) fn per_page(self) -> u32 {
        if self.limit == 0 { 200 } else { self.limit }
    }
}
