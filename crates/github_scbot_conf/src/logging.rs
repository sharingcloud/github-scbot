//! Log configuration.

use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

const DEFAULT_ENV_CONFIG: &str = "info,github_scbot=debug";

pub fn configure_logging() {
    if std::env::var("RUST_LOG").unwrap_or_default().is_empty() {
        std::env::set_var("RUST_LOG", DEFAULT_ENV_CONFIG);
    }

    LogTracer::init().expect("Unable to setup log tracer.");

    let app_name = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")).to_string();
    let layer = BunyanFormattingLayer::new(app_name, std::io::stdout);
    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(JsonStorageLayer)
        .with(layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
