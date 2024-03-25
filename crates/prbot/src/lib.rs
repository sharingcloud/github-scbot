//! CLI module.

use anyhow::Result;
use args::{Args, CommandExecutor};
use clap::Parser;
use prbot_config::Config;
use prbot_logging::configure_logging;
use shadow_rs::shadow;
use tracing::info;

pub(crate) mod args;
mod commands;
mod config_validator;
#[cfg(test)]
mod testutils;
pub(crate) mod utils;

shadow!(build);

/// Get version data.
pub fn get_version_data() -> String {
    format!(
        "{} {} (commit {} - {})",
        build::PROJECT_NAME,
        build::PKG_VERSION,
        build::COMMIT_HASH,
        build::COMMIT_DATE_3339
    )
}

/// Initialize command line.
pub fn initialize_command_line() -> Result<()> {
    dotenv::dotenv().ok();

    let config = Config::from_env(env!("CARGO_PKG_VERSION").to_string());
    configure_logging(&config)?;
    config_validator::validate_configuration(&config)?;

    info!("{}", get_version_data());

    let args = Args::parse();
    CommandExecutor::parse_args(config, args)
}
