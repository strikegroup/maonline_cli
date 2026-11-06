// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

pub(crate) fn non_empty(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() {
        return Err("value must not be empty".into());
    }
    Ok(value.to_owned())
}

pub(crate) fn sec_code(value: &str) -> Result<String, String> {
    let value = non_empty(value)?;
    if value.len() != 4 || !value.bytes().all(|c| c.is_ascii_alphanumeric()) {
        return Err("security code must be 4 ASCII alphanumeric characters".into());
    }
    Ok(value.to_ascii_uppercase())
}

pub(crate) fn edinet_code(value: &str) -> Result<String, String> {
    let value = non_empty(value)?.to_ascii_uppercase();
    if value.len() != 6
        || !value.starts_with('E')
        || !value.as_bytes()[1..].iter().all(u8::is_ascii_digit)
    {
        return Err("EDINET code must be E followed by 5 digits".into());
    }
    Ok(value)
}

pub(crate) fn date(value: &str) -> Result<String, String> {
    let value = non_empty(value)?;
    if value.len() != 10 || chrono::NaiveDate::parse_from_str(&value, "%Y-%m-%d").is_err() {
        return Err("date must be a valid YYYY-MM-DD date".into());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_codes_and_text_without_changing_internal_spaces() {
        assert_eq!(
            non_empty("  サンプル  株式会社  ").unwrap(),
            "サンプル  株式会社"
        );
        assert_eq!(sec_code(" 130a ").unwrap(), "130A");
        assert_eq!(edinet_code(" e02144 ").unwrap(), "E02144");
        for value in ["", "  ", "123", "12345", "１２３４", "12-4"] {
            assert!(sec_code(value).is_err());
        }
        for value in ["E1234", "X12345", "E1234A", "Ｅ12345"] {
            assert!(edinet_code(value).is_err());
        }
    }

    #[test]
    fn validates_calendar_dates_and_normalizes_whitespace() {
        assert_eq!(date(" 2024-02-29 ").unwrap(), "2024-02-29");
        for value in ["2025-02-29", "2026-13-01", "2026-1-01", "", "yesterday"] {
            assert!(date(value).is_err(), "{value}");
        }
    }
}
