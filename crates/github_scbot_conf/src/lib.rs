//! Configuration module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod errors;
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
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt().json().init();
    let config = Config::from_env();

    self::validation::validate_configuration(&config)?;
    Ok(config)
}
