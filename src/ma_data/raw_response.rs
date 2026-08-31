// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct RawResponse {
    pub(super) data: Vec<RawMaData>,
    pub(super) meta: RawMeta,
}

#[derive(Default, Debug, Deserialize)]
pub(super) struct RawMaData {
    pub(super) disdate: String,
    pub(super) title: String,
    pub(super) price: Option<i64>,
    pub(super) total: Option<i64>,
    pub(super) maoscheme: String,
    pub(super) maoschemename: String,
    pub(super) maoschemesub: String,
    pub(super) maoschemesubname: String,
    pub(super) inout: String,
    pub(super) gyosyudiff: String,
    pub(super) buyer_info: RawBuyerInfo,
    pub(super) target_info: RawTargetInfo,
    pub(super) seller_info: RawSellerInfo,
}

#[derive(Default, Debug, Deserialize)]
pub(super) struct RawBuyerInfo {
    pub(super) name: String,
    pub(super) code: String,
    pub(super) industry: String,
    pub(super) area: String,
}

#[derive(Default, Debug, Deserialize)]
pub(super) struct RawTargetInfo {
    pub(super) name: String,
    pub(super) code: String,
    pub(super) industry: String,
    pub(super) area: String,
}

#[derive(Default, Debug, Deserialize)]
pub(super) struct RawSellerInfo {
    pub(super) name: String,
    pub(super) code: String,
    pub(super) industry: String,
    pub(super) area: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct RawMeta {
    pub(super) next_page: Option<u32>,
}

#[cfg(test)]
mod tests {
    use super::RawResponse;

    #[test]
    fn deserializes_ma_data_response() {
        let response: RawResponse = serde_json::from_str(
            r#"{
                "data": [{
                    "disdate": "2024-05-20",
                    "title": "案件",
                    "price": 10000000000,
                    "total": 20000000000,
                    "maoscheme": "stock",
                    "maoschemename": "株式譲渡",
                    "maoschemesub": "",
                    "maoschemesubname": "",
                    "inout": "IN-IN",
                    "gyosyudiff": "",
                    "buyer_info": {"name": "", "code": "", "industry": "", "area": ""},
                    "target_info": {"name": "", "code": "", "industry": "", "area": ""},
                    "seller_info": {"name": "", "code": "", "industry": "", "area": ""}
                }],
                "meta": {"current_page": 1, "next_page": null, "total_count": 1}
            }"#,
        )
        .expect("response must be deserializable");

        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].price, Some(10_000_000_000));
        assert_eq!(response.data[0].total, Some(20_000_000_000));
        assert!(response.meta.next_page.is_none());
    }
}
