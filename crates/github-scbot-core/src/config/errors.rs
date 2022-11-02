//! Configuration errors.

use thiserror::Error;

/// Configuration error.
#[derive(Debug, Error)]
pub enum ConfError {
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
    #[error("Errors on environment variables:\n{}", errors)]
    EnvVarsError { errors: String },
}

/// Result alias for `ConfError`.
pub type Result<T, E = ConfError> = core::result::Result<T, E>;
