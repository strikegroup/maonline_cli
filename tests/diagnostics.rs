// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use std::process::Command;

#[test]
fn diagnostics_go_to_stderr_and_global_flags_work_after_subcommands() {
    for args in [vec!["-v", "companies"], vec!["companies", "-v"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_maonline"))
            .args(args)
            .args(["--name", "sample", "--key", " "])
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("command started"));
        assert!(stderr.contains("api key must not be empty"));
    }
    let output = Command::new(env!("CARGO_BIN_EXE_maonline"))
        .args(["companies", "--name", "sample", "--key", " ", "-qqq"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
