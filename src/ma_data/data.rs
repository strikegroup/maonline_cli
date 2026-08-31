// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct MaData {
    disdate: String,
    title: String,
    price: Option<i64>,
    total: Option<i64>,
    maoscheme: String,
    maoschemename: String,
    maoschemesub: String,
    maoschemesubname: String,
    inout: String,
    gyosyudiff: String,
    buyer_info: BuyerInfo,
    target_info: TargetInfo,
    seller_info: SellerInfo,
}

#[derive(Debug, Serialize)]
struct BuyerInfo {
    name: String,
    code: String,
    industry: String,
    area: String,
}

#[derive(Debug, Serialize)]
struct TargetInfo {
    name: String,
    code: String,
    industry: String,
    area: String,
}

#[derive(Debug, Serialize)]
struct SellerInfo {
    name: String,
    code: String,
    industry: String,
    area: String,
}

impl From<super::raw_response::RawMaData> for MaData {
    fn from(raw: super::raw_response::RawMaData) -> Self {
        MaData {
            disdate: raw.disdate,
            title: raw.title,
            price: raw.price,
            total: raw.total,
            maoscheme: raw.maoscheme,
            maoschemename: raw.maoschemename,
            maoschemesub: raw.maoschemesub,
            maoschemesubname: raw.maoschemesubname,
            inout: raw.inout,
            gyosyudiff: raw.gyosyudiff,
            buyer_info: BuyerInfo {
                name: raw.buyer_info.name,
                code: raw.buyer_info.code,
                industry: raw.buyer_info.industry,
                area: raw.buyer_info.area,
            },
            target_info: TargetInfo {
                name: raw.target_info.name,
                code: raw.target_info.code,
                industry: raw.target_info.industry,
                area: raw.target_info.area,
            },
            seller_info: SellerInfo {
                name: raw.seller_info.name,
                code: raw.seller_info.code,
                industry: raw.seller_info.industry,
                area: raw.seller_info.area,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MaData;
    use crate::ma_data::raw_response::{RawBuyerInfo, RawMaData, RawSellerInfo, RawTargetInfo};

    #[test]
    fn converts_raw_ma_data() {
        let ma_data = MaData::from(RawMaData {
            disdate: "2024-05-20".to_owned(),
            title: "案件".to_owned(),
            price: Some(10_000_000_000),
            total: Some(20_000_000_000),
            maoscheme: "stock".to_owned(),
            maoschemename: "株式譲渡".to_owned(),
            maoschemesub: String::new(),
            maoschemesubname: String::new(),
            inout: "IN-IN".to_owned(),
            gyosyudiff: String::new(),
            buyer_info: RawBuyerInfo::default(),
            target_info: RawTargetInfo::default(),
            seller_info: RawSellerInfo::default(),
        });

        assert_eq!(ma_data.title, "案件");
        assert_eq!(ma_data.price, Some(10_000_000_000));
        assert_eq!(ma_data.total, Some(20_000_000_000));
    }
}
