// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[test]
fn rejects_invalid_search_inputs_before_resolving_credentials() {
    let cases = [
        (vec!["companies", "--name", "  "], "value must not be empty"),
        (
            vec!["companies", "--sec-code", "123"],
            "security code must be",
        ),
        (
            vec!["companies", "--edinet-code", "E1234X"],
            "EDINET code must be",
        ),
        (
            vec!["news", "search", "--date-from", "2025-02-29"],
            "valid YYYY-MM-DD",
        ),
        (
            vec![
                "news",
                "search",
                "--date-from",
                "2026-02-01",
                "--date-to",
                "2026-01-01",
            ],
            "--date-from must be earlier",
        ),
        (
            vec!["ma-data", "--min-price", "2", "--max-price", "1"],
            "--min-price must be less",
        ),
        (vec!["news", "get", " "], "value must not be empty"),
    ];
    for (args, message) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_maonline"))
            .env("MAONLINE_API_KEY", "")
            .args(&args)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains(message), "{args:?}: {stderr}");
    }
}
