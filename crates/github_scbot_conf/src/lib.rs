//! Configuration module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod errors;
mod logging;
pub mod sentry;
pub mod validation;

use github_scbot_libs::{dotenv, stable_eyre};

pub use crate::{
    config::Config,
    errors::{ConfError, Result},
};

/// Configure application startup.
pub fn configure_startup() -> Result<Config> {
    dotenv::dotenv().ok();
    stable_eyre::install().ok();

    self::logging::configure_logging();
    let config = Config::from_env();

    self::validation::validate_configuration(&config)?;
    Ok(config)
}
