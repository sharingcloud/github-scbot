//! Startup utils

use color_eyre::Result;

/// Configure startup
///
/// # Errors
///
/// Error if called twice.
///
pub fn configure_startup() -> Result<()> {
    dotenv::dotenv().ok();
    std::env::set_var("RUST_LOG", "info");

    pretty_env_logger::init();
    color_eyre::install()?;

    Ok(())
}
