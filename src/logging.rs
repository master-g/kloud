//! Logging configuration module
//!
//! Provides tracing subscriber configuration for console logging.

use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize logging with the given level.
///
/// If `RUST_LOG` environment variable is set, it takes precedence.
/// Otherwise, uses the provided level string.
pub fn init(level: &str) {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    let pretty = std::env::var("KLOUD_LOG_PRETTY").map(|v| v == "true").unwrap_or(true);

    let subscriber = tracing_subscriber::registry().with(env_filter);

    if pretty {
        subscriber
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true)
                    .pretty(),
            )
            .init();
    } else {
        subscriber
            .with(
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_file(true)
                    .with_line_number(true),
            )
            .init();
    }
}
