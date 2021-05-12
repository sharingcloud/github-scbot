//! Configuration module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod errors;
pub mod sentry;
pub mod validation;

use tracing_subscriber::EnvFilter;

pub use crate::{
    config::Config,
    errors::{ConfError, Result},
};

/// Configure application startup.
pub fn configure_startup() -> Result<Config> {
    dotenv::dotenv().ok();
    stable_eyre::install().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .json()
        .init();
    let config = Config::from_env();

    self::validation::validate_configuration(&config)?;
    Ok(config)
}
