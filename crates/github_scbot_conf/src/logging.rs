//! Log configuration.

use std::str::FromStr;

use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

use crate::Config;

const DEFAULT_ENV_CONFIG: &str = "info,github_scbot=debug";

pub fn configure_logging(config: &Config) {
    let log_config = std::env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_ENV_CONFIG.to_string());

    if config.logging_use_bunyan {
        let app_name = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")).to_string();
        let layer = BunyanFormattingLayer::new(app_name, std::io::stdout);
        let subscriber = Registry::default()
            .with(EnvFilter::from_str(&log_config).expect("Bad log configuration"))
            .with(JsonStorageLayer)
            .with(layer);
        tracing::subscriber::set_global_default(subscriber).unwrap();
    } else {
        let subscriber = tracing_subscriber::fmt().finish();
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    LogTracer::init().expect("Unable to setup log tracer.");
}
