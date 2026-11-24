// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[test]
fn rejects_invalid_search_inputs_before_resolving_credentials() {
    let cases = [
        (
            vec!["ma-data", "--company", "  "],
            "value must not be empty",
        ),
        (vec!["ma-data", "--scheme", "  "], "value must not be empty"),
        (vec!["ma-data", "--inout", "  "], "value must not be empty"),
        (vec!["ma-data", "--area", "  "], "value must not be empty"),
        (
            vec!["ma-data", "--min-price", "2", "--max-price", "1"],
            "--min-price must be less",
        ),
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

#[test]
fn rejects_invalid_pagination_on_ma_data() {
    for (options, message) in [
        (vec!["--page", "0"], "invalid value"),
        (vec!["--limit", "201"], "invalid value"),
        (vec!["--page", "2"], "requires --limit"),
        (vec!["--limit", "0", "--page", "2"], "requires --limit"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_maonline"))
            .arg("ma-data")
            .args(&options)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains(message), "{options:?}: {stderr}");
    }
}
