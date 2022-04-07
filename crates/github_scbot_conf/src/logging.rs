//! Log configuration.

use std::str::FromStr;

use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

use github_scbot_sentry::eyre::Result;
use tracing_tree::HierarchicalLayer;

use crate::Config;

const DEFAULT_ENV_CONFIG: &str = "info,github_scbot=debug";

pub fn configure_logging(config: &Config) -> Result<()> {
    LogTracer::init().expect("Unable to setup log tracer.");

    let log_config = std::env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_ENV_CONFIG.to_string());
    let app_name = concat!(env!("CARGO_PKG_NAME"), "-", env!("CARGO_PKG_VERSION")).to_string();

    let filter_layer = EnvFilter::from_str(&log_config)?;
    let hierarchical_layer = HierarchicalLayer::new(2)
        .with_targets(true)
        .with_bracketed_fields(true);
    let error_layer = ErrorLayer::default();
    let json_storage_layer = {
        if config.logging_use_bunyan {
            Some(JsonStorageLayer)
        } else {
            None
        }
    };
    let bunyan_layer = {
        if config.logging_use_bunyan {
            Some(BunyanFormattingLayer::new(app_name, std::io::stdout))
        } else {
            None
        }
    };
    let telemetry_layer = {
        if !config.telemetry_url.is_empty() {
            let tracer = opentelemetry_jaeger::new_pipeline()
                .with_agent_endpoint(config.telemetry_url.as_str())
                .install_batch(opentelemetry::runtime::Tokio)?;
            let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
            Some(telemetry)
        } else {
            None
        }
    };

    let subscriber = tracing_subscriber::registry()
        .with(error_layer)
        .with(hierarchical_layer)
        .with(filter_layer)
        .with(json_storage_layer)
        .with(bunyan_layer)
        .with(telemetry_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
