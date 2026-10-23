// SPDX-FileCopyrightText: 2026 Strike Group Co., Ltd.
//
// SPDX-License-Identifier: Apache-2.0

use std::io::IsTerminal;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{filter::Targets, prelude::*};

fn level(verbose: u8, quiet: u8) -> LevelFilter {
    match (verbose, quiet) {
        (_, 3..) => LevelFilter::OFF,
        (_, 2) => LevelFilter::ERROR,
        (_, 1) => LevelFilter::WARN,
        (0, _) => LevelFilter::INFO,
        (1, _) => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    }
}

fn targets(verbose: u8, quiet: u8) -> Targets {
    // Dependency trace output can include HTTP headers or payloads. Enable only
    // application diagnostics, even at -vv; do not inherit RUST_LOG.
    Targets::new().with_target("maonline", level(verbose, quiet))
}

pub(crate) fn init(verbose: u8, quiet: u8) {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stderr)
                .with_ansi(std::io::stderr().is_terminal())
                .without_time()
                .with_target(false),
        )
        .with(targets(verbose, quiet))
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn verbosity_levels_and_dependency_filter_protect_diagnostics() {
        assert_eq!(level(0, 0), LevelFilter::INFO);
        assert_eq!(level(1, 0), LevelFilter::DEBUG);
        assert_eq!(level(2, 0), LevelFilter::TRACE);
        assert_eq!(level(0, 1), LevelFilter::WARN);
        assert_eq!(level(0, 2), LevelFilter::ERROR);
        assert_eq!(level(0, 3), LevelFilter::OFF);
        #[derive(Clone)]
        struct Buffer(Arc<Mutex<Vec<u8>>>);
        impl std::io::Write for Buffer {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let buffer = Buffer(Arc::default());
        let writer = buffer.clone();
        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_ansi(false)
                    .without_time()
                    .with_writer(move || writer.clone()),
            )
            .with(targets(2, 0));
        tracing::subscriber::with_default(subscriber, || {
            tracing::debug!(target: "maonline::api_client", "safe diagnostic");
            tracing::trace!(target: "reqwest", "secret-key");
            tracing::trace!(target: "hyper", "secret-body");
        });
        let output = String::from_utf8(buffer.0.lock().unwrap().clone()).unwrap();
        assert!(output.contains("safe diagnostic"));
        assert!(!output.contains("secret"));
    }
}
