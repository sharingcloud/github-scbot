//! Configuration module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod errors;
mod logging;
pub mod validation;

pub use crate::{
    config::Config,
    errors::{ConfError, Result},
};

/// Configure application startup.
pub fn configure_startup() -> Result<Config> {
    dotenv::dotenv().ok();
    github_scbot_sentry::eyre::install().ok();

    let config = Config::from_env();
    self::logging::configure_logging(&config);

    self::validation::validate_configuration(&config)?;
    Ok(config)
}
