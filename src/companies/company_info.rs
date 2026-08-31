// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;

use super::raw_response::RawCompanyInfo;

#[derive(Debug, Serialize)]
pub(super) struct CompanyInfo {
    sec_code: Option<String>,
    edinet_code: String,
    company_name: String,
    market: Option<String>,
    address: Option<String>,
    homepage: Option<String>,
}

impl From<RawCompanyInfo> for CompanyInfo {
    fn from(raw: RawCompanyInfo) -> Self {
        Self {
            sec_code: Some(raw.edsyokencode),
            edinet_code: raw.edcode,
            company_name: raw.edname,
            market: Some(raw.market),
            address: Some(raw.address),
            homepage: Some(raw.homepage),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CompanyInfo, RawCompanyInfo};

    #[test]
    fn converts_raw_company_info() {
        let company = CompanyInfo::from(RawCompanyInfo {
            edsyokencode: "1234".to_owned(),
            edcode: "E01234".to_owned(),
            edname: "株式会社サンプル".to_owned(),
            market: "東証プライム".to_owned(),
            address: "東京都".to_owned(),
            homepage: "https://example.com".to_owned(),
        });

        assert_eq!(company.sec_code.as_deref(), Some("1234"));
        assert_eq!(company.company_name, "株式会社サンプル");
    }
}
