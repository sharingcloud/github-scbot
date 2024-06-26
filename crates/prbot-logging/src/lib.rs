//! Log configuration.

use std::str::FromStr;

use prbot_config::Config;
use prbot_sentry::sentry;
use thiserror::Error;
use tracing::subscriber::{DefaultGuard, NoSubscriber};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_error::ErrorLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};
use tracing_tree::HierarchicalLayer;

const DEFAULT_ENV_CONFIG: &str = "info,sqlx=error,prbot=debug";

#[derive(Debug, Error)]
pub enum LoggingError {
    #[error(
        "Could not set tracing global default subscriber,\n  caused by: {}",
        source
    )]
    TracingSetGlobalDefaultError {
        source: tracing::dispatcher::SetGlobalDefaultError,
    },
    #[error("Could not initialize tracing log tracer,\n  caused by: {}", source)]
    TracingLogTracerError {
        source: tracing::log::SetLoggerError,
    },
    #[error(
        "Wrong env filter configuration: {}\n  caused by: {}",
        configuration,
        source
    )]
    EnvFilterConfigurationError {
        source: tracing_subscriber::filter::ParseError,
        configuration: String,
    },
}

/// Configre logging.
pub fn configure_logging(config: &Config) -> Result<(), LoggingError> {
    LogTracer::init().map_err(|e| LoggingError::TracingLogTracerError { source: e })?;

    let log_config = std::env::var("RUST_LOG").unwrap_or_else(|_| DEFAULT_ENV_CONFIG.to_string());
    let app_name = format!("prbot/{}", config.version);

    let filter_layer = EnvFilter::from_str(&log_config).map_err(|e| {
        LoggingError::EnvFilterConfigurationError {
            source: e,
            configuration: log_config,
        }
    })?;
    let hierarchical_layer = HierarchicalLayer::new(2)
        .with_targets(true)
        .with_bracketed_fields(true);
    let error_layer = ErrorLayer::default();
    let json_storage_layer = {
        if config.logging.use_bunyan {
            Some(JsonStorageLayer)
        } else {
            None
        }
    };
    let bunyan_layer = {
        if config.logging.use_bunyan {
            Some(BunyanFormattingLayer::new(app_name, std::io::stdout))
        } else {
            None
        }
    };
    let sentry_layer = {
        if !config.sentry.url.is_empty() {
            Some(sentry::integrations::tracing::layer())
        } else {
            None
        }
    };

    let subscriber = tracing_subscriber::registry()
        .with(sentry_layer)
        .with(error_layer)
        .with(hierarchical_layer)
        .with(filter_layer)
        .with(json_storage_layer)
        .with(bunyan_layer);

    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| LoggingError::TracingSetGlobalDefaultError { source: e })?;

    Ok(())
}

pub fn temporarily_disable_logging() -> DefaultGuard {
    tracing::subscriber::set_default(NoSubscriber::default())
}
