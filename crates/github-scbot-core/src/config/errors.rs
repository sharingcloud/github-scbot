//! Configuration errors.

use snafu::prelude::*;

/// Configuration error.
#[allow(missing_docs)]
#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum ConfError {
    #[snafu(display(
        "Could not set tracing global default subscriber,\n  caused by: {}",
        source
    ))]
    TracingSetGlobalDefaultError {
        source: tracing::dispatcher::SetGlobalDefaultError,
    },
    #[snafu(display("Could not initialize tracing log tracer,\n  caused by: {}", source))]
    TracingLogTracerError {
        source: tracing::log::SetLoggerError,
    },
    #[snafu(display(
        "Wrong env filter configuration: {}\n  caused by: {}",
        configuration,
        source
    ))]
    EnvFilterConfigurationError {
        source: tracing_subscriber::filter::ParseError,
        configuration: String,
    },
    #[snafu(display("Errors on environment variables:\n{}", errors))]
    EnvVarsError { errors: String },
}

/// Result alias for `ConfError`.
pub type Result<T, E = ConfError> = core::result::Result<T, E>;
