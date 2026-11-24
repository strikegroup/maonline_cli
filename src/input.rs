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

#[cfg(test)]
mod tests {
    use super::non_empty;

    #[test]
    fn trims_text_without_changing_internal_spaces() {
        assert_eq!(
            non_empty("  サンプル  株式会社  ").unwrap(),
            "サンプル  株式会社"
        );
        for value in ["", "  "] {
            assert!(non_empty(value).is_err());
        }
    }
}
