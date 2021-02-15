//! Configuration module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod config;
pub mod errors;
pub mod validation;

pub use crate::{
    config::Config,
    errors::{ConfError, Result},
};

/// Configure application startup.
pub fn configure_startup() -> Result<Config> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();
    let config = Config::from_env();

    self::validation::validate_configuration(&config)?;
    Ok(config)
}
