//! CLI module.

use anyhow::Result;

use args::{Args, CommandExecutor};
use clap::Parser;
use github_scbot_core::config::configure_startup;

pub(crate) mod args;
mod commands;
pub(crate) mod utils;
#[cfg(test)]
mod testutils;

/// Initialize command line.
pub fn initialize_command_line() -> Result<()> {
    let config = configure_startup()?;
    let args = Args::parse();

    CommandExecutor::parse_args(config, args)
}
