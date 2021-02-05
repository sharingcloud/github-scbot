//! Core module.

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod constants;
pub mod errors;
pub mod validation;

pub use self::errors::{CoreError, Result};

/// Configure application startup.
pub fn configure_startup() -> Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");
    tracing_subscriber::fmt::init();

    self::validation::validate_configuration()
}
