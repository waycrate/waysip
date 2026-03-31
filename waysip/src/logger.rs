//! Stderr logging via `tracing-subscriber`.

use std::io;

use crate::cli::Cli;

/// Initialise the tracing subscriber, writing formatted logs to stderr.
///
/// The log level is taken from `--log-level` if supplied, otherwise [`tracing::Level::INFO`].
pub fn setup(cli: &Cli) {
    let level = cli.log_level.unwrap_or(tracing::Level::INFO);
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_writer(io::stderr)
        .init();
}
