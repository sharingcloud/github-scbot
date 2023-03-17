//! CLI module.

use anyhow::Result;
use args::{Args, CommandExecutor};
use clap::Parser;
use github_scbot_config::Config;
use github_scbot_logging::configure_logging;

pub(crate) mod args;
mod commands;
mod config_validator;
#[cfg(test)]
mod testutils;
pub(crate) mod utils;

/// Initialize command line.
pub fn initialize_command_line() -> Result<()> {
    dotenv::dotenv().ok();

    let config = Config::from_env();
    configure_logging(&config)?;
    config_validator::validate_configuration(&config)?;

    let args = Args::parse();
    CommandExecutor::parse_args(config, args)
}
