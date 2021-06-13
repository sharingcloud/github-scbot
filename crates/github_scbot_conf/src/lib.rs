//! Configuration module.

#![warn(missing_docs)]
#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::missing_errors_doc,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::must_use_candidate,
    clippy::module_name_repetitions,
    clippy::struct_excessive_bools
)]

pub mod config;
pub mod errors;
mod logging;
pub mod sentry;
pub mod validation;

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
